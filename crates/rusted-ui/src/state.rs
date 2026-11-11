//! The UI model. Deliberately free of GPUI entities: everything here can be
//! constructed and asserted on in a unit test, and the shell only renders it.

use crate::services::{DirectoryLoader, PreviewCache, SearchService, ThumbnailCache};
use rusted_core::{
    Action, FileEntry, Keymap, Notice, PaneId, Settings, SortSpec, TabId, ViewMode, Workspace,
};
use rusted_db::Database;
use rusted_fs::DriveInfo;
use rusted_operations::{OperationProgress, OperationQueue, SharedControl, UndoStack};
use rusted_search::SearchHit;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Which transient overlay owns the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    GoTo,
    CommandPalette,
    Search,
    ContextMenu,
    TabMenu,
}

impl Overlay {
    pub fn is_open(self) -> bool {
        !matches!(self, Overlay::None)
    }

    /// Overlays that capture typed text into [`AppState::overlay_input`].
    pub fn wants_text(self) -> bool {
        matches!(self, Overlay::GoTo | Overlay::CommandPalette | Overlay::Search)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    None,
    Rename,
    BatchRename,
    NewFolder,
    NewFile,
    PermanentDelete,
    Settings,
    Shortcuts,
    StorageAnalyzer,
    Properties,
    TagEditor,
    CreateArchive,
    ExtractAll,
    Errors,
}

impl Dialog {
    /// Dialogs with a single text field that the keyboard should focus.
    pub fn has_input(self) -> bool {
        matches!(self, Dialog::Rename | Dialog::NewFolder | Dialog::NewFile | Dialog::ExtractAll | Dialog::TagEditor)
    }

    pub fn is_open(self) -> bool {
        !matches!(self, Dialog::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelTab {
    Preview,
    Inspector,
    Operations,
    Search,
    Storage,
}

impl PanelTab {
    pub const ALL: [Self; 5] = [Self::Preview, Self::Inspector, Self::Operations, Self::Search, Self::Storage];

    pub fn label(self) -> &'static str {
        match self {
            Self::Preview => "Preview",
            Self::Inspector => "Details",
            Self::Operations => "Queue",
            Self::Search => "Search",
            Self::Storage => "Storage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Toggle,
    Range,
}

impl SelectionMode {
    pub fn from_modifiers(shift: bool, toggle: bool) -> Self {
        if shift {
            Self::Range
        } else if toggle {
            Self::Toggle
        } else {
            Self::Replace
        }
    }
}

/// A context menu entry. `enabled == false` is rendered disabled *and* cannot be
/// invoked, so no menu entry can be a dead command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub action: Action,
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub separator_before: bool,
}

/// A pending filesystem question that needs an explicit answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prompt {
    /// Overwrite an existing destination?
    Conflict { sources: Vec<PathBuf>, destination: PathBuf },
    /// Permanently delete without the Recycle Bin?
    PermanentDelete { paths: Vec<PathBuf> },
    /// Destructive or irreversible action confirmation in general.
    Confirm { title: String, detail: String, action: Action },
}

/// Everything the shell needs that is not pure model state.
pub struct AppState {
    pub settings: Settings,
    pub keymap: Keymap,
    pub workspace: Workspace,
    pub theme_name: rusted_core::ThemeName,

    /// Streamed entries per pane, so switching panes does not reload.
    pub entries: BTreeMap<PaneId, Vec<FileEntry>>,
    pub loading: BTreeSet<PaneId>,
    /// Errors from the last load per pane; shown instead of a fake empty folder.
    pub load_errors: BTreeMap<PaneId, String>,

    pub drives: Vec<DriveInfo>,
    pub bookmarks: Vec<rusted_db::Bookmark>,
    pub recent_locations: Vec<PathBuf>,

    pub overlay: Overlay,
    pub overlay_input: String,
    pub overlay_selection: usize,
    pub menu_entries: Vec<MenuEntry>,
    pub menu_anchor: (f32, f32),
    pub tab_menu_tab: Option<TabId>,
    pub prompt: Option<Prompt>,

    pub dialog: Dialog,
    pub dialog_input: String,
    pub dialog_secondary: String,
    pub dialog_error: Option<String>,
    pub dialog_confirmed: bool,
    pub rename_spec: rusted_operations::BatchRenameSpec,
    pub rename_preview: Vec<rusted_operations::RenamePreview>,

    pub panel_tab: PanelTab,
    pub sidebar_collapsed: bool,
    pub panel_visible: bool,
    pub tree: crate::views::tree::TreeState,

    pub notices: Vec<Notice>,
    pub status: String,
    pub notices_seen: usize,

    pub operations: Vec<OperationProgress>,
    pub operation_controls: BTreeMap<rusted_core::Uuid, SharedControl>,
    pub clipboard: Vec<PathBuf>,
    pub clipboard_cut: bool,
    pub undo: UndoStack,
    pub search_hits: Vec<SearchHit>,
    pub search_filters: rusted_core::SearchFilters,
    pub search_hint: String,
    pub storage: Option<rusted_storage::StorageReport>,

    pub loader: DirectoryLoader,
    pub cancel_slot: Option<Arc<AtomicBool>>,
    pub cancel_slots: BTreeMap<PaneId, Arc<AtomicBool>>,
    pub thumbnails: ThumbnailCache,
    pub previews: PreviewCache,
    pub search: Option<SearchService>,
    pub database: Option<Database>,
    pub queue: OperationQueue,
    pub pending_rename_paths: Vec<PathBuf>,
    /// Filesystem paths decoded into GPU textures, keyed by path.
    pub textures: BTreeMap<PathBuf, Arc<gpui::RenderImage>>,
    /// Milliseconds spent in the most recent directory load, for the status bar.
    pub last_load_ms: u128,
}

impl AppState {
    pub fn new(settings: Settings, keymap: Keymap, workspace: Workspace) -> Self {
        let settings = settings.sanitized();
        let thumbnails = ThumbnailCache::new(512, settings.thumbnail_size as u32);
        let previews = PreviewCache::new(64, &settings);
        let sidebar_collapsed = settings.sidebar_collapsed;
        Self {
            theme_name: settings.theme,
            workspace,
            settings,
            keymap,
            entries: BTreeMap::new(),
            loading: BTreeSet::new(),
            load_errors: BTreeMap::new(),
            drives: rusted_fs::drives(),
            bookmarks: Vec::new(),
            recent_locations: Vec::new(),
            overlay: Overlay::None,
            overlay_input: String::new(),
            overlay_selection: 0,
            menu_entries: Vec::new(),
            menu_anchor: (0.0, 0.0),
            tab_menu_tab: None,
            prompt: None,
            dialog: Dialog::None,
            dialog_input: String::new(),
            dialog_secondary: String::new(),
            dialog_error: None,
            dialog_confirmed: false,
            rename_spec: rusted_operations::BatchRenameSpec::default(),
            rename_preview: Vec::new(),
            panel_tab: PanelTab::Inspector,
            sidebar_collapsed,
            panel_visible: false,
            tree: crate::views::tree::TreeState::default(),
            notices: Vec::new(),
            status: String::new(),
            notices_seen: 0,
            operations: Vec::new(),
            operation_controls: BTreeMap::new(),
            clipboard: Vec::new(),
            clipboard_cut: false,
            undo: UndoStack::new(64),
            search_hits: Vec::new(),
            search_filters: rusted_core::SearchFilters::default(),
            search_hint: String::new(),
            storage: None,
            loader: DirectoryLoader::new(),
            cancel_slot: None,
            cancel_slots: BTreeMap::new(),
            thumbnails,
            previews,
            search: None,
            database: None,
            queue: OperationQueue::new(2),
            pending_rename_paths: Vec::new(),
            textures: BTreeMap::new(),
            last_load_ms: 0,
        }
    }

    // ---------------------------------------------------------------- entries

    pub fn entries(&self) -> &[FileEntry] {
        self.entries_for(self.workspace.active_pane)
    }

    pub fn entries_for(&self, pane: PaneId) -> &[FileEntry] {
        self.entries.get(&pane).map(Vec::as_slice).unwrap_or_default()
    }

    pub fn set_entries(&mut self, pane: PaneId, entries: Vec<FileEntry>) {
        self.entries.insert(pane, entries);
        self.load_errors.remove(&pane);
    }

    pub fn set_load_error(&mut self, pane: PaneId, message: String) {
        self.entries.insert(pane, Vec::new());
        self.load_errors.insert(pane, message);
    }

    pub fn is_loading(&self, pane: PaneId) -> bool {
        self.loading.contains(&pane)
    }

    pub fn load_error(&self, pane: PaneId) -> Option<&String> {
        self.load_errors.get(&pane)
    }

    // -------------------------------------------------------------- selection

    pub fn selected_paths(&self) -> Vec<PathBuf> {
        self.workspace.active_tab().selection.paths().cloned().collect()
    }

    pub fn primary_selection(&self) -> Option<&FileEntry> {
        let tab = self.workspace.active_tab();
        let entries = self.entries();
        if let Some(anchor) = tab.selection.anchor.as_ref()
            && let Some(entry) = entries.iter().find(|entry| &entry.path == anchor)
        {
            return Some(entry);
        }
        tab.selection.paths().find_map(|path| entries.iter().find(|entry| &entry.path == path))
    }

    pub fn selected_entries(&self) -> Vec<FileEntry> {
        let entries = self.entries();
        entries.iter().filter(|entry| self.workspace.active_tab().selection.contains(&entry.path)).cloned().collect()
    }

    pub fn focused_index(&self) -> Option<usize> {
        let anchor = self.workspace.active_tab().selection.anchor.as_ref()?;
        self.visible_entries().iter().position(|entry| &entry.path == anchor)
    }

    /// Applies a click/arrow selection against the *visible* (possibly filtered)
    /// ordering so ranges always match what the user sees.
    pub fn select_index(&mut self, index: usize, mode: SelectionMode) {
        let visible = self.visible_entries();
        let Some(entry) = visible.get(index) else {
            return;
        };
        let path = entry.path.clone();
        let ordered = visible;
        let previous_anchor = self.workspace.active_tab().selection.anchor.clone();
        let selection = &mut self.workspace.active_tab_mut().selection;
        match mode {
            SelectionMode::Replace => selection.select_only(path),
            SelectionMode::Toggle => selection.toggle(path),
            SelectionMode::Range => match previous_anchor {
                Some(anchor) => selection.select_range(&ordered, &anchor, &path),
                None => selection.select_only(path),
            },
        }
    }

    pub fn select_path(&mut self, path: PathBuf, mode: SelectionMode) {
        let visible = self.visible_entries();
        match visible.iter().position(|entry| entry.path == path) {
            Some(index) => self.select_index(index, mode),
            None => {
                let selection = &mut self.workspace.active_tab_mut().selection;
                match mode {
                    SelectionMode::Toggle => selection.toggle(path),
                    _ => selection.select_only(path),
                }
            }
        }
    }

    /// Moves the selection by `delta` rows and returns the new focused index.
    pub fn move_selection(&mut self, delta: isize) -> Option<usize> {
        let visible = self.visible_entries();
        if visible.is_empty() {
            return None;
        }
        let current = self.focused_index().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, visible.len() as isize - 1) as usize;
        self.select_index(next, SelectionMode::Replace);
        Some(next)
    }

    pub fn select_all(&mut self) {
        let visible = self.visible_entries();
        self.workspace.active_tab_mut().selection.select_all(&visible);
    }

    pub fn clear_selection(&mut self) {
        self.workspace.active_tab_mut().selection.clear();
    }

    pub fn invert_selection(&mut self) {
        let visible = self.visible_entries();
        let tab = self.workspace.active_tab_mut();
        let currently: BTreeSet<PathBuf> = tab.selection.paths().cloned().collect();
        tab.selection.clear();
        for entry in &visible {
            if !currently.contains(&entry.path) {
                tab.selection.toggle(entry.path.clone());
            }
        }
    }

    // --------------------------------------------------------------- filtering

    pub fn visible_entries(&self) -> Vec<FileEntry> {
        let tab = self.workspace.active_tab();
        let base: Vec<FileEntry> = if !self.search_hits.is_empty() && self.overlay == Overlay::Search {
            // Global (indexed) results replace the listing while the overlay is up.
            self.search_hits
                .iter()
                .map(|hit| FileEntry {
                    path: hit.path.clone(),
                    name: hit.name.clone(),
                    kind: rusted_core::EntryKind::File,
                    size: hit.size,
                    modified: hit.modified.and_then(|stamp| chrono::DateTime::from_timestamp(stamp, 0)),
                    created: None,
                    readonly: false,
                    hidden: false,
                    extension: hit.path.extension().map(|ext| ext.to_string_lossy().to_lowercase()),
                })
                .collect()
        } else {
            self.entries().to_vec()
        };
        let query = tab.search.query.trim();
        if query.is_empty() {
            base
        } else {
            crate::services::directory::filter_entries(&base, query)
        }
    }

    // ---------------------------------------------------------------- notices

    pub fn notify(&mut self, notice: Notice) {
        self.status = notice.message.clone();
        let is_error = notice.is_error;
        if is_error {
            self.notices.push(notice);
            if self.notices.len() > 200 {
                self.notices.remove(0);
            }
        }
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.notify(Notice::info(message));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.notify(Notice::error(message));
    }

    pub fn error_count(&self) -> usize {
        self.notices.len()
    }

    // -------------------------------------------------------------- commands

    /// Whether a command can do real work right now. The palette, context menus
    /// and toolbar all consult this, so nothing dead is ever offered.
    pub fn action_available(&self, action: Action) -> bool {
        use Action::*;
        let tab = self.workspace.active_tab();
        let has_selection = !tab.selection.is_empty();
        let has_clipboard = !self.clipboard.is_empty();
        let has_entries = !self.entries().is_empty();
        let multi_pane = self.workspace.root.pane_count() > 1;
        match action {
            Back => tab.history.can_back(),
            Forward => tab.history.can_forward(),
            Up => rusted_fs::parent(tab.path()).is_some(),
            Undo => self.undo.can_undo(),
            Redo => self.undo.can_redo(),
            Paste => has_clipboard,
            ClosePane => multi_pane,
            ReopenPane => multi_pane,
            CloseTab => !tab_has_single_tab(self),
            CyclePane => multi_pane,
            Open | OpenWithDefault | RevealInExplorer | Rename | BatchRename | Copy | Cut | Duplicate
            | DeleteRecycle | DeletePermanent | Properties | CopyPath | CopyName | TagSelection
            | AddToStarred | RemoveFromStarred | CreateArchive | ExtractAll | PreviewFocused => has_selection,
            SearchIndexed | SearchRecursive | StorageAnalyzer => !tab.path().as_os_str().is_empty(),
            SelectAll | InvertSelection | SelectNext | SelectPrevious => has_entries,
            ReopenClosedTab => !self.workspace.recently_closed().is_empty(),
            NextTab | PreviousTab | DuplicateTab | RenameTab | MoveTabToPane => !self.workspace.active_pane().is_empty(),
            ToggleInspector | TogglePreview | ToggleSidebar | OpenSettings | OpenShortcuts | NewTab
            | NewWindow | CommandPalette | GoTo | Refresh | FocusLocation | ViewGrid | ViewList
            | ViewDetails | ViewMiller | ViewTree | NextView | ToggleHidden | SortByName | SortBySize
            | SortByModified | SortByKind | ToggleFoldersFirst | NewFolder | NewFile | SearchHere
            | OpenArchive | Quit => true,
            ClearSelection => has_selection,
        }
    }

    /// Shortcut text shown next to palette and context-menu rows.
    pub fn shortcut_label(&self, action: Action) -> Option<String> {
        let text = self.keymap.shortcut_text(action);
        if text.trim().is_empty() {
            None
        } else {
            rusted_core::Shortcut::parse(&text).map(|chord| chord.display())
        }
    }

    /// Builds the context menu for the current selection. Entries mirror the real
    /// availability rules so disabled rows are visibly disabled, not silently dead.
    pub fn context_menu_for(&self, target: Option<&FileEntry>) -> Vec<MenuEntry> {
        use Action::*;
        // `None` marks a separator; every other slot is a real command.
        let layout: Vec<Option<Action>> = match target {
            Some(entry) if entry.is_dir() && !entry.is_link() => vec![
                Some(Open),
                Some(OpenWithDefault),
                None,
                Some(NewFolder),
                Some(NewFile),
                None,
                Some(Copy),
                Some(Cut),
                Some(Duplicate),
                None,
                Some(Rename),
                Some(BatchRename),
                None,
                Some(DeleteRecycle),
                Some(DeletePermanent),
                None,
                Some(CopyPath),
                Some(Properties),
                None,
                Some(CreateArchive),
                Some(StorageAnalyzer),
                None,
                Some(AddToStarred),
                Some(TagSelection),
            ],
            Some(_) => vec![
                Some(Open),
                Some(OpenWithDefault),
                Some(RevealInExplorer),
                None,
                Some(Copy),
                Some(Cut),
                Some(Duplicate),
                None,
                Some(Rename),
                Some(BatchRename),
                None,
                Some(DeleteRecycle),
                Some(DeletePermanent),
                None,
                Some(CopyPath),
                Some(CopyName),
                Some(Properties),
                None,
                Some(CreateArchive),
                Some(ExtractAll),
                None,
                Some(AddToStarred),
                Some(TagSelection),
            ],
            None => vec![
                Some(NewFolder),
                Some(NewFile),
                None,
                Some(Paste),
                None,
                Some(SelectAll),
                Some(InvertSelection),
                None,
                Some(Refresh),
                Some(ToggleHidden),
                None,
                Some(OpenSettings),
                Some(CommandPalette),
            ],
        };
        let mut entries = Vec::new();
        let mut separator_before = false;
        for slot in layout {
            match slot {
                None => separator_before = !entries.is_empty(),
                Some(action) => {
                    entries.push(MenuEntry {
                        action,
                        label: action.label().to_string(),
                        shortcut: self.shortcut_label(action),
                        enabled: self.action_available(action),
                        separator_before,
                    });
                    separator_before = false;
                }
            }
        }
        entries
    }

    // ----------------------------------------------------------------- panes

    /// Directory listing for one pane, following the tab that pane has active.
    pub fn pane_directory(&self, pane: PaneId) -> Option<PathBuf> {
        self.workspace.root.pane(pane).and_then(|pane| pane.active_tab().map(|tab| tab.path().to_path_buf()))
    }

    pub fn pane_view(&self, pane: PaneId) -> ViewMode {
        self.workspace
            .root
            .pane(pane)
            .and_then(|pane| pane.active_tab().map(|tab| tab.view))
            .unwrap_or(self.settings.default_view)
    }

    pub fn pane_sort(&self, pane: PaneId) -> SortSpec {
        self.workspace
            .root
            .pane(pane)
            .and_then(|pane| pane.active_tab().map(|tab| tab.sort))
            .unwrap_or_default()
    }

    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.workspace.root.pane_ids()
    }

    // ------------------------------------------------------------------ misc

    pub fn sidebar_shortcuts(&self) -> Vec<PathBuf> {
        if self.settings.sidebar_shortcuts.is_empty() {
            rusted_fs::KnownFolder::ALL.iter().filter_map(|folder| folder.path()).collect()
        } else {
            self.settings.sidebar_shortcuts.clone()
        }
    }

    pub fn breadcrumbs(&self) -> Vec<(String, PathBuf)> {
        self.workspace.active_tab().history.breadcrumbs()
    }

    /// Textures for the current listing, in listing order, for the grid view.
    pub fn thumbnail_for(&mut self, path: &Path) -> Option<Arc<gpui::RenderImage>> {
        if let Some(texture) = self.textures.get(path) {
            return Some(texture.clone());
        }
        let preview = self.thumbnails.load(path)?;
        let texture = crate::views::texture_from_preview(&preview)?;
        self.textures.insert(path.to_path_buf(), texture.clone());
        Some(texture)
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }
}

fn tab_has_single_tab(state: &AppState) -> bool {
    state.workspace.active_pane().tabs.len() <= 1 && state.workspace.root.pane_count() <= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusted_core::ThemeName;

    fn state() -> AppState {
        let dir = std::env::temp_dir();
        AppState::new(Settings::default(), Keymap::default(), Workspace::new(dir, ViewMode::Details))
    }

    fn entry(name: &str) -> FileEntry {
        FileEntry {
            path: PathBuf::from(format!("/entries/{name}")),
            name: name.into(),
            kind: rusted_core::EntryKind::File,
            size: 1,
            modified: None,
            created: None,
            readonly: false,
            hidden: false,
            extension: None,
        }
    }

    #[test]
    fn fresh_state_starts_on_a_real_directory() {
        let state = state();
        assert!(state.workspace.active_directory().is_absolute());
        assert_eq!(state.theme_name, ThemeName::ObsidianBlack);
        assert!(!state.action_available(Action::Paste));
        assert!(!state.action_available(Action::Undo));
        assert!(state.action_available(Action::NewTab));
        assert!(!state.action_available(Action::ClosePane));
    }

    #[test]
    fn notices_keep_errors_and_status_text() {
        let mut state = state();
        state.info("copied");
        assert_eq!(state.status, "copied");
        assert!(state.notices.is_empty());
        state.error("permission denied");
        assert_eq!(state.status, "permission denied");
        assert_eq!(state.notices.len(), 1);
        assert!(state.notices[0].is_error);
        assert_eq!(state.error_count(), 1);
    }

    #[test]
    fn actions_report_availability_from_real_state() {
        let mut state = state();
        assert!(!state.action_available(Action::Back));
        let here = state.workspace.active_directory().to_path_buf();
        state.workspace.active_tab_mut().history.visit(here);
        assert!(state.action_available(Action::Back));
    }

    #[test]
    fn selection_modes_match_visible_order() {
        let mut state = state();
        let pane = state.workspace.active_pane;
        state.set_entries(pane, vec![entry("a"), entry("b"), entry("c"), entry("d")]);
        state.select_index(1, SelectionMode::Replace);
        assert_eq!(state.selected_paths(), vec![PathBuf::from("/entries/b")]);
        state.select_index(3, SelectionMode::Range);
        assert_eq!(state.selected_paths().len(), 3);
        state.select_index(3, SelectionMode::Toggle);
        assert_eq!(state.selected_paths().len(), 2);
        state.invert_selection();
        assert_eq!(state.selected_paths().len(), 2);
        state.select_all();
        assert_eq!(state.selected_paths().len(), 4);
        state.clear_selection();
        assert!(state.selected_paths().is_empty());
    }

    #[test]
    fn arrow_movement_clamps_at_the_edges() {
        let mut state = state();
        let pane = state.workspace.active_pane;
        state.set_entries(pane, vec![entry("a"), entry("b"), entry("c")]);
        assert_eq!(state.move_selection(-1), Some(0));
        assert_eq!(state.move_selection(2), Some(2));
        assert_eq!(state.move_selection(10), Some(2));
        assert_eq!(state.focused_index(), Some(2));
    }

    #[test]
    fn load_errors_are_visible_and_clear_entries() {
        let mut state = state();
        let pane = state.workspace.active_pane;
        state.set_entries(pane, vec![entry("a")]);
        state.set_load_error(pane, "access denied".into());
        assert!(state.entries().is_empty());
        assert_eq!(state.load_error(pane).map(String::as_str), Some("access denied"));
        state.set_entries(pane, vec![entry("b")]);
        assert!(state.load_error(pane).is_none());
    }

    #[test]
    fn context_menu_never_offers_a_dead_command() {
        let mut state = state();
        state.clipboard.push(PathBuf::from("/entries/a"));
        let entries = state.context_menu_for(None);
        assert!(entries.iter().any(|entry| entry.action == Action::Paste && entry.enabled));
        assert!(entries.iter().all(|entry| entry.label == entry.action.label()));
        for entry in &entries {
            assert_eq!(entry.enabled, state.action_available(entry.action));
        }
    }
}
