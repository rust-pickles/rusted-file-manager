import React from 'react';
import { useAppStore } from '../../store/useAppStore';
import { FileItem } from '../../types/filesystem';
import { getFileIcon, formatBytes } from '../../utils/formatters';

export const CompactView: React.FC = () => {
  const {
    directoryItems,
    selectedIds,
    selectItem,
    setActiveItem,
    navigateTab,
    activeTabId,
    openContextMenu,
    currentPath,
  } = useAppStore();

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
      className="flex-1 overflow-auto p-3 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150"
    >
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-1.5 p-3 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] shadow-xs">
        {directoryItems.map((item) => {
          const isSelected = selectedIds.includes(item.id);

          return (
            <div
              key={item.id}
              onClick={(e) => handleItemClick(e, item)}
              onDoubleClick={() => handleDoubleClick(item)}
              onContextMenu={(e) => handleContextMenu(e, item)}
              className={`flex items-center justify-between px-2.5 py-1.5 rounded-[8px] cursor-pointer transition-colors border ${
                isSelected
                  ? 'bg-[var(--selection-bg)] border-[var(--selection-border)] text-[var(--text-primary)] font-medium shadow-2xs'
                  : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              <div className="flex items-center gap-1.5 min-w-0 flex-1">
                <span className="shrink-0">{getFileIcon(item, 'w-3.5 h-3.5')}</span>
                <span className="truncate text-xs">{item.name}</span>
              </div>

              <span className="font-mono text-[10px] text-[var(--text-muted)] ml-1.5 shrink-0">
                {item.type === 'folder'
                  ? `${item.itemCount || 0}`
                  : formatBytes(item.size, 0)}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
};
