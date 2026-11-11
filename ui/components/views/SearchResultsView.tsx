import React, { useEffect, useState } from 'react';
import { Search, X } from 'lucide-react';
import { useAppStore } from '../../store/useAppStore';
import { fileSystemService } from '../../services/FileSystemService';
import { FileItem, PreviewKind } from '../../types/filesystem';
import { getFileIcon, formatBytes, formatDate } from '../../utils/formatters';

export const SearchResultsView: React.FC = () => {
  const {
    searchQuery,
    setSearchQuery,
    searchFilters,
    setSearchFilters,
    navigateTab,
    activeTabId,
    selectItem,
    setActiveItem,
    openContextMenu,
  } = useAppStore();

  const [results, setResults] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function executeSearch() {
      setLoading(true);
      const res = await fileSystemService.search(searchQuery, searchFilters);
      if (!cancelled) {
        setResults(res);
        setLoading(false);
      }
    }

    executeSearch();

    return () => {
      cancelled = true;
    };
  }, [searchQuery, searchFilters]);

  const handleOpen = (item: FileItem) => {
    if (item.type === 'folder') {
      navigateTab(activeTabId, item.path);
    } else {
      selectItem(item.id, false);
      setActiveItem(item);
      navigateTab(activeTabId, item.parentPath);
    }
  };

  const toggleTypeFilter = (kind: PreviewKind) => {
    const current = searchFilters.types || [];
    const next = current.includes(kind) ? current.filter((k) => k !== kind) : [...current, kind];
    setSearchFilters({ types: next });
  };

  const handleSizeChange = (val: 'all' | 'small' | 'medium' | 'large') => {
    setSearchFilters({ sizeRange: val });
  };

  const renderHighlightedName = (name: string, q: string) => {
    if (!q.trim()) return name;
    const parts = name.split(new RegExp(`(${q})`, 'gi'));
    return (
      <span>
        {parts.map((part, i) =>
          part.toLowerCase() === q.toLowerCase() ? (
            <mark key={i} className="bg-[var(--selection-bg)] text-[var(--accent)] font-semibold px-0.5 rounded-[4px]">
              {part}
            </mark>
          ) : (
            part
          )
        )}
      </span>
    );
  };

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150">
      <div className="max-w-5xl mx-auto space-y-4">
        {/* Search Query & Filters Header */}
        <div className="p-4 bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] space-y-3 shadow-xs">
          <div className="flex items-center gap-3">
            <div className="relative flex-1">
              <Search className="w-4 h-4 text-[var(--text-muted)] absolute left-3 top-2.5" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by filename or keyword..."
                className="w-full h-9 pl-9 pr-8 bg-[var(--bg-app)] border border-[var(--border-subtle)] rounded-[10px] text-[var(--text-primary)] placeholder-[var(--text-muted)] text-sm focus:border-[var(--accent)] focus:outline-none"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  className="absolute right-2.5 top-2.5 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                >
                  <X className="w-4 h-4" />
                </button>
              )}
            </div>
            <span className="font-mono text-[var(--text-secondary)] text-xs shrink-0">
              {results.length} matches
            </span>
          </div>

          {/* Filter Chips */}
          <div className="flex flex-wrap items-center gap-2 pt-2 border-t border-[var(--border-subtle)]">
            <span className="text-[var(--text-muted)] text-[11px] font-semibold">Filter:</span>

            {/* Type Toggles */}
            {(['code', 'image', 'video', 'audio', 'pdf', 'archive'] as PreviewKind[]).map((k) => {
              const active = searchFilters.types?.includes(k);
              return (
                <button
                  key={k}
                  onClick={() => toggleTypeFilter(k)}
                  className={`px-2.5 py-1 rounded-[8px] capitalize text-[11px] transition-colors border ${
                    active
                      ? 'bg-[var(--selection-bg)] border-[var(--accent)] text-[var(--text-primary)] font-semibold'
                      : 'bg-[var(--bg-app)] border-[var(--border-subtle)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
                  }`}
                >
                  {k}
                </button>
              );
            })}

            <div className="h-4 w-px bg-[var(--border-subtle)] mx-1" />

            {/* Size options */}
            <span className="text-[var(--text-muted)] text-[11px]">Size:</span>
            {(['all', 'small', 'medium', 'large'] as const).map((s) => (
              <button
                key={s}
                onClick={() => handleSizeChange(s)}
                className={`px-2.5 py-1 rounded-[8px] capitalize text-[11px] transition-colors border ${
                  searchFilters.sizeRange === s
                    ? 'bg-[var(--selection-bg)] border-[var(--accent)] text-[var(--text-primary)] font-semibold'
                    : 'bg-[var(--bg-app)] border-[var(--border-subtle)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
                }`}
              >
                {s === 'all'
                  ? 'Any'
                  : s === 'small'
                  ? '< 1MB'
                  : s === 'medium'
                  ? '1-50MB'
                  : '> 50MB'}
              </button>
            ))}
          </div>
        </div>

        {/* Results List */}
        <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] overflow-hidden shadow-xs">
          {loading ? (
            <div className="p-8 text-center text-[var(--text-muted)]">Searching virtual filesystem...</div>
          ) : results.length === 0 ? (
            <div className="p-12 text-center text-[var(--text-muted)]">
              <p className="text-sm font-medium text-[var(--text-secondary)]">No matching files found</p>
              <p className="text-xs text-[var(--text-muted)] mt-1">Try broadening your search query or filters</p>
            </div>
          ) : (
            <div className="divide-y divide-[var(--border-subtle)]/40">
              {results.map((item) => (
                <div
                  key={item.id}
                  onClick={() => handleOpen(item)}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    openContextMenu({
                      x: e.clientX,
                      y: e.clientY,
                      type: item.type === 'folder' ? 'folder' : 'file',
                      targetItem: item,
                      targetPath: item.parentPath,
                    });
                  }}
                  className="flex items-center px-4 py-2.5 hover:bg-[var(--bg-hover)] cursor-pointer transition-colors group"
                >
                  <div className="flex-1 flex items-center gap-2.5 min-w-0 pr-2">
                    <span className="shrink-0">{getFileIcon(item)}</span>
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-[var(--text-primary)] font-medium">
                        {renderHighlightedName(item.name, searchQuery)}
                      </p>
                      <p className="truncate text-[10px] text-[var(--text-muted)] font-mono">
                        {item.parentPath}
                      </p>
                    </div>
                  </div>

                  <span className="text-[10px] text-[var(--text-muted)] font-mono shrink-0 mr-4">
                    {item.type === 'folder' ? 'Folder' : formatBytes(item.size)}
                  </span>

                  <span className="text-[10px] text-[var(--text-muted)] font-mono shrink-0 hidden sm:block">
                    {formatDate(item.modifiedAt)}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
