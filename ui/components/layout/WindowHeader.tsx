import React from 'react';
import {
  X,
  Plus,
  Sun,
  Moon,
  Activity,
  Folder,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { BrandIcon } from '../common/BrandIcon';

export const WindowHeader: React.FC = () => {
  const {
    tabs,
    activeTabId,
    createTab,
    closeTab,
    switchTab,
    duplicateTab,
    theme,
    toggleTheme,
    operationJobs,
    toggleOperationQueue,
  } = useAppStore();

  const activeJobs = operationJobs.filter((j) => j.status === 'running');
  const avgProgress =
    activeJobs.length > 0
      ? Math.round(activeJobs.reduce((acc, j) => acc + j.progress, 0) / activeJobs.length)
      : 0;

  const isDark = theme !== 'snow' && theme !== 'warm-sand';

  return (
    <header className="h-11 bg-[var(--bg-sidebar)] border-b border-[var(--border-subtle)] flex items-center justify-between px-3 select-none shrink-0 text-xs transition-colors duration-150">
      {/* Left: Window Dots & RUSTED Geometric Brand */}
      <div className="flex items-center gap-3.5 shrink-0">
        <div className="flex items-center gap-1.5 mr-1 group">
          <button
            aria-label="Close window"
            className="w-3 h-3 rounded-full bg-rose-500/80 hover:bg-rose-500 transition-colors flex items-center justify-center text-[8px] text-rose-950 opacity-80 hover:opacity-100"
          >
            <span className="opacity-0 group-hover:opacity-100">×</span>
          </button>
          <button
            aria-label="Minimize window"
            className="w-3 h-3 rounded-full bg-amber-500/80 hover:bg-amber-500 transition-colors flex items-center justify-center text-[8px] text-amber-950 opacity-80 hover:opacity-100"
          >
            <span className="opacity-0 group-hover:opacity-100">–</span>
          </button>
          <button
            aria-label="Maximize window"
            className="w-3 h-3 rounded-full bg-emerald-500/80 hover:bg-emerald-500 transition-colors flex items-center justify-center text-[8px] text-emerald-950 opacity-80 hover:opacity-100"
          >
            <span className="opacity-0 group-hover:opacity-100">+</span>
          </button>
        </div>

        {/* Sophisticated RUSTED Branding */}
        <div className="hidden sm:flex items-center gap-2">
          <div className="w-5 h-5 rounded-[6px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--accent)] shadow-2xs">
            <BrandIcon size={13} />
          </div>
          <div className="flex items-baseline gap-1.5">
            <span className="font-bold tracking-wider text-[12px] text-[var(--text-primary)]">
              RUSTED
            </span>
            <span className="text-[10px] text-[var(--text-muted)] font-mono">v2.4</span>
          </div>
        </div>
      </div>

      {/* Center: Spacious, Refined Tab Bar */}
      <div className="flex-1 mx-3 min-w-0 max-w-4xl flex items-center">
        <div className="w-full flex items-center gap-1 overflow-x-auto no-scrollbar p-0.5 rounded-[13px] bg-[var(--bg-app)]/70 border border-[var(--border-subtle)]">
          {tabs.map((tab) => {
            const isActive = tab.id === activeTabId;
            return (
              <div
                key={tab.id}
                onClick={() => switchTab(tab.id)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  duplicateTab(tab.id);
                }}
                title={`${tab.title} (${tab.path}) — Right-click to duplicate`}
                className={`group flex items-center gap-2 h-8 px-3 rounded-[11px] transition-all duration-150 cursor-pointer text-xs min-w-[140px] max-w-[240px] flex-1 border ${
                  isActive
                    ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] shadow-xs font-medium'
                    : 'bg-transparent border-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
                }`}
              >
                <Folder
                  className={`w-3.5 h-3.5 shrink-0 transition-colors ${
                    isActive ? 'text-[var(--accent)]' : 'text-[var(--text-muted)] group-hover:text-[var(--text-secondary)]'
                  }`}
                />
                <span className="truncate flex-1 font-medium text-[11px] tracking-tight">
                  {tab.title}
                </span>

                {tabs.length > 1 && (
                  <button
                    type="button"
                    aria-label="Close tab"
                    onClick={(e) => {
                      e.stopPropagation();
                      closeTab(tab.id);
                    }}
                    className="w-4 h-4 rounded-md flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                  >
                    <X className="w-3 h-3" />
                  </button>
                )}
              </div>
            );
          })}

          <button
            onClick={() => createTab('/Home', 'Home')}
            title="New Tab (Ctrl+T)"
            aria-label="New Tab"
            className="w-8 h-8 rounded-[11px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] border border-transparent hover:border-[var(--border-subtle)] transition-colors shrink-0 ml-0.5"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Right: Transfer Activity & System Theme Quick Toggle */}
      <div className="flex items-center gap-2 shrink-0">
        {activeJobs.length > 0 && (
          <button
            onClick={toggleOperationQueue}
            title={`${activeJobs.length} transfers in progress (${avgProgress}%)`}
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-[10px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] text-[var(--accent)] hover:bg-[var(--bg-hover)] transition-colors animate-pulse"
          >
            <Activity className="w-3 h-3" />
            <span className="font-mono text-[11px] font-semibold">{avgProgress}%</span>
          </button>
        )}

        <button
          onClick={toggleTheme}
          title={`Switch Theme (Current: ${theme})`}
          aria-label="Toggle Theme"
          className="w-8 h-8 rounded-[10px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] border border-[var(--border-subtle)] transition-colors"
        >
          {isDark ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
        </button>
      </div>
    </header>
  );
};
