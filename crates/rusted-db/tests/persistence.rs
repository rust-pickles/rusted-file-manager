use rusted_core::{Keymap, Settings, Shortcut, ThemeName, ViewMode, Workspace};
use rusted_db::{Database, Profile};
use std::path::Path;

#[test]
fn settings_theme_and_session_roundtrip() {
    let db = Database::in_memory().unwrap();
    let mut settings = Settings::default();
    settings.theme = ThemeName::Snow;
    db.save_settings(&settings).unwrap();
    assert_eq!(db.load_settings().unwrap().unwrap().theme, ThemeName::Snow);

    let mut workspace = Workspace::new(".".into(), ViewMode::Details);
    workspace.new_tab("..".into(), ViewMode::Grid);
    db.save_session("autosave", &workspace).unwrap();
    let restored = db.load_session("autosave").unwrap().unwrap();
    assert_eq!(restored.active_pane().tabs.len(), 2);
    assert_eq!(restored.active_tab().view, ViewMode::Grid);
    assert_eq!(db.session_names().unwrap(), vec!["autosave".to_string()]);
}

#[test]
fn bookmarks_tags_recents_and_profiles_persist() {
    let db = Database::in_memory().unwrap();
    db.pin(Path::new("/tmp/a"), "A", "Custom").unwrap();
    db.tag(Path::new("/tmp/a"), "work", "#aaaaaa").unwrap();
    db.visit(Path::new("/tmp/a")).unwrap();
    db.opened_file(Path::new("/tmp/a/file.txt")).unwrap();

    assert_eq!(db.bookmarks().unwrap().len(), 1);
    assert_eq!(db.tags(Some("work")).unwrap().len(), 1);
    assert_eq!(db.tags_for(Path::new("/tmp/a")).unwrap().len(), 1);
    assert_eq!(db.recent_locations(10).unwrap().len(), 1);
    assert_eq!(db.recent_files(10).unwrap().len(), 1);

    db.untag(Path::new("/tmp/a"), "work").unwrap();
    assert!(db.tags(None).unwrap().is_empty());

    let profile = Profile {
        name: "Work".into(),
        workspace: Workspace::new("/tmp".into(), ViewMode::List),
        settings: Settings::default(),
    };
    db.save_profile(&profile).unwrap();
    assert_eq!(db.load_profile("Work").unwrap().unwrap().workspace.active_tab().view, ViewMode::List);
    assert_eq!(db.profiles().unwrap(), vec!["Work".to_string()]);
    db.delete_profile("Work").unwrap();
    assert!(db.profiles().unwrap().is_empty());
}

#[test]
fn keymap_overrides_persist() {
    let db = Database::in_memory().unwrap();
    let mut keymap = Keymap::default();
    keymap.bind(rusted_core::Action::StorageAnalyzer, Shortcut::parse("ctrl+alt+s"));
    db.save_keymap(&keymap).unwrap();
    let restored = db.load_keymap().unwrap().unwrap();
    assert_eq!(restored.shortcut_text(rusted_core::Action::StorageAnalyzer), "ctrl+alt+s");
}
