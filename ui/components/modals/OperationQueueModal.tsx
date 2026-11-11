import React from 'react';
import { Activity, X, CheckCircle2 } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { formatBytes } from '../../utils/formatters';

export const OperationQueueModal: React.FC = () => {
  const {
    isOperationQueueOpen,
    toggleOperationQueue,
    operationJobs,
    cancelOperation,
  } = useAppStore();

  if (!isOperationQueueOpen) return null;

  return (
    <div
      onClick={toggleOperationQueue}
      className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none animate-in fade-in"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-lg bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[18px] shadow-2xl p-5 text-xs text-[var(--text-primary)] transition-colors duration-150"
      >
        <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
          <div className="flex items-center gap-2 font-semibold text-sm">
            <Activity className="w-4 h-4 text-[var(--accent)]" />
            <span>Filesystem Operation Queue</span>
          </div>
          <button
            onClick={toggleOperationQueue}
            className="w-6 h-6 rounded-[6px] flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="mt-4 space-y-3 max-h-80 overflow-y-auto pr-1">
          {operationJobs.length === 0 ? (
            <div className="py-12 text-center text-[var(--text-muted)]">
              <p className="text-sm font-medium text-[var(--text-secondary)]">Queue is empty</p>
              <p className="text-[11px] text-[var(--text-muted)] mt-1">No active transfers or batch operations</p>
            </div>
          ) : (
            operationJobs.map((job) => {
              const isRunning = job.status === 'running';
              const isCompleted = job.status === 'completed';
              const isCancelled = job.status === 'cancelled';

              return (
                <div
                  key={job.id}
                  className="p-3.5 rounded-[12px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)] space-y-2.5"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="font-semibold text-[var(--text-primary)]">{job.title}</p>
                      <p className="text-[10px] text-[var(--text-muted)] font-mono mt-0.5 truncate max-w-sm">
                        To: {job.destinationPath}
                      </p>
                    </div>

                    <div className="flex items-center gap-2">
                      {isCompleted ? (
                        <span className="flex items-center gap-1 text-emerald-400 font-mono text-[11px]">
                          <CheckCircle2 className="w-3.5 h-3.5" />
                          <span>Finished</span>
                        </span>
                      ) : isCancelled ? (
                        <span className="text-rose-400 font-mono text-[11px]">Cancelled</span>
                      ) : (
                        <button
                          onClick={() => cancelOperation(job.id)}
                          className="px-2.5 py-1 rounded-[6px] bg-[var(--bg-surface)] hover:bg-rose-500/20 text-[var(--text-secondary)] hover:text-rose-400 border border-[var(--border-subtle)] transition-colors text-[10px]"
                        >
                          Cancel
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Progress Bar */}
                  <div className="w-full h-1.5 bg-[var(--bg-surface)] rounded-full overflow-hidden border border-[var(--border-subtle)]">
                    <div
                      className={`h-full rounded-full transition-all duration-300 ${
                        isCompleted
                          ? 'bg-emerald-400'
                          : isCancelled
                          ? 'bg-rose-500'
                          : 'bg-[var(--accent)]'
                      }`}
                      style={{ width: `${job.progress}%` }}
                    />
                  </div>

                  {/* Telemetry info */}
                  <div className="flex justify-between items-center text-[10px] text-[var(--text-muted)] font-mono">
                    <span>
                      {formatBytes(job.processedBytes)} / {formatBytes(job.totalBytes)} ({job.progress}%)
                    </span>
                    {isRunning && <span className="text-[var(--text-primary)] font-semibold">{job.speed}</span>}
                  </div>
                </div>
              );
            })
          )}
        </div>

        <div className="flex justify-end pt-3 border-t border-[var(--border-subtle)] mt-4">
          <button
            onClick={toggleOperationQueue}
            className="px-4 py-1.5 rounded-[10px] bg-[var(--bg-hover)] text-[var(--text-primary)] border border-[var(--border-subtle)] transition-colors hover:bg-[var(--bg-active)]"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
