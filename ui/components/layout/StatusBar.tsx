import React from 'react';
import {
  HardDrive,
  Eye,
  EyeOff,
  Activity,
  CheckCircle2,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { formatBytes } from '../../utils/formatters';

export const StatusBar: React.FC = () => {
  const {
    directoryItems,
    selectedIds,
    showHiddenFiles,
    toggleHiddenFiles,
    operationJobs,
    toggleOperationQueue,
    tabs,
    activeTabId,
    gridDensity,
    setGridDensity,
  } = useAppStore();

  const activeTab = tabs.find((t) => t.id === activeTabId);
  const isGridView = activeTab?.viewMode === 'grid';

  const totalItemsCount = directoryItems.length;
  const selectedItems = directoryItems.filter((i) => selectedIds.includes(i.id));
  const selectedCount = selectedItems.length;
  const selectedBytes = selectedItems.reduce((acc, i) => acc + (i.size || 0), 0);

  const runningOperations = operationJobs.filter((j) => j.status === 'running');
  const recentCompleted = operationJobs.filter((j) => j.status === 'completed')[0];

  return (
    <footer className="h-6 bg-[var(--bg-sidebar)] border-t border-[var(--border-subtle)] flex items-center justify-between px-3 text-[11px] text-[var(--text-secondary)] select-none shrink-0 font-mono transition-colors duration-150">
      {/* Left: Item Stats */}
      <div className="flex items-center gap-2.5">
        <span>
          {totalItemsCount} {totalItemsCount === 1 ? 'item' : 'items'}
        </span>

        {selectedCount > 0 && (
          <>
            <span className="text-[var(--text-muted)]">·</span>
            <span className="text-[var(--text-primary)] font-semibold">
              {selectedCount} selected ({formatBytes(selectedBytes)})
            </span>
          </>
        )}
      </div>

      {/* Center: Active Operations Notification */}
      <div className="flex items-center gap-2">
        {runningOperations.length > 0 ? (
          <button
            onClick={toggleOperationQueue}
            className="flex items-center gap-1.5 text-[var(--text-primary)] hover:underline transition-colors cursor-pointer"
          >
            <Activity className="w-3 h-3 animate-pulse text-[var(--accent)]" />
            <span>
              {runningOperations[0].title} ({runningOperations[0].progress}%)
            </span>
          </button>
        ) : recentCompleted ? (
          <div className="flex items-center gap-1 text-emerald-400 text-[10px]">
            <CheckCircle2 className="w-3 h-3" />
            <span>Last transfer finished</span>
          </div>
        ) : null}
      </div>

      {/* Right: Storage, Hidden Files Toggle, Grid Size */}
      <div className="flex items-center gap-3">
        {isGridView && (
          <div className="flex items-center gap-1.5 text-[10px] text-[var(--text-muted)]">
            <span>Size</span>
            <div className="flex items-center gap-0.5 bg-[var(--bg-surface)] px-1 py-0.5 rounded-[6px] border border-[var(--border-subtle)]">
              <button
                onClick={() => setGridDensity('compact')}
                className={`px-1.5 rounded-[4px] ${gridDensity === 'compact' ? 'bg-[var(--bg-hover)] text-[var(--text-primary)] font-bold' : 'hover:text-[var(--text-primary)]'}`}
              >
                S
              </button>
              <button
                onClick={() => setGridDensity('normal')}
                className={`px-1.5 rounded-[4px] ${gridDensity === 'normal' ? 'bg-[var(--bg-hover)] text-[var(--text-primary)] font-bold' : 'hover:text-[var(--text-primary)]'}`}
              >
                M
              </button>
              <button
                onClick={() => setGridDensity('spacious')}
                className={`px-1.5 rounded-[4px] ${gridDensity === 'spacious' ? 'bg-[var(--bg-hover)] text-[var(--text-primary)] font-bold' : 'hover:text-[var(--text-primary)]'}`}
              >
                L
              </button>
            </div>
          </div>
        )}

        <button
          onClick={toggleHiddenFiles}
          title={showHiddenFiles ? 'Hide hidden files' : 'Show hidden files'}
          className={`flex items-center gap-1 hover:text-[var(--text-primary)] transition-colors ${
            showHiddenFiles ? 'text-[var(--text-primary)] font-medium' : 'text-[var(--text-muted)]'
          }`}
        >
          {showHiddenFiles ? <Eye className="w-3 h-3 text-[var(--accent)]" /> : <EyeOff className="w-3 h-3" />}
          <span>{showHiddenFiles ? 'Hidden on' : 'Hidden off'}</span>
        </button>

        <span className="text-[var(--text-muted)]">·</span>

        <div className="flex items-center gap-1 text-[var(--text-secondary)]">
          <HardDrive className="w-3 h-3 text-[var(--text-muted)]" />
          <span>200 GB available</span>
        </div>
      </div>
    </footer>
  );
};
