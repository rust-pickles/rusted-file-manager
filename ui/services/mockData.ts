import { FileItem, DriveInfo } from '../types/filesystem';

export const INITIAL_DRIVES: DriveInfo[] = [
  {
    id: 'drive-main',
    name: 'Rusted System (C:)',
    mountPoint: 'C:',
    fileSystem: 'APFS / Ext4',
    totalBytes: 512 * 1024 * 1024 * 1024, // 512 GB
    usedBytes: 312 * 1024 * 1024 * 1024,
    freeBytes: 200 * 1024 * 1024 * 1024,
    iconType: 'internal',
  },
  {
    id: 'drive-data',
    name: 'Fast NVMe Data (D:)',
    mountPoint: 'D:',
    fileSystem: 'Btrfs / NTFS',
    totalBytes: 2048 * 1024 * 1024 * 1024, // 2 TB
    usedBytes: 1140 * 1024 * 1024 * 1024,
    freeBytes: 908 * 1024 * 1024 * 1024,
    iconType: 'external',
  },
  {
    id: 'drive-nas',
    name: 'Studio NAS Archive (Z:)',
    mountPoint: '//nas.local/volume1',
    fileSystem: 'NFS / SMB',
    totalBytes: 8192 * 1024 * 1024 * 1024, // 8 TB
    usedBytes: 4620 * 1024 * 1024 * 1024,
    freeBytes: 3572 * 1024 * 1024 * 1024,
    iconType: 'network',
  },
];

export const INITIAL_FILES: FileItem[] = [
  // ROOT LEVEL FOLDERS
  {
    id: 'folder-home',
    name: 'Home',
    path: '/Home',
    parentPath: '/',
    type: 'folder',
    size: 48920140280,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-11T09:30:00Z',
    isStarred: true,
    itemCount: 8,
  },
  {
    id: 'folder-desktop',
    name: 'Desktop',
    path: '/Home/Desktop',
    parentPath: '/Home',
    type: 'folder',
    size: 24500000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-10T14:22:00Z',
    isStarred: true,
    itemCount: 4,
  },
  {
    id: 'folder-downloads',
    name: 'Downloads',
    path: '/Home/Downloads',
    parentPath: '/Home',
    type: 'folder',
    size: 1480000000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-11T08:15:00Z',
    isStarred: true,
    itemCount: 6,
  },
  {
    id: 'folder-documents',
    name: 'Documents',
    path: '/Home/Documents',
    parentPath: '/Home',
    type: 'folder',
    size: 3200000000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-11T02:40:00Z',
    isStarred: true,
    itemCount: 4,
  },
  {
    id: 'folder-pictures',
    name: 'Pictures',
    path: '/Home/Pictures',
    parentPath: '/Home',
    type: 'folder',
    size: 18400000000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-09T18:10:00Z',
    isStarred: true,
    itemCount: 3,
  },
  {
    id: 'folder-videos',
    name: 'Videos',
    path: '/Home/Videos',
    parentPath: '/Home',
    type: 'folder',
    size: 24500000000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-08T11:20:00Z',
    isStarred: false,
    itemCount: 3,
  },
  {
    id: 'folder-music',
    name: 'Music',
    path: '/Home/Music',
    parentPath: '/Home',
    type: 'folder',
    size: 1400000000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-07T16:45:00Z',
    isStarred: false,
    itemCount: 3,
  },
  {
    id: 'folder-trash',
    name: 'Trash',
    path: '/Trash',
    parentPath: '/',
    type: 'folder',
    size: 145000000,
    createdAt: '2026-01-10T08:00:00Z',
    modifiedAt: '2026-11-10T19:00:00Z',
    itemCount: 2,
  },

  // DESKTOP FILES
  {
    id: 'file-desktop-1',
    name: 'Atlas_Architecture_Spec.pdf',
    path: '/Home/Desktop/Atlas_Architecture_Spec.pdf',
    parentPath: '/Home/Desktop',
    type: 'file',
    extension: 'pdf',
    size: 4820000,
    createdAt: '2026-11-09T10:14:00Z',
    modifiedAt: '2026-11-10T11:32:00Z',
    mimeType: 'application/pdf',
    isStarred: true,
    previewKind: 'pdf',
    textContent: 'Atlas File Manager Desktop Architecture v2.4\nComprehensive system layout for Tauri 2 Rust Core and React Frontend.',
  },
  {
    id: 'file-desktop-2',
    name: 'Applet_Sprint_Notes.md',
    path: '/Home/Desktop/Applet_Sprint_Notes.md',
    parentPath: '/Home/Desktop',
    type: 'file',
    extension: 'md',
    size: 24500,
    createdAt: '2026-11-08T09:00:00Z',
    modifiedAt: '2026-11-11T03:15:00Z',
    mimeType: 'text/markdown',
    isStarred: false,
    previewKind: 'markdown',
    textContent: `# Atlas Development Roadmap

- [x] Miller Columns Navigation Engine
- [x] Multi-Tab Session Synchronization
- [x] High-Density List & Compact Viewports
- [x] Instant QuickLook Preview Inspector
- [ ] Rust Tauri 2 IPC Command Bindings

### Target Performance Invariants
- Frame latency: < 8ms on 120Hz displays
- Subdirectory traversal: < 15ms in-memory cache
- Keyboard hotkeys: Zero conflict with native desktop compositor`,
  },
  {
    id: 'file-desktop-3',
    name: 'Desktop_Quick_Mockup.png',
    path: '/Home/Desktop/Desktop_Quick_Mockup.png',
    parentPath: '/Home/Desktop',
    type: 'file',
    extension: 'png',
    size: 3200000,
    createdAt: '2026-11-10T14:15:00Z',
    modifiedAt: '2026-11-10T14:22:00Z',
    mimeType: 'image/png',
    previewKind: 'image',
    dimensions: { width: 2560, height: 1440 },
    previewUrl: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?auto=format&fit=crop&w=1200&q=80',
  },
  {
    id: 'file-desktop-4',
    name: '.dev_workspace.json',
    path: '/Home/Desktop/.dev_workspace.json',
    parentPath: '/Home/Desktop',
    type: 'file',
    extension: 'json',
    size: 1420,
    createdAt: '2026-11-01T12:00:00Z',
    modifiedAt: '2026-11-01T12:00:00Z',
    mimeType: 'application/json',
    isHidden: true,
    previewKind: 'code',
    codeLanguage: 'json',
    textContent: '{\n  "workspaceId": "atlas-dev-2026",\n  "recentProfiles": ["default", "dark-slate"],\n  "telemetry": false\n}',
  },

  // DOWNLOADS FILES
  {
    id: 'file-down-1',
    name: 'tauri-v2.3.0-linux-x86_64.AppImage',
    path: '/Home/Downloads/tauri-v2.3.0-linux-x86_64.AppImage',
    parentPath: '/Home/Downloads',
    type: 'file',
    extension: 'appimage',
    size: 89400000,
    createdAt: '2026-11-11T07:10:00Z',
    modifiedAt: '2026-11-11T07:11:00Z',
    mimeType: 'application/x-executable',
    previewKind: 'binary',
  },
  {
    id: 'file-down-2',
    name: 'Rust_Toolchain_1.85.tar.gz',
    path: '/Home/Downloads/Rust_Toolchain_1.85.tar.gz',
    parentPath: '/Home/Downloads',
    type: 'file',
    extension: 'tar.gz',
    size: 214000000,
    createdAt: '2026-11-10T16:20:00Z',
    modifiedAt: '2026-11-10T16:22:00Z',
    mimeType: 'application/gzip',
    previewKind: 'archive',
  },
  {
    id: 'file-down-3',
    name: 'Quarterly_Financial_Summary.xlsx',
    path: '/Home/Downloads/Quarterly_Financial_Summary.xlsx',
    parentPath: '/Home/Downloads',
    type: 'file',
    extension: 'xlsx',
    size: 1840000,
    createdAt: '2026-11-09T11:00:00Z',
    modifiedAt: '2026-11-09T11:05:00Z',
    mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    previewKind: 'document',
    textContent: 'Sheet 1: Budget Analysis 2026\nQ1 Revenue: $840,000\nQ2 Revenue: $1,120,000\nQ3 Revenue: $1,450,000\nOperating Margin: 44.8%',
  },
  {
    id: 'file-down-4',
    name: 'Figma_Design_Tokens_Kit.zip',
    path: '/Home/Downloads/Figma_Design_Tokens_Kit.zip',
    parentPath: '/Home/Downloads',
    type: 'file',
    extension: 'zip',
    size: 42100000,
    createdAt: '2026-11-08T15:30:00Z',
    modifiedAt: '2026-11-08T15:30:00Z',
    mimeType: 'application/zip',
    previewKind: 'archive',
  },
  {
    id: 'folder-down-apps',
    name: 'Installed_Packages',
    path: '/Home/Downloads/Installed_Packages',
    parentPath: '/Home/Downloads',
    type: 'folder',
    size: 580000000,
    createdAt: '2026-10-15T09:00:00Z',
    modifiedAt: '2026-11-05T18:00:00Z',
    itemCount: 2,
  },
  {
    id: 'file-down-pkg1',
    name: 'vscode-1.98.deb',
    path: '/Home/Downloads/Installed_Packages/vscode-1.98.deb',
    parentPath: '/Home/Downloads/Installed_Packages',
    type: 'file',
    extension: 'deb',
    size: 98000000,
    createdAt: '2026-11-04T10:00:00Z',
    modifiedAt: '2026-11-04T10:00:00Z',
    previewKind: 'binary',
  },
  {
    id: 'file-down-pkg2',
    name: 'postman-agent.tar.gz',
    path: '/Home/Downloads/Installed_Packages/postman-agent.tar.gz',
    parentPath: '/Home/Downloads/Installed_Packages',
    type: 'file',
    extension: 'tar.gz',
    size: 48000000,
    createdAt: '2026-11-02T12:00:00Z',
    modifiedAt: '2026-11-02T12:00:00Z',
    previewKind: 'archive',
  },

  // DOCUMENTS HIERARCHY
  {
    id: 'folder-doc-projects',
    name: 'Projects',
    path: '/Home/Documents/Projects',
    parentPath: '/Home/Documents',
    type: 'folder',
    size: 2100000000,
    createdAt: '2026-02-01T10:00:00Z',
    modifiedAt: '2026-11-11T02:30:00Z',
    isStarred: true,
    itemCount: 3,
  },
  {
    id: 'folder-doc-work',
    name: 'Enterprise_Work',
    path: '/Home/Documents/Enterprise_Work',
    parentPath: '/Home/Documents',
    type: 'folder',
    size: 840000000,
    createdAt: '2026-03-12T11:00:00Z',
    modifiedAt: '2026-11-08T17:00:00Z',
    itemCount: 3,
  },
  {
    id: 'folder-doc-personal',
    name: 'Personal_Records',
    path: '/Home/Documents/Personal_Records',
    parentPath: '/Home/Documents',
    type: 'folder',
    size: 260000000,
    createdAt: '2026-04-05T14:00:00Z',
    modifiedAt: '2026-10-20T12:00:00Z',
    itemCount: 2,
  },
  {
    id: 'file-doc-readme',
    name: 'Documentation_Index.pdf',
    path: '/Home/Documents/Documentation_Index.pdf',
    parentPath: '/Home/Documents',
    type: 'file',
    extension: 'pdf',
    size: 3400000,
    createdAt: '2026-08-10T10:00:00Z',
    modifiedAt: '2026-11-05T09:00:00Z',
    mimeType: 'application/pdf',
    previewKind: 'pdf',
    textContent: 'Documents Directory Master Index:\n1. Projects (Atlas Engine, Quantum Compute, Brand Systems)\n2. Enterprise Work (Client deliverables, NDAs, roadmaps)\n3. Personal (Records, contracts)',
  },

  // NESTED: Documents -> Projects -> Atlas-Desktop
  {
    id: 'folder-proj-atlas',
    name: 'Atlas-Desktop',
    path: '/Home/Documents/Projects/Atlas-Desktop',
    parentPath: '/Home/Documents/Projects',
    type: 'folder',
    size: 1450000000,
    createdAt: '2026-09-01T08:00:00Z',
    modifiedAt: '2026-11-11T03:40:00Z',
    isStarred: true,
    itemCount: 4,
  },
  {
    id: 'folder-proj-quantum',
    name: 'Quantum-Engine',
    path: '/Home/Documents/Projects/Quantum-Engine',
    parentPath: '/Home/Documents/Projects',
    type: 'folder',
    size: 450000000,
    createdAt: '2026-07-15T09:00:00Z',
    modifiedAt: '2026-11-06T15:30:00Z',
    itemCount: 3,
  },
  {
    id: 'folder-proj-brand',
    name: 'Brand-Assets',
    path: '/Home/Documents/Projects/Brand-Assets',
    parentPath: '/Home/Documents/Projects',
    type: 'folder',
    size: 200000000,
    createdAt: '2026-08-20T11:00:00Z',
    modifiedAt: '2026-10-30T16:00:00Z',
    itemCount: 3,
  },

  // NESTED: Atlas-Desktop -> src
  {
    id: 'folder-atlas-src',
    name: 'src',
    path: '/Home/Documents/Projects/Atlas-Desktop/src',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop',
    type: 'folder',
    size: 890000,
    createdAt: '2026-09-01T08:10:00Z',
    modifiedAt: '2026-11-11T03:30:00Z',
    itemCount: 3,
  },
  {
    id: 'file-atlas-cargo',
    name: 'Cargo.toml',
    path: '/Home/Documents/Projects/Atlas-Desktop/Cargo.toml',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop',
    type: 'file',
    extension: 'toml',
    size: 1840,
    createdAt: '2026-09-01T08:05:00Z',
    modifiedAt: '2026-11-10T22:00:00Z',
    previewKind: 'code',
    codeLanguage: 'toml',
    textContent: `[package]
name = "atlas-desktop"
version = "2.4.0"
edition = "2024"
authors = ["Atlas Native Team <core@atlasnative.com>"]
description = "Blazing fast desktop file manager core powered by Tauri 2"

[dependencies]
tauri = { version = "2.3", features = ["tray-icon", "protocol-asset"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.38", features = ["full"] }
rayon = "1.10"
walkdir = "2.5"
notify = "6.1"`,
  },
  {
    id: 'file-atlas-readme',
    name: 'README.md',
    path: '/Home/Documents/Projects/Atlas-Desktop/README.md',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop',
    type: 'file',
    extension: 'md',
    size: 4500,
    createdAt: '2026-09-01T08:00:00Z',
    modifiedAt: '2026-11-09T18:00:00Z',
    previewKind: 'markdown',
    textContent: `# Atlas Desktop File Manager

Built for engineers, designers, and creators who value instant response times and architectural clarity.

## Features
- **Cascading Miller Columns**: Seamless directory navigation without losing parent context.
- **Unified Inspector**: Native previews for code, vector graphics, audio waveforms, and video frames.
- **Command Palette (Ctrl+K)**: Frictionless keyboard navigation and file dispatch.
- **Tauri 2 Native Core**: Minimal memory footprint, multi-threaded directory indexing.`,
  },
  {
    id: 'file-atlas-spec',
    name: 'design_tokens.json',
    path: '/Home/Documents/Projects/Atlas-Desktop/design_tokens.json',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop',
    type: 'file',
    extension: 'json',
    size: 8900,
    createdAt: '2026-09-15T12:00:00Z',
    modifiedAt: '2026-11-10T16:00:00Z',
    previewKind: 'code',
    codeLanguage: 'json',
    textContent: `{\n  "theme": "atlas-obsidian",\n  "colors": {\n    "bg-canvas": "#0B0D10",\n    "bg-surface": "#161A21",\n    "border-subtle": "#252B38",\n    "accent-electric": "#38BDF8"\n  }\n}`,
  },

  // Files inside Atlas-Desktop/src
  {
    id: 'file-atlas-mainrs',
    name: 'main.rs',
    path: '/Home/Documents/Projects/Atlas-Desktop/src/main.rs',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop/src',
    type: 'file',
    extension: 'rs',
    size: 14200,
    createdAt: '2026-09-01T08:15:00Z',
    modifiedAt: '2026-11-11T03:30:00Z',
    isStarred: true,
    previewKind: 'code',
    codeLanguage: 'rust',
    textContent: `// Atlas File Manager - Desktop Core Engine
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, State};
use std::sync::Arc;

mod filesystem;
mod watcher;
mod indexing;

#[tauri::command]
async fn list_directory_fast(
    path: String,
    show_hidden: bool,
) -> Result<Vec<filesystem::FileMetadata>, String> {
    filesystem::scanner::scan_path(&path, show_hidden).await
}

#[tauri::command]
async fn transfer_files(
    sources: Vec<String>,
    destination: String,
    operation: String,
) -> Result<String, String> {
    filesystem::operations::queue_transfer(sources, destination, &operation).await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_directory_fast,
            transfer_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running atlas file manager desktop application");
}`,
  },
  {
    id: 'file-atlas-windowrs',
    name: 'window.rs',
    path: '/Home/Documents/Projects/Atlas-Desktop/src/window.rs',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop/src',
    type: 'file',
    extension: 'rs',
    size: 7800,
    createdAt: '2026-09-05T10:00:00Z',
    modifiedAt: '2026-11-08T14:00:00Z',
    previewKind: 'code',
    codeLanguage: 'rust',
    textContent: `// Native Window Configuration & Borderless Compositor Styling
pub fn setup_window(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").unwrap();
    #[cfg(target_os = "macos")]
    window.set_title_bar_style(tauri::TitleBarStyle::Overlay)?;
    Ok(())
}`,
  },
  {
    id: 'file-atlas-fsrs',
    name: 'filesystem.rs',
    path: '/Home/Documents/Projects/Atlas-Desktop/src/filesystem.rs',
    parentPath: '/Home/Documents/Projects/Atlas-Desktop/src',
    type: 'file',
    extension: 'rs',
    size: 18900,
    createdAt: '2026-09-08T11:00:00Z',
    modifiedAt: '2026-11-10T18:20:00Z',
    previewKind: 'code',
    codeLanguage: 'rust',
    textContent: `// High-performance asynchronous filesystem watcher and batch I/O operations
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified_timestamp: i64,
}`,
  },

  // PICTURES FOLDERS & FILES
  {
    id: 'folder-pic-wallpapers',
    name: 'Wallpapers',
    path: '/Home/Pictures/Wallpapers',
    parentPath: '/Home/Pictures',
    type: 'folder',
    size: 94000000,
    createdAt: '2026-03-01T10:00:00Z',
    modifiedAt: '2026-11-09T18:00:00Z',
    itemCount: 3,
  },
  {
    id: 'folder-pic-screenshots',
    name: 'Screenshots',
    path: '/Home/Pictures/Screenshots',
    parentPath: '/Home/Pictures',
    type: 'folder',
    size: 48000000,
    createdAt: '2026-04-10T12:00:00Z',
    modifiedAt: '2026-11-10T15:30:00Z',
    itemCount: 2,
  },
  {
    id: 'folder-pic-camera',
    name: 'Studio_Shoots',
    path: '/Home/Pictures/Studio_Shoots',
    parentPath: '/Home/Pictures',
    type: 'folder',
    size: 1800000000,
    createdAt: '2026-05-20T14:00:00Z',
    modifiedAt: '2026-10-18T16:00:00Z',
    itemCount: 2,
  },
  {
    id: 'file-wall-1',
    name: 'Obsidian_Ridge_4K.jpg',
    path: '/Home/Pictures/Wallpapers/Obsidian_Ridge_4K.jpg',
    parentPath: '/Home/Pictures/Wallpapers',
    type: 'file',
    extension: 'jpg',
    size: 14200000,
    createdAt: '2026-11-01T08:00:00Z',
    modifiedAt: '2026-11-01T08:00:00Z',
    mimeType: 'image/jpeg',
    previewKind: 'image',
    dimensions: { width: 3840, height: 2160 },
    previewUrl: 'https://images.unsplash.com/photo-1506744038136-46273834b3fb?auto=format&fit=crop&w=1200&q=80',
  },
  {
    id: 'file-wall-2',
    name: 'Nordic_Glacier_Minimal.jpg',
    path: '/Home/Pictures/Wallpapers/Nordic_Glacier_Minimal.jpg',
    parentPath: '/Home/Pictures/Wallpapers',
    type: 'file',
    extension: 'jpg',
    size: 11800000,
    createdAt: '2026-11-03T10:00:00Z',
    modifiedAt: '2026-11-03T10:00:00Z',
    mimeType: 'image/jpeg',
    previewKind: 'image',
    dimensions: { width: 3840, height: 2160 },
    previewUrl: 'https://images.unsplash.com/photo-1464822759023-fed622ff2c3b?auto=format&fit=crop&w=1200&q=80',
  },
  {
    id: 'file-wall-3',
    name: 'Cyber_Grid_Vector.png',
    path: '/Home/Pictures/Wallpapers/Cyber_Grid_Vector.png',
    parentPath: '/Home/Pictures/Wallpapers',
    type: 'file',
    extension: 'png',
    size: 8400000,
    createdAt: '2026-11-05T14:00:00Z',
    modifiedAt: '2026-11-05T14:00:00Z',
    mimeType: 'image/png',
    previewKind: 'image',
    dimensions: { width: 2560, height: 1440 },
    previewUrl: 'https://images.unsplash.com/photo-1550751827-4bd374c3f58b?auto=format&fit=crop&w=1200&q=80',
  },

  // VIDEOS
  {
    id: 'file-vid-1',
    name: 'Atlas_Feature_Reel_60fps.mp4',
    path: '/Home/Videos/Atlas_Feature_Reel_60fps.mp4',
    parentPath: '/Home/Videos',
    type: 'file',
    extension: 'mp4',
    size: 1420000000,
    createdAt: '2026-11-08T10:00:00Z',
    modifiedAt: '2026-11-08T11:20:00Z',
    mimeType: 'video/mp4',
    previewKind: 'video',
    duration: '02:45',
    dimensions: { width: 1920, height: 1080 },
    previewUrl: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ForBiggerBlazes.mp4',
  },
  {
    id: 'file-vid-2',
    name: 'Miller_Columns_Interaction_Test.mp4',
    path: '/Home/Videos/Miller_Columns_Interaction_Test.mp4',
    parentPath: '/Home/Videos',
    type: 'file',
    extension: 'mp4',
    size: 890000000,
    createdAt: '2026-11-07T14:00:00Z',
    modifiedAt: '2026-11-07T14:30:00Z',
    mimeType: 'video/mp4',
    previewKind: 'video',
    duration: '01:18',
    dimensions: { width: 1920, height: 1080 },
    previewUrl: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4',
  },
  {
    id: 'file-vid-3',
    name: 'Keynote_Presentation_Cut.mov',
    path: '/Home/Videos/Keynote_Presentation_Cut.mov',
    parentPath: '/Home/Videos',
    type: 'file',
    extension: 'mov',
    size: 2100000000,
    createdAt: '2026-11-06T09:00:00Z',
    modifiedAt: '2026-11-06T09:30:00Z',
    mimeType: 'video/quicktime',
    previewKind: 'video',
    duration: '05:12',
    dimensions: { width: 3840, height: 2160 },
  },

  // MUSIC / AUDIO
  {
    id: 'file-mus-1',
    name: 'Ambient_Focus_Flow_Master.mp3',
    path: '/Home/Music/Ambient_Focus_Flow_Master.mp3',
    parentPath: '/Home/Music',
    type: 'file',
    extension: 'mp3',
    size: 8400000,
    createdAt: '2026-11-05T12:00:00Z',
    modifiedAt: '2026-11-05T12:00:00Z',
    mimeType: 'audio/mpeg',
    previewKind: 'audio',
    duration: '03:36',
    previewUrl: 'https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3',
  },
  {
    id: 'file-mus-2',
    name: 'Synthwave_Drum_Stem_90bpm.wav',
    path: '/Home/Music/Synthwave_Drum_Stem_90bpm.wav',
    parentPath: '/Home/Music',
    type: 'file',
    extension: 'wav',
    size: 48000000,
    createdAt: '2026-11-04T15:00:00Z',
    modifiedAt: '2026-11-04T15:00:00Z',
    mimeType: 'audio/wav',
    previewKind: 'audio',
    duration: '02:15',
  },
  {
    id: 'file-mus-3',
    name: 'Podcast_Episode_48_Raw.aac',
    path: '/Home/Music/Podcast_Episode_48_Raw.aac',
    parentPath: '/Home/Music',
    type: 'file',
    extension: 'aac',
    size: 62000000,
    createdAt: '2026-11-02T18:00:00Z',
    modifiedAt: '2026-11-02T18:00:00Z',
    mimeType: 'audio/aac',
    previewKind: 'audio',
    duration: '42:10',
  },

  // TRASH ITEMS
  {
    id: 'file-trash-1',
    name: 'Old_Prototype_v1.zip',
    path: '/Trash/Old_Prototype_v1.zip',
    parentPath: '/Trash',
    type: 'file',
    extension: 'zip',
    size: 120000000,
    createdAt: '2026-10-01T10:00:00Z',
    modifiedAt: '2026-11-08T18:00:00Z',
    previewKind: 'archive',
  },
  {
    id: 'file-trash-2',
    name: 'Deprecated_Config.yml',
    path: '/Trash/Deprecated_Config.yml',
    parentPath: '/Trash',
    type: 'file',
    extension: 'yml',
    size: 4200,
    createdAt: '2026-10-12T14:00:00Z',
    modifiedAt: '2026-11-09T12:00:00Z',
    previewKind: 'code',
    codeLanguage: 'yaml',
    textContent: 'version: 1.0\ndeprecated: true\nreason: "Migrated to Tauri 2 config schema"',
  },
];
