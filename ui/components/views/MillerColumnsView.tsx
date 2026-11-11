import React, { useState, useEffect, useRef } from 'react';
import { ChevronRight, Folder, Loader2 } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { FileItem } from '../../types/filesystem';
import { getFileIcon, formatBytes } from '../../utils/formatters';

interface ColumnLevel {
  path: string;
  items: FileItem[];
  selectedItemPath: string | null;
  loading: boolean;
}

export const MillerColumnsView: React.FC = () => {
  const {
    currentPath,
    navigateTab,
    activeTabId,
    showHiddenFiles,
    selectItem,
    setActiveItem,
    openContextMenu,
  } = useAppStore();

  const [columns, setColumns] = useState<ColumnLevel[]>([]);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dragOverPath, setDragOverPath] = useState<string | null>(null);

  // Rebuild columns hierarchy whenever currentPath changes
  useEffect(() => {
    let isCancelled = false;

    async function loadColumns() {
      const clean = currentPath === '/' ? ['/Home'] : currentPath.split('/').filter(Boolean);
      const levelPaths: string[] = [];

      let running = '';
      for (const seg of clean) {
        running += `/${seg}`;
        levelPaths.push(running);
      }

      if (!levelPaths.some((p) => p === '/Home') && clean.length > 0) {
        levelPaths.unshift('/Home');
      }

      const newCols: ColumnLevel[] = [];

      for (let i = 0; i < levelPaths.length; i++) {
        const path = levelPaths[i];
        const nextSubPath = levelPaths[i + 1] || null;

        try {
          const items = await fileSystemService.listDirectory(path, showHiddenFiles);
          if (isCancelled) return;

          newCols.push({
            path,
            items,
            selectedItemPath: nextSubPath,
            loading: false,
          });
        } catch {
          // Path might not exist
        }
      }

      if (!isCancelled) {
        setColumns(newCols);
        setTimeout(() => {
          if (containerRef.current) {
            containerRef.current.scrollTo({
              left: containerRef.current.scrollWidth,
              behavior: 'smooth',
            });
          }
        }, 60);
      }
    }

    loadColumns();

    return () => {
      isCancelled = true;
    };
  }, [currentPath, showHiddenFiles]);

  const handleItemClick = (item: FileItem, colIndex: number) => {
    selectItem(item.id, false);
    setActiveItem(item);

    if (item.type === 'folder') {
      navigateTab(activeTabId, item.path);
    } else {
      setColumns((prev) =>
        prev.map((col, idx) =>
          idx === colIndex ? { ...col, selectedItemPath: item.path } : col
        )
      );
    }
  };

  const handleItemDoubleClick = (item: FileItem) => {
    if (item.type === 'folder') {
      navigateTab(activeTabId, item.path);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, item?: FileItem, colPath?: string) => {
    e.preventDefault();
    e.stopPropagation();
    openContextMenu({
      x: e.clientX,
      y: e.clientY,
      type: item ? (item.type === 'folder' ? 'folder' : 'file') : 'background',
      targetItem: item,
      targetPath: colPath || currentPath,
    });
  };

  const handleDragStart = (e: React.DragEvent, item: FileItem) => {
    e.dataTransfer.setData('text/plain', item.path);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent, targetFolder: FileItem) => {
    if (targetFolder.type === 'folder') {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
      setDragOverPath(targetFolder.path);
    }
  };

  const handleDrop = async (e: React.DragEvent, targetFolder: FileItem) => {
    e.preventDefault();
    setDragOverPath(null);
    const srcPath = e.dataTransfer.getData('text/plain');
    if (srcPath && srcPath !== targetFolder.path) {
      await fileSystemService.moveItems([srcPath], targetFolder.path);
      navigateTab(activeTabId, currentPath);
    }
  };

  return (
    <div
      ref={containerRef}
      onContextMenu={(e) => handleContextMenu(e, undefined, currentPath)}
      className="flex-1 flex overflow-x-auto overflow-y-hidden p-3 gap-3 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150"
    >
      {columns.length === 0 ? (
        <div className="flex-1 flex items-center justify-center text-[var(--text-muted)]">
          <Loader2 className="w-5 h-5 animate-spin mr-2" />
          <span>Traversing directory hierarchy...</span>
        </div>
      ) : (
        columns.map((column, colIndex) => {
          const colName = column.path === '/' ? 'Root' : column.path.split('/').pop();
          return (
            <div
              key={column.path}
              className="w-68 min-w-[250px] max-w-[290px] h-full flex flex-col bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] overflow-hidden shrink-0 shadow-xs transition-all duration-150"
              onContextMenu={(e) => handleContextMenu(e, undefined, column.path)}
            >
              {/* Column Header */}
              <div className="h-8 px-3.5 flex items-center justify-between border-b border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] text-[11px] font-semibold text-[var(--text-secondary)]">
                <span className="truncate">{colName}</span>
                <span className="font-mono text-[10px] text-[var(--text-muted)]">
                  {column.items.length}
                </span>
              </div>

              {/* Items List */}
              <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
                {column.items.length === 0 ? (
                  <div className="p-4 text-center text-[var(--text-muted)] text-xs italic">
                    Empty folder
                  </div>
                ) : (
                  column.items.map((item) => {
                    const isSelected =
                      column.selectedItemPath === item.path ||
                      currentPath === item.path;
                    const isDragTarget = dragOverPath === item.path;

                    return (
                      <div
                        key={item.id}
                        draggable
                        onDragStart={(e) => handleDragStart(e, item)}
                        onDragOver={(e) => handleDragOver(e, item)}
                        onDragLeave={() => setDragOverPath(null)}
                        onDrop={(e) => handleDrop(e, item)}
                        onClick={() => handleItemClick(item, colIndex)}
                        onDoubleClick={() => handleItemDoubleClick(item)}
                        onContextMenu={(e) => handleContextMenu(e, item, column.path)}
                        className={`group flex items-center justify-between px-2.5 py-1.5 rounded-[10px] cursor-pointer transition-colors text-left border ${
                          isDragTarget
                            ? 'bg-[var(--selection-bg)] border-[var(--accent)]'
                            : isSelected
                            ? 'bg-[var(--selection-bg)] border-[var(--selection-border)] text-[var(--text-primary)] font-medium shadow-2xs'
                            : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                        }`}
                      >
                        <div className="flex items-center gap-2 min-w-0 flex-1">
                          <span className="shrink-0">{getFileIcon(item)}</span>
                          <span className="truncate text-xs">{item.name}</span>
                        </div>

                        {item.type === 'folder' ? (
                          <ChevronRight
                            className={`w-3.5 h-3.5 shrink-0 transition-colors ${
                              isSelected
                                ? 'text-[var(--accent)]'
                                : 'text-[var(--text-muted)] group-hover:text-[var(--text-secondary)]'
                            }`}
                          />
                        ) : (
                          <span className="font-mono text-[10px] text-[var(--text-muted)] ml-1 shrink-0">
                            {formatBytes(item.size)}
                          </span>
                        )}
                      </div>
                    );
                  })
                )}
              </div>
            </div>
          );
        })
      )}
    </div>
  );
};
