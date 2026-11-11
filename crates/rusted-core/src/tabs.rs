use crate::{NavigationHistory, SearchState, Selection, SortSpec, ViewMode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// One browsing session. Everything that should survive a tab switch lives here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: Uuid,
    pub custom_title: Option<String>,
    pub history: NavigationHistory,
    pub selection: Selection,
    pub view: ViewMode,
    pub sort: SortSpec,
    pub scroll: f32,
    pub search: SearchState,
    pub preview_visible: bool,
    pub inspector_visible: bool,
}

impl Tab {
    pub fn new(path: PathBuf, view: ViewMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            custom_title: None,
            history: NavigationHistory::new(path),
            selection: Selection::default(),
            view,
            sort: SortSpec::default(),
            scroll: 0.0,
            search: SearchState::default(),
            preview_visible: false,
            inspector_visible: false,
        }
    }

    pub fn path(&self) -> &Path {
        self.history.current()
    }

    pub fn title(&self) -> String {
        self.custom_title.clone().unwrap_or_else(|| {
            self.path()
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| self.path().display().to_string())
        })
    }

    fn reset_for_new_location(&mut self) {
        self.selection.clear();
        self.scroll = 0.0;
        self.search = SearchState::default();
    }

    pub fn navigate(&mut self, path: PathBuf) {
        self.history.visit(path);
        self.reset_for_new_location();
    }

    pub fn back(&mut self) -> Option<PathBuf> {
        let path = self.history.back().map(Path::to_path_buf);
        if path.is_some() {
            self.reset_for_new_location();
        }
        path
    }

    pub fn forward(&mut self) -> Option<PathBuf> {
        let path = self.history.forward().map(Path::to_path_buf);
        if path.is_some() {
            self.reset_for_new_location();
        }
        path
    }

    /// Duplicates the location, view and sort but not transient selection.
    pub fn duplicate(&self) -> Self {
        let mut tab = Self::new(self.path().to_path_buf(), self.view);
        tab.sort = self.sort;
        tab.custom_title = self.custom_title.clone();
        tab.preview_visible = self.preview_visible;
        tab.inspector_visible = self.inspector_visible;
        tab
    }

    pub fn set_view(&mut self, view: ViewMode) {
        self.view = view;
        self.scroll = 0.0;
    }

    pub fn set_sort(&mut self, key: crate::SortKey) {
        self.sort = self.sort.toggled(key);
        self.scroll = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SortKey;

    #[test]
    fn navigation_clears_transient_state_but_keeps_view_and_sort() {
        let mut tab = Tab::new(PathBuf::from("root"), ViewMode::Grid);
        tab.set_sort(SortKey::Size);
        tab.selection.select_only(PathBuf::from("root/file"));
        tab.scroll = 100.0;
        tab.navigate(PathBuf::from("root/child"));
        assert!(tab.selection.is_empty());
        assert_eq!(tab.scroll, 0.0);
        assert_eq!(tab.view, ViewMode::Grid);
        assert_eq!(tab.sort.key, SortKey::Size);
        assert_eq!(tab.back().as_deref(), Some(Path::new("root")));
        assert_eq!(tab.forward().as_deref(), Some(Path::new("root/child")));
    }

    #[test]
    fn duplicate_keeps_location_and_sort_but_gets_a_new_identity() {
        let mut tab = Tab::new(PathBuf::from("root"), ViewMode::List);
        tab.custom_title = Some("Docs".into());
        let copy = tab.duplicate();
        assert_ne!(copy.id, tab.id);
        assert_eq!(copy.path(), tab.path());
        assert_eq!(copy.custom_title.as_deref(), Some("Docs"));
        assert_eq!(copy.view, ViewMode::List);
    }

    #[test]
    fn title_falls_back_to_drive_or_root() {
        let tab = Tab::new(PathBuf::from("C:\\"), ViewMode::Details);
        assert_eq!(tab.title(), "C:\\");
        let tab = Tab::new(PathBuf::from("/home/user"), ViewMode::Details);
        assert_eq!(tab.title(), "user");
    }
}
