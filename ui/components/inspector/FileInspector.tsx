import React, { useState } from 'react';
import {
  X,
  Star,
  StarOff,
  Copy,
  ExternalLink,
  Play,
  Pause,
  Folder,
  Info,
  Layers,
  Check,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { getFileIcon, formatBytes, formatDate, getFileTypeLabel } from '../../utils/formatters';

export const FileInspector: React.FC = () => {
  const {
    activeItem,
    isInspectorOpen,
    toggleInspector,
    createTab,
    openPropertiesModal,
    setActiveItem,
  } = useAppStore();

  const [copiedPath, setCopiedPath] = useState(false);
  const [isPlayingAudio, setIsPlayingAudio] = useState(false);

  if (!isInspectorOpen) return null;

  const handleCopyPath = () => {
    if (activeItem) {
      navigator.clipboard.writeText(activeItem.path);
      setCopiedPath(true);
      setTimeout(() => setCopiedPath(false), 1500);
    }
  };

  const handleToggleStar = async () => {
    if (activeItem) {
      const updated = await fileSystemService.toggleStarred(activeItem.path);
      setActiveItem(updated);
    }
  };

  return (
    <aside className="w-80 bg-[var(--bg-sidebar)] border-l border-[var(--border-subtle)] flex flex-col select-none shrink-0 text-xs text-[var(--text-primary)] overflow-hidden transition-colors duration-150">
      {/* Header */}
      <div className="h-9 px-3.5 flex items-center justify-between border-b border-[var(--border-subtle)] text-[var(--text-secondary)] bg-[var(--bg-sidebar)]">
        <div className="flex items-center gap-1.5 font-semibold text-[11px] text-[var(--text-muted)]">
          <Info className="w-3.5 h-3.5 text-[var(--accent)]" />
          <span>INSPECTOR</span>
        </div>
        <button
          onClick={toggleInspector}
          title="Close Inspector (Ctrl+I)"
          aria-label="Close Inspector"
          className="w-6 h-6 rounded-[6px] flex items-center justify-center hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {!activeItem ? (
        <div className="flex-1 flex flex-col items-center justify-center p-6 text-center text-[var(--text-muted)]">
          <Layers className="w-10 h-10 mb-2 stroke-[1] opacity-30" />
          <p className="text-xs font-medium text-[var(--text-secondary)]">No item selected</p>
          <p className="text-[11px] text-[var(--text-muted)] mt-0.5">
            Select a file or folder to preview metadata and contents
          </p>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* Quick Preview Area */}
          <div className="w-full rounded-[14px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] overflow-hidden flex flex-col items-center justify-center p-3 relative shadow-xs">
            {activeItem.previewKind === 'image' && activeItem.previewUrl ? (
              <div className="w-full space-y-2">
                <img
                  src={activeItem.previewUrl}
                  alt={activeItem.name}
                  referrerPolicy="no-referrer"
                  className="w-full h-44 object-cover rounded-[10px] border border-[var(--border-subtle)]"
                />
                {activeItem.dimensions && (
                  <div className="text-center font-mono text-[10px] text-[var(--text-muted)]">
                    {activeItem.dimensions.width} × {activeItem.dimensions.height} px
                  </div>
                )}
              </div>
            ) : activeItem.previewKind === 'video' && activeItem.previewUrl ? (
              <div className="w-full space-y-2">
                <video
                  src={activeItem.previewUrl}
                  controls
                  className="w-full h-44 rounded-[10px] bg-black object-cover"
                />
                <div className="flex justify-between items-center text-[10px] font-mono text-[var(--text-muted)] px-1">
                  <span>Duration: {activeItem.duration || '02:45'}</span>
                  <span>1080p 60fps</span>
                </div>
              </div>
            ) : activeItem.previewKind === 'audio' ? (
              <div className="w-full py-4 px-2 flex flex-col items-center space-y-3">
                <div className="w-12 h-12 rounded-full bg-[var(--bg-hover)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--accent)]">
                  <button
                    onClick={() => setIsPlayingAudio(!isPlayingAudio)}
                    className="w-full h-full flex items-center justify-center"
                  >
                    {isPlayingAudio ? (
                      <Pause className="w-5 h-5" />
                    ) : (
                      <Play className="w-5 h-5 ml-0.5" />
                    )}
                  </button>
                </div>

                {/* Simulated Waveform Visualizer */}
                <div className="w-full h-8 flex items-center justify-center gap-1 px-2">
                  {[40, 65, 30, 85, 95, 55, 75, 45, 90, 60, 35, 70, 80, 50, 65, 90].map(
                    (val, i) => (
                      <div
                        key={i}
                        className={`w-1 rounded-full transition-all ${
                          isPlayingAudio
                            ? 'bg-[var(--accent)] animate-pulse'
                            : 'bg-[var(--text-muted)]/40'
                        }`}
                        style={{ height: `${val}%` }}
                      />
                    )
                  )}
                </div>

                <div className="flex justify-between w-full text-[10px] font-mono text-[var(--text-muted)]">
                  <span>00:45</span>
                  <span>{activeItem.duration || '03:36'}</span>
                </div>
              </div>
            ) : activeItem.previewKind === 'code' || activeItem.previewKind === 'markdown' ? (
              <div className="w-full text-left">
                <div className="flex items-center justify-between pb-1.5 mb-2 border-b border-[var(--border-subtle)] text-[10px] font-mono text-[var(--text-muted)]">
                  <span>{activeItem.codeLanguage || activeItem.extension || 'code'}</span>
                  <span>{activeItem.textContent?.split('\n').length || 1} lines</span>
                </div>
                <pre className="font-mono text-[10px] text-[var(--text-primary)] overflow-x-auto max-h-48 p-2.5 rounded-[10px] bg-[var(--bg-app)] border border-[var(--border-subtle)] leading-relaxed select-text">
                  <code>{activeItem.textContent || '// Empty file content'}</code>
                </pre>
              </div>
            ) : activeItem.type === 'folder' ? (
              <div className="py-6 flex flex-col items-center">
                <Folder className="w-14 h-14 text-[var(--accent)] fill-[var(--accent)]/15 mb-2" />
                <span className="font-medium text-[var(--text-primary)]">{activeItem.name}</span>
                <span className="text-[10px] text-[var(--text-muted)] font-mono mt-0.5">
                  {activeItem.itemCount || 0} items inside
                </span>
              </div>
            ) : (
              <div className="py-6 flex flex-col items-center">
                {getFileIcon(activeItem, 'w-12 h-12')}
                <span className="font-medium text-[var(--text-primary)] mt-2">{activeItem.name}</span>
                <span className="text-[10px] text-[var(--text-muted)] font-mono mt-0.5">
                  {getFileTypeLabel(activeItem)}
                </span>
              </div>
            )}
          </div>

          {/* Title & Quick Actions */}
          <div>
            <div className="flex items-start justify-between gap-2">
              <h2 className="font-bold text-sm text-[var(--text-primary)] break-words flex-1">
                {activeItem.name}
              </h2>
              <button
                onClick={handleToggleStar}
                title={activeItem.isStarred ? 'Remove from Starred' : 'Add to Starred'}
                className="text-[var(--text-muted)] hover:text-amber-400 transition-colors p-1"
              >
                {activeItem.isStarred ? (
                  <Star className="w-4 h-4 text-amber-400 fill-amber-400" />
                ) : (
                  <StarOff className="w-4 h-4" />
                )}
              </button>
            </div>

            <p className="text-[11px] text-[var(--text-muted)] font-mono mt-0.5">
              {getFileTypeLabel(activeItem)}
            </p>

            {/* Quick Action Buttons */}
            <div className="flex items-center gap-1.5 mt-3">
              {activeItem.type === 'folder' && (
                <button
                  onClick={() => createTab(activeItem.path, activeItem.name)}
                  className="flex-1 py-1.5 px-2 rounded-[8px] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] border border-[var(--border-subtle)] text-[var(--text-primary)] flex items-center justify-center gap-1 text-[11px] font-medium transition-colors"
                >
                  <ExternalLink className="w-3 h-3 text-[var(--accent)]" />
                  <span>Open in Tab</span>
                </button>
              )}

              <button
                onClick={handleCopyPath}
                title="Copy Full Path"
                className="py-1.5 px-2.5 rounded-[8px] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] border border-[var(--border-subtle)] text-[var(--text-primary)] flex items-center justify-center gap-1 text-[11px] transition-colors"
              >
                {copiedPath ? (
                  <Check className="w-3 h-3 text-emerald-400" />
                ) : (
                  <Copy className="w-3 h-3" />
                )}
                <span>Path</span>
              </button>

              <button
                onClick={() => openPropertiesModal(activeItem)}
                className="py-1.5 px-2.5 rounded-[8px] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] border border-[var(--border-subtle)] text-[var(--text-primary)] text-[11px] transition-colors"
              >
                Properties
              </button>
            </div>
          </div>

          {/* Metadata Attributes */}
          <div className="space-y-2.5 pt-3 border-t border-[var(--border-subtle)] font-mono text-[11px]">
            <div className="flex justify-between items-center">
              <span className="text-[var(--text-muted)]">Size:</span>
              <span className="text-[var(--text-primary)] font-semibold">{formatBytes(activeItem.size)}</span>
            </div>

            <div className="flex justify-between items-center">
              <span className="text-[var(--text-muted)]">Kind:</span>
              <span className="text-[var(--text-secondary)]">{getFileTypeLabel(activeItem)}</span>
            </div>

            <div className="flex justify-between items-start gap-2">
              <span className="text-[var(--text-muted)] shrink-0">Location:</span>
              <span className="text-[var(--text-secondary)] text-right truncate flex-1" title={activeItem.parentPath}>
                {activeItem.parentPath}
              </span>
            </div>

            <div className="flex justify-between items-center">
              <span className="text-[var(--text-muted)]">Modified:</span>
              <span className="text-[var(--text-secondary)]">{formatDate(activeItem.modifiedAt)}</span>
            </div>

            <div className="flex justify-between items-center">
              <span className="text-[var(--text-muted)]">Created:</span>
              <span className="text-[var(--text-secondary)]">{formatDate(activeItem.createdAt)}</span>
            </div>

            {activeItem.dimensions && (
              <div className="flex justify-between items-center">
                <span className="text-[var(--text-muted)]">Dimensions:</span>
                <span className="text-[var(--text-secondary)]">
                  {activeItem.dimensions.width} × {activeItem.dimensions.height} px
                </span>
              </div>
            )}
          </div>
        </div>
      )}
    </aside>
  );
};
