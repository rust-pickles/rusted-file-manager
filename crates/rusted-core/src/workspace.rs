use crate::{DropZone, Pane, PaneId, SplitNode, SplitError, Tab, TabId, ViewMode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Upper bound on visible panes; the flat split tree is bounded by the same number.
pub const MAX_VISIBLE_PANES: usize = crate::split::MAX_PANES;
const RECENTLY_CLOSED_LIMIT: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub root: SplitNode,
    pub active_pane: PaneId,
    /// Used to reopen a tab when the user closes the very last one, so the shell
    /// always has a directory to render.
    #[serde(default)]
    pub default_location: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    recently_closed: Vec<Tab>,
}

impl Workspace {
    pub fn new(initial: PathBuf, view: ViewMode) -> Self {
        let root = SplitNode::new(Tab::new(initial.clone(), view));
        let active_pane = root.panes()[0].id;
        Self { root, active_pane, default_location: initial, recently_closed: Vec::new() }
    }

    /// Rebuilds a workspace from restored panes, dropping empty ones.
    pub fn from_root(root: SplitNode) -> Self {
        let mut root = root;
        root.prune();
        let active_pane = root.panes()[0].id;
        let default_location = root.panes()[0].tabs.first().map(|tab| tab.path().to_path_buf()).unwrap_or_default();
        Self { root, active_pane, default_location, recently_closed: Vec::new() }
    }

    /// Ensures the active pane still exists and that every pane holds a tab.
    pub fn normalize(&mut self) {
        self.root.prune();
        self.ensure_usable();
        if self.root.pane(self.active_pane).is_none() {
            self.active_pane = self.root.panes()[0].id;
        }
    }

    /// Guarantees one non-empty pane by reopening the default location.
    fn ensure_usable(&mut self) {
        if self.root.tab_count() > 0 {
            return;
        }
        let fallback = if self.default_location.as_os_str().is_empty() {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        } else {
            self.default_location.clone()
        };
        self.root = SplitNode::new(Tab::new(fallback, ViewMode::Details));
        self.active_pane = self.root.panes()[0].id;
    }

    pub fn active_pane(&self) -> &Pane {
        self.root.pane(self.active_pane).expect("active pane invariant")
    }

    pub fn active_pane_mut(&mut self) -> &mut Pane {
        self.root.pane_mut(self.active_pane).expect("active pane invariant")
    }

    pub fn active_tab(&self) -> &Tab {
        self.active_pane().active_tab().expect("non-empty pane invariant")
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        self.active_pane_mut().active_tab_mut().expect("non-empty pane invariant")
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.active_pane().tabs
    }

    pub fn focus_pane(&mut self, id: PaneId) -> bool {
        if self.root.pane(id).is_some() {
            self.active_pane = id;
            true
        } else {
            false
        }
    }

    pub fn cycle_pane(&mut self) {
        let ids = self.root.pane_ids();
        if ids.is_empty() {
            return;
        }
        let index = ids.iter().position(|id| *id == self.active_pane).unwrap_or(0);
        self.active_pane = ids[(index + 1) % ids.len()];
    }

    pub fn new_tab(&mut self, path: PathBuf, view: ViewMode) -> TabId {
        let tab = Tab::new(path, view);
        let id = tab.id;
        self.active_pane_mut().add_tab(tab);
        id
    }

    pub fn duplicate_tab(&mut self) -> Option<TabId> {
        let tab = self.active_tab().duplicate();
        let id = tab.id;
        self.active_pane_mut().add_tab(tab);
        Some(id)
    }

    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.root.panes().into_iter().find_map(|pane| pane.tabs.iter().find(|tab| tab.id == id))
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        let pane = self.root.panes_mut().into_iter().find(|pane| pane.tabs.iter().any(|tab| tab.id == id))?;
        pane.tabs.iter_mut().find(|tab| tab.id == id)
    }

    /// Closes a tab. The owning pane is removed when its last tab goes and other
    /// panes exist, so the workspace never keeps an unusable empty pane.
    pub fn close_tab(&mut self, id: TabId) -> bool {
        let owner = self
            .root
            .panes()
            .into_iter()
            .find(|pane| pane.tabs.iter().any(|tab| tab.id == id))
            .map(|pane| pane.id);
        let Some(owner) = owner else { return false };
        let will_be_empty = self.root.pane(owner).map(|pane| pane.tabs.len() == 1).unwrap_or(false);
        let Some((tab, _)) = self.root.take_tab(id) else { return false };
        self.remember_closed(tab);
        if will_be_empty && self.root.pane_count() > 1 {
            self.root.prune();
        }
        self.normalize();
        true
    }

    /// Closes the active tab, reporting whether another pane absorbed the focus.
    pub fn close_active_tab(&mut self) -> bool {
        let id = self.active_tab().id;
        self.close_tab(id)
    }

    pub fn reopen_closed(&mut self) -> Option<TabId> {
        let tab = self.recently_closed.pop()?;
        let id = tab.id;
        self.active_pane_mut().add_tab(tab);
        Some(id)
    }

    pub fn recently_closed(&self) -> &[Tab] {
        &self.recently_closed
    }

    fn remember_closed(&mut self, tab: Tab) {
        self.recently_closed.push(tab);
        if self.recently_closed.len() > RECENTLY_CLOSED_LIMIT {
            self.recently_closed.remove(0);
        }
    }

    pub fn activate_tab_index(&mut self, index: usize) -> bool {
        self.active_pane_mut().activate(index)
    }

    pub fn activate_tab(&mut self, id: TabId) -> bool {
        let Some(pane) = self.root.panes_mut().into_iter().find(|pane| pane.tabs.iter().any(|tab| tab.id == id)) else {
            return false;
        };
        let index = pane.tabs.iter().position(|tab| tab.id == id).expect("found");
        pane.active = index;
        let pane_id = pane.id;
        self.active_pane = pane_id;
        true
    }

    /// Moves or splits an existing tab. Empty panes left behind are pruned, and
    /// focus follows the destination so the drag feels immediate.
    pub fn move_or_split_tab(&mut self, tab_id: TabId, target: PaneId, zone: DropZone) -> Result<PaneId, SplitError> {
        let source = self
            .root
            .panes()
            .into_iter()
            .find(|pane| pane.tabs.iter().any(|tab| tab.id == tab_id))
            .map(|pane| pane.id)
            .ok_or(SplitError::PaneNotFound(target))?;
        if zone == DropZone::Center {
            if source == target {
                let pane = self.root.pane_mut(target).ok_or(SplitError::PaneNotFound(target))?;
                let index = pane.tabs.iter().position(|tab| tab.id == tab_id).ok_or(SplitError::InvalidLayout)?;
                pane.active = index;
                self.active_pane = target;
                return Ok(target);
            }
            if self.root.pane(target).is_none() {
                return Err(SplitError::PaneNotFound(target));
            }
            let (mut tab, _) = self.root.take_tab(tab_id).ok_or(SplitError::InvalidLayout)?;
            tab.scroll = tab.scroll.max(0.0);
            let destination = self.root.drop_tab(tab, target, zone)?;
            self.root.prune();
            self.normalize();
            self.active_pane = destination;
            return Ok(destination);
        }
        if self.root.pane_count() >= MAX_VISIBLE_PANES {
            return Err(SplitError::LayoutLimit(MAX_VISIBLE_PANES));
        }
        let (tab, _) = self.root.take_tab(tab_id).ok_or(SplitError::InvalidLayout)?;
        let destination = self.root.drop_tab(tab, target, zone)?;
        self.root.prune();
        self.normalize();
        self.active_pane = destination;
        Ok(destination)
    }

    pub fn split_active(&mut self, zone: DropZone) -> Result<PaneId, SplitError> {
        if !zone.splits() {
            return Ok(self.active_pane);
        }
        let target = self.active_pane;
        let duplicate = self.active_tab().duplicate();
        if self.root.pane_count() >= MAX_VISIBLE_PANES {
            return Err(SplitError::LayoutLimit(MAX_VISIBLE_PANES));
        }
        self.root.drop_tab(duplicate, target, zone)
    }

    pub fn close_pane_merge(&mut self, id: PaneId) -> Result<Vec<Tab>, SplitError> {
        let merged = self.root.close_pane_merge(id)?;
        self.normalize();
        Ok(merged)
    }

    pub fn resize(&mut self, first: PaneId, second: PaneId, ratio: f32) -> bool {
        self.root.resize(first, second, ratio)
    }

    pub fn rename_tab(&mut self, id: TabId, title: Option<String>) -> bool {
        match self.tab_mut(id) {
            Some(tab) => {
                tab.custom_title = title.filter(|value| !value.trim().is_empty());
                true
            }
            None => false,
        }
    }

    pub fn active_directory(&self) -> &Path {
        self.active_tab().path()
    }

    pub fn pane_tabs(&self) -> Vec<(PaneId, usize)> {
        self.root.panes().into_iter().map(|pane| (pane.id, pane.tabs.len())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn tab(name: &str) -> Tab {
        Tab::new(PathBuf::from(name), ViewMode::Details)
    }

    #[test]
    fn five_zone_drop_builds_nested_layout_and_preserves_tabs() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let first = workspace.active_tab().id;
        workspace.new_tab("b".into(), ViewMode::Grid);
        let second = workspace.active_tab().id;
        let destination = workspace.move_or_split_tab(second, workspace.active_pane, DropZone::Right).unwrap();
        assert_eq!(workspace.root.pane_count(), 2);
        assert!(workspace.tab(first).is_some());
        workspace.focus_pane(destination);
        let third = workspace.new_tab("c".into(), ViewMode::List);
        workspace.move_or_split_tab(third, destination, DropZone::Bottom).unwrap();
        assert_eq!(workspace.root.pane_count(), 3);
        assert!(workspace.root.panes().iter().flat_map(|pane| &pane.tabs).any(|tab| tab.path() == Path::new("a")));
        workspace.move_or_split_tab(third, destination, DropZone::Center).unwrap();
        assert_eq!(workspace.root.pane_count(), 2);
    }

    #[test]
    fn moving_last_tab_prunes_empty_source() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let id = workspace.active_tab().id;
        let destination = workspace.move_or_split_tab(id, workspace.active_pane, DropZone::Left).unwrap();
        assert_eq!(workspace.root.pane_count(), 2);
        let other = workspace.active_tab().id;
        workspace.move_or_split_tab(other, destination, DropZone::Center).unwrap();
        assert_eq!(workspace.root.pane_count(), 1);
        assert_eq!(workspace.active_pane().tabs.len(), 2);
    }

    #[test]
    fn close_pane_merges_all_tabs_into_neighbour() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let id = workspace.active_tab().id;
        let destination = workspace.move_or_split_tab(id, workspace.active_pane, DropZone::Right).unwrap();
        workspace.focus_pane(destination);
        workspace.new_tab("b".into(), ViewMode::Details);
        let merged = workspace.close_pane_merge(destination).unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(workspace.root.pane_count(), 1);
        assert_eq!(workspace.active_pane().tabs.len(), 2);
    }

    #[test]
    fn resize_is_bounded() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let original = workspace.active_pane;
        let id = workspace.active_tab().id;
        let destination = workspace.move_or_split_tab(id, original, DropZone::Right).unwrap();
        assert!(workspace.resize(original, destination, 2.0));
        match workspace.root {
            SplitNode::Split { ratio, .. } => {
                assert!((ratio - (1.0 - crate::MIN_PANE_RATIO)).abs() < f32::EPSILON)
            }
            _ => panic!("expected split"),
        }
    }

    #[test]
    fn closed_tabs_are_restored_with_state() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let id = workspace.active_tab().id;
        workspace.active_tab_mut().scroll = 42.0;
        assert!(workspace.close_tab(id));
        assert_eq!(workspace.reopen_closed().unwrap(), id);
        assert_eq!(workspace.active_tab().scroll, 42.0);
    }

    #[test]
    fn closing_the_last_tab_reopens_the_default_location() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let id = workspace.active_tab().id;
        assert!(workspace.close_tab(id));
        assert_eq!(workspace.active_pane().tabs.len(), 1);
        assert_eq!(workspace.active_tab().path(), Path::new("a"));
        assert_eq!(workspace.reopen_closed().unwrap(), id);
        assert_eq!(workspace.active_pane().tabs.len(), 2);
    }

    #[test]
    fn tab_focus_cycles_in_layout_order() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let first = workspace.active_pane;
        let id = workspace.active_tab().id;
        let second = workspace.move_or_split_tab(id, first, DropZone::Right).unwrap();
        workspace.focus_pane(first);
        workspace.cycle_pane();
        assert_eq!(workspace.active_pane, second);
        workspace.cycle_pane();
        assert_eq!(workspace.active_pane, first);
    }

    #[test]
    fn duplicate_and_rename_tab() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Grid);
        let copy = workspace.duplicate_tab().unwrap();
        assert_ne!(copy, workspace.active_tab().id);
        assert!(workspace.rename_tab(copy, Some("Pinned".into())));
        assert_eq!(workspace.tab(copy).unwrap().title(), "Pinned");
        assert!(workspace.rename_tab(copy, None));
        assert_ne!(workspace.tab(copy).unwrap().title(), "Pinned");
    }

    #[test]
    fn workspace_survives_a_json_roundtrip() {
        let mut workspace = Workspace::new("a".into(), ViewMode::Details);
        let id = workspace.active_tab().id;
        workspace.move_or_split_tab(id, workspace.active_pane, DropZone::Top).unwrap();
        let json = serde_json::to_string(&workspace).unwrap();
        let restored: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.root.pane_count(), workspace.root.pane_count());
        assert_eq!(restored.active_pane, workspace.active_pane);
    }
}
