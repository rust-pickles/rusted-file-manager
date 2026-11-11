use crate::{copy_path,delete_permanent,move_path,recycle,OperationControl,OperationError,OperationKind,OperationProgress,OperationRequest,OperationStatus,SharedControl};
use crossbeam_channel::{Receiver,Sender};
use std::{collections::HashMap,path::{Path,PathBuf},sync::{Arc,Mutex},thread,time::Instant};
use uuid::Uuid;

struct Job{request:OperationRequest,control:SharedControl}
pub struct OperationQueue{sender:Sender<Job>,progress:Receiver<OperationProgress>,controls:Arc<Mutex<HashMap<Uuid,SharedControl>>>}
impl OperationQueue{
    pub fn new(worker_count:usize)->Self{
        let(jobs_tx,jobs_rx)=crossbeam_channel::unbounded::<Job>();let(progress_tx,progress_rx)=crossbeam_channel::unbounded();let controls=Arc::new(Mutex::new(HashMap::new()));let jobs=Arc::new(Mutex::new(jobs_rx));
        for _ in 0..worker_count.clamp(1,4){let jobs=jobs.clone();let events=progress_tx.clone();let controls=controls.clone();thread::spawn(move||loop{let job={let lock=match jobs.lock(){Ok(v)=>v,Err(_)=>break};match lock.recv(){Ok(j)=>j,Err(_)=>break}};let id=job.request.id;let kind=job.request.kind;send(&events,&job.request,OperationStatus::Running,None,0,0,None);let started=Instant::now();let mut bytes=0;let result=execute(&job.request,&job.control,|done|{bytes=done;send(&events,&job.request,OperationStatus::Running,None,0,bytes,Some(bytes as f64/started.elapsed().as_secs_f64().max(0.001)));});let status=match &result{Ok(())=>OperationStatus::Completed,Err(OperationError::Cancelled)=>OperationStatus::Cancelled,Err(_)=>OperationStatus::Failed};let mut update=progress(id,kind,status);update.bytes_done=bytes;update.bytes_per_second=Some(bytes as f64/started.elapsed().as_secs_f64().max(0.001));if let Err(e)=result{update.error=Some(e.to_string());}let _=events.send(update);controls.lock().unwrap().remove(&id);});}
        Self{sender:jobs_tx,progress:progress_rx,controls}
    }
    pub fn submit(&self,request:OperationRequest)->Result<Uuid,OperationError>{let id=request.id;let control=Arc::new(OperationControl::default());self.controls.lock().unwrap().insert(id,control.clone());self.sender.send(Job{request,control}).map_err(|_|OperationError::Invalid("operation workers stopped".into()))?;Ok(id)}
    pub fn control(&self,id:Uuid)->Option<SharedControl>{self.controls.lock().ok()?.get(&id).cloned()}
    pub fn pause(&self,id:Uuid)->bool{self.control(id).inspect(|c|c.pause()).is_some()}
    pub fn resume(&self,id:Uuid)->bool{self.control(id).inspect(|c|c.resume()).is_some()}
    pub fn cancel(&self,id:Uuid)->bool{self.control(id).inspect(|c|c.cancel()).is_some()}
    pub fn try_progress(&self)->Option<OperationProgress>{self.progress.try_recv().ok()}
}
fn progress(id:Uuid,kind:OperationKind,status:OperationStatus)->OperationProgress{OperationProgress{id,kind,status,current:None,completed_files:0,total_files:None,bytes_done:0,total_bytes:None,bytes_per_second:None,error:None}}
fn send(events:&Sender<OperationProgress>,request:&OperationRequest,status:OperationStatus,current:Option<PathBuf>,completed:usize,bytes:u64,speed:Option<f64>){let mut p=progress(request.id,request.kind,status);p.current=current;p.completed_files=completed;p.bytes_done=bytes;p.bytes_per_second=speed;let _=events.send(p);}
fn execute(request:&OperationRequest,control:&OperationControl,mut progress:impl FnMut(u64))->Result<(),OperationError>{
    match request.kind{
        OperationKind::Copy|OperationKind::Move=>{let destination_is_dir=request.destination.is_dir()||request.sources.len()>1;let mut cumulative=0;for source in &request.sources{control.checkpoint()?;let destination=if destination_is_dir{request.destination.join(source.file_name().ok_or_else(||OperationError::Invalid("source has no filename".into()))?)}else{request.destination.clone()};let before=cumulative;if request.kind==OperationKind::Copy{copy_path(source,&destination,request.conflict,control,|n|progress(before+n))?;}else{move_path(source,&destination,request.conflict,control,|n|progress(before+n))?;}cumulative+=fs_size(source).unwrap_or(0);}Ok(())},
        OperationKind::Delete=>{for source in &request.sources{control.checkpoint()?;}if request.destination==Path::new("$permanent"){for source in &request.sources{delete_permanent(source,true)?;}Ok(())}else{recycle(&request.sources)}},
        _=>Err(OperationError::Invalid("this request kind must use its dedicated engine".into())),
    }
}
fn fs_size(path:&Path)->Option<u64>{let m=std::fs::symlink_metadata(path).ok()?;if m.is_file(){Some(m.len())}else if m.is_dir()&&!m.file_type().is_symlink(){std::fs::read_dir(path).ok()?.filter_map(Result::ok).map(|e|fs_size(&e.path()).unwrap_or(0)).sum::<u64>().into()}else{Some(0)}}
trait SumOption{fn into(self)->Option<u64>;}impl SumOption for u64{fn into(self)->Option<u64>{Some(self)}}
