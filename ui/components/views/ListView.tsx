import React from 'react';
import {
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Star,
  Folder,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { FileItem, SortField } from '../../types/filesystem';
import {
  getFileIcon,
  formatBytes,
  formatDate,
  getFileTypeLabel,
} from '../../utils/formatters';

export const ListView: React.FC = () => {
  const {
    directoryItems,
    selectedIds,
    selectItem,
    setActiveItem,
    navigateTab,
    activeTabId,
    tabs,
    setSort,
    openContextMenu,
    currentPath,
  } = useAppStore();

  const activeTab = tabs.find((t) => t.id === activeTabId);
  const currentSortField = activeTab?.sortField || 'name';
  const currentSortOrder = activeTab?.sortOrder || 'asc';

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

  const renderSortIndicator = (field: SortField) => {
    if (currentSortField !== field) {
      return <ArrowUpDown className="w-3 h-3 text-[var(--text-muted)] ml-1 opacity-0 group-hover:opacity-100" />;
    }
    return currentSortOrder === 'asc' ? (
      <ArrowUp className="w-3 h-3 text-[var(--accent)] ml-1" />
    ) : (
      <ArrowDown className="w-3 h-3 text-[var(--accent)] ml-1" />
    );
  };

  return (
    <div
      onContextMenu={(e) => handleContextMenu(e)}
      className="flex-1 flex flex-col p-3 bg-[var(--bg-main)] overflow-hidden text-xs select-none transition-colors duration-150"
    >
      <div className="flex-1 flex flex-col bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] overflow-hidden shadow-xs">
        {/* Table Header */}
        <div className="h-8 border-b border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] flex items-center px-4 font-semibold text-[var(--text-secondary)] shrink-0 text-[11px]">
          <div
            onClick={() => setSort('name')}
            className="group flex-1 flex items-center cursor-pointer hover:text-[var(--text-primary)]"
          >
            <span>Name</span>
            {renderSortIndicator('name')}
          </div>

          <div
            onClick={() => setSort('type')}
            className="group w-32 hidden sm:flex items-center cursor-pointer hover:text-[var(--text-primary)]"
          >
            <span>Kind</span>
            {renderSortIndicator('type')}
          </div>

          <div
            onClick={() => setSort('size')}
            className="group w-28 flex items-center justify-end cursor-pointer hover:text-[var(--text-primary)]"
          >
            <span>Size</span>
            {renderSortIndicator('size')}
          </div>

          <div
            onClick={() => setSort('modifiedAt')}
            className="group w-44 hidden md:flex items-center justify-end cursor-pointer hover:text-[var(--text-primary)]"
          >
            <span>Date Modified</span>
            {renderSortIndicator('modifiedAt')}
          </div>
        </div>

        {/* Table Body */}
        <div className="flex-1 overflow-y-auto divide-y divide-[var(--border-subtle)]/40 p-1">
          {directoryItems.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-[var(--text-muted)] py-12">
              <Folder className="w-10 h-10 mb-2 stroke-[1] opacity-40" />
              <p className="text-xs text-[var(--text-secondary)]">Empty directory</p>
            </div>
          ) : (
            directoryItems.map((item) => {
              const isSelected = selectedIds.includes(item.id);

              return (
                <div
                  key={item.id}
                  onClick={(e) => handleItemClick(e, item)}
                  onDoubleClick={() => handleDoubleClick(item)}
                  onContextMenu={(e) => handleContextMenu(e, item)}
                  className={`flex items-center px-3 py-1.5 rounded-[8px] cursor-pointer transition-colors border ${
                    isSelected
                      ? 'bg-[var(--selection-bg)] border-[var(--selection-border)] text-[var(--text-primary)] font-medium shadow-2xs'
                      : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  {/* Name & Icon */}
                  <div className="flex-1 flex items-center gap-2.5 min-w-0 pr-2">
                    <span className="shrink-0">{getFileIcon(item)}</span>
                    <span className="truncate">{item.name}</span>
                    {item.isStarred && (
                      <Star className="w-3 h-3 text-amber-400 fill-amber-400 shrink-0" />
                    )}
                  </div>

                  {/* Kind */}
                  <div className="w-32 hidden sm:block text-[var(--text-muted)] truncate text-[11px]">
                    {getFileTypeLabel(item)}
                  </div>

                  {/* Size */}
                  <div className="w-28 text-right font-mono text-[var(--text-muted)] text-[11px] shrink-0">
                    {item.type === 'folder'
                      ? `${item.itemCount || 0} items`
                      : formatBytes(item.size)}
                  </div>

                  {/* Date Modified */}
                  <div className="w-44 hidden md:block text-right font-mono text-[var(--text-muted)] text-[11px] shrink-0">
                    {formatDate(item.modifiedAt)}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
