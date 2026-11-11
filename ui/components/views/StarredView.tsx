import React, { useEffect, useState } from 'react';
import { Star, StarOff } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { FileItem } from '../../types/filesystem';
import { getFileIcon, formatBytes } from '../../utils/formatters';

export const StarredView: React.FC = () => {
  const { navigateTab, activeTabId, selectItem, setActiveItem, openContextMenu } = useAppStore();
  const [items, setItems] = useState<FileItem[]>([]);

  useEffect(() => {
    async function load() {
      const starred = await fileSystemService.getStarredItems();
      setItems(starred);
    }
    load();
  }, []);

  const handleOpen = (item: FileItem) => {
    if (item.type === 'folder') {
      navigateTab(activeTabId, item.path);
    } else {
      selectItem(item.id, false);
      setActiveItem(item);
      navigateTab(activeTabId, item.parentPath);
    }
  };

  const handleUnstar = async (e: React.MouseEvent, item: FileItem) => {
    e.stopPropagation();
    await fileSystemService.toggleStarred(item.path);
    setItems((prev) => prev.filter((i) => i.id !== item.id));
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150">
      <div className="max-w-5xl mx-auto space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
          <div className="flex items-center gap-2.5">
            <Star className="w-5 h-5 text-amber-400 fill-amber-400" />
            <div>
              <h1 className="text-lg font-bold text-[var(--text-primary)]">Starred Favorites</h1>
              <p className="text-[var(--text-secondary)]">Pinned shortcuts and high-priority assets</p>
            </div>
          </div>
          <span className="font-mono text-[var(--text-muted)] text-xs">{items.length} pinned items</span>
        </div>

        {items.length === 0 ? (
          <div className="h-64 flex flex-col items-center justify-center text-[var(--text-muted)]">
            <Star className="w-10 h-10 mb-2 stroke-[1] opacity-30" />
            <p className="text-sm font-medium text-[var(--text-secondary)]">No starred items yet</p>
            <p className="text-xs text-[var(--text-muted)] mt-0.5">
              Right-click any file or folder and select "Add to Starred"
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
            {items.map((item) => (
              <div
                key={item.id}
                onClick={() => handleOpen(item)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  openContextMenu({
                    x: e.clientX,
                    y: e.clientY,
                    type: item.type === 'folder' ? 'folder' : 'file',
                    targetItem: item,
                    targetPath: item.parentPath,
                  });
                }}
                className="p-3.5 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:border-[var(--border-strong)] hover:bg-[var(--bg-hover)] cursor-pointer transition-all flex items-center justify-between group shadow-2xs"
              >
                <div className="flex items-center gap-2.5 min-w-0 flex-1">
                  <div className="w-8 h-8 rounded-[8px] bg-[var(--bg-app)] flex items-center justify-center shrink-0">
                    {getFileIcon(item)}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="font-medium text-[var(--text-primary)] truncate">
                      {item.name}
                    </p>
                    <p className="text-[10px] text-[var(--text-muted)] font-mono truncate">
                      {item.type === 'folder' ? 'Folder' : formatBytes(item.size)}
                    </p>
                  </div>
                </div>

                <button
                  onClick={(e) => handleUnstar(e, item)}
                  title="Remove from Starred"
                  className="opacity-0 group-hover:opacity-100 text-[var(--text-muted)] hover:text-rose-400 p-1 rounded-[6px] transition-opacity"
                >
                  <StarOff className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
