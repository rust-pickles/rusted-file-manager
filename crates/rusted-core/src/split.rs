use crate::Tab;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use uuid::Uuid;

pub type PaneId = Uuid;
pub type TabId = Uuid;

/// No pane is ever laid out smaller than this fraction of its parent split.
pub const MIN_PANE_RATIO: f32 = 0.12;
/// Hard ceiling on simultaneously visible panes; keeps layout arithmetic sane.
pub const MAX_PANES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    /// Pane split into left / right halves.
    Horizontal,
    /// Pane split into top / bottom halves.
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropZone {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

impl DropZone {
    pub const ALL: [Self; 5] = [Self::Center, Self::Left, Self::Right, Self::Top, Self::Bottom];

    pub fn label(self) -> &'static str {
        match self {
            Self::Center => "Move into pane",
            Self::Left => "Split left",
            Self::Right => "Split right",
            Self::Top => "Split above",
            Self::Bottom => "Split below",
        }
    }

    pub fn splits(self) -> bool {
        self != Self::Center
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    pub id: PaneId,
    pub tabs: Vec<Tab>,
    pub active: usize,
}

impl Pane {
    pub fn new(tab: Tab) -> Self {
        Self { id: Uuid::new_v4(), tabs: vec![tab], active: 0 }
    }

    pub fn empty() -> Self {
        Self { id: Uuid::new_v4(), tabs: Vec::new(), active: 0 }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    /// Inserts directly after the active tab so a new tab appears where expected.
    pub fn add_tab(&mut self, tab: Tab) {
        let at = (self.active + 1).min(self.tabs.len());
        self.tabs.insert(at, tab);
        self.active = at;
    }

    pub fn remove_tab(&mut self, id: TabId) -> Option<Tab> {
        let index = self.tabs.iter().position(|tab| tab.id == id)?;
        let tab = self.tabs.remove(index);
        self.active = self.active.min(self.tabs.len().saturating_sub(1));
        Some(tab)
    }

    pub fn close_tab(&mut self, id: TabId) -> Option<Tab> {
        self.remove_tab(id)
    }

    pub fn activate(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active = index;
            true
        } else {
            false
        }
    }

    pub fn next_tab(&mut self, reverse: bool) {
        if self.tabs.is_empty() {
            return;
        }
        let len = self.tabs.len();
        self.active = if reverse { self.active.checked_sub(1).unwrap_or(len - 1) } else { (self.active + 1) % len };
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitNode {
    Leaf(Pane),
    Split { axis: Axis, ratio: f32, first: Box<SplitNode>, second: Box<SplitNode> },
}

impl SplitNode {
    pub fn new(tab: Tab) -> Self {
        Self::Leaf(Pane::new(tab))
    }

    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        match self {
            Self::Leaf(pane) if pane.id == id => Some(pane),
            Self::Leaf(_) => None,
            Self::Split { first, second, .. } => first.pane(id).or_else(|| second.pane(id)),
        }
    }

    pub fn pane_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        match self {
            Self::Leaf(pane) if pane.id == id => Some(pane),
            Self::Leaf(_) => None,
            Self::Split { first, second, .. } => first.pane_mut(id).or_else(|| second.pane_mut(id)),
        }
    }

    /// Pane traversal order is left-to-right / top-to-bottom; focus cycling uses it.
    pub fn panes(&self) -> Vec<&Pane> {
        match self {
            Self::Leaf(pane) => vec![pane],
            Self::Split { first, second, .. } => {
                let mut panes = first.panes();
                panes.extend(second.panes());
                panes
            }
        }
    }

    pub fn panes_mut(&mut self) -> Vec<&mut Pane> {
        match self {
            Self::Leaf(pane) => vec![pane],
            Self::Split { first, second, .. } => {
                let mut panes = first.panes_mut();
                panes.extend(second.panes_mut());
                panes
            }
        }
    }

    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.panes().into_iter().map(|pane| pane.id).collect()
    }

    pub fn pane_count(&self) -> usize {
        self.panes().len()
    }

    pub fn contains_tab(&self, id: TabId) -> bool {
        self.panes().iter().any(|pane| pane.tabs.iter().any(|tab| tab.id == id))
    }

    /// Removes a tab from wherever it lives and reports the pane it came from.
    pub fn take_tab(&mut self, id: TabId) -> Option<(Tab, PaneId)> {
        match self {
            Self::Leaf(pane) => pane.remove_tab(id).map(|tab| (tab, pane.id)),
            Self::Split { first, second, .. } => first.take_tab(id).or_else(|| second.take_tab(id)),
        }
    }

    /// Applies a five-zone drop. `result` carries the destination pane id.
    pub fn drop_tab(&mut self, tab: Tab, target: PaneId, zone: DropZone) -> Result<PaneId, SplitError> {
        split_leaf(self, target, tab, zone)
    }

    /// Removes empty panes. Returns whether any leaf survives; the caller must not
    /// leave the tree empty, so a fully pruned root keeps one placeholder pane.
    pub fn prune(&mut self) {
        let taken = std::mem::replace(self, Self::Leaf(Pane::empty()));
        *self = prune_node(taken).unwrap_or_else(|| Self::Leaf(Pane::empty()));
    }

    /// Sets the ratio of the innermost split that separates `first` and `second`.
    pub fn resize(&mut self, first: PaneId, second: PaneId, ratio: f32) -> bool {
        match self {
            Self::Leaf(_) => false,
            Self::Split { ratio: current, first: a, second: b, .. } => {
                let in_a = a.pane(first).is_some() && a.pane(second).is_some();
                let in_b = b.pane(first).is_some() && b.pane(second).is_some();
                if in_a || in_b {
                    *current = ratio.clamp(MIN_PANE_RATIO, 1.0 - MIN_PANE_RATIO);
                    true
                } else {
                    a.resize(first, second, ratio) || b.resize(first, second, ratio)
                }
            }
        }
    }

    /// Resets the innermost split separating the two panes to an even ratio.
    pub fn reset_ratio(&mut self, first: PaneId, second: PaneId) -> bool {
        self.resize(first, second, 0.5)
    }

    /// Removes a pane and moves its tabs into the nearest surviving pane.
    /// Returns the tabs that were relocated, or `LastPane` when nothing would remain.
    pub fn close_pane_merge(&mut self, id: PaneId) -> Result<Vec<Tab>, SplitError> {
        if self.pane(id).is_none() {
            return Err(SplitError::PaneNotFound(id));
        }
        if self.pane_count() <= 1 {
            return Err(SplitError::LastPane);
        }
        let taken = std::mem::replace(self, Self::Leaf(Pane::empty()));
        let (mut tree, removed) = remove_leaf(taken, id);
        let Some(mut pane) = removed else {
            *self = tree;
            return Err(SplitError::PaneNotFound(id));
        };
        let tabs = std::mem::take(&mut pane.tabs);
        // Prefer a sibling, otherwise the first surviving pane in layout order.
        let target = tree.panes().first().map(|p| p.id).ok_or(SplitError::LastPane)?;
        if let Some(destination) = tree.pane_mut(target) {
            for tab in &tabs {
                destination.tabs.push(tab.clone());
            }
            destination.active = destination.tabs.len().saturating_sub(1);
        }
        *self = tree;
        Ok(tabs)
    }

    /// Total tab count; used by layout limits and status reporting.
    pub fn tab_count(&self) -> usize {
        self.panes().iter().map(|pane| pane.tabs.len()).sum()
    }
}

/// Replaces the target leaf with a split built by `build`. Unrelated subtrees are
/// preserved verbatim; the boolean reports whether the target was found.
fn wrap_target(node: SplitNode, target: PaneId, build: &mut dyn FnMut(SplitNode) -> SplitNode) -> (SplitNode, bool) {
    match node {
        SplitNode::Leaf(pane) => {
            let found = pane.id == target;
            let leaf = SplitNode::Leaf(pane);
            if found { (build(leaf), true) } else { (leaf, false) }
        }
        SplitNode::Split { axis, ratio, first, second } => {
            let (first, found) = wrap_target(*first, target, build);
            if found {
                return (SplitNode::Split { axis, ratio, first: Box::new(first), second }, true);
            }
            let (second, found) = wrap_target(*second, target, build);
            (SplitNode::Split { axis, ratio, first: Box::new(first), second: Box::new(second) }, found)
        }
    }
}

/// Pure layout transformation used by both pointer drops and context-menu actions.
pub fn split_leaf(node: &mut SplitNode, target: PaneId, incoming: Tab, zone: DropZone) -> Result<PaneId, SplitError> {
    if zone == DropZone::Center {
        let pane = node.pane_mut(target).ok_or(SplitError::PaneNotFound(target))?;
        let id = pane.id;
        pane.add_tab(incoming);
        return Ok(id);
    }
    if node.pane(target).is_none() {
        return Err(SplitError::PaneNotFound(target));
    }
    if node.pane_count() >= MAX_PANES {
        return Err(SplitError::LayoutLimit(MAX_PANES));
    }
    let new_pane = Pane::new(incoming);
    let destination = new_pane.id;
    let mut pending = Some(new_pane);
    let mut build = |old: SplitNode| -> SplitNode {
        let pane = pending.take().expect("split builder runs once");
        let new_leaf = SplitNode::Leaf(pane);
        let axis = match zone {
            DropZone::Left | DropZone::Right => Axis::Horizontal,
            _ => Axis::Vertical,
        };
        let (first, second) = match zone {
            DropZone::Left | DropZone::Top => (new_leaf, old),
            _ => (old, new_leaf),
        };
        SplitNode::Split { axis, ratio: 0.5, first: Box::new(first), second: Box::new(second) }
    };
    let taken = std::mem::replace(node, SplitNode::Leaf(Pane::empty()));
    let (rebuilt, found) = wrap_target(taken, target, &mut build);
    *node = rebuilt;
    if found {
        Ok(destination)
    } else {
        Err(SplitError::PaneNotFound(target))
    }
}

fn prune_node(node: SplitNode) -> Option<SplitNode> {
    match node {
        SplitNode::Leaf(pane) => (!pane.tabs.is_empty()).then_some(SplitNode::Leaf(pane)),
        SplitNode::Split { axis, ratio, first, second } => {
            match (prune_node(*first), prune_node(*second)) {
                (Some(first), Some(second)) => {
                    Some(SplitNode::Split { axis, ratio, first: Box::new(first), second: Box::new(second) })
                }
                (Some(only), None) | (None, Some(only)) => Some(only),
                (None, None) => None,
            }
        }
    }
}

fn remove_leaf(node: SplitNode, id: PaneId) -> (SplitNode, Option<Pane>) {
    match node {
        SplitNode::Leaf(pane) => {
            if pane.id == id {
                (SplitNode::Leaf(Pane::empty()), Some(pane))
            } else {
                (SplitNode::Leaf(pane), None)
            }
        }
        SplitNode::Split { axis, ratio, first, second } => {
            let (first, removed) = remove_leaf(*first, id);
            if removed.is_some() {
                return (SplitNode::Split { axis, ratio, first: Box::new(first), second }, removed);
            }
            let (second, removed) = remove_leaf(*second, id);
            let tree = SplitNode::Split { axis, ratio, first: Box::new(first), second: Box::new(second) };
            match removed {
                Some(pane) => (tree, Some(pane)),
                None => (tree, None),
            }
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SplitError {
    #[error("pane {0} was not found")]
    PaneNotFound(PaneId),
    #[error("cannot close the last pane")]
    LastPane,
    #[error("layout limit of {0} panes reached")]
    LayoutLimit(usize),
    #[error("invalid split layout")]
    InvalidLayout,
}

/// Tab id to owning pane, used by drag/drop hit-testing and persistence checks.
pub fn tab_locations(node: &SplitNode) -> BTreeMap<TabId, PaneId> {
    node.panes().into_iter().flat_map(|pane| pane.tabs.iter().map(move |tab| (tab.id, pane.id))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ViewMode;
    use std::path::PathBuf;

    fn tab(name: &str) -> Tab {
        Tab::new(PathBuf::from(name), ViewMode::Details)
    }

    #[test]
    fn center_drop_adds_to_target_group() {
        let mut node = SplitNode::new(tab("a"));
        let pane = node.pane_ids()[0];
        let added = tab("b");
        let added_id = added.id;
        assert_eq!(node.drop_tab(added, pane, DropZone::Center).unwrap(), pane);
        assert_eq!(node.pane(pane).unwrap().tabs.len(), 2);
        assert_eq!(node.pane(pane).unwrap().active_tab().unwrap().id, added_id);
    }

    #[test]
    fn each_zone_creates_a_split_on_the_requested_side() {
        for zone in [DropZone::Left, DropZone::Right, DropZone::Top, DropZone::Bottom] {
            let mut node = SplitNode::new(tab("old"));
            let original = node.pane_ids()[0];
            let destination = node.drop_tab(tab("new"), original, zone).unwrap();
            assert_eq!(node.pane_count(), 2, "{zone:?}");
            match &node {
                SplitNode::Split { axis, first, second, .. } => {
                    let expected_axis = if matches!(zone, DropZone::Left | DropZone::Right) {
                        Axis::Horizontal
                    } else {
                        Axis::Vertical
                    };
                    assert_eq!(*axis, expected_axis);
                    let first_has_new = first.pane(destination).is_some();
                    let expect_first = matches!(zone, DropZone::Left | DropZone::Top);
                    assert_eq!(first_has_new, expect_first, "new pane placed on the wrong side for {zone:?}");
                    assert!(second.pane(if first_has_new { original } else { destination }).is_some());
                }
                SplitNode::Leaf(_) => panic!("expected a split"),
            }
        }
    }

    #[test]
    fn nested_split_only_touches_its_target_subtree() {
        let mut node = SplitNode::new(tab("a"));
        let a = node.pane_ids()[0];
        let b = node.drop_tab(tab("b"), a, DropZone::Right).unwrap();
        let c = node.drop_tab(tab("c"), b, DropZone::Bottom).unwrap();
        assert_eq!(node.pane_count(), 3);
        assert!(node.pane(a).is_some() && node.pane(b).is_some() && node.pane(c).is_some());
        let d = node.drop_tab(tab("d"), a, DropZone::Top).unwrap();
        assert_eq!(node.pane_count(), 4);
        assert!(node.pane(d).is_some());
        // The untouched nested split under b must still hold exactly c.
        assert_eq!(node.pane(b).unwrap().tabs.len(), 1);
        assert_eq!(node.pane(c).unwrap().tabs.len(), 1);
    }

    #[test]
    fn prune_drops_empty_leaves_and_keeps_the_tree_connected() {
        let mut node = SplitNode::new(tab("a"));
        let a = node.pane_ids()[0];
        let b = node.drop_tab(tab("b"), a, DropZone::Right).unwrap();
        if let Some(pane) = node.pane_mut(b) {
            let id = pane.tabs[0].id;
            pane.remove_tab(id);
        }
        node.prune();
        assert_eq!(node.pane_count(), 1);
        assert_eq!(node.panes()[0].id, a);
    }

    #[test]
    fn resize_is_clamped_and_reset() {
        let mut node = SplitNode::new(tab("a"));
        let a = node.pane_ids()[0];
        let b = node.drop_tab(tab("b"), a, DropZone::Right).unwrap();
        assert!(node.resize(a, b, 2.0));
        match &node {
            SplitNode::Split { ratio, .. } => assert!((*ratio - (1.0 - MIN_PANE_RATIO)).abs() < f32::EPSILON),
            _ => panic!("expected split"),
        }
        assert!(node.reset_ratio(a, b));
        match &node {
            SplitNode::Split { ratio, .. } => assert!((*ratio - 0.5).abs() < f32::EPSILON),
            _ => panic!("expected split"),
        }
    }

    #[test]
    fn close_pane_merge_relocates_tabs_and_rejects_last_pane() {
        let mut node = SplitNode::new(tab("a"));
        let a = node.pane_ids()[0];
        let b = node.drop_tab(tab("b"), a, DropZone::Bottom).unwrap();
        let moved = node.close_pane_merge(b).unwrap();
        assert_eq!(moved.len(), 1);
        assert_eq!(node.pane_count(), 1);
        assert_eq!(node.panes()[0].tabs.len(), 2);
        assert_eq!(node.close_pane_merge(a), Err(SplitError::LastPane));
    }

    #[test]
    fn pane_limit_is_enforced() {
        let mut node = SplitNode::new(tab("a"));
        let mut target = node.pane_ids()[0];
        for index in 0..MAX_PANES - 1 {
            target = node.drop_tab(tab(&format!("t{index}")), target, DropZone::Right).unwrap();
        }
        assert_eq!(node.pane_count(), MAX_PANES);
        assert!(matches!(
            node.drop_tab(tab("overflow"), target, DropZone::Left),
            Err(SplitError::LayoutLimit(MAX_PANES))
        ));
    }
}
