import React, { useState } from 'react';
import {
  Settings,
  X,
  Keyboard,
  Columns,
  LayoutGrid,
  List,
  AlignJustify,
  Eye,
  PanelRight,
  Check,
  Info,
} from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { ViewMode, ThemeId } from '../../types/filesystem';
import { THEMES } from '../../services/themes';
import { BrandIcon } from '../common/BrandIcon';

export const SettingsModal: React.FC = () => {
  const {
    isSettingsOpen,
    closeSettings,
    theme,
    setTheme,
    gridDensity,
    setGridDensity,
    showHiddenFiles,
    toggleHiddenFiles,
    isInspectorOpen,
    toggleInspector,
    setViewMode,
    tabs,
    activeTabId,
  } = useAppStore();

  const [activeTab, setActiveTab] = useState<'themes' | 'shortcuts' | 'about'>('themes');

  const currentTab = tabs.find((t) => t.id === activeTabId);
  const currentViewMode: ViewMode = currentTab?.viewMode || 'miller';

  if (!isSettingsOpen) return null;

  const shortcutsList = [
    { key: 'Ctrl + T', desc: 'Open new tab' },
    { key: 'Ctrl + W', desc: 'Close active tab' },
    { key: 'Ctrl + L', desc: 'Focus and edit address bar path' },
    { key: 'Ctrl + F', desc: 'Instant file search' },
    { key: 'Ctrl + K', desc: 'Open Command Palette' },
    { key: 'Ctrl + Shift + N', desc: 'Create new folder' },
    { key: 'Alt + Left', desc: 'Navigate back in tab history' },
    { key: 'Alt + Right', desc: 'Navigate forward in tab history' },
    { key: 'Ctrl + 1', desc: 'Switch to Miller Columns View' },
    { key: 'Ctrl + 2', desc: 'Switch to Grid View' },
    { key: 'Ctrl + 3', desc: 'Switch to Detailed List View' },
    { key: 'Ctrl + 4', desc: 'Switch to Compact Power-User View' },
    { key: 'Ctrl + B', desc: 'Toggle Sidebar' },
    { key: 'Ctrl + I', desc: 'Toggle Info / Preview Inspector' },
    { key: 'F2', desc: 'Rename selected file or folder' },
    { key: 'Delete', desc: 'Delete selected items' },
    { key: 'Ctrl + A', desc: 'Select all items in directory' },
    { key: 'Ctrl + C / X / V', desc: 'Copy / Cut / Paste clipboard' },
    { key: 'Ctrl + R', desc: 'Refresh directory' },
  ];

  return (
    <div
      onClick={closeSettings}
      className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 select-none animate-in fade-in"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-2xl bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[18px] shadow-2xl p-5 text-xs text-[var(--text-primary)] transition-colors duration-150 flex flex-col max-h-[85vh]"
      >
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)] shrink-0">
          <div className="flex items-center gap-2 font-semibold text-sm">
            <Settings className="w-4 h-4 text-[var(--accent)]" />
            <span>Rusted Preferences</span>
          </div>
          <button
            onClick={closeSettings}
            className="w-7 h-7 rounded-[8px] flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center gap-1.5 border-b border-[var(--border-subtle)] pt-2.5 pb-2 shrink-0">
          <button
            onClick={() => setActiveTab('themes')}
            className={`px-3 py-1.5 rounded-[10px] font-medium transition-colors border ${
              activeTab === 'themes'
                ? 'bg-[var(--bg-surface-elevated)] border-[var(--border-strong)] text-[var(--text-primary)] shadow-2xs'
                : 'border-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
            }`}
          >
            Appearance & Themes
          </button>
          <button
            onClick={() => setActiveTab('shortcuts')}
            className={`px-3 py-1.5 rounded-[10px] font-medium transition-colors border ${
              activeTab === 'shortcuts'
                ? 'bg-[var(--bg-surface-elevated)] border-[var(--border-strong)] text-[var(--text-primary)] shadow-2xs'
                : 'border-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
            }`}
          >
            Keyboard Shortcuts
          </button>
          <button
            onClick={() => setActiveTab('about')}
            className={`px-3 py-1.5 rounded-[10px] font-medium transition-colors border ${
              activeTab === 'about'
                ? 'bg-[var(--bg-surface-elevated)] border-[var(--border-strong)] text-[var(--text-primary)] shadow-2xs'
                : 'border-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
            }`}
          >
            About Rusted
          </button>
        </div>

        {/* Tab Content */}
        <div className="py-4 overflow-y-auto flex-1 space-y-5 pr-1">
          {activeTab === 'themes' ? (
            <div className="space-y-5">
              {/* Theme Grid */}
              <div>
                <div className="mb-2.5">
                  <p className="font-semibold text-sm text-[var(--text-primary)]">Color Theme</p>
                  <p className="text-[11px] text-[var(--text-muted)]">
                    Choose from 6 curated visual themes. Obsidian Black is the default desktop aesthetic.
                  </p>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
                  {THEMES.map((th) => {
                    const isActive = theme === th.id;
                    const c = th.previewColors;

                    return (
                      <div
                        key={th.id}
                        onClick={() => setTheme(th.id)}
                        className={`p-3 rounded-[12px] border cursor-pointer transition-all flex flex-col justify-between ${
                          isActive
                            ? 'border-[var(--accent)] ring-1 ring-[var(--accent)]/40 bg-[var(--bg-surface-elevated)] shadow-xs'
                            : 'border-[var(--border-subtle)] hover:border-[var(--border-strong)] bg-[var(--bg-app)]/50 hover:bg-[var(--bg-hover)]'
                        }`}
                      >
                        {/* Visual Swatch Mockup */}
                        <div
                          className="w-full h-16 rounded-[8px] p-1.5 mb-2.5 flex flex-col justify-between overflow-hidden border"
                          style={{
                            backgroundColor: c.bg,
                            borderColor: c.border,
                          }}
                        >
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-1">
                              <div
                                className="w-2 h-2 rounded-full"
                                style={{ backgroundColor: c.accent }}
                              />
                              <div
                                className="w-8 h-2 rounded-xs"
                                style={{ backgroundColor: c.border }}
                              />
                            </div>
                            <div
                              className="w-4 h-2 rounded-xs"
                              style={{ backgroundColor: c.border }}
                            />
                          </div>

                          <div className="flex gap-1.5 items-end">
                            <div
                              className="w-10 h-7 rounded-xs border"
                              style={{
                                backgroundColor: c.sidebar,
                                borderColor: c.border,
                              }}
                            />
                            <div
                              className="flex-1 h-7 rounded-xs border p-1"
                              style={{
                                backgroundColor: c.surface,
                                borderColor: c.border,
                              }}
                            >
                              <div
                                className="w-6 h-1 rounded-xs mb-1"
                                style={{ backgroundColor: c.text, opacity: 0.8 }}
                              />
                              <div
                                className="w-10 h-1 rounded-xs"
                                style={{ backgroundColor: c.text, opacity: 0.4 }}
                              />
                            </div>
                          </div>
                        </div>

                        {/* Theme Name & Badge */}
                        <div>
                          <div className="flex items-center justify-between gap-1 mb-1">
                            <span className="font-semibold text-xs text-[var(--text-primary)]">
                              {th.name}
                            </span>
                            {isActive && (
                              <span className="flex items-center gap-0.5 text-[10px] text-emerald-400 font-medium">
                                <Check className="w-3 h-3" />
                                <span>Active</span>
                              </span>
                            )}
                          </div>
                          <p className="text-[10px] text-[var(--text-muted)] line-clamp-2 leading-relaxed">
                            {th.description}
                          </p>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Default View Mode */}
              <div className="p-3.5 rounded-[12px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)] space-y-2">
                <p className="font-semibold text-xs text-[var(--text-primary)]">Default Browsing View</p>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                  {[
                    { mode: 'miller' as const, label: 'Miller Columns', icon: Columns },
                    { mode: 'grid' as const, label: 'Grid Thumbnails', icon: LayoutGrid },
                    { mode: 'list' as const, label: 'Detailed List', icon: List },
                    { mode: 'compact' as const, label: 'Compact Dense', icon: AlignJustify },
                  ].map((m) => {
                    const Icon = m.icon;
                    const active = currentViewMode === m.mode;
                    return (
                      <button
                        key={m.mode}
                        onClick={() => setViewMode(m.mode)}
                        className={`flex flex-col items-center justify-center p-2.5 rounded-[10px] border text-center transition-all ${
                          active
                            ? 'bg-[var(--bg-surface)] border-[var(--accent)] text-[var(--text-primary)] font-medium shadow-2xs'
                            : 'bg-[var(--bg-surface)] border-[var(--border-subtle)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
                        }`}
                      >
                        <Icon className="w-4 h-4 mb-1 text-[var(--accent)]" />
                        <span className="text-[11px]">{m.label}</span>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Grid Density */}
              <div className="flex items-center justify-between p-3.5 rounded-[12px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)]">
                <div>
                  <p className="font-semibold text-xs text-[var(--text-primary)]">Grid Thumbnail Size</p>
                  <p className="text-[11px] text-[var(--text-muted)]">Adjust thumbnail dimensions for Grid mode</p>
                </div>
                <div className="flex items-center gap-1 bg-[var(--bg-surface)] p-1 rounded-[8px] border border-[var(--border-subtle)]">
                  {(['compact', 'normal', 'spacious'] as const).map((d) => (
                    <button
                      key={d}
                      onClick={() => setGridDensity(d)}
                      className={`px-2.5 py-1 rounded-[6px] capitalize text-[11px] transition-colors ${
                        gridDensity === d
                          ? 'bg-[var(--bg-hover)] text-[var(--text-primary)] font-bold'
                          : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                      }`}
                    >
                      {d}
                    </button>
                  ))}
                </div>
              </div>

              {/* Layout Toggles */}
              <div className="space-y-2">
                <div className="flex items-center justify-between p-3.5 rounded-[12px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)]">
                  <div className="flex items-center gap-2.5">
                    <Eye className="w-4 h-4 text-[var(--text-secondary)]" />
                    <div>
                      <p className="font-semibold text-xs text-[var(--text-primary)]">Show Hidden Files & Dotfiles</p>
                      <p className="text-[11px] text-[var(--text-muted)]">
                        Display dotfiles starting with "." or hidden system attribute
                      </p>
                    </div>
                  </div>
                  <input
                    type="checkbox"
                    checked={showHiddenFiles}
                    onChange={toggleHiddenFiles}
                    className="w-4 h-4 rounded border-[var(--border-strong)] bg-[var(--bg-surface)] text-[var(--accent)] cursor-pointer"
                  />
                </div>

                <div className="flex items-center justify-between p-3.5 rounded-[12px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)]">
                  <div className="flex items-center gap-2.5">
                    <PanelRight className="w-4 h-4 text-[var(--text-secondary)]" />
                    <div>
                      <p className="font-semibold text-xs text-[var(--text-primary)]">Enable Right Inspector Panel</p>
                      <p className="text-[11px] text-[var(--text-muted)]">
                        Real-time quick preview and extended metadata attributes
                      </p>
                    </div>
                  </div>
                  <input
                    type="checkbox"
                    checked={isInspectorOpen}
                    onChange={toggleInspector}
                    className="w-4 h-4 rounded border-[var(--border-strong)] bg-[var(--bg-surface)] text-[var(--accent)] cursor-pointer"
                  />
                </div>
              </div>
            </div>
          ) : activeTab === 'shortcuts' ? (
            <div className="space-y-1">
              <div className="grid grid-cols-2 gap-2 text-[var(--text-muted)] font-semibold text-[11px] px-2.5 pb-1.5 border-b border-[var(--border-subtle)]">
                <span>Action</span>
                <span className="text-right">Keyboard Shortcut</span>
              </div>
              {shortcutsList.map((sc, i) => (
                <div
                  key={i}
                  className="grid grid-cols-2 gap-2 items-center px-2.5 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)]"
                >
                  <span className="text-[var(--text-primary)]">{sc.desc}</span>
                  <div className="text-right">
                    <kbd className="px-2 py-0.5 rounded-[6px] bg-[var(--bg-app)] border border-[var(--border-subtle)] font-mono text-[10px] text-[var(--text-secondary)]">
                      {sc.key}
                    </kbd>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-4 py-2">
              <div className="flex items-center gap-3.5 p-4 rounded-[14px] bg-[var(--bg-app)]/60 border border-[var(--border-subtle)]">
                <div className="w-12 h-12 rounded-[12px] bg-[var(--bg-surface)] border border-[var(--border-subtle)] flex items-center justify-center text-[var(--accent)] shadow-xs">
                  <BrandIcon size={28} />
                </div>
                <div>
                  <h3 className="text-base font-bold text-[var(--text-primary)]">RUSTED</h3>
                  <p className="text-xs text-[var(--text-secondary)]">Rusted File Manager — Version 2.4.0</p>
                  <p className="text-[10px] text-[var(--text-muted)] font-mono mt-0.5">Tauri 2 & Rust Core Architecture</p>
                </div>
              </div>

              <div className="space-y-2.5 text-[var(--text-secondary)] leading-relaxed text-xs p-3">
                <p>
                  <strong className="text-[var(--text-primary)]">Rusted</strong> is a next-generation desktop file manager engineered for speed, architectural clarity, and power-user workflows.
                </p>
                <p>
                  Featuring cascading Miller Columns navigation, an instant Command Palette (<kbd className="font-mono px-1 rounded bg-[var(--bg-app)] border border-[var(--border-subtle)] text-[10px]">Ctrl+K</kbd>), multi-tab session management, and rich in-place file previews for code, vector graphics, audio waveforms, and video frames.
                </p>
                <p className="text-[11px] text-[var(--text-muted)] border-t border-[var(--border-subtle)] pt-3 font-mono">
                  Engine: React 19 + TypeScript + Tailwind CSS · Decoupled IFileSystemService · Tauri 2 Ready
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end pt-3 border-t border-[var(--border-subtle)] shrink-0 mt-2">
          <button
            onClick={closeSettings}
            className="px-4 py-1.5 rounded-[10px] bg-[var(--accent)] text-[var(--accent-text)] font-semibold transition-colors hover:opacity-90 shadow-2xs"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
};
