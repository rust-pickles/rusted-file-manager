import React, { useState } from 'react';
import { Star, Eye, MoreVertical, Folder } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { FileItem } from '../../types/filesystem';
import { getFileIcon, formatBytes, getFileTypeLabel } from '../../utils/formatters';

export const GridView: React.FC = () => {
  const {
    directoryItems,
    selectedIds,
    selectItem,
    setActiveItem,
    navigateTab,
    activeTabId,
    gridDensity,
    openContextMenu,
    currentPath,
    openRenameModal,
  } = useAppStore();

  const [dragOverFolderId, setDragOverFolderId] = useState<string | null>(null);

  const cardDimensions =
    gridDensity === 'compact'
      ? 'w-28 h-28'
      : gridDensity === 'spacious'
      ? 'w-44 h-44'
      : 'w-36 h-36';

  const previewHeight =
    gridDensity === 'compact' ? 'h-16' : gridDensity === 'spacious' ? 'h-28' : 'h-20';

  const handleItemClick = (e: React.MouseEvent, item: FileItem) => {
    e.stopPropagation();
    const isMulti = e.ctrlKey || e.metaKey;
    selectItem(item.id, isMulti);
    setActiveItem(item);
  };

  const handleDoubleClick = (item: FileItem) => {
    if (item.type === 'folder') {
      navigateTab(activeTabId, item.path);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, item?: FileItem) => {
    e.preventDefault();
    e.stopPropagation();
    openContextMenu({
      x: e.clientX,
      y: e.clientY,
      type: item ? (item.type === 'folder' ? 'folder' : 'file') : 'background',
      targetItem: item,
      targetPath: currentPath,
    });
  };

  return (
    <div
      onContextMenu={(e) => handleContextMenu(e)}
      className="flex-1 overflow-y-auto p-4 bg-[var(--bg-main)] select-none transition-colors duration-150"
    >
      {directoryItems.length === 0 ? (
        <div className="h-full flex flex-col items-center justify-center text-[var(--text-muted)]">
          <Folder className="w-12 h-12 mb-2 stroke-[1] opacity-40" />
          <p className="text-sm font-medium text-[var(--text-secondary)]">Folder is empty</p>
          <p className="text-xs text-[var(--text-muted)] mt-0.5">
            Right-click or press Ctrl+Shift+N to create a folder
          </p>
        </div>
      ) : (
        <div className="flex flex-wrap gap-3 items-start">
          {directoryItems.map((item) => {
            const isSelected = selectedIds.includes(item.id);
            const isDragOver = dragOverFolderId === item.id;

            return (
              <div
                key={item.id}
                onClick={(e) => handleItemClick(e, item)}
                onDoubleClick={() => handleDoubleClick(item)}
                onContextMenu={(e) => handleContextMenu(e, item)}
                className={`group relative flex flex-col items-center rounded-[12px] p-2 cursor-pointer transition-all border ${cardDimensions} ${
                  isSelected
                    ? 'bg-[var(--selection-bg)] border-[var(--selection-border)] shadow-xs'
                    : isDragOver
                    ? 'bg-[var(--selection-bg)] border-[var(--accent)]'
                    : 'bg-[var(--bg-surface)] border-[var(--border-subtle)] hover:bg-[var(--bg-hover)] hover:border-[var(--border-strong)]'
                }`}
              >
                {/* Thumbnail / Icon Container */}
                <div
                  className={`w-full ${previewHeight} rounded-[8px] flex items-center justify-center relative overflow-hidden bg-[var(--bg-app)]/60`}
                >
                  {item.previewKind === 'image' && item.previewUrl ? (
                    <img
                      src={item.previewUrl}
                      alt={item.name}
                      referrerPolicy="no-referrer"
                      className="w-full h-full object-cover rounded-[8px]"
                    />
                  ) : item.type === 'folder' ? (
                    <Folder className="w-10 h-10 text-[var(--accent)] fill-[var(--accent)]/15" />
                  ) : (
                    <div className="flex flex-col items-center justify-center">
                      {getFileIcon(item, 'w-9 h-9')}
                      {item.extension && (
                        <span className="text-[9px] uppercase font-mono font-bold text-[var(--text-muted)] mt-1">
                          {item.extension}
                        </span>
                      )}
                    </div>
                  )}

                  {/* Star Badge */}
                  {item.isStarred && (
                    <div className="absolute top-1 left-1">
                      <Star className="w-3 h-3 text-amber-400 fill-amber-400" />
                    </div>
                  )}

                  {/* Quick Action Overlay on Hover */}
                  <div className="absolute inset-0 bg-[var(--bg-app)]/80 backdrop-blur-2xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-1 rounded-[8px]">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setActiveItem(item);
                      }}
                      title="Quick Preview"
                      className="w-6 h-6 rounded-[6px] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-primary)] border border-[var(--border-subtle)] flex items-center justify-center"
                    >
                      <Eye className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        openRenameModal(item);
                      }}
                      title="Rename"
                      className="w-6 h-6 rounded-[6px] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-primary)] border border-[var(--border-subtle)] flex items-center justify-center"
                    >
                      <MoreVertical className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {/* File Label & Meta */}
                <div className="w-full mt-1.5 text-center">
                  <p
                    title={item.name}
                    className={`text-xs truncate font-medium ${
                      isSelected ? 'text-[var(--text-primary)] font-semibold' : 'text-[var(--text-primary)]'
                    }`}
                  >
                    {item.name}
                  </p>
                  <p className="text-[10px] text-[var(--text-muted)] font-mono mt-0.5 truncate">
                    {item.type === 'folder'
                      ? `${item.itemCount || 0} items`
                      : formatBytes(item.size)}
                  </p>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
