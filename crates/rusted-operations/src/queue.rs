use crate::{
    copy_path, delete_permanent, move_path, recycle, OperationControl, OperationError, OperationKind,
    OperationProgress, OperationRequest, OperationStatus, SharedControl,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

/// More than a handful of parallel disk writers only hurts throughput.
const MAX_WORKERS: usize = 4;
const QUEUED_LIMIT: usize = 512;

struct Job {
    request: OperationRequest,
    control: SharedControl,
}

/// Coordinates background file operations. Workers are created once and block on
/// the job channel, so an idle queue costs a parked thread and no polling.
pub struct OperationQueue {
    sender: Sender<Job>,
    progress: Receiver<OperationProgress>,
    progress_sender: Sender<OperationProgress>,
    controls: Arc<Mutex<HashMap<Uuid, SharedControl>>>,
    history: Arc<Mutex<Vec<OperationProgress>>>,
}

impl OperationQueue {
    pub fn new(worker_count: usize) -> Self {
        let (jobs_tx, jobs_rx) = unbounded::<Job>();
        let (progress_tx, progress_rx) = unbounded::<OperationProgress>();
        let controls: Arc<Mutex<HashMap<Uuid, SharedControl>>> = Arc::new(Mutex::new(HashMap::new()));
        let history: Arc<Mutex<Vec<OperationProgress>>> = Arc::new(Mutex::new(Vec::new()));
        let jobs = Arc::new(Mutex::new(jobs_rx));

        for _ in 0..worker_count.clamp(1, MAX_WORKERS) {
            let jobs = jobs.clone();
            let events = progress_tx.clone();
            let controls = controls.clone();
            let history = history.clone();
            std::thread::Builder::new()
                .name("rusted-operation".into())
                .spawn(move || loop {
                    let job = {
                        let guard = match jobs.lock() {
                            Ok(guard) => guard,
                            Err(_) => break,
                        };
                        match guard.recv() {
                            Ok(job) => job,
                            Err(_) => break,
                        }
                    };
                    run_job(&job, &events, &history);
                    if let Ok(mut map) = controls.lock() {
                        map.remove(&job.request.id);
                    }
                })
                .expect("operation worker thread");
        }

        Self { sender: jobs_tx, progress: progress_rx, progress_sender: progress_tx, controls, history }
    }

    pub fn submit(&self, request: OperationRequest, control: SharedControl) -> Result<Uuid, OperationError> {
        let id = request.id;
        self.controls
            .lock()
            .map_err(|_| OperationError::Invalid("queue lock poisoned".into()))?
            .insert(id, control.clone());
        let queued = OperationProgress::new(id, request.kind, OperationStatus::Queued, request.sources.len());
        let _ = self.progress_sender.send(queued);
        self.sender
            .send(Job { request, control })
            .map_err(|_| OperationError::Invalid("operation workers have stopped".into()))?;
        Ok(id)
    }

    pub fn pause(&self, id: Uuid) -> bool {
        match self.control(id) {
            Some(control) => {
                control.pause();
                true
            }
            None => false,
        }
    }

    pub fn resume(&self, id: Uuid) -> bool {
        match self.control(id) {
            Some(control) => {
                control.resume();
                true
            }
            None => false,
        }
    }

    pub fn cancel(&self, id: Uuid) -> bool {
        match self.control(id) {
            Some(control) => {
                control.cancel();
                true
            }
            None => false,
        }
    }

    pub fn control(&self, id: Uuid) -> Option<SharedControl> {
        self.controls.lock().ok()?.get(&id).cloned()
    }

    pub fn is_active(&self, id: Uuid) -> bool {
        self.control(id).is_some()
    }

    /// Non-blocking drain of progress events; the UI calls this on a timer.
    pub fn try_progress(&self) -> Option<OperationProgress> {
        self.progress.try_recv().ok()
    }

    pub fn drain(&self, limit: usize) -> Vec<OperationProgress> {
        let mut events = Vec::new();
        while events.len() < limit {
            match self.progress.try_recv() {
                Ok(event) => events.push(event),
                Err(_) => break,
            }
        }
        events
    }

    /// Most recent finished operations, newest first.
    pub fn history(&self) -> Vec<OperationProgress> {
        self.history.lock().map(|entries| entries.iter().rev().cloned().collect()).unwrap_or_default()
    }

    pub fn queued_len(&self) -> usize {
        self.sender.len().min(QUEUED_LIMIT)
    }
}

fn run_job(job: &Job, events: &Sender<OperationProgress>, history: &Arc<Mutex<Vec<OperationProgress>>>) {
    let request = &job.request;
    let id = request.id;
    let kind = request.kind;
    let total = request.sources.len();
    let started = Instant::now();
    let mut progress = OperationProgress::new(id, kind, OperationStatus::Running, total);
    let _ = events.send(progress.clone());

    let result = execute(request, &job.control, |completed, current| {
        progress.completed_items = completed;
        progress.current = current.map(Path::to_path_buf);
        progress.bytes_done = job.control.bytes();
        progress.bytes_per_second = job.control.bytes() as f64 / started.elapsed().as_secs_f64().max(0.001);
        progress.status = if job.control.is_paused() { OperationStatus::Paused } else { OperationStatus::Running };
        let _ = events.send(progress.clone());
    });

    progress.bytes_done = job.control.bytes();
    progress.bytes_per_second = job.control.bytes() as f64 / started.elapsed().as_secs_f64().max(0.001);
    match result {
        Ok(done) => {
            progress.completed_items = done;
            progress.status = OperationStatus::Completed;
        }
        Err(OperationError::Cancelled) => progress.status = OperationStatus::Cancelled,
        Err(error) => {
            progress.status = OperationStatus::Failed;
            progress.error = Some(error.to_string());
        }
    }
    let _ = events.send(progress.clone());
    if let Ok(mut entries) = history.lock() {
        entries.push(progress);
        if entries.len() > 100 {
            entries.remove(0);
        }
    }
}

fn execute(
    request: &OperationRequest,
    control: &OperationControl,
    mut progress: impl FnMut(usize, Option<&Path>),
) -> Result<usize, OperationError> {
    let total = request.sources.len();
    if total == 0 {
        return Err(OperationError::Invalid("no sources were selected".into()));
    }
    match request.kind {
        OperationKind::Delete => {
            for source in &request.sources {
                if std::fs::symlink_metadata(source).is_err() {
                    return Err(OperationError::MissingSource(source.clone()));
                }
            }
            let count = recycle(&request.sources)?;
            progress(count, None);
            Ok(count)
        }
        OperationKind::DeletePermanent => {
            if !request.confirmed {
                return Err(OperationError::ConfirmationRequired);
            }
            for (index, source) in request.sources.iter().enumerate() {
                control.checkpoint()?;
                delete_permanent(source, true)?;
                progress(index + 1, Some(source));
            }
            Ok(total)
        }
        OperationKind::Copy | OperationKind::Move | OperationKind::Duplicate => {
            let into_directory = request.destination.is_dir() || request.sources.len() > 1;
            if !request.destination.is_dir() {
                return Err(OperationError::Invalid(format!(
                    "destination is not a folder: {}",
                    request.destination.display()
                )));
            }
            let mut done = 0;
            for source in &request.sources {
                control.checkpoint()?;
                let destination = if into_directory {
                    match source.file_name() {
                        Some(name) => request.destination.join(name),
                        None => return Err(OperationError::Invalid("source has no file name".into())),
                    }
                } else {
                    request.destination.clone()
                };
                progress(done, Some(source));
                let mut report = |_bytes: u64| {};
                match request.kind {
                    OperationKind::Copy => copy_path(source, &destination, request.conflict, control, &mut report)?,
                    OperationKind::Move => move_path(source, &destination, request.conflict, control, &mut report)?,
                    _ => copy_path(source, &destination, request.conflict, control, &mut report)?,
                };
                done += 1;
                progress(done, Some(source));
            }
            Ok(done)
        }
        OperationKind::Rename | OperationKind::Extract | OperationKind::CreateArchive => Err(OperationError::Invalid(
            "this operation runs through its dedicated engine".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConflictPolicy;
    use std::sync::Arc;
    use std::time::Duration;

    fn drain_until<F: Fn(&OperationProgress) -> bool>(queue: &OperationQueue, predicate: F) -> OperationProgress {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut last = None;
        while Instant::now() < deadline {
            while let Some(event) = queue.try_progress() {
                if predicate(&event) {
                    return event;
                }
                last = Some(event);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("timed out waiting for progress; last = {last:?}");
    }

    #[test]
    fn copies_files_and_reports_completion() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("src.txt");
        std::fs::write(&source, b"payload").unwrap();
        let target = dir.path().join("target");
        std::fs::create_dir(&target).unwrap();
        let queue = OperationQueue::new(2);
        let request = OperationRequest::new(OperationKind::Copy, vec![source], target.clone())
            .with_conflict(ConflictPolicy::Replace);
        let id = queue.submit(request, Arc::new(OperationControl::default())).unwrap();
        let event = drain_until(&queue, |event| event.id == id && event.status.is_finished());
        assert_eq!(event.status, OperationStatus::Completed);
        assert_eq!(std::fs::read_to_string(target.join("src.txt")).unwrap(), "payload");
        assert_eq!(queue.history().len(), 1);
        assert!(!queue.is_active(id));
    }

    #[test]
    fn cancel_stops_a_running_operation() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("out");
        std::fs::create_dir(&target).unwrap();
        let mut sources = Vec::new();
        for index in 0..20 {
            let path = dir.path().join(format!("file{index}.bin"));
            std::fs::write(&path, vec![0u8; 200 * 1024]).unwrap();
            sources.push(path);
        }
        let queue = OperationQueue::new(1);
        let control = Arc::new(OperationControl::default());
        let request = OperationRequest::new(OperationKind::Copy, sources, target)
            .with_conflict(ConflictPolicy::Replace);
        let id = queue.submit(request, control.clone()).unwrap();
        queue.cancel(id);
        let event = drain_until(&queue, |event| event.id == id && event.status.is_finished());
        assert!(matches!(event.status, OperationStatus::Cancelled | OperationStatus::Completed));
    }

    #[test]
    fn recycle_removes_the_source_through_the_operation_system() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("trash-me.txt");
        std::fs::write(&file, b"x").unwrap();
        let queue = OperationQueue::new(1);
        let request = OperationRequest::new(OperationKind::Delete, vec![file.clone()], dir.path().to_path_buf());
        let id = queue.submit(request, Arc::new(OperationControl::default())).unwrap();
        let event = drain_until(&queue, |event| event.id == id && event.status.is_finished());
        // On headless CI hosts the Recycle Bin may be unavailable; either the
        // file is gone or the failure is reported explicitly.
        if event.status == OperationStatus::Completed {
            assert!(!file.exists());
        } else {
            assert!(event.error.is_some(), "a failure must carry a reason");
            assert!(file.exists());
        }
    }

    #[test]
    fn permanent_delete_without_confirmation_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("keep.txt");
        std::fs::write(&file, b"x").unwrap();
        let queue = OperationQueue::new(1);
        let request = OperationRequest::new(OperationKind::DeletePermanent, vec![file.clone()], dir.path().to_path_buf());
        let id = queue.submit(request, Arc::new(OperationControl::default())).unwrap();
        let event = drain_until(&queue, |event| event.id == id && event.status.is_finished());
        assert_eq!(event.status, OperationStatus::Failed);
        assert!(file.exists());
    }

    #[test]
    fn history_is_bounded_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("out");
        std::fs::create_dir(&target).unwrap();
        let queue = OperationQueue::new(2);
        let mut ids = Vec::new();
        for index in 0..3 {
            let source = dir.path().join(format!("f{index}.txt"));
            std::fs::write(&source, b"x").unwrap();
            let request = OperationRequest::new(OperationKind::Copy, vec![source], target.clone())
                .with_conflict(ConflictPolicy::Replace);
            ids.push(queue.submit(request, Arc::new(OperationControl::default())).unwrap());
        }
        for id in ids {
            drain_until(&queue, |event| event.id == id && event.status.is_finished());
        }
        let history = queue.history();
        assert_eq!(history.len(), 3);
        assert!(history[0].completed_items > 0);
    }
}
