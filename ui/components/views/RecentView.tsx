import React, { useEffect, useState } from 'react';
import { Clock } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { FileItem } from '../../types/filesystem';
import { getFileIcon, formatBytes, formatDate } from '../../utils/formatters';

export const RecentView: React.FC = () => {
  const { navigateTab, activeTabId, selectItem, setActiveItem, openContextMenu } = useAppStore();
  const [files, setFiles] = useState<FileItem[]>([]);
  const [filterType, setFilterType] = useState<string>('all');

  useEffect(() => {
    async function load() {
      const recents = await fileSystemService.getRecentFiles(30);
      setFiles(recents);
    }
    load();
  }, []);

  const filtered = files.filter((f) => {
    if (filterType === 'all') return true;
    if (filterType === 'code') return f.previewKind === 'code';
    if (filterType === 'media') return ['image', 'video', 'audio'].includes(f.previewKind || '');
    if (filterType === 'docs') return ['document', 'pdf', 'markdown'].includes(f.previewKind || '');
    return true;
  });

  const handleOpen = (item: FileItem) => {
    selectItem(item.id, false);
    setActiveItem(item);
    navigateTab(activeTabId, item.parentPath);
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150">
      <div className="max-w-5xl mx-auto space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
          <div className="flex items-center gap-2.5">
            <Clock className="w-5 h-5 text-[var(--accent)]" />
            <div>
              <h1 className="text-lg font-bold text-[var(--text-primary)]">Recent Files</h1>
              <p className="text-[var(--text-secondary)]">Recently modified and accessed workspace items</p>
            </div>
          </div>

          {/* Filter Pills */}
          <div className="flex items-center gap-1 bg-[var(--bg-surface)] p-1 rounded-[10px] border border-[var(--border-subtle)]">
            {['all', 'code', 'media', 'docs'].map((t) => (
              <button
                key={t}
                onClick={() => setFilterType(t)}
                className={`px-3 py-1 rounded-[8px] capitalize text-xs transition-colors ${
                  filterType === t
                    ? 'bg-[var(--bg-hover)] text-[var(--text-primary)] font-semibold'
                    : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                {t}
              </button>
            ))}
          </div>
        </div>

        {/* Table of Recents */}
        <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] overflow-hidden shadow-xs">
          <div className="h-8 border-b border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] flex items-center px-4 font-semibold text-[var(--text-secondary)] text-[11px]">
            <div className="flex-1">Name</div>
            <div className="w-48 hidden sm:block">Path</div>
            <div className="w-24 text-right">Size</div>
            <div className="w-40 text-right">Last Modified</div>
          </div>

          <div className="divide-y divide-[var(--border-subtle)]/40">
            {filtered.map((file) => (
              <div
                key={file.id}
                onClick={() => handleOpen(file)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  openContextMenu({
                    x: e.clientX,
                    y: e.clientY,
                    type: 'file',
                    targetItem: file,
                    targetPath: file.parentPath,
                  });
                }}
                className="flex items-center px-4 py-2 hover:bg-[var(--bg-hover)] cursor-pointer transition-colors group"
              >
                <div className="flex-1 flex items-center gap-2.5 min-w-0 pr-2">
                  <span className="shrink-0">{getFileIcon(file)}</span>
                  <span className="truncate text-[var(--text-primary)] font-medium">
                    {file.name}
                  </span>
                </div>

                <div className="w-48 hidden sm:block text-[var(--text-muted)] font-mono text-[10px] truncate">
                  {file.parentPath}
                </div>

                <div className="w-24 text-right font-mono text-[var(--text-secondary)] text-[11px] shrink-0">
                  {formatBytes(file.size)}
                </div>

                <div className="w-40 text-right font-mono text-[var(--text-muted)] text-[11px] shrink-0">
                  {formatDate(file.modifiedAt)}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
