import React, { useEffect, useState } from 'react';
import {
  Clock,
  Star,
  HardDrive,
  ArrowRight,
  Monitor,
  Download,
  FileText,
  Image,
  Video,
  Music,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { FileItem, DriveInfo } from '../../types/filesystem';
import { getFileIcon, formatBytes } from '../../utils/formatters';
import { BrandIcon } from '../common/BrandIcon';

export const HomeView: React.FC = () => {
  const { navigateTab, activeTabId, selectItem, setActiveItem } = useAppStore();
  const [recentFiles, setRecentFiles] = useState<FileItem[]>([]);
  const [starredItems, setStarredItems] = useState<FileItem[]>([]);
  const [drives, setDrives] = useState<DriveInfo[]>([]);

  useEffect(() => {
    async function loadData() {
      const recents = await fileSystemService.getRecentFiles(8);
      const starred = await fileSystemService.getStarredItems();
      const overview = await fileSystemService.getStorageOverview();
      setRecentFiles(recents);
      setStarredItems(starred);
      setDrives(overview.drives);
    }
    loadData();
  }, []);

  const handleOpenItem = (item: FileItem) => {
    if (item.type === 'folder') {
      navigateTab(activeTabId, item.path);
    } else {
      selectItem(item.id, false);
      setActiveItem(item);
      navigateTab(activeTabId, item.parentPath);
    }
  };

  const quickFolders = [
    { name: 'Desktop', path: '/Home/Desktop', icon: Monitor },
    { name: 'Downloads', path: '/Home/Downloads', icon: Download },
    { name: 'Documents', path: '/Home/Documents', icon: FileText },
    { name: 'Pictures', path: '/Home/Pictures', icon: Image },
    { name: 'Videos', path: '/Home/Videos', icon: Video },
    { name: 'Music', path: '/Home/Music', icon: Music },
  ];

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* Welcome Header */}
        <div className="flex items-center justify-between pb-4 border-b border-[var(--border-subtle)]">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--accent)] shadow-xs">
              <BrandIcon size={22} />
            </div>
            <div>
              <h1 className="text-xl font-bold text-[var(--text-primary)] tracking-tight">
                RUSTED Workspace
              </h1>
              <p className="text-[var(--text-secondary)] mt-0.5">
                Rusted File Manager — Modern, high-performance desktop filesystem
              </p>
            </div>
          </div>

          <button
            onClick={() => navigateTab(activeTabId, '/Home/Storage')}
            className="flex items-center gap-1.5 px-3.5 py-2 rounded-[10px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:bg-[var(--bg-hover)] text-[var(--text-primary)] transition-colors shadow-2xs"
          >
            <HardDrive className="w-3.5 h-3.5 text-[var(--accent)]" />
            <span>Manage Storage</span>
          </button>
        </div>

        {/* Quick Locations Grid */}
        <div>
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3">
            Quick Locations
          </h2>
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-2.5">
            {quickFolders.map((qf) => {
              const Icon = qf.icon;
              return (
                <button
                  key={qf.path}
                  onClick={() => navigateTab(activeTabId, qf.path)}
                  className="flex flex-col items-center justify-center p-3.5 rounded-[14px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:bg-[var(--bg-hover)] hover:border-[var(--border-strong)] transition-all text-center group cursor-pointer shadow-2xs"
                >
                  <div className="w-10 h-10 rounded-[10px] bg-[var(--bg-app)] flex items-center justify-center mb-2.5 group-hover:scale-105 transition-transform text-[var(--accent)]">
                    <Icon className="w-5 h-5 opacity-90" />
                  </div>
                  <span className="font-medium text-[var(--text-primary)] text-xs">{qf.name}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Two Columns: Recent Files & Starred Items */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {/* Recent Files */}
          <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] p-4 shadow-xs">
            <div className="flex items-center justify-between mb-3 pb-2 border-b border-[var(--border-subtle)]">
              <div className="flex items-center gap-2">
                <Clock className="w-4 h-4 text-[var(--accent)]" />
                <h3 className="font-semibold text-[var(--text-primary)]">Recent Files</h3>
              </div>
              <button
                onClick={() => navigateTab(activeTabId, '/Home/Recent')}
                className="text-[var(--text-muted)] hover:text-[var(--text-primary)] flex items-center gap-1 text-[11px] transition-colors"
              >
                <span>View all</span>
                <ArrowRight className="w-3 h-3" />
              </button>
            </div>

            <div className="space-y-1">
              {recentFiles.map((file) => (
                <div
                  key={file.id}
                  onClick={() => handleOpenItem(file)}
                  className="flex items-center justify-between p-2 rounded-[8px] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors group"
                >
                  <div className="flex items-center gap-2.5 min-w-0 flex-1">
                    <span className="shrink-0">{getFileIcon(file)}</span>
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-[var(--text-primary)] font-medium">
                        {file.name}
                      </p>
                      <p className="truncate text-[10px] text-[var(--text-muted)] font-mono">
                        {file.parentPath}
                      </p>
                    </div>
                  </div>
                  <span className="text-[10px] text-[var(--text-muted)] font-mono shrink-0 ml-2">
                    {formatBytes(file.size)}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Starred Folders & Assets */}
          <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] p-4 shadow-xs">
            <div className="flex items-center justify-between mb-3 pb-2 border-b border-[var(--border-subtle)]">
              <div className="flex items-center gap-2">
                <Star className="w-4 h-4 text-amber-400 fill-amber-400" />
                <h3 className="font-semibold text-[var(--text-primary)]">Starred Locations & Files</h3>
              </div>
              <button
                onClick={() => navigateTab(activeTabId, '/Home/Starred')}
                className="text-[var(--text-muted)] hover:text-[var(--text-primary)] flex items-center gap-1 text-[11px] transition-colors"
              >
                <span>View all</span>
                <ArrowRight className="w-3 h-3" />
              </button>
            </div>

            <div className="space-y-1">
              {starredItems.slice(0, 7).map((item) => (
                <div
                  key={item.id}
                  onClick={() => handleOpenItem(item)}
                  className="flex items-center justify-between p-2 rounded-[8px] hover:bg-[var(--bg-hover)] cursor-pointer transition-colors group"
                >
                  <div className="flex items-center gap-2.5 min-w-0 flex-1">
                    <span className="shrink-0">{getFileIcon(item)}</span>
                    <span className="truncate text-[var(--text-primary)] font-medium">
                      {item.name}
                    </span>
                  </div>
                  <span className="text-[10px] text-[var(--text-muted)] font-mono shrink-0">
                    {item.type === 'folder' ? 'Folder' : formatBytes(item.size)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Drives Quick Overview */}
        <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] p-4 shadow-xs">
          <h3 className="font-semibold text-[var(--text-primary)] mb-3">Storage Devices</h3>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            {drives.map((d) => {
              const pct = Math.round((d.usedBytes / d.totalBytes) * 100);
              return (
                <div
                  key={d.id}
                  onClick={() => navigateTab(activeTabId, '/Home/Storage')}
                  className="p-3.5 rounded-[12px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)] hover:border-[var(--border-strong)] cursor-pointer transition-colors"
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium text-[var(--text-primary)] truncate">{d.name}</span>
                    <span className="text-[10px] font-mono text-[var(--text-secondary)]">{pct}%</span>
                  </div>
                  <div className="w-full h-1.5 bg-[var(--bg-hover)] rounded-full overflow-hidden mb-2">
                    <div
                      className={`h-full rounded-full transition-all ${
                        pct > 85 ? 'bg-amber-500' : 'bg-[var(--accent)]'
                      }`}
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                  <div className="flex justify-between text-[10px] text-[var(--text-muted)] font-mono">
                    <span>{formatBytes(d.freeBytes)} free</span>
                    <span>{formatBytes(d.totalBytes)}</span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
