//! Local SQLite persistence: settings, bookmarks, tags, recents, sessions and
//! profiles. A single connection guarded by the app keeps writes serialized.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use rusted_core::{Settings, Workspace};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("state encoding error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("local storage could not be prepared: {0}")]
    Io(#[from] std::io::Error),
    #[error("no local application data directory is available")]
    NoDataDirectory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub path: PathBuf,
    pub label: String,
    pub position: i64,
    pub section: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub path: PathBuf,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub workspace: Workspace,
    pub settings: Settings,
}

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 3000;
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS bookmarks (
    path TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    position INTEGER NOT NULL,
    section TEXT NOT NULL DEFAULT 'Pinned'
);
CREATE TABLE IF NOT EXISTS tags (
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    PRIMARY KEY (path, name)
);
CREATE TABLE IF NOT EXISTS recent_locations (path TEXT PRIMARY KEY, visited_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS recent_files (path TEXT PRIMARY KEY, opened_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS sessions (name TEXT PRIMARY KEY, workspace TEXT NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS profiles (name TEXT PRIMARY KEY, workspace TEXT NOT NULL, settings TEXT NOT NULL);
"#;

pub struct Database {
    connection: Connection,
}

impl Database {
    /// Opens (or creates) the database under the user's local app data folder.
    pub fn open_default() -> Result<Self, DbError> {
        let project =
            directories::ProjectDirs::from("com", "Rusted", "Rusted").ok_or(DbError::NoDataDirectory)?;
        let directory = project.data_local_dir();
        std::fs::create_dir_all(directory)?;
        Self::open(&directory.join("rusted.db"))
    }

    pub fn open(path: &Path) -> Result<Self, DbError> {
        Self::init(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self, DbError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(connection: Connection) -> Result<Self, DbError> {
        connection.execute_batch(SCHEMA)?;
        Ok(Self { connection })
    }

    pub fn path_of_default_database() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "Rusted", "Rusted")
            .map(|project| project.data_local_dir().join("rusted.db"))
    }

    // ---- settings -------------------------------------------------------

    pub fn save_settings(&self, settings: &Settings) -> Result<(), DbError> {
        self.set_json("settings", settings)
    }

    pub fn load_settings(&self) -> Result<Option<Settings>, DbError> {
        Ok(self.get_json::<Settings>("settings")?.map(Settings::sanitized))
    }

    pub fn save_keymap(&self, keymap: &rusted_core::Keymap) -> Result<(), DbError> {
        self.set_json("keymap", keymap)
    }

    pub fn load_keymap(&self) -> Result<Option<rusted_core::Keymap>, DbError> {
        self.get_json("keymap")
    }

    // ---- bookmarks ------------------------------------------------------

    pub fn pin(&self, path: &Path, label: &str, section: &str) -> Result<(), DbError> {
        let position: i64 = self.connection.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM bookmarks WHERE section = ?1",
            [section],
            |row| row.get(0),
        )?;
        self.connection.execute(
            "INSERT INTO bookmarks(path, label, position, section) VALUES(?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET label = excluded.label, section = excluded.section",
            params![path.to_string_lossy(), label, position, section],
        )?;
        Ok(())
    }

    pub fn unpin(&self, path: &Path) -> Result<(), DbError> {
        self.connection.execute("DELETE FROM bookmarks WHERE path = ?1", [path.to_string_lossy()])?;
        Ok(())
    }

    pub fn is_pinned(&self, path: &Path) -> Result<bool, DbError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE path = ?1",
            [path.to_string_lossy()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn move_bookmark(&self, path: &Path, position: i64) -> Result<(), DbError> {
        self.connection.execute(
            "UPDATE bookmarks SET position = ?2 WHERE path = ?1",
            params![path.to_string_lossy(), position],
        )?;
        Ok(())
    }

    pub fn bookmarks(&self) -> Result<Vec<Bookmark>, DbError> {
        let mut statement = self
            .connection
            .prepare("SELECT path, label, position, section FROM bookmarks ORDER BY section, position, label")?;
        let rows = statement.query_map([], |row| {
            Ok(Bookmark {
                path: PathBuf::from(row.get::<_, String>(0)?),
                label: row.get(1)?,
                position: row.get(2)?,
                section: row.get(3)?,
            })
        })?;
        let mut bookmarks = Vec::new();
        for row in rows {
            bookmarks.push(row?);
        }
        Ok(bookmarks)
    }

    // ---- tags -----------------------------------------------------------

    pub fn tag(&self, path: &Path, name: &str, color: &str) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT INTO tags(path, name, color) VALUES(?1, ?2, ?3)
             ON CONFLICT(path, name) DO UPDATE SET color = excluded.color",
            params![path.to_string_lossy(), name, color],
        )?;
        Ok(())
    }

    pub fn untag(&self, path: &Path, name: &str) -> Result<(), DbError> {
        self.connection.execute(
            "DELETE FROM tags WHERE path = ?1 AND name = ?2",
            params![path.to_string_lossy(), name],
        )?;
        Ok(())
    }

    pub fn tags(&self, filter: Option<&str>) -> Result<Vec<Tag>, DbError> {
        let mut statement = self.connection.prepare(
            "SELECT path, name, color FROM tags WHERE (?1 IS NULL OR name = ?1) ORDER BY name, path",
        )?;
        let rows = statement.query_map([filter], |row| {
            Ok(Tag {
                path: PathBuf::from(row.get::<_, String>(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    pub fn tags_for(&self, path: &Path) -> Result<Vec<Tag>, DbError> {
        let mut statement = self
            .connection
            .prepare("SELECT path, name, color FROM tags WHERE path = ?1 ORDER BY name")?;
        let rows = statement.query_map([path.to_string_lossy()], |row| {
            Ok(Tag {
                path: PathBuf::from(row.get::<_, String>(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    // ---- recents --------------------------------------------------------

    pub fn visit(&self, path: &Path) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT INTO recent_locations(path, visited_at) VALUES(?1, ?2)
             ON CONFLICT(path) DO UPDATE SET visited_at = excluded.visited_at",
            params![path.to_string_lossy(), Utc::now().timestamp()],
        )?;
        self.trim_recent("recent_locations", "visited_at")
    }

    pub fn opened_file(&self, path: &Path) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT INTO recent_files(path, opened_at) VALUES(?1, ?2)
             ON CONFLICT(path) DO UPDATE SET opened_at = excluded.opened_at",
            params![path.to_string_lossy(), Utc::now().timestamp()],
        )?;
        self.trim_recent("recent_files", "opened_at")
    }

    pub fn forget_recent(&self, path: &Path) -> Result<(), DbError> {
        self.connection.execute("DELETE FROM recent_locations WHERE path = ?1", [path.to_string_lossy()])?;
        Ok(())
    }

    fn trim_recent(&self, table: &str, column: &str) -> Result<(), DbError> {
        // Table and column are internal constants, never user input.
        self.connection.execute(
            &format!(
                "DELETE FROM {table} WHERE path NOT IN (SELECT path FROM {table} ORDER BY {column} DESC LIMIT 200)"
            ),
            [],
        )?;
        Ok(())
    }

    pub fn recent_locations(&self, limit: usize) -> Result<Vec<PathBuf>, DbError> {
        let mut statement = self
            .connection
            .prepare("SELECT path FROM recent_locations ORDER BY visited_at DESC LIMIT ?1")?;
        let rows = statement.query_map([limit as i64], |row| Ok(PathBuf::from(row.get::<_, String>(0)?)))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    pub fn recent_files(&self, limit: usize) -> Result<Vec<PathBuf>, DbError> {
        let mut statement =
            self.connection.prepare("SELECT path FROM recent_files ORDER BY opened_at DESC LIMIT ?1")?;
        let rows = statement.query_map([limit as i64], |row| Ok(PathBuf::from(row.get::<_, String>(0)?)))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    // ---- sessions and profiles ------------------------------------------

    pub fn save_session(&self, name: &str, workspace: &Workspace) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT INTO sessions(name, workspace, updated_at) VALUES(?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET workspace = excluded.workspace, updated_at = excluded.updated_at",
            params![name, serde_json::to_string(workspace)?, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    pub fn load_session(&self, name: &str) -> Result<Option<Workspace>, DbError> {
        let value = self
            .connection
            .query_row("SELECT workspace FROM sessions WHERE name = ?1", [name], |row| row.get::<_, String>(0))
            .optional()?;
        match value {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub fn session_names(&self) -> Result<Vec<String>, DbError> {
        let mut statement = self.connection.prepare("SELECT name FROM sessions ORDER BY updated_at DESC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    }

    pub fn save_profile(&self, profile: &Profile) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT INTO profiles(name, workspace, settings) VALUES(?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET workspace = excluded.workspace, settings = excluded.settings",
            params![
                profile.name,
                serde_json::to_string(&profile.workspace)?,
                serde_json::to_string(&profile.settings)?
            ],
        )?;
        Ok(())
    }

    pub fn load_profile(&self, name: &str) -> Result<Option<Profile>, DbError> {
        let row = self
            .connection
            .query_row("SELECT workspace, settings FROM profiles WHERE name = ?1", [name], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .optional()?;
        match row {
            Some((workspace, settings)) => Ok(Some(Profile {
                name: name.to_string(),
                workspace: serde_json::from_str(&workspace)?,
                settings: serde_json::from_str(&settings)?,
            })),
            None => Ok(None),
        }
    }

    pub fn profiles(&self) -> Result<Vec<String>, DbError> {
        let mut statement = self.connection.prepare("SELECT name FROM profiles ORDER BY name")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    }

    pub fn delete_profile(&self, name: &str) -> Result<(), DbError> {
        self.connection.execute("DELETE FROM profiles WHERE name = ?1", [name])?;
        Ok(())
    }

    // ---- helpers --------------------------------------------------------

    fn set_json<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), DbError> {
        self.connection.execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, serde_json::to_string(value)?],
        )?;
        Ok(())
    }

    fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, DbError> {
        let value = self
            .connection
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| row.get::<_, String>(0))
            .optional()?;
        match value {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusted_core::{ThemeName, ViewMode};

    #[test]
    fn settings_roundtrip_keeps_theme_and_are_sanitized() {
        let db = Database::in_memory().unwrap();
        assert!(db.load_settings().unwrap().is_none());
        let mut settings = Settings::default();
        settings.theme = ThemeName::WarmSand;
        settings.thumbnail_size = 9_000;
        db.save_settings(&settings).unwrap();
        let loaded = db.load_settings().unwrap().unwrap();
        assert_eq!(loaded.theme, ThemeName::WarmSand);
        assert_eq!(loaded.thumbnail_size, 320);
    }

    #[test]
    fn bookmarks_keep_insertion_order_and_can_be_removed() {
        let db = Database::in_memory().unwrap();
        db.pin(Path::new("C:\\Work"), "Work", "Pinned").unwrap();
        db.pin(Path::new("C:\\Home"), "Home", "Pinned").unwrap();
        let bookmarks = db.bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 2);
        assert_eq!(bookmarks[0].label, "Work");
        assert!(db.is_pinned(Path::new("C:\\Work")).unwrap());
        db.unpin(Path::new("C:\\Work")).unwrap();
        assert!(!db.is_pinned(Path::new("C:\\Work")).unwrap());
        assert_eq!(db.bookmarks().unwrap().len(), 1);
    }
}
