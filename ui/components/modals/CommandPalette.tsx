import React, { useState, useEffect, useRef } from 'react';
import {
  Folder,
  Columns,
  LayoutGrid,
  List,
  AlignJustify,
  Sun,
  Moon,
  Eye,
  Settings,
  FolderPlus,
  PanelLeft,
  PanelRight,
  RotateCw,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { BrandIcon } from '../common/BrandIcon';

interface CommandItem {
  id: string;
  category: 'Navigation' | 'View' | 'Action' | 'Settings';
  title: string;
  subtitle?: string;
  shortcut?: string;
  icon: React.ComponentType<{ className?: string }>;
  action: () => void;
}

export const CommandPalette: React.FC = () => {
  const {
    isCommandPaletteOpen,
    closeCommandPalette,
    navigateTab,
    activeTabId,
    setViewMode,
    toggleTheme,
    theme,
    toggleSidebar,
    toggleInspector,
    toggleHiddenFiles,
    showHiddenFiles,
    openSettings,
    openNewFolderModal,
    currentPath,
    refreshTab,
  } = useAppStore();

  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isCommandPaletteOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isCommandPaletteOpen]);

  const isDark = theme !== 'snow' && theme !== 'warm-sand';

  const allCommands: CommandItem[] = [
    // Navigation
    {
      id: 'nav-home',
      category: 'Navigation',
      title: 'Go to Home',
      subtitle: '/Home',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home'),
    },
    {
      id: 'nav-desktop',
      category: 'Navigation',
      title: 'Go to Desktop',
      subtitle: '/Home/Desktop',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home/Desktop'),
    },
    {
      id: 'nav-downloads',
      category: 'Navigation',
      title: 'Go to Downloads',
      subtitle: '/Home/Downloads',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home/Downloads'),
    },
    {
      id: 'nav-docs',
      category: 'Navigation',
      title: 'Go to Documents',
      subtitle: '/Home/Documents',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home/Documents'),
    },
    {
      id: 'nav-atlas',
      category: 'Navigation',
      title: 'Go to Atlas-Desktop Project',
      subtitle: '/Home/Documents/Projects/Atlas-Desktop',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home/Documents/Projects/Atlas-Desktop'),
    },
    {
      id: 'nav-pictures',
      category: 'Navigation',
      title: 'Go to Pictures',
      subtitle: '/Home/Pictures',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home/Pictures'),
    },
    {
      id: 'nav-storage',
      category: 'Navigation',
      title: 'Go to Storage Overview',
      subtitle: '/Home/Storage',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Home/Storage'),
    },
    {
      id: 'nav-trash',
      category: 'Navigation',
      title: 'Go to Trash',
      subtitle: '/Trash',
      icon: Folder,
      action: () => navigateTab(activeTabId, '/Trash'),
    },

    // View Modes
    {
      id: 'view-miller',
      category: 'View',
      title: 'Switch to Miller Columns View',
      shortcut: 'Ctrl+1',
      icon: Columns,
      action: () => setViewMode('miller'),
    },
    {
      id: 'view-grid',
      category: 'View',
      title: 'Switch to Grid View',
      shortcut: 'Ctrl+2',
      icon: LayoutGrid,
      action: () => setViewMode('grid'),
    },
    {
      id: 'view-list',
      category: 'View',
      title: 'Switch to Detailed List View',
      shortcut: 'Ctrl+3',
      icon: List,
      action: () => setViewMode('list'),
    },
    {
      id: 'view-compact',
      category: 'View',
      title: 'Switch to Compact Power-User View',
      shortcut: 'Ctrl+4',
      icon: AlignJustify,
      action: () => setViewMode('compact'),
    },
    {
      id: 'view-sidebar',
      category: 'View',
      title: 'Toggle Sidebar Navigation',
      shortcut: 'Ctrl+B',
      icon: PanelLeft,
      action: () => toggleSidebar(),
    },
    {
      id: 'view-inspector',
      category: 'View',
      title: 'Toggle File Inspector Panel',
      shortcut: 'Ctrl+I',
      icon: PanelRight,
      action: () => toggleInspector(),
    },

    // Actions
    {
      id: 'action-new-folder',
      category: 'Action',
      title: 'Create New Folder',
      shortcut: 'Ctrl+Shift+N',
      icon: FolderPlus,
      action: () => openNewFolderModal(currentPath),
    },
    {
      id: 'action-refresh',
      category: 'Action',
      title: 'Refresh Current Directory',
      shortcut: 'Ctrl+R',
      icon: RotateCw,
      action: () => refreshTab(),
    },
    {
      id: 'action-hidden',
      category: 'Action',
      title: showHiddenFiles ? 'Hide Dotfiles & System Files' : 'Show Hidden Dotfiles & System Files',
      shortcut: 'Ctrl+H',
      icon: Eye,
      action: () => toggleHiddenFiles(),
    },
    {
      id: 'action-theme',
      category: 'Action',
      title: isDark ? 'Switch to Light Theme' : 'Switch to Dark Theme',
      icon: isDark ? Sun : Moon,
      action: () => toggleTheme(),
    },

    // Settings
    {
      id: 'set-pref',
      category: 'Settings',
      title: 'Open Preferences & Themes',
      shortcut: 'Ctrl+,',
      icon: Settings,
      action: () => openSettings(),
    },
  ];

  const filteredCommands = allCommands.filter((cmd) => {
    if (!query.trim()) return true;
    const q = query.toLowerCase();
    return (
      cmd.title.toLowerCase().includes(q) ||
      (cmd.subtitle && cmd.subtitle.toLowerCase().includes(q)) ||
      cmd.category.toLowerCase().includes(q)
    );
  });

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % Math.max(1, filteredCommands.length));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) =>
        prev <= 0 ? filteredCommands.length - 1 : prev - 1
      );
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filteredCommands[selectedIndex]) {
        filteredCommands[selectedIndex].action();
        closeCommandPalette();
      }
    } else if (e.key === 'Escape') {
      closeCommandPalette();
    }
  };

  if (!isCommandPaletteOpen) return null;

  return (
    <div
      onClick={closeCommandPalette}
      className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-start justify-center pt-20 p-4 select-none animate-in fade-in duration-100"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-lg bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[16px] shadow-2xl overflow-hidden flex flex-col text-xs text-[var(--text-primary)] transition-colors duration-150"
      >
        {/* Search Input Bar */}
        <div className="flex items-center px-4 py-3 border-b border-[var(--border-subtle)] gap-2.5 bg-[var(--bg-app)]/50">
          <div className="w-5 h-5 rounded-[6px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--accent)] shrink-0">
            <BrandIcon size={12} />
          </div>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or jump to path..."
            className="w-full bg-transparent text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none"
          />
          <kbd className="px-1.5 py-0.5 rounded-[6px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] text-[10px] font-mono text-[var(--text-muted)]">
            ESC
          </kbd>
        </div>

        {/* Command Results */}
        <div className="max-h-80 overflow-y-auto p-1.5 space-y-0.5">
          {filteredCommands.length === 0 ? (
            <div className="py-8 text-center text-[var(--text-muted)]">No matching commands</div>
          ) : (
            filteredCommands.map((cmd, idx) => {
              const isSelected = idx === selectedIndex;
              const Icon = cmd.icon;

              return (
                <div
                  key={cmd.id}
                  onClick={() => {
                    cmd.action();
                    closeCommandPalette();
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`flex items-center justify-between px-3 py-2 rounded-[10px] cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-[var(--selection-bg)] text-[var(--text-primary)] font-medium shadow-2xs'
                      : 'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div
                      className={`w-6 h-6 rounded-[6px] flex items-center justify-center shrink-0 ${
                        isSelected
                          ? 'bg-[var(--bg-surface)] text-[var(--accent)] border border-[var(--border-subtle)]'
                          : 'bg-[var(--bg-app)] text-[var(--text-muted)]'
                      }`}
                    >
                      <Icon className="w-3.5 h-3.5" />
                    </div>
                    <div className="min-w-0">
                      <p className="font-medium truncate">{cmd.title}</p>
                      {cmd.subtitle && (
                        <p className="text-[10px] text-[var(--text-muted)] font-mono truncate">
                          {cmd.subtitle}
                        </p>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-2 shrink-0 ml-3">
                    <span className="text-[10px] text-[var(--text-muted)] font-mono">
                      {cmd.category}
                    </span>
                    {cmd.shortcut && (
                      <kbd className="px-1.5 py-0.5 rounded-[6px] bg-[var(--bg-app)] border border-[var(--border-subtle)] text-[10px] font-mono text-[var(--text-secondary)]">
                        {cmd.shortcut}
                      </kbd>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="px-3.5 py-2 border-t border-[var(--border-subtle)] bg-[var(--bg-app)]/60 flex items-center justify-between text-[10px] text-[var(--text-muted)] font-mono">
          <div className="flex items-center gap-2">
            <span>↑↓ navigate</span>
            <span>·</span>
            <span>↵ execute</span>
          </div>
          <span>RUSTED Command Engine</span>
        </div>
      </div>
    </div>
  );
};
