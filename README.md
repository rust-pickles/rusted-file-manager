# RUSTED — desktop file manager

A Windows file manager. React + TypeScript interface, Rust backend, shipped as a
Tauri 2 desktop app.

- Miller columns, tabs, split panes, command palette, quick preview
- Real filesystem work: list, create, rename, delete (Recycle Bin), copy, move,
  duplicate, star, search, storage overview, recent and starred lists
- Stars and recent files persist in a local SQLite database

## Install it (Windows 10/11)

You only need [Node.js 20+](https://nodejs.org). Rust installs itself if missing.

1. Download the source (the **Code** button, or `git clone`).
2. Double-click **`build-installer.bat`** and wait 10–20 minutes. It installs
   Rust via rustup when needed, builds in release mode, and writes
   `dist-installer\RUSTED_<version>_x64-setup.exe`.
3. Run that setup file. It installs the app for your user and registers it in
   Settings → Apps.

**`install.bat`** does steps 2–3 together: it finds an already-built installer,
or builds one, then runs it.

Uninstall from Settings → Apps, or run `install.bat` and choose uninstall.

Unattended build (CI uses this path):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-installer.ps1
```

Add `-Portable` to skip the installer and produce only `target\release\rusted-shell.exe`.

## Develop

```bash
npm install
npm run dev     # vite dev server on http://localhost:3000
npm run build   # type-check and build ui/ into dist/
npm run tauri dev
```

`npm run dev` alone runs the interface against an in-memory mock filesystem,
because a web page cannot read your disk. The real backend is only active inside
the Tauri window.

## Layout

| Path | Contents |
| --- | --- |
| `ui/` | React interface (a git subtree of the frontend project) |
| `src-tauri/` | Tauri shell, commands, virtual-path mapping |
| `crates/rusted-core` | State model: tabs, panes, selection, history |
| `crates/rusted-fs` | Directory enumeration, metadata, drives |
| `crates/rusted-operations` | Copy, move, rename, delete, duplicate |
| `crates/rusted-search` | Filename index and query |
| `crates/rusted-db` | SQLite persistence for stars and recents |
| `crates/rusted-preview` | Preview type detection |
| `crates/rusted-archives` | Archive browsing |
| `crates/rusted-storage` | Disk usage |

## Known gaps

- Restore from Recycle Bin and Empty Recycle Bin are not implemented; the
  interface reports that instead of pretending to succeed.
- Thumbnails are not generated yet, so the inspector shows no image preview.
- Folders refresh on navigation, not on external file changes.

## Licence

See `LICENSE`.
