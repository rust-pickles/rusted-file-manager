import { create } from 'zustand';
import {
  FileItem,
  TabState,
  ViewMode,
  SortField,
  SortOrder,
  ContextMenuPosition,
  FileOperationJob,
  SearchFilterState,
  ThemeId,
} from '../types/filesystem';
import { fileSystemService } from '../services/FileSystemService';

interface AppState {
  // Tabs
  tabs: TabState[];
  activeTabId: string;
  createTab: (path?: string, title?: string) => void;
  closeTab: (tabId: string) => void;
  duplicateTab: (tabId: string) => void;
  switchTab: (tabId: string) => void;
  navigateTab: (tabId: string, path: string) => void;
  goBack: (tabId?: string) => void;
  goForward: (tabId?: string) => void;
  refreshTab: (tabId?: string) => void;
  setViewMode: (mode: ViewMode) => void;
  setSort: (field: SortField, order?: SortOrder) => void;

  // Selection
  selectedIds: string[];
  selectItem: (id: string, multiSelect?: boolean, rangeSelect?: boolean) => void;
  selectAll: (allIds: string[]) => void;
  clearSelection: () => void;
  activeItem: FileItem | null;
  setActiveItem: (item: FileItem | null) => void;

  // Navigation & Breadcrumbs
  currentPath: string;
  isEditingAddress: boolean;
  setIsEditingAddress: (editing: boolean) => void;

  // UI Panels
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
  isInspectorOpen: boolean;
  toggleInspector: () => void;
  theme: ThemeId;
  setTheme: (theme: ThemeId) => void;
  toggleTheme: () => void;
  showHiddenFiles: boolean;
  toggleHiddenFiles: () => void;
  gridDensity: 'compact' | 'normal' | 'spacious';
  setGridDensity: (density: 'compact' | 'normal' | 'spacious') => void;

  // Modals & Overlays
  isCommandPaletteOpen: boolean;
  openCommandPalette: () => void;
  closeCommandPalette: () => void;

  isSettingsOpen: boolean;
  openSettings: () => void;
  closeSettings: () => void;

  isNewFolderModalOpen: boolean;
  openNewFolderModal: (parentPath?: string) => void;
  closeNewFolderModal: () => void;

  isRenameModalOpen: boolean;
  itemToRename: FileItem | null;
  openRenameModal: (item: FileItem) => void;
  closeRenameModal: () => void;

  isDeleteModalOpen: boolean;
  itemsToDelete: FileItem[];
  openDeleteModal: (items: FileItem[]) => void;
  closeDeleteModal: () => void;

  isPropertiesModalOpen: boolean;
  propertiesTargetItem: FileItem | null;
  openPropertiesModal: (item: FileItem) => void;
  closePropertiesModal: () => void;

  isOperationQueueOpen: boolean;
  toggleOperationQueue: () => void;

  // Search
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  searchFilters: SearchFilterState;
  setSearchFilters: (filters: Partial<SearchFilterState>) => void;
  isSearching: boolean;
  setIsSearching: (searching: boolean) => void;

  // Context Menu
  contextMenu: ContextMenuPosition | null;
  openContextMenu: (pos: ContextMenuPosition) => void;
  closeContextMenu: () => void;

  // Clipboard & File Operations
  clipboard: { items: FileItem[]; operation: 'copy' | 'cut' } | null;
  copySelection: () => void;
  cutSelection: () => void;
  pasteToPath: (targetPath?: string) => Promise<void>;

  // Operations Queue
  operationJobs: FileOperationJob[];
  cancelOperation: (jobId: string) => void;

  // Current directory items (cached for active tab)
  directoryItems: FileItem[];
  isLoadingDirectory: boolean;
  loadCurrentDirectory: () => Promise<void>;
}

const DEFAULT_FILTERS: SearchFilterState = {
  query: '',
  extensions: [],
  types: [],
  sizeRange: 'all',
  modifiedWithin: 'all',
};

const INITIAL_TAB: TabState = {
  id: 'tab-1',
  title: 'Documents',
  path: '/Home/Documents',
  history: ['/Home', '/Home/Documents'],
  historyIndex: 1,
  viewMode: 'miller',
  selectedIds: [],
  sortField: 'name',
  sortOrder: 'asc',
};

export const useAppStore = create<AppState>((set, get) => ({
  tabs: [INITIAL_TAB],
  activeTabId: 'tab-1',
  selectedIds: [],
  activeItem: null,
  currentPath: '/Home/Documents',
  isEditingAddress: false,
  isSidebarOpen: true,
  isInspectorOpen: true,
  theme: 'obsidian',
  showHiddenFiles: false,
  gridDensity: 'normal',
  isCommandPaletteOpen: false,
  isSettingsOpen: false,
  isNewFolderModalOpen: false,
  isRenameModalOpen: false,
  itemToRename: null,
  isDeleteModalOpen: false,
  itemsToDelete: [],
  isPropertiesModalOpen: false,
  propertiesTargetItem: null,
  isOperationQueueOpen: false,
  searchQuery: '',
  searchFilters: DEFAULT_FILTERS,
  isSearching: false,
  contextMenu: null,
  clipboard: null,
  operationJobs: [],
  directoryItems: [],
  isLoadingDirectory: false,

  createTab: (path = '/Home', title) => {
    const newId = `tab-${Date.now()}`;
    const cleanTitle = title || (path === '/' ? 'Root' : path.split('/').pop() || 'Folder');
    const newTab: TabState = {
      id: newId,
      title: cleanTitle,
      path,
      history: [path],
      historyIndex: 0,
      viewMode: 'miller',
      selectedIds: [],
      sortField: 'name',
      sortOrder: 'asc',
    };
    set((state) => ({
      tabs: [...state.tabs, newTab],
      activeTabId: newId,
      currentPath: path,
      selectedIds: [],
      activeItem: null,
      isSearching: false,
    }));
    get().loadCurrentDirectory();
  },

  closeTab: (tabId: string) => {
    const { tabs, activeTabId } = get();
    if (tabs.length <= 1) return; // keep at least 1 tab

    const index = tabs.findIndex((t) => t.id === tabId);
    const newTabs = tabs.filter((t) => t.id !== tabId);
    let nextActiveId = activeTabId;

    if (activeTabId === tabId) {
      const nextTab = newTabs[Math.max(0, index - 1)];
      nextActiveId = nextTab.id;
      set({
        tabs: newTabs,
        activeTabId: nextActiveId,
        currentPath: nextTab.path,
        selectedIds: nextTab.selectedIds,
        activeItem: null,
      });
      get().loadCurrentDirectory();
    } else {
      set({ tabs: newTabs });
    }
  },

  duplicateTab: (tabId: string) => {
    const tab = get().tabs.find((t) => t.id === tabId);
    if (!tab) return;
    const newId = `tab-${Date.now()}`;
    const duplicated: TabState = {
      ...tab,
      id: newId,
      title: `${tab.title}`,
      history: [...tab.history],
      selectedIds: [...tab.selectedIds],
    };
    set((state) => ({
      tabs: [...state.tabs, duplicated],
      activeTabId: newId,
      currentPath: tab.path,
      selectedIds: tab.selectedIds,
    }));
    get().loadCurrentDirectory();
  },

  switchTab: (tabId: string) => {
    const tab = get().tabs.find((t) => t.id === tabId);
    if (!tab) return;
    set({
      activeTabId: tabId,
      currentPath: tab.path,
      selectedIds: tab.selectedIds,
      activeItem: null,
      isSearching: false,
    });
    get().loadCurrentDirectory();
  },

  navigateTab: (tabId: string, path: string) => {
    const cleanPath = path === '/' ? '/' : path.replace(/\/$/, '');
    const cleanTitle = cleanPath === '/' ? 'Root' : cleanPath.split('/').pop() || 'Folder';

    set((state) => {
      const targetTabId = tabId || state.activeTabId;
      const updatedTabs = state.tabs.map((tab) => {
        if (tab.id !== targetTabId) return tab;
        const newHistory = tab.history.slice(0, tab.historyIndex + 1);
        newHistory.push(cleanPath);
        return {
          ...tab,
          title: cleanTitle,
          path: cleanPath,
          history: newHistory,
          historyIndex: newHistory.length - 1,
          selectedIds: [],
        };
      });

      return {
        tabs: updatedTabs,
        currentPath: cleanPath,
        selectedIds: [],
        activeItem: null,
        isSearching: false,
      };
    });

    get().loadCurrentDirectory();
  },

  goBack: (tabId?: string) => {
    const { tabs, activeTabId } = get();
    const id = tabId || activeTabId;
    const tab = tabs.find((t) => t.id === id);
    if (!tab || tab.historyIndex <= 0) return;

    const newIndex = tab.historyIndex - 1;
    const prevPath = tab.history[newIndex];
    const prevTitle = prevPath === '/' ? 'Root' : prevPath.split('/').pop() || 'Folder';

    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === id
          ? {
              ...t,
              path: prevPath,
              title: prevTitle,
              historyIndex: newIndex,
              selectedIds: [],
            }
          : t
      ),
      currentPath: prevPath,
      selectedIds: [],
      activeItem: null,
    }));

    get().loadCurrentDirectory();
  },

  goForward: (tabId?: string) => {
    const { tabs, activeTabId } = get();
    const id = tabId || activeTabId;
    const tab = tabs.find((t) => t.id === id);
    if (!tab || tab.historyIndex >= tab.history.length - 1) return;

    const newIndex = tab.historyIndex + 1;
    const nextPath = tab.history[newIndex];
    const nextTitle = nextPath === '/' ? 'Root' : nextPath.split('/').pop() || 'Folder';

    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === id
          ? {
              ...t,
              path: nextPath,
              title: nextTitle,
              historyIndex: newIndex,
              selectedIds: [],
            }
          : t
      ),
      currentPath: nextPath,
      selectedIds: [],
      activeItem: null,
    }));

    get().loadCurrentDirectory();
  },

  refreshTab: () => {
    get().loadCurrentDirectory();
  },

  setViewMode: (mode: ViewMode) => {
    const { activeTabId } = get();
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === activeTabId ? { ...t, viewMode: mode } : t)),
    }));
  },

  setSort: (field: SortField, order?: SortOrder) => {
    const { activeTabId, tabs } = get();
    const tab = tabs.find((t) => t.id === activeTabId);
    if (!tab) return;

    const newOrder = order || (tab.sortField === field && tab.sortOrder === 'asc' ? 'desc' : 'asc');

    set((state) => ({
      tabs: state.tabs.map((t) =>
        t.id === activeTabId ? { ...t, sortField: field, sortOrder: newOrder } : t
      ),
    }));
    get().loadCurrentDirectory();
  },

  selectItem: (id: string, multiSelect = false, _rangeSelect = false) => {
    const { selectedIds, directoryItems } = get();
    let newSelected: string[] = [];

    if (multiSelect) {
      if (selectedIds.includes(id)) {
        newSelected = selectedIds.filter((item) => item !== id);
      } else {
        newSelected = [...selectedIds, id];
      }
    } else {
      newSelected = [id];
    }

    const clickedItem = directoryItems.find((f) => f.id === id) || null;

    set((state) => ({
      selectedIds: newSelected,
      activeItem: clickedItem,
      tabs: state.tabs.map((t) =>
        t.id === state.activeTabId ? { ...t, selectedIds: newSelected } : t
      ),
    }));
  },

  selectAll: (allIds: string[]) => {
    set((state) => ({
      selectedIds: allIds,
      tabs: state.tabs.map((t) =>
        t.id === state.activeTabId ? { ...t, selectedIds: allIds } : t
      ),
    }));
  },

  clearSelection: () => {
    set((state) => ({
      selectedIds: [],
      activeItem: null,
      tabs: state.tabs.map((t) =>
        t.id === state.activeTabId ? { ...t, selectedIds: [] } : t
      ),
    }));
  },

  setActiveItem: (item: FileItem | null) => {
    set({ activeItem: item });
  },

  setIsEditingAddress: (editing: boolean) => {
    set({ isEditingAddress: editing });
  },

  toggleSidebar: () => {
    set((state) => ({ isSidebarOpen: !state.isSidebarOpen }));
  },

  toggleInspector: () => {
    set((state) => ({ isInspectorOpen: !state.isInspectorOpen }));
  },

  setTheme: (newTheme: ThemeId) => {
    document.documentElement.setAttribute('data-theme', newTheme);
    if (newTheme === 'snow' || newTheme === 'warm-sand') {
      document.documentElement.classList.remove('dark');
    } else {
      document.documentElement.classList.add('dark');
    }
    set({ theme: newTheme });
  },

  toggleTheme: () => {
    const current = get().theme;
    const isDark = current !== 'snow' && current !== 'warm-sand';
    const next: ThemeId = isDark ? 'snow' : 'obsidian';
    get().setTheme(next);
  },

  toggleHiddenFiles: () => {
    set((state) => ({ showHiddenFiles: !state.showHiddenFiles }));
    get().loadCurrentDirectory();
  },

  setGridDensity: (density: 'compact' | 'normal' | 'spacious') => {
    set({ gridDensity: density });
  },

  openCommandPalette: () => set({ isCommandPaletteOpen: true }),
  closeCommandPalette: () => set({ isCommandPaletteOpen: false }),

  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),

  openNewFolderModal: (_parentPath) => set({ isNewFolderModalOpen: true }),
  closeNewFolderModal: () => set({ isNewFolderModalOpen: false }),

  openRenameModal: (item: FileItem) => set({ isRenameModalOpen: true, itemToRename: item }),
  closeRenameModal: () => set({ isRenameModalOpen: false, itemToRename: null }),

  openDeleteModal: (items: FileItem[]) => set({ isDeleteModalOpen: true, itemsToDelete: items }),
  closeDeleteModal: () => set({ isDeleteModalOpen: false, itemsToDelete: [] }),

  openPropertiesModal: (item: FileItem) =>
    set({ isPropertiesModalOpen: true, propertiesTargetItem: item }),
  closePropertiesModal: () => set({ isPropertiesModalOpen: false, propertiesTargetItem: null }),

  toggleOperationQueue: () =>
    set((state) => ({ isOperationQueueOpen: !state.isOperationQueueOpen })),

  setSearchQuery: (query: string) => {
    set({ searchQuery: query, isSearching: !!query.trim() });
  },

  setSearchFilters: (filters: Partial<SearchFilterState>) => {
    set((state) => ({
      searchFilters: { ...state.searchFilters, ...filters },
    }));
  },

  setIsSearching: (searching: boolean) => {
    set({ isSearching: searching });
  },

  openContextMenu: (pos: ContextMenuPosition) => {
    set({ contextMenu: pos });
  },

  closeContextMenu: () => {
    set({ contextMenu: null });
  },

  copySelection: () => {
    const { selectedIds, directoryItems } = get();
    const items = directoryItems.filter((i) => selectedIds.includes(i.id));
    if (items.length > 0) {
      set({ clipboard: { items, operation: 'copy' } });
    }
  },

  cutSelection: () => {
    const { selectedIds, directoryItems } = get();
    const items = directoryItems.filter((i) => selectedIds.includes(i.id));
    if (items.length > 0) {
      set({ clipboard: { items, operation: 'cut' } });
    }
  },

  pasteToPath: async (targetPath?: string) => {
    const { clipboard, currentPath } = get();
    if (!clipboard || clipboard.items.length === 0) return;

    const dest = targetPath || currentPath;
    const srcPaths = clipboard.items.map((i) => i.path);
    const totalBytes = clipboard.items.reduce((acc, i) => acc + i.size, 0);

    const jobId = `job-${Date.now()}`;
    const newJob: FileOperationJob = {
      id: jobId,
      type: clipboard.operation === 'copy' ? 'copy' : 'move',
      title: `${clipboard.operation === 'copy' ? 'Copying' : 'Moving'} ${clipboard.items.length} item(s)`,
      sourcePaths: srcPaths,
      destinationPath: dest,
      progress: 5,
      totalBytes,
      processedBytes: Math.floor(totalBytes * 0.05),
      speed: '142 MB/s',
      status: 'running',
      startedAt: new Date().toISOString(),
    };

    set((state) => ({
      operationJobs: [newJob, ...state.operationJobs],
      isOperationQueueOpen: true,
    }));

    // Simulate transfer progress for authentic desktop feel
    const interval = setInterval(() => {
      set((state) => {
        const jobs = state.operationJobs.map((j) => {
          if (j.id !== jobId) return j;
          const nextProg = Math.min(100, j.progress + 25);
          return {
            ...j,
            progress: nextProg,
            processedBytes: Math.floor(totalBytes * (nextProg / 100)),
            status: nextProg >= 100 ? ('completed' as const) : ('running' as const),
          };
        });
        return { operationJobs: jobs };
      });
    }, 200);

    setTimeout(async () => {
      clearInterval(interval);
      if (clipboard.operation === 'copy') {
        await fileSystemService.copyItems(srcPaths, dest);
      } else {
        await fileSystemService.moveItems(srcPaths, dest);
        set({ clipboard: null });
      }
      get().loadCurrentDirectory();
    }, 900);
  },

  cancelOperation: (jobId: string) => {
    set((state) => ({
      operationJobs: state.operationJobs.map((j) =>
        j.id === jobId ? { ...j, status: 'cancelled' as const } : j
      ),
    }));
  },

  loadCurrentDirectory: async () => {
    const { currentPath, showHiddenFiles, tabs, activeTabId } = get();
    const tab = tabs.find((t) => t.id === activeTabId);
    const sortField = tab?.sortField || 'name';
    const sortOrder = tab?.sortOrder || 'asc';

    set({ isLoadingDirectory: true });

    try {
      const items = await fileSystemService.listDirectory(currentPath, showHiddenFiles);

      // Apply sorting
      items.sort((a, b) => {
        if (a.type !== b.type) {
          return a.type === 'folder' ? -1 : 1;
        }

        let result = 0;
        if (sortField === 'name') {
          result = a.name.localeCompare(b.name, undefined, { numeric: true });
        } else if (sortField === 'size') {
          result = (a.size || 0) - (b.size || 0);
        } else if (sortField === 'modifiedAt') {
          result = a.modifiedAt.localeCompare(b.modifiedAt);
        } else if (sortField === 'type') {
          result = (a.extension || '').localeCompare(b.extension || '');
        }

        return sortOrder === 'asc' ? result : -result;
      });

      set({ directoryItems: items, isLoadingDirectory: false });

      // If activeItem is set, ensure it reflects current state
      const { activeItem } = get();
      if (activeItem) {
        const fresh = items.find((i) => i.id === activeItem.id);
        if (fresh) set({ activeItem: fresh });
      }
    } catch {
      set({ directoryItems: [], isLoadingDirectory: false });
    }
  },
}));
