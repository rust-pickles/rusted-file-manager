//! Single source of truth for every user-triggerable action. The command
//! palette, keyboard router and settings screen all read this table, so a
//! command cannot appear in the UI without being dispatchable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Tabs
    NewTab,
    CloseTab,
    ReopenClosedTab,
    NextTab,
    PreviousTab,
    DuplicateTab,
    RenameTab,
    MoveTabToPane,
    // Navigation
    Back,
    Forward,
    Up,
    GoTo,
    Refresh,
    FocusLocation,
    // Workspace
    SplitLeft,
    SplitRight,
    SplitAbove,
    SplitBelow,
    CyclePane,
    ClosePane,
    ReopenPane,
    // Views
    ViewGrid,
    ViewList,
    ViewDetails,
    ViewMiller,
    ViewTree,
    NextView,
    ToggleHidden,
    SortByName,
    SortBySize,
    SortByModified,
    SortByKind,
    ToggleFoldersFirst,
    // Selection
    SelectAll,
    ClearSelection,
    InvertSelection,
    SelectNext,
    SelectPrevious,
    // Files
    Open,
    OpenWithDefault,
    RevealInExplorer,
    Rename,
    BatchRename,
    NewFolder,
    NewFile,
    Copy,
    Cut,
    Paste,
    Duplicate,
    DeleteRecycle,
    DeletePermanent,
    Properties,
    CopyPath,
    CopyName,
    Undo,
    Redo,
    // Search
    SearchHere,
    SearchRecursive,
    SearchIndexed,
    CommandPalette,
    // Panels
    TogglePreview,
    ToggleInspector,
    ToggleSidebar,
    PreviewFocused,
    // Tools
    OpenSettings,
    OpenShortcuts,
    StorageAnalyzer,
    NewWindow,
    OpenArchive,
    CreateArchive,
    ExtractAll,
    AddToStarred,
    RemoveFromStarred,
    TagSelection,
    Quit,
}

impl Action {
    pub const ALL: &'static [Action] = &[
        Action::NewTab, Action::CloseTab, Action::ReopenClosedTab, Action::NextTab, Action::PreviousTab,
        Action::DuplicateTab, Action::RenameTab, Action::MoveTabToPane, Action::Back, Action::Forward,
        Action::Up, Action::GoTo, Action::Refresh, Action::FocusLocation, Action::SplitLeft,
        Action::SplitRight, Action::SplitAbove, Action::SplitBelow, Action::CyclePane, Action::ClosePane,
        Action::ReopenPane, Action::ViewGrid, Action::ViewList, Action::ViewDetails, Action::ViewMiller,
        Action::ViewTree, Action::NextView, Action::ToggleHidden, Action::SortByName, Action::SortBySize,
        Action::SortByModified, Action::SortByKind, Action::ToggleFoldersFirst, Action::SelectAll,
        Action::ClearSelection, Action::InvertSelection, Action::SelectNext, Action::SelectPrevious,
        Action::Open, Action::OpenWithDefault, Action::RevealInExplorer, Action::Rename,
        Action::BatchRename, Action::NewFolder, Action::NewFile, Action::Copy, Action::Cut, Action::Paste,
        Action::Duplicate, Action::DeleteRecycle, Action::DeletePermanent, Action::Properties,
        Action::CopyPath, Action::CopyName, Action::Undo, Action::Redo, Action::SearchHere,
        Action::SearchRecursive, Action::SearchIndexed, Action::CommandPalette, Action::TogglePreview,
        Action::ToggleInspector, Action::ToggleSidebar, Action::PreviewFocused, Action::OpenSettings,
        Action::OpenShortcuts, Action::StorageAnalyzer, Action::NewWindow, Action::OpenArchive,
        Action::CreateArchive, Action::ExtractAll, Action::AddToStarred, Action::RemoveFromStarred,
        Action::TagSelection, Action::Quit,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Action::NewTab => "tab.new", Action::CloseTab => "tab.close", Action::ReopenClosedTab => "tab.reopen",
            Action::NextTab => "tab.next", Action::PreviousTab => "tab.previous", Action::DuplicateTab => "tab.duplicate",
            Action::RenameTab => "tab.rename", Action::MoveTabToPane => "tab.move", Action::Back => "nav.back",
            Action::Forward => "nav.forward", Action::Up => "nav.up", Action::GoTo => "nav.goto",
            Action::Refresh => "nav.refresh", Action::FocusLocation => "nav.location",
            Action::SplitLeft => "split.left", Action::SplitRight => "split.right", Action::SplitAbove => "split.above",
            Action::SplitBelow => "split.below", Action::CyclePane => "pane.cycle", Action::ClosePane => "pane.close",
            Action::ReopenPane => "pane.reopen", Action::ViewGrid => "view.grid", Action::ViewList => "view.list",
            Action::ViewDetails => "view.details", Action::ViewMiller => "view.miller", Action::ViewTree => "view.tree",
            Action::NextView => "view.next", Action::ToggleHidden => "view.hidden", Action::SortByName => "sort.name",
            Action::SortBySize => "sort.size", Action::SortByModified => "sort.modified", Action::SortByKind => "sort.kind",
            Action::ToggleFoldersFirst => "sort.folders_first", Action::SelectAll => "selection.all",
            Action::ClearSelection => "selection.clear", Action::InvertSelection => "selection.invert",
            Action::SelectNext => "selection.next", Action::SelectPrevious => "selection.previous",
            Action::Open => "file.open", Action::OpenWithDefault => "file.open_default",
            Action::RevealInExplorer => "file.reveal", Action::Rename => "file.rename",
            Action::BatchRename => "file.batch_rename", Action::NewFolder => "file.new_folder",
            Action::NewFile => "file.new_file", Action::Copy => "file.copy", Action::Cut => "file.cut",
            Action::Paste => "file.paste", Action::Duplicate => "file.duplicate", Action::DeleteRecycle => "file.delete",
            Action::DeletePermanent => "file.delete_permanent", Action::Properties => "file.properties",
            Action::CopyPath => "file.copy_path", Action::CopyName => "file.copy_name", Action::Undo => "edit.undo",
            Action::Redo => "edit.redo", Action::SearchHere => "search.here", Action::SearchRecursive => "search.recursive",
            Action::SearchIndexed => "search.indexed", Action::CommandPalette => "app.palette",
            Action::TogglePreview => "panel.preview", Action::ToggleInspector => "panel.inspector",
            Action::ToggleSidebar => "panel.sidebar", Action::PreviewFocused => "panel.quick_preview",
            Action::OpenSettings => "app.settings", Action::OpenShortcuts => "app.shortcuts",
            Action::StorageAnalyzer => "tools.storage", Action::NewWindow => "app.new_window",
            Action::OpenArchive => "archive.open", Action::CreateArchive => "archive.create",
            Action::ExtractAll => "archive.extract_all", Action::AddToStarred => "star.add",
            Action::RemoveFromStarred => "star.remove", Action::TagSelection => "tag.selection", Action::Quit => "app.quit",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Action::NewTab => "New tab", Action::CloseTab => "Close tab", Action::ReopenClosedTab => "Reopen closed tab",
            Action::NextTab => "Next tab", Action::PreviousTab => "Previous tab", Action::DuplicateTab => "Duplicate tab",
            Action::RenameTab => "Rename tab…", Action::MoveTabToPane => "Move tab to pane…",
            Action::Back => "Back", Action::Forward => "Forward", Action::Up => "Go to parent folder",
            Action::GoTo => "Go to folder…", Action::Refresh => "Refresh", Action::FocusLocation => "Edit location",
            Action::SplitLeft => "Split pane left", Action::SplitRight => "Split pane right",
            Action::SplitAbove => "Split pane above", Action::SplitBelow => "Split pane below",
            Action::CyclePane => "Cycle panes", Action::ClosePane => "Close pane", Action::ReopenPane => "Reopen pane",
            Action::ViewGrid => "View: Grid", Action::ViewList => "View: List", Action::ViewDetails => "View: Details",
            Action::ViewMiller => "View: Miller columns", Action::ViewTree => "View: Tree", Action::NextView => "Cycle view mode",
            Action::ToggleHidden => "Toggle hidden files", Action::SortByName => "Sort by name",
            Action::SortBySize => "Sort by size", Action::SortByModified => "Sort by date modified",
            Action::SortByKind => "Sort by type", Action::ToggleFoldersFirst => "Toggle folders first",
            Action::SelectAll => "Select all", Action::ClearSelection => "Clear selection",
            Action::InvertSelection => "Invert selection", Action::SelectNext => "Select next item",
            Action::SelectPrevious => "Select previous item", Action::Open => "Open", Action::OpenWithDefault => "Open with default app",
            Action::RevealInExplorer => "Reveal in File Explorer", Action::Rename => "Rename…",
            Action::BatchRename => "Batch rename…", Action::NewFolder => "New folder", Action::NewFile => "New file",
            Action::Copy => "Copy", Action::Cut => "Cut", Action::Paste => "Paste", Action::Duplicate => "Duplicate",
            Action::DeleteRecycle => "Move to Recycle Bin", Action::DeletePermanent => "Delete permanently…",
            Action::Properties => "Properties", Action::CopyPath => "Copy full path", Action::CopyName => "Copy file name",
            Action::Undo => "Undo", Action::Redo => "Redo", Action::SearchHere => "Search this folder",
            Action::SearchRecursive => "Search subfolders", Action::SearchIndexed => "Search indexed locations",
            Action::CommandPalette => "Command palette", Action::TogglePreview => "Toggle preview panel",
            Action::ToggleInspector => "Toggle inspector", Action::ToggleSidebar => "Toggle sidebar",
            Action::PreviewFocused => "Quick preview", Action::OpenSettings => "Settings…",
            Action::OpenShortcuts => "Keyboard shortcuts…", Action::StorageAnalyzer => "Storage analyzer…",
            Action::NewWindow => "New window", Action::OpenArchive => "Open archive…",
            Action::CreateArchive => "Create archive from selection…", Action::ExtractAll => "Extract all…",
            Action::AddToStarred => "Add to starred", Action::RemoveFromStarred => "Remove from starred",
            Action::TagSelection => "Tag selection…", Action::Quit => "Quit RUSTED",
        }
    }

    pub fn category(self) -> &'static str {
        use Action::*;
        match self {
            NewTab | CloseTab | ReopenClosedTab | NextTab | PreviousTab | DuplicateTab | RenameTab
            | MoveTabToPane => "Tabs",
            Back | Forward | Up | GoTo | Refresh | FocusLocation => "Navigation",
            SplitLeft | SplitRight | SplitAbove | SplitBelow | CyclePane | ClosePane | ReopenPane => "Layout",
            ViewGrid | ViewList | ViewDetails | ViewMiller | ViewTree | NextView | ToggleHidden => "View",
            SortByName | SortBySize | SortByModified | SortByKind | ToggleFoldersFirst => "Sort",
            SelectAll | ClearSelection | InvertSelection | SelectNext | SelectPrevious => "Selection",
            Open | OpenWithDefault | RevealInExplorer | Rename | BatchRename | NewFolder | NewFile | Copy
            | Cut | Paste | Duplicate | DeleteRecycle | DeletePermanent | Properties | CopyPath | CopyName
            | Undo | Redo => "Files",
            SearchHere | SearchRecursive | SearchIndexed | CommandPalette => "Search",
            TogglePreview | ToggleInspector | ToggleSidebar | PreviewFocused => "Panels",
            OpenSettings | OpenShortcuts | StorageAnalyzer | NewWindow | OpenArchive | CreateArchive
            | ExtractAll | AddToStarred | RemoveFromStarred | TagSelection | Quit => "Application",
        }
    }

    /// Modifier bitmask shortcuts: `ctrl+shift+t`, `alt+left`, `F5`, `ctrl+1`.
    pub fn default_shortcut(self) -> &'static str {
        use Action::*;
        match self {
            NewTab => "ctrl+t", CloseTab => "ctrl+w", ReopenClosedTab => "ctrl+shift+t", NextTab => "ctrl+tab",
            PreviousTab => "ctrl+shift+tab", DuplicateTab => "ctrl+shift+k", RenameTab => "",
            MoveTabToPane => "", Back => "alt+left", Forward => "alt+right", Up => "alt+up", GoTo => "ctrl+k",
            Refresh => "f5", FocusLocation => "ctrl+l", SplitLeft => "ctrl+shift+left",
            SplitRight => "ctrl+shift+right", SplitAbove => "ctrl+shift+up", SplitBelow => "ctrl+shift+down",
            CyclePane => "f6", ClosePane => "ctrl+shift+w", ReopenPane => "", ViewGrid => "alt+1",
            ViewList => "alt+2", ViewDetails => "alt+3", ViewMiller => "alt+4", ViewTree => "alt+5",
            NextView => "ctrl+shift+v", ToggleHidden => "ctrl+h", SortByName => "", SortBySize => "",
            SortByModified => "", SortByKind => "", ToggleFoldersFirst => "", SelectAll => "ctrl+a",
            ClearSelection => "escape", InvertSelection => "ctrl+shift+i", SelectNext => "down",
            SelectPrevious => "up", Open => "enter", OpenWithDefault => "", RevealInExplorer => "ctrl+shift+e",
            Rename => "f2", BatchRename => "ctrl+shift+r", NewFolder => "ctrl+shift+n", NewFile => "",
            Copy => "ctrl+c", Cut => "ctrl+x", Paste => "ctrl+v", Duplicate => "",
            DeleteRecycle => "delete", DeletePermanent => "shift+delete", Properties => "alt+enter",
            CopyPath => "ctrl+shift+c", CopyName => "", Undo => "ctrl+z", Redo => "ctrl+shift+z",
            SearchHere => "ctrl+f", SearchRecursive => "ctrl+shift+f", SearchIndexed => "ctrl+shift+o",
            CommandPalette => "ctrl+shift+p", TogglePreview => "", ToggleInspector => "ctrl+i",
            ToggleSidebar => "ctrl+b", PreviewFocused => "space", OpenSettings => "ctrl+comma",
            OpenShortcuts => "ctrl+shift+slash", StorageAnalyzer => "", NewWindow => "ctrl+n",
            OpenArchive => "", CreateArchive => "", ExtractAll => "", AddToStarred => "",
            RemoveFromStarred => "", TagSelection => "", Quit => "ctrl+q",
        }
    }
}

/// Normalised keyboard chord used for matching, display and rebinding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: String,
}

impl Shortcut {
    pub const NONE: &'static str = "";

    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut key = String::new();
        for part in text.split('+') {
            let part = part.trim();
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                "" => {}
                other => key = canonical_key(other),
            }
        }
        (!key.is_empty()).then_some(Self { ctrl, shift, alt, key })
    }

    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        parts.push(match self.key.as_str() {
            "comma" => ",".into(),
            "slash" => "/".into(),
            "left" => "←".into(),
            "right" => "→".into(),
            "up" => "↑".into(),
            "down" => "↓".into(),
            "escape" => "Esc".into(),
            "delete" => "Del".into(),
            "enter" => "Enter".into(),
            "space" => "Space".into(),
            "tab" => "Tab".into(),
            other => other.to_uppercase(),
        });
        parts.join(" + ")
    }
}

fn canonical_key(key: &str) -> String {
    match key {
        "," => "comma".into(),
        "/" => "slash".into(),
        "arrowleft" | "left" => "left".into(),
        "arrowright" | "right" => "right".into(),
        "arrowup" | "up" => "up".into(),
        "arrowdown" | "down" => "down".into(),
        "esc" => "escape".into(),
        "del" => "delete".into(),
        "return" => "enter".into(),
        " " => "space".into(),
        other => other.to_string(),
    }
}

/// Shortcut overrides persisted in settings, keyed by [`Action::id`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keymap {
    #[serde(default)]
    pub overrides: std::collections::BTreeMap<String, String>,
}

impl Keymap {
    pub fn shortcut_text(&self, action: Action) -> String {
        self.overrides.get(action.id()).cloned().unwrap_or_else(|| action.default_shortcut().to_string())
    }

    pub fn shortcut(&self, action: Action) -> Option<Shortcut> {
        Shortcut::parse(&self.shortcut_text(action))
    }

    /// Rebinding clears the same chord from any other action to avoid ambiguity.
    pub fn bind(&mut self, action: Action, chord: Option<Shortcut>) {
        match chord {
            Some(chord) => {
                let text = serialize(&chord);
                for other in Action::ALL {
                    if *other != action && self.shortcut(*other).as_ref() == Some(&chord) {
                        self.overrides.insert(other.id().to_string(), String::new());
                    }
                }
                self.overrides.insert(action.id().to_string(), text);
            }
            None => {
                self.overrides.insert(action.id().to_string(), String::new());
            }
        }
    }

    #[must_use]
    pub fn reset(&mut self, action: Action) -> &mut Self {
        self.overrides.remove(action.id());
        self
    }

    /// Resolves a chord to exactly one action, or None when unbound/ambiguous.
    pub fn resolve(&self, chord: &Shortcut) -> Option<Action> {
        Action::ALL.iter().copied().find(|action| self.shortcut(*action).as_ref() == Some(chord))
    }
}

fn serialize(chord: &Shortcut) -> String {
    let mut parts = Vec::new();
    if chord.ctrl {
        parts.push("ctrl".to_string());
    }
    if chord.shift {
        parts.push("shift".to_string());
    }
    if chord.alt {
        parts.push("alt".to_string());
    }
    parts.push(chord.key.clone());
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_unique_id_and_label() {
        let mut ids = std::collections::BTreeSet::new();
        for action in Action::ALL {
            assert!(ids.insert(action.id()), "duplicate id {}", action.id());
            assert!(!action.label().is_empty());
            assert!(!action.category().is_empty());
        }
    }

    #[test]
    fn shortcuts_parse_and_display() {
        let chord = Shortcut::parse("ctrl+shift+t").unwrap();
        assert!(chord.ctrl && chord.shift && !chord.alt);
        assert_eq!(chord.key, "t");
        assert_eq!(chord.display(), "Ctrl + Shift + T");
        assert_eq!(Shortcut::parse("ctrl+comma").unwrap().display(), "Ctrl + ,");
        assert!(Shortcut::parse("").is_none());
    }

    #[test]
    fn keymap_rebinding_is_exclusive_and_resettable() {
        let mut keymap = Keymap::default();
        assert_eq!(keymap.resolve(&Shortcut::parse("ctrl+t").unwrap()), Some(Action::NewTab));
        keymap.bind(Action::Refresh, Shortcut::parse("ctrl+t"));
        assert_eq!(keymap.resolve(&Shortcut::parse("ctrl+t").unwrap()), Some(Action::Refresh));
        assert_eq!(keymap.shortcut(Action::NewTab), None);
        keymap.reset(Action::Refresh);
        assert_eq!(keymap.shortcut(Action::Refresh).unwrap().display(), "F5");
    }

    #[test]
    fn unbound_actions_report_none() {
        let keymap = Keymap::default();
        assert!(keymap.shortcut(Action::MoveTabToPane).is_none());
        assert!(keymap.shortcut(Action::CopyName).is_none());
    }
}
