import React, { useEffect, useRef } from 'react';
import {
  FolderOpen,
  ExternalLink,
  Edit2,
  Copy,
  Scissors,
  Trash2,
  Star,
  StarOff,
  Info,
  FolderPlus,
  FilePlus,
  ClipboardPaste,
  RotateCw,
  CheckSquare,
  Eye,
  FileDown,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';

export const ContextMenu: React.FC = () => {
  const {
    contextMenu,
    closeContextMenu,
    navigateTab,
    activeTabId,
    createTab,
    openRenameModal,
    openDeleteModal,
    openPropertiesModal,
    openNewFolderModal,
    copySelection,
    cutSelection,
    pasteToPath,
    clipboard,
    refreshTab,
    selectAll,
    directoryItems,
    currentPath,
    setActiveItem,
    selectItem,
  } = useAppStore();

  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleOutsideClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        closeContextMenu();
      }
    };

    window.addEventListener('mousedown', handleOutsideClick);
    window.addEventListener('scroll', closeContextMenu, true);
    window.addEventListener('resize', closeContextMenu);

    return () => {
      window.removeEventListener('mousedown', handleOutsideClick);
      window.removeEventListener('scroll', closeContextMenu, true);
      window.removeEventListener('resize', closeContextMenu);
    };
  }, [closeContextMenu]);

  if (!contextMenu) return null;

  const { x, y, type, targetItem, targetPath } = contextMenu;

  const menuWidth = 210;
  const menuHeight = 290;
  const safeX = Math.min(x, window.innerWidth - menuWidth - 12);
  const safeY = Math.min(y, window.innerHeight - menuHeight - 12);

  const handleOpen = () => {
    if (!targetItem) return;
    if (targetItem.type === 'folder') {
      navigateTab(activeTabId, targetItem.path);
    } else {
      selectItem(targetItem.id, false);
      setActiveItem(targetItem);
    }
    closeContextMenu();
  };

  const handleOpenNewTab = () => {
    if (targetItem && targetItem.type === 'folder') {
      createTab(targetItem.path, targetItem.name);
    }
    closeContextMenu();
  };

  const handleToggleStar = async () => {
    if (targetItem) {
      await fileSystemService.toggleStarred(targetItem.path);
      refreshTab();
    }
    closeContextMenu();
  };

  const handleCopy = () => {
    if (targetItem) {
      selectItem(targetItem.id, false);
      copySelection();
    }
    closeContextMenu();
  };

  const handleCut = () => {
    if (targetItem) {
      selectItem(targetItem.id, false);
      cutSelection();
    }
    closeContextMenu();
  };

  const handlePaste = async () => {
    await pasteToPath(targetPath || currentPath);
    closeContextMenu();
  };

  const handleDuplicate = async () => {
    if (targetItem) {
      await fileSystemService.duplicateItem(targetItem.path);
      refreshTab();
    }
    closeContextMenu();
  };

  const handleDelete = () => {
    if (targetItem) {
      openDeleteModal([targetItem]);
    }
    closeContextMenu();
  };

  const handleRename = () => {
    if (targetItem) {
      openRenameModal(targetItem);
    }
    closeContextMenu();
  };

  const handleCopyPath = () => {
    if (targetItem) {
      navigator.clipboard.writeText(targetItem.path);
    } else if (targetPath) {
      navigator.clipboard.writeText(targetPath);
    }
    closeContextMenu();
  };

  const handleProperties = () => {
    if (targetItem) {
      openPropertiesModal(targetItem);
    }
    closeContextMenu();
  };

  const handleNewFile = async () => {
    await fileSystemService.createFile(targetPath || currentPath, 'new_document.txt', '');
    refreshTab();
    closeContextMenu();
  };

  return (
    <div
      ref={menuRef}
      style={{ left: safeX, top: safeY }}
      className="fixed z-50 w-54 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[12px] shadow-2xl p-1 text-xs text-[var(--text-primary)] select-none animate-in fade-in duration-75 divide-y divide-[var(--border-subtle)]"
    >
      {/* Target Item Context Menu (File or Folder) */}
      {targetItem && (
        <>
          <div className="space-y-0.5 pb-1">
            <button
              onClick={handleOpen}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <FolderOpen className="w-3.5 h-3.5 text-[var(--accent)]" />
                <span>Open</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">↵</span>
            </button>

            {targetItem.type === 'folder' && (
              <button
                onClick={handleOpenNewTab}
                className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
              >
                <ExternalLink className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Open in New Tab</span>
              </button>
            )}

            {targetItem.type === 'file' && (
              <button
                onClick={() => {
                  setActiveItem(targetItem);
                  closeContextMenu();
                }}
                className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
              >
                <div className="flex items-center gap-2">
                  <Eye className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                  <span>Preview</span>
                </div>
                <span className="text-[10px] text-[var(--text-muted)] font-mono">Space</span>
              </button>
            )}
          </div>

          <div className="space-y-0.5 py-1">
            <button
              onClick={handleCut}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <Scissors className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Cut</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+X</span>
            </button>

            <button
              onClick={handleCopy}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <Copy className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Copy</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+C</span>
            </button>

            {targetItem.type === 'file' && (
              <button
                onClick={handleDuplicate}
                className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
              >
                <div className="flex items-center gap-2">
                  <FileDown className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                  <span>Duplicate</span>
                </div>
                <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+D</span>
              </button>
            )}

            <button
              onClick={handleRename}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <Edit2 className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Rename</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">F2</span>
            </button>
          </div>

          <div className="space-y-0.5 py-1">
            <button
              onClick={handleToggleStar}
              className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              {targetItem.isStarred ? (
                <>
                  <StarOff className="w-3.5 h-3.5 text-amber-400" />
                  <span>Remove from Starred</span>
                </>
              ) : (
                <>
                  <Star className="w-3.5 h-3.5 text-amber-400" />
                  <span>Add to Starred</span>
                </>
              )}
            </button>

            <button
              onClick={handleCopyPath}
              className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <Copy className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
              <span>Copy Full Path</span>
            </button>

            <button
              onClick={handleProperties}
              className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <Info className="w-3.5 h-3.5 text-[var(--accent)]" />
              <span>Properties</span>
            </button>
          </div>

          <div className="pt-1">
            <button
              onClick={handleDelete}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-rose-500/20 text-rose-400 transition-colors"
            >
              <div className="flex items-center gap-2">
                <Trash2 className="w-3.5 h-3.5" />
                <span>Delete</span>
              </div>
              <span className="text-[10px] font-mono">Del</span>
            </button>
          </div>
        </>
      )}

      {/* Background Context Menu */}
      {type === 'background' && (
        <>
          <div className="space-y-0.5 pb-1">
            <button
              onClick={() => {
                openNewFolderModal(targetPath || currentPath);
                closeContextMenu();
              }}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <FolderPlus className="w-3.5 h-3.5 text-[var(--accent)]" />
                <span>New Folder</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+Shift+N</span>
            </button>

            <button
              onClick={handleNewFile}
              className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <FilePlus className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
              <span>New Text Document</span>
            </button>
          </div>

          <div className="space-y-0.5 py-1">
            <button
              onClick={handlePaste}
              disabled={!clipboard || clipboard.items.length === 0}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] disabled:opacity-40 transition-colors"
            >
              <div className="flex items-center gap-2">
                <ClipboardPaste className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Paste {clipboard?.items.length ? `(${clipboard.items.length})` : ''}</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+V</span>
            </button>

            <button
              onClick={() => {
                refreshTab();
                closeContextMenu();
              }}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <RotateCw className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Refresh</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+R</span>
            </button>
          </div>

          <div className="pt-1">
            <button
              onClick={() => {
                selectAll(directoryItems.map((i) => i.id));
                closeContextMenu();
              }}
              className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)] transition-colors"
            >
              <div className="flex items-center gap-2">
                <CheckSquare className="w-3.5 h-3.5 text-[var(--text-secondary)]" />
                <span>Select All</span>
              </div>
              <span className="text-[10px] text-[var(--text-muted)] font-mono">Ctrl+A</span>
            </button>
          </div>
        </>
      )}
    </div>
  );
};
