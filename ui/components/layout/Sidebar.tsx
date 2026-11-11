import React, { useState } from 'react';
import {
  Home,
  Clock,
  Star,
  Monitor,
  Download,
  FileText,
  Image,
  Video,
  Music,
  HardDrive,
  Network,
  Trash2,
  ChevronDown,
  ChevronRight,
  Settings,
  PanelLeftClose,
  PanelLeft,
  Pin,
  PinOff,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { INITIAL_DRIVES } from '../../services/mockData';
import { formatBytes } from '../../utils/formatters';

export const Sidebar: React.FC = () => {
  const {
    currentPath,
    navigateTab,
    activeTabId,
    isSidebarOpen,
    toggleSidebar,
    openSettings,
  } = useAppStore();

  const [collapsedSections, setCollapsedSections] = useState<Record<string, boolean>>({
    storage: false,
    quickAccess: false,
    home: false,
    additional: false,
  });

  const [pinnedPaths, setPinnedPaths] = useState<string[]>([
    '/Home/Documents/Projects/Atlas-Desktop',
    '/Home/Pictures/Wallpapers',
  ]);

  const toggleSection = (section: string) => {
    setCollapsedSections((prev) => ({ ...prev, [section]: !prev[section] }));
  };

  const isCurrent = (path: string) => currentPath === path;

  const handleNav = (path: string) => {
    navigateTab(activeTabId, path);
  };

  const togglePin = (e: React.MouseEvent, path: string) => {
    e.stopPropagation();
    if (pinnedPaths.includes(path)) {
      setPinnedPaths(pinnedPaths.filter((p) => p !== path));
    } else {
      setPinnedPaths([...pinnedPaths, path]);
    }
  };

  if (!isSidebarOpen) {
    return (
      <aside className="w-12 bg-[var(--bg-sidebar)] border-r border-[var(--border-subtle)] flex flex-col items-center py-3 select-none shrink-0 text-[var(--text-secondary)] transition-colors duration-150">
        <button
          onClick={toggleSidebar}
          title="Expand Sidebar (Ctrl+B)"
          aria-label="Expand Sidebar"
          className="w-8 h-8 rounded-[8px] flex items-center justify-center hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors mb-3"
        >
          <PanelLeft className="w-4 h-4" />
        </button>

        <div className="flex flex-col items-center gap-1 w-full px-2">
          <button
            onClick={() => handleNav('/Home')}
            title="Home"
            aria-label="Home"
            className={`w-8 h-8 rounded-[8px] flex items-center justify-center transition-colors ${
              isCurrent('/Home')
                ? 'bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--border-strong)]'
                : 'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Home className="w-4 h-4" />
          </button>
          <button
            onClick={() => handleNav('/Home/Documents')}
            title="Documents"
            aria-label="Documents"
            className={`w-8 h-8 rounded-[8px] flex items-center justify-center transition-colors ${
              isCurrent('/Home/Documents')
                ? 'bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--border-strong)]'
                : 'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
            }`}
          >
            <FileText className="w-4 h-4" />
          </button>
          <button
            onClick={() => handleNav('/Home/Downloads')}
            title="Downloads"
            aria-label="Downloads"
            className={`w-8 h-8 rounded-[8px] flex items-center justify-center transition-colors ${
              isCurrent('/Home/Downloads')
                ? 'bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--border-strong)]'
                : 'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Download className="w-4 h-4" />
          </button>
          <button
            onClick={() => handleNav('/Trash')}
            title="Trash"
            aria-label="Trash"
            className={`w-8 h-8 rounded-[8px] flex items-center justify-center transition-colors ${
              isCurrent('/Trash')
                ? 'bg-[var(--bg-surface)] text-[var(--text-primary)] border border-[var(--border-strong)]'
                : 'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>

        <div className="mt-auto">
          <button
            onClick={openSettings}
            title="Settings (Ctrl+,)"
            aria-label="Settings"
            className="w-8 h-8 rounded-[8px] flex items-center justify-center hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors"
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </aside>
    );
  }

  return (
    <aside className="w-60 bg-[var(--bg-sidebar)] border-r border-[var(--border-subtle)] flex flex-col select-none shrink-0 text-xs text-[var(--text-primary)] overflow-hidden transition-colors duration-150">
      {/* Top Header / Toggle */}
      <div className="h-9 px-3.5 flex items-center justify-between border-b border-[var(--border-subtle)] text-[var(--text-secondary)]">
        <span className="font-semibold text-[10px] tracking-wider text-[var(--text-muted)]">
          EXPLORER
        </span>
        <button
          onClick={toggleSidebar}
          title="Collapse Sidebar (Ctrl+B)"
          aria-label="Collapse Sidebar"
          className="w-6 h-6 rounded-[6px] flex items-center justify-center hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors"
        >
          <PanelLeftClose className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Nav Scroll Area */}
      <div className="flex-1 overflow-y-auto px-2 py-2.5 space-y-4">
        {/* HOME SECTION */}
        <div>
          <button
            onClick={() => toggleSection('home')}
            className="w-full flex items-center justify-between px-2 py-1 text-[10px] font-semibold tracking-wider text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
          >
            <span>HOME</span>
            {collapsedSections.home ? (
              <ChevronRight className="w-3 h-3" />
            ) : (
              <ChevronDown className="w-3 h-3" />
            )}
          </button>

          {!collapsedSections.home && (
            <div className="mt-1 space-y-0.5">
              <button
                onClick={() => handleNav('/Home')}
                className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-[10px] transition-colors text-left border ${
                  isCurrent('/Home')
                    ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                    : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <Home className="w-3.5 h-3.5 shrink-0" />
                <span className="truncate flex-1">Home</span>
              </button>

              <button
                onClick={() => handleNav('/Home/Recent')}
                className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-[10px] transition-colors text-left border ${
                  isCurrent('/Home/Recent')
                    ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                    : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <Clock className="w-3.5 h-3.5 shrink-0" />
                <span className="truncate flex-1">Recent Files</span>
              </button>

              <button
                onClick={() => handleNav('/Home/Starred')}
                className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-[10px] transition-colors text-left border ${
                  isCurrent('/Home/Starred')
                    ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                    : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <Star className="w-3.5 h-3.5 shrink-0" />
                <span className="truncate flex-1">Starred</span>
              </button>
            </div>
          )}
        </div>

        {/* QUICK ACCESS SECTION */}
        <div>
          <button
            onClick={() => toggleSection('quickAccess')}
            className="w-full flex items-center justify-between px-2 py-1 text-[10px] font-semibold tracking-wider text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
          >
            <span>QUICK ACCESS</span>
            {collapsedSections.quickAccess ? (
              <ChevronRight className="w-3 h-3" />
            ) : (
              <ChevronDown className="w-3 h-3" />
            )}
          </button>

          {!collapsedSections.quickAccess && (
            <div className="mt-1 space-y-0.5">
              {[
                { label: 'Desktop', path: '/Home/Desktop', icon: Monitor },
                { label: 'Downloads', path: '/Home/Downloads', icon: Download },
                { label: 'Documents', path: '/Home/Documents', icon: FileText },
                { label: 'Pictures', path: '/Home/Pictures', icon: Image },
                { label: 'Videos', path: '/Home/Videos', icon: Video },
                { label: 'Music', path: '/Home/Music', icon: Music },
              ].map((item) => {
                const Icon = item.icon;
                const active = isCurrent(item.path);
                return (
                  <button
                    key={item.path}
                    onClick={() => handleNav(item.path)}
                    className={`group w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-[10px] transition-colors text-left border ${
                      active
                        ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                        : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                    }`}
                  >
                    <Icon className="w-3.5 h-3.5 shrink-0 opacity-80 group-hover:opacity-100" />
                    <span className="truncate flex-1">{item.label}</span>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* PINNED SHORTCUTS */}
        {pinnedPaths.length > 0 && (
          <div>
            <div className="px-2 py-1 text-[10px] font-semibold tracking-wider text-[var(--text-muted)] flex items-center justify-between">
              <span>PINNED SHORTCUTS</span>
            </div>
            <div className="mt-1 space-y-0.5">
              {pinnedPaths.map((p) => {
                const name = p.split('/').pop() || p;
                const active = isCurrent(p);
                return (
                  <div
                    key={p}
                    onClick={() => handleNav(p)}
                    className={`group w-full flex items-center justify-between px-2.5 py-1.5 rounded-[10px] cursor-pointer transition-colors border ${
                      active
                        ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                        : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                    }`}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <Pin className="w-3 h-3 text-[var(--accent)] shrink-0 rotate-45" />
                      <span className="truncate text-xs">{name}</span>
                    </div>
                    <button
                      onClick={(e) => togglePin(e, p)}
                      title="Unpin folder"
                      className="opacity-0 group-hover:opacity-100 p-0.5 hover:text-rose-400 text-[var(--text-muted)] transition-opacity"
                    >
                      <PinOff className="w-3 h-3" />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* STORAGE SECTION */}
        <div>
          <button
            onClick={() => toggleSection('storage')}
            className="w-full flex items-center justify-between px-2 py-1 text-[10px] font-semibold tracking-wider text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
          >
            <span>STORAGE</span>
            {collapsedSections.storage ? (
              <ChevronRight className="w-3 h-3" />
            ) : (
              <ChevronDown className="w-3 h-3" />
            )}
          </button>

          {!collapsedSections.storage && (
            <div className="mt-1 space-y-1.5">
              {INITIAL_DRIVES.map((drive) => {
                const usedPct = Math.round((drive.usedBytes / drive.totalBytes) * 100);
                return (
                  <button
                    key={drive.id}
                    onClick={() => handleNav('/Home/Storage')}
                    className={`w-full p-2.5 rounded-[12px] transition-colors text-left border ${
                      currentPath === '/Home/Storage'
                        ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] shadow-2xs'
                        : 'bg-[var(--bg-surface-secondary)] border-[var(--border-subtle)] hover:bg-[var(--bg-surface)]'
                    }`}
                  >
                    <div className="flex items-center justify-between gap-1 mb-1.5">
                      <div className="flex items-center gap-1.5 min-w-0">
                        <HardDrive className="w-3.5 h-3.5 text-[var(--accent)] shrink-0" />
                        <span className="truncate font-medium text-[11px] text-[var(--text-primary)]">
                          {drive.name}
                        </span>
                      </div>
                      <span className="text-[10px] text-[var(--text-secondary)] font-mono shrink-0">
                        {usedPct}%
                      </span>
                    </div>

                    <div className="w-full h-1 bg-[var(--bg-hover)] rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all duration-300 ${
                          usedPct > 85 ? 'bg-amber-500' : 'bg-[var(--accent)]'
                        }`}
                        style={{ width: `${usedPct}%` }}
                      />
                    </div>

                    <div className="flex justify-between items-center text-[10px] text-[var(--text-muted)] mt-1 font-mono">
                      <span>{formatBytes(drive.freeBytes)} free</span>
                      <span>{formatBytes(drive.totalBytes)}</span>
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* ADDITIONAL */}
        <div>
          <button
            onClick={() => toggleSection('additional')}
            className="w-full flex items-center justify-between px-2 py-1 text-[10px] font-semibold tracking-wider text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
          >
            <span>SYSTEM</span>
            {collapsedSections.additional ? (
              <ChevronRight className="w-3 h-3" />
            ) : (
              <ChevronDown className="w-3 h-3" />
            )}
          </button>

          {!collapsedSections.additional && (
            <div className="mt-1 space-y-0.5">
              <button
                onClick={() => handleNav('/Home/Network')}
                className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-[10px] transition-colors text-left border ${
                  isCurrent('/Home/Network')
                    ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                    : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <Network className="w-3.5 h-3.5 shrink-0 opacity-80" />
                <span className="truncate flex-1">Network Locations</span>
              </button>

              <button
                onClick={() => handleNav('/Trash')}
                className={`w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-[10px] transition-colors text-left border ${
                  isCurrent('/Trash')
                    ? 'bg-[var(--bg-surface)] border-[var(--border-strong)] text-[var(--text-primary)] font-medium shadow-2xs'
                    : 'border-transparent hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <Trash2 className="w-3.5 h-3.5 shrink-0 opacity-80" />
                <span className="truncate flex-1">Trash</span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Bottom Footer / Settings */}
      <div className="h-10 px-3.5 border-t border-[var(--border-subtle)] flex items-center justify-between text-[var(--text-secondary)] bg-[var(--bg-sidebar)]">
        <button
          onClick={openSettings}
          className="flex items-center gap-2 text-xs hover:text-[var(--text-primary)] transition-colors"
        >
          <Settings className="w-3.5 h-3.5" />
          <span>Preferences</span>
        </button>
        <span className="text-[10px] text-[var(--text-muted)] font-mono">⌘,</span>
      </div>
    </aside>
  );
};
