import React, { useState, useEffect, useRef } from 'react';
import { FolderPlus, Edit3, Trash2, Info, X } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { formatBytes, formatDate, getFileTypeLabel, getFileIcon } from '../../utils/formatters';

export const ActionDialogs: React.FC = () => {
  const {
    isNewFolderModalOpen,
    closeNewFolderModal,
    isRenameModalOpen,
    itemToRename,
    closeRenameModal,
    isDeleteModalOpen,
    itemsToDelete,
    closeDeleteModal,
    isPropertiesModalOpen,
    propertiesTargetItem,
    closePropertiesModal,
    currentPath,
    refreshTab,
    clearSelection,
  } = useAppStore();

  const [folderName, setFolderName] = useState('New Folder');
  const folderInputRef = useRef<HTMLInputElement>(null);

  const [newName, setNewName] = useState('');
  const renameInputRef = useRef<HTMLInputElement>(null);

  const [permanentDelete, setPermanentDelete] = useState(false);

  useEffect(() => {
    if (isNewFolderModalOpen) {
      setFolderName('New Folder');
      setTimeout(() => {
        folderInputRef.current?.focus();
        folderInputRef.current?.select();
      }, 50);
    }
  }, [isNewFolderModalOpen]);

  useEffect(() => {
    if (isRenameModalOpen && itemToRename) {
      setNewName(itemToRename.name);
      setTimeout(() => {
        renameInputRef.current?.focus();
        if (itemToRename.type === 'file' && itemToRename.name.includes('.')) {
          const dotIdx = itemToRename.name.lastIndexOf('.');
          renameInputRef.current?.setSelectionRange(0, dotIdx);
        } else {
          renameInputRef.current?.select();
        }
      }, 50);
    }
  }, [isRenameModalOpen, itemToRename]);

  const handleCreateFolder = async (e: React.FormEvent) => {
    e.preventDefault();
    if (folderName.trim()) {
      await fileSystemService.createFolder(currentPath, folderName.trim());
      closeNewFolderModal();
      refreshTab();
    }
  };

  const handleRename = async (e: React.FormEvent) => {
    e.preventDefault();
    if (itemToRename && newName.trim()) {
      try {
        await fileSystemService.renameItem(itemToRename.path, newName.trim());
        closeRenameModal();
        refreshTab();
      } catch (err: unknown) {
        alert(err instanceof Error ? err.message : 'Error renaming item');
      }
    }
  };

  const handleDelete = async () => {
    for (const item of itemsToDelete) {
      await fileSystemService.deleteItem(item.path, permanentDelete);
    }
    clearSelection();
    closeDeleteModal();
    refreshTab();
  };

  return (
    <>
      {/* 1. New Folder Modal */}
      {isNewFolderModalOpen && (
        <div
          onClick={closeNewFolderModal}
          className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none animate-in fade-in"
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-sm bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[18px] shadow-2xl p-5 text-xs text-[var(--text-primary)] transition-colors duration-150"
          >
            <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
              <div className="flex items-center gap-2 font-semibold">
                <FolderPlus className="w-4 h-4 text-[var(--accent)]" />
                <span>Create New Folder</span>
              </div>
              <button
                onClick={closeNewFolderModal}
                className="w-6 h-6 rounded-[6px] flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreateFolder} className="mt-4 space-y-4">
              <div>
                <label className="text-[var(--text-secondary)] block mb-1.5 font-medium">Folder Name:</label>
                <input
                  ref={folderInputRef}
                  type="text"
                  value={folderName}
                  onChange={(e) => setFolderName(e.target.value)}
                  className="w-full h-8 px-3 rounded-[10px] bg-[var(--bg-app)] border border-[var(--border-subtle)] text-[var(--text-primary)] text-xs focus:border-[var(--accent)] focus:outline-none"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={closeNewFolderModal}
                  className="px-3.5 py-1.5 rounded-[10px] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 rounded-[10px] bg-[var(--accent)] text-[var(--accent-text)] font-semibold transition-colors hover:opacity-90 shadow-2xs"
                >
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* 2. Rename Modal */}
      {isRenameModalOpen && itemToRename && (
        <div
          onClick={closeRenameModal}
          className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none animate-in fade-in"
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-sm bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[18px] shadow-2xl p-5 text-xs text-[var(--text-primary)] transition-colors duration-150"
          >
            <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
              <div className="flex items-center gap-2 font-semibold">
                <Edit3 className="w-4 h-4 text-[var(--accent)]" />
                <span>Rename {itemToRename.type === 'folder' ? 'Folder' : 'File'}</span>
              </div>
              <button
                onClick={closeRenameModal}
                className="w-6 h-6 rounded-[6px] flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleRename} className="mt-4 space-y-4">
              <div>
                <label className="text-[var(--text-secondary)] block mb-1.5 font-medium">New Name:</label>
                <input
                  ref={renameInputRef}
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  className="w-full h-8 px-3 rounded-[10px] bg-[var(--bg-app)] border border-[var(--border-subtle)] text-[var(--text-primary)] text-xs focus:border-[var(--accent)] focus:outline-none"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={closeRenameModal}
                  className="px-3.5 py-1.5 rounded-[10px] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 rounded-[10px] bg-[var(--accent)] text-[var(--accent-text)] font-semibold transition-colors hover:opacity-90 shadow-2xs"
                >
                  Rename
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* 3. Delete Confirmation Modal */}
      {isDeleteModalOpen && itemsToDelete.length > 0 && (
        <div
          onClick={closeDeleteModal}
          className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none animate-in fade-in"
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-sm bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[18px] shadow-2xl p-5 text-xs text-[var(--text-primary)] transition-colors duration-150"
          >
            <div className="flex items-center gap-3 pb-3 border-b border-[var(--border-subtle)]">
              <div className="w-8 h-8 rounded-full bg-rose-500/15 flex items-center justify-center shrink-0 text-rose-400">
                <Trash2 className="w-4 h-4" />
              </div>
              <div>
                <span className="font-semibold text-sm">
                  Delete {itemsToDelete.length} {itemsToDelete.length === 1 ? 'item' : 'items'}?
                </span>
                <p className="text-[11px] text-[var(--text-muted)] mt-0.5">
                  Are you sure you want to remove these items?
                </p>
              </div>
            </div>

            <div className="my-3 max-h-32 overflow-y-auto space-y-1 bg-[var(--bg-app)] p-2 rounded-[10px] border border-[var(--border-subtle)]">
              {itemsToDelete.map((it) => (
                <div key={it.id} className="flex items-center gap-2 text-[var(--text-secondary)]">
                  <span className="shrink-0">{getFileIcon(it, 'w-3 h-3')}</span>
                  <span className="truncate">{it.name}</span>
                </div>
              ))}
            </div>

            <label className="flex items-center gap-2 cursor-pointer text-[var(--text-secondary)] hover:text-[var(--text-primary)] py-1">
              <input
                type="checkbox"
                checked={permanentDelete}
                onChange={(e) => setPermanentDelete(e.target.checked)}
                className="rounded-[4px] border-[var(--border-strong)] bg-[var(--bg-app)] text-rose-500"
              />
              <span>Delete permanently (bypass Trash)</span>
            </label>

            <div className="flex items-center justify-end gap-2 pt-3 border-t border-[var(--border-subtle)] mt-2">
              <button
                onClick={closeDeleteModal}
                className="px-3.5 py-1.5 rounded-[10px] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleDelete}
                className="px-4 py-1.5 rounded-[10px] bg-rose-600 hover:bg-rose-500 text-white font-semibold transition-colors shadow-2xs"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 4. Properties Sheet Modal */}
      {isPropertiesModalOpen && propertiesTargetItem && (
        <div
          onClick={closePropertiesModal}
          className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none animate-in fade-in"
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-md bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[18px] shadow-2xl p-5 text-xs text-[var(--text-primary)] transition-colors duration-150"
          >
            <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
              <div className="flex items-center gap-2 font-semibold">
                <Info className="w-4 h-4 text-[var(--accent)]" />
                <span>Properties: {propertiesTargetItem.name}</span>
              </div>
              <button
                onClick={closePropertiesModal}
                className="w-6 h-6 rounded-[6px] flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="py-4 flex items-center gap-3">
              <div className="w-12 h-12 rounded-[12px] bg-[var(--bg-app)] flex items-center justify-center shrink-0 border border-[var(--border-subtle)]">
                {getFileIcon(propertiesTargetItem, 'w-7 h-7')}
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-sm font-bold truncate">
                  {propertiesTargetItem.name}
                </p>
                <p className="text-[var(--text-muted)] font-mono text-[11px] mt-0.5">
                  {getFileTypeLabel(propertiesTargetItem)}
                </p>
              </div>
            </div>

            <div className="space-y-2 border-t border-[var(--border-subtle)] pt-3 font-mono text-[11px]">
              <div className="flex justify-between py-1 border-b border-[var(--border-subtle)]/40">
                <span className="text-[var(--text-muted)]">Full Path:</span>
                <span className="text-[var(--text-secondary)] truncate max-w-xs">{propertiesTargetItem.path}</span>
              </div>
              <div className="flex justify-between py-1 border-b border-[var(--border-subtle)]/40">
                <span className="text-[var(--text-muted)]">Size:</span>
                <span className="text-[var(--text-primary)] font-semibold">
                  {formatBytes(propertiesTargetItem.size)} ({propertiesTargetItem.size.toLocaleString()} bytes)
                </span>
              </div>
              <div className="flex justify-between py-1 border-b border-[var(--border-subtle)]/40">
                <span className="text-[var(--text-muted)]">Modified:</span>
                <span className="text-[var(--text-secondary)]">{formatDate(propertiesTargetItem.modifiedAt)}</span>
              </div>
              <div className="flex justify-between py-1 border-b border-[var(--border-subtle)]/40">
                <span className="text-[var(--text-muted)]">Created:</span>
                <span className="text-[var(--text-secondary)]">{formatDate(propertiesTargetItem.createdAt)}</span>
              </div>
              <div className="flex justify-between py-1 border-b border-[var(--border-subtle)]/40">
                <span className="text-[var(--text-muted)]">Permissions:</span>
                <span className="text-emerald-400">Read / Write (0644)</span>
              </div>
              <div className="flex justify-between py-1">
                <span className="text-[var(--text-muted)]">Device:</span>
                <span className="text-[var(--text-secondary)]">Rusted System (C:)</span>
              </div>
            </div>

            <div className="flex justify-end pt-4 border-t border-[var(--border-subtle)] mt-4">
              <button
                onClick={closePropertiesModal}
                className="px-4 py-1.5 rounded-[10px] bg-[var(--bg-hover)] text-[var(--text-primary)] border border-[var(--border-subtle)] font-medium transition-colors hover:bg-[var(--bg-active)]"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
