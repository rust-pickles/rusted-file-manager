import React, { useState, useEffect, useRef } from 'react';
import {
  ArrowLeft,
  ArrowRight,
  RotateCw,
  Columns,
  LayoutGrid,
  List,
  AlignJustify,
  Search,
  FolderPlus,
  PanelRight,
  Terminal,
  ChevronRight,
  Home,
  Copy,
  Check,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { ViewMode } from '../../types/filesystem';

export const NavigationToolbar: React.FC = () => {
  const {
    tabs,
    activeTabId,
    currentPath,
    navigateTab,
    goBack,
    goForward,
    refreshTab,
    setViewMode,
    isInspectorOpen,
    toggleInspector,
    openNewFolderModal,
    openCommandPalette,
    searchQuery,
    setSearchQuery,
    isEditingAddress,
    setIsEditingAddress,
  } = useAppStore();

  const activeTab = tabs.find((t) => t.id === activeTabId);
  const canGoBack = (activeTab?.historyIndex ?? 0) > 0;
  const canGoForward =
    (activeTab?.historyIndex ?? 0) < (activeTab?.history?.length ?? 0) - 1;
  const currentViewMode: ViewMode = activeTab?.viewMode || 'miller';

  const [pathInput, setPathInput] = useState(currentPath);
  const [copiedPath, setCopiedPath] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setPathInput(currentPath);
  }, [currentPath]);

  useEffect(() => {
    if (isEditingAddress && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditingAddress]);

  const handleAddressSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (pathInput.trim()) {
      navigateTab(activeTabId, pathInput.trim());
      setIsEditingAddress(false);
    }
  };

  const copyCurrentPath = () => {
    navigator.clipboard.writeText(currentPath);
    setCopiedPath(true);
    setTimeout(() => setCopiedPath(false), 1500);
  };

  const pathParts = currentPath.split('/').filter(Boolean);

  return (
    <div className="h-11 bg-[var(--bg-app)] border-b border-[var(--border-subtle)] flex items-center justify-between px-3 gap-2.5 select-none text-xs shrink-0 transition-colors duration-150">
      {/* Left: History Nav & Refresh */}
      <div className="flex items-center gap-1 shrink-0">
        <button
          onClick={() => goBack()}
          disabled={!canGoBack}
          title="Back (Alt+Left)"
          aria-label="Back"
          className="w-7 h-7 rounded-[8px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] disabled:opacity-30 disabled:hover:bg-transparent transition-colors"
        >
          <ArrowLeft className="w-3.5 h-3.5" />
        </button>

        <button
          onClick={() => goForward()}
          disabled={!canGoForward}
          title="Forward (Alt+Right)"
          aria-label="Forward"
          className="w-7 h-7 rounded-[8px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] disabled:opacity-30 disabled:hover:bg-transparent transition-colors"
        >
          <ArrowRight className="w-3.5 h-3.5" />
        </button>

        <button
          onClick={() => refreshTab()}
          title="Refresh (Ctrl+R)"
          aria-label="Refresh"
          className="w-7 h-7 rounded-[8px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors ml-0.5"
        >
          <RotateCw className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Center: Interactive Breadcrumbs / Editable Address Bar */}
      <div className="flex-1 max-w-2xl min-w-0">
        {isEditingAddress ? (
          <form onSubmit={handleAddressSubmit} className="relative w-full">
            <input
              ref={inputRef}
              type="text"
              value={pathInput}
              onChange={(e) => setPathInput(e.target.value)}
              onBlur={() => setIsEditingAddress(false)}
              onKeyDown={(e) => {
                if (e.key === 'Escape') setIsEditingAddress(false);
              }}
              className="w-full h-7 px-3 bg-[var(--bg-surface)] border border-[var(--accent)] rounded-[10px] text-[var(--text-primary)] font-mono text-xs focus:outline-none shadow-xs"
              placeholder="/Home/Documents..."
            />
          </form>
        ) : (
          <div
            onClick={() => setIsEditingAddress(true)}
            className="group flex items-center h-7 px-2.5 bg-[var(--bg-surface)] border border-[var(--border-subtle)] hover:border-[var(--border-strong)] rounded-[10px] transition-colors cursor-text overflow-hidden"
          >
            <div className="flex items-center gap-1 min-w-0 flex-1 overflow-x-auto no-scrollbar font-medium">
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  navigateTab(activeTabId, '/Home');
                }}
                className="text-[var(--text-secondary)] hover:text-[var(--text-primary)] p-0.5 rounded-[6px] transition-colors shrink-0"
              >
                <Home className="w-3.5 h-3.5" />
              </button>

              {pathParts.map((part, index) => {
                const subPath = '/' + pathParts.slice(0, index + 1).join('/');
                const isLast = index === pathParts.length - 1;
                return (
                  <React.Fragment key={subPath}>
                    <ChevronRight className="w-3 h-3 text-[var(--text-muted)] shrink-0" />
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        navigateTab(activeTabId, subPath);
                      }}
                      className={`truncate max-w-[140px] px-1.5 py-0.5 rounded-[6px] transition-colors ${
                        isLast
                          ? 'text-[var(--text-primary)] font-semibold'
                          : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
                      }`}
                    >
                      {part}
                    </button>
                  </React.Fragment>
                );
              })}
            </div>

            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                copyCurrentPath();
              }}
              title="Copy path"
              className="ml-2 text-[var(--text-muted)] hover:text-[var(--text-primary)] opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
            >
              {copiedPath ? (
                <Check className="w-3.5 h-3.5 text-emerald-400" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </button>
          </div>
        )}
      </div>

      {/* Right: Search, View Mode Segmented Controls, Actions */}
      <div className="flex items-center gap-1.5 shrink-0">
        {/* Quick Search Field */}
        <div className="relative">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search files... (Ctrl+F)"
            className="w-36 lg:w-48 h-7 pl-7 pr-2 bg-[var(--bg-surface)] border border-[var(--border-subtle)] focus:border-[var(--border-strong)] rounded-[10px] text-[var(--text-primary)] placeholder-[var(--text-muted)] text-xs focus:outline-none transition-all"
          />
          <Search className="w-3.5 h-3.5 text-[var(--text-muted)] absolute left-2 top-1.5 pointer-events-none" />
        </div>

        {/* View Mode Segmented Switcher */}
        <div className="flex items-center p-0.5 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[10px]">
          <button
            onClick={() => setViewMode('miller')}
            title="Miller Columns View (Ctrl+1)"
            aria-label="Miller Columns"
            className={`w-6 h-6 rounded-[8px] flex items-center justify-center transition-colors ${
              currentViewMode === 'miller'
                ? 'bg-[var(--bg-surface-elevated)] text-[var(--text-primary)] shadow-2xs'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Columns className="w-3.5 h-3.5" />
          </button>

          <button
            onClick={() => setViewMode('grid')}
            title="Grid View (Ctrl+2)"
            aria-label="Grid View"
            className={`w-6 h-6 rounded-[8px] flex items-center justify-center transition-colors ${
              currentViewMode === 'grid'
                ? 'bg-[var(--bg-surface-elevated)] text-[var(--text-primary)] shadow-2xs'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <LayoutGrid className="w-3.5 h-3.5" />
          </button>

          <button
            onClick={() => setViewMode('list')}
            title="Detailed List View (Ctrl+3)"
            aria-label="List View"
            className={`w-6 h-6 rounded-[8px] flex items-center justify-center transition-colors ${
              currentViewMode === 'list'
                ? 'bg-[var(--bg-surface-elevated)] text-[var(--text-primary)] shadow-2xs'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <List className="w-3.5 h-3.5" />
          </button>

          <button
            onClick={() => setViewMode('compact')}
            title="Compact View (Ctrl+4)"
            aria-label="Compact View"
            className={`w-6 h-6 rounded-[8px] flex items-center justify-center transition-colors ${
              currentViewMode === 'compact'
                ? 'bg-[var(--bg-surface-elevated)] text-[var(--text-primary)] shadow-2xs'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <AlignJustify className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* New Folder */}
        <button
          onClick={() => openNewFolderModal(currentPath)}
          title="New Folder (Ctrl+Shift+N)"
          aria-label="New Folder"
          className="w-7 h-7 rounded-[8px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] border border-[var(--border-subtle)] transition-colors"
        >
          <FolderPlus className="w-3.5 h-3.5" />
        </button>

        {/* Command Palette */}
        <button
          onClick={openCommandPalette}
          title="Command Palette (Ctrl+K)"
          aria-label="Command Palette"
          className="w-7 h-7 rounded-[8px] flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] border border-[var(--border-subtle)] transition-colors"
        >
          <Terminal className="w-3.5 h-3.5" />
        </button>

        {/* Inspector Toggle */}
        <button
          onClick={toggleInspector}
          title="Toggle Info Inspector (Ctrl+I)"
          aria-label="Toggle Inspector"
          className={`w-7 h-7 rounded-[8px] flex items-center justify-center border transition-colors ${
            isInspectorOpen
              ? 'bg-[var(--bg-surface-elevated)] border-[var(--border-strong)] text-[var(--text-primary)]'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] border-[var(--border-subtle)] hover:bg-[var(--bg-hover)]'
          }`}
        >
          <PanelRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
