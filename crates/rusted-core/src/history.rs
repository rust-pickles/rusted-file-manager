use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Per-tab, bounded linear history. Navigating after Back discards the forward branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationHistory {
    current: PathBuf,
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
    capacity: usize,
}

impl NavigationHistory {
    pub fn new(initial: PathBuf) -> Self {
        Self { current: initial, back: Vec::new(), forward: Vec::new(), capacity: 256 }
    }

    pub fn current(&self) -> &Path {
        &self.current
    }

    pub fn can_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn can_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    pub fn back_depth(&self) -> usize {
        self.back.len()
    }

    pub fn forward_depth(&self) -> usize {
        self.forward.len()
    }

    pub fn visit(&mut self, path: PathBuf) {
        if path == self.current {
            return;
        }
        self.back.push(std::mem::replace(&mut self.current, path));
        if self.back.len() > self.capacity {
            self.back.remove(0);
        }
        self.forward.clear();
    }

    pub fn back(&mut self) -> Option<&Path> {
        let next = self.back.pop()?;
        self.forward.push(std::mem::replace(&mut self.current, next));
        if self.forward.len() > self.capacity {
            self.forward.remove(0);
        }
        Some(&self.current)
    }

    pub fn forward(&mut self) -> Option<&Path> {
        let next = self.forward.pop()?;
        self.back.push(std::mem::replace(&mut self.current, next));
        if self.back.len() > self.capacity {
            self.back.remove(0);
        }
        Some(&self.current)
    }

    /// Breadcrumb segments of the current location, as (label, full path).
    pub fn breadcrumbs(&self) -> Vec<(String, PathBuf)> {
        let mut segments = Vec::new();
        let mut accumulated = PathBuf::new();
        for component in self.current.components() {
            accumulated.push(component.as_os_str());
            let label = match component {
                std::path::Component::RootDir => accumulated.display().to_string(),
                std::path::Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned(),
                _ => component.as_os_str().to_string_lossy().into_owned(),
            };
            segments.push((label, accumulated.clone()));
        }
        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_discards_forward_branch_and_ignores_duplicates() {
        let mut history = NavigationHistory::new("a".into());
        history.visit("b".into());
        history.visit("c".into());
        assert_eq!(history.back(), Some(Path::new("b")));
        history.visit("d".into());
        assert!(!history.can_forward());
        assert_eq!(history.back(), Some(Path::new("b")));
        history.visit("b".into());
        assert_eq!(history.current(), Path::new("b"));
        assert_eq!(history.back(), Some(Path::new("a")));
        assert!(!history.can_back());
    }

    #[test]
    fn history_is_bounded() {
        let mut history = NavigationHistory::new("start".into());
        for index in 0..300 {
            history.visit(PathBuf::from(format!("dir{index}")));
        }
        assert!(history.back_depth() <= 256);
    }

    #[test]
    fn breadcrumbs_reconstruct_paths() {
        let history = NavigationHistory::new(PathBuf::from("/home/user/docs"));
        let crumbs = history.breadcrumbs();
        assert_eq!(crumbs.len(), 4);
        assert_eq!(crumbs.last().unwrap().0, "docs");
        assert_eq!(crumbs.last().unwrap().1, PathBuf::from("/home/user/docs"));
    }
}
