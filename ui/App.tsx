import React, { useEffect } from 'react';
import { useAppStore } from './store/useAppStore';
import { WindowHeader } from './components/layout/WindowHeader';
import { NavigationToolbar } from './components/layout/NavigationToolbar';
import { Sidebar } from './components/layout/Sidebar';
import { StatusBar } from './components/layout/StatusBar';
import { MillerColumnsView } from './components/views/MillerColumnsView';
import { GridView } from './components/views/GridView';
import { ListView } from './components/views/ListView';
import { CompactView } from './components/views/CompactView';
import { HomeView } from './components/views/HomeView';
import { StorageView } from './components/views/StorageView';
import { RecentView } from './components/views/RecentView';
import { StarredView } from './components/views/StarredView';
import { SearchResultsView } from './components/views/SearchResultsView';
import { FileInspector } from './components/inspector/FileInspector';
import { CommandPalette } from './components/modals/CommandPalette';
import { ContextMenu } from './components/modals/ContextMenu';
import { ActionDialogs } from './components/modals/ActionDialogs';
import { OperationQueueModal } from './components/modals/OperationQueueModal';
import { SettingsModal } from './components/modals/SettingsModal';

export default function App() {
  const {
    currentPath,
    tabs,
    activeTabId,
    loadCurrentDirectory,
    isSearching,
    createTab,
    closeTab,
    goBack,
    goForward,
    setViewMode,
    toggleSidebar,
    toggleInspector,
    openCommandPalette,
    openNewFolderModal,
    openRenameModal,
    openDeleteModal,
    copySelection,
    cutSelection,
    pasteToPath,
    selectAll,
    clearSelection,
    directoryItems,
    selectedIds,
    setIsEditingAddress,
    theme,
  } = useAppStore();

  const activeTab = tabs.find((t) => t.id === activeTabId);
  const currentViewMode = activeTab?.viewMode || 'miller';

  // Initialize directory on mount
  useEffect(() => {
    loadCurrentDirectory();
  }, []);

  // Sync theme with data-theme attribute and dark class
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    if (theme === 'snow' || theme === 'warm-sand') {
      document.documentElement.classList.remove('dark');
    } else {
      document.documentElement.classList.add('dark');
    }
  }, [theme]);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore shortcuts if user is typing inside an input or textarea
      const targetTag = (e.target as HTMLElement)?.tagName?.toLowerCase();
      const isInput = targetTag === 'input' || targetTag === 'textarea';

      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const mod = isMac ? e.metaKey : e.ctrlKey;

      if (mod && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        openCommandPalette();
        return;
      }

      if (mod && e.key.toLowerCase() === 't') {
        e.preventDefault();
        createTab('/Home', 'Home');
        return;
      }

      if (mod && e.key.toLowerCase() === 'w') {
        e.preventDefault();
        if (tabs.length > 1) {
          closeTab(activeTabId);
        }
        return;
      }

      if (mod && e.key.toLowerCase() === 'l') {
        e.preventDefault();
        setIsEditingAddress(true);
        return;
      }

      if (mod && e.key.toLowerCase() === 'b') {
        e.preventDefault();
        toggleSidebar();
        return;
      }

      if (mod && e.key.toLowerCase() === 'i') {
        e.preventDefault();
        toggleInspector();
        return;
      }

      if (mod && e.shiftKey && e.key.toLowerCase() === 'n') {
        e.preventDefault();
        openNewFolderModal(currentPath);
        return;
      }

      if (mod && e.key === '1') {
        e.preventDefault();
        setViewMode('miller');
        return;
      }
      if (mod && e.key === '2') {
        e.preventDefault();
        setViewMode('grid');
        return;
      }
      if (mod && e.key === '3') {
        e.preventDefault();
        setViewMode('list');
        return;
      }
      if (mod && e.key === '4') {
        e.preventDefault();
        setViewMode('compact');
        return;
      }

      if (e.altKey && e.key === 'ArrowLeft') {
        e.preventDefault();
        goBack();
        return;
      }
      if (e.altKey && e.key === 'ArrowRight') {
        e.preventDefault();
        goForward();
        return;
      }

      // Non-input selection and file operation hotkeys
      if (!isInput) {
        if (mod && e.key.toLowerCase() === 'a') {
          e.preventDefault();
          selectAll(directoryItems.map((i) => i.id));
          return;
        }

        if (mod && e.key.toLowerCase() === 'c') {
          e.preventDefault();
          copySelection();
          return;
        }

        if (mod && e.key.toLowerCase() === 'x') {
          e.preventDefault();
          cutSelection();
          return;
        }

        if (mod && e.key.toLowerCase() === 'v') {
          e.preventDefault();
          pasteToPath();
          return;
        }

        if (e.key === 'F2') {
          e.preventDefault();
          const target = directoryItems.find((i) => selectedIds.includes(i.id));
          if (target) openRenameModal(target);
          return;
        }

        if (e.key === 'Delete') {
          e.preventDefault();
          const targets = directoryItems.filter((i) => selectedIds.includes(i.id));
          if (targets.length > 0) openDeleteModal(targets);
          return;
        }

        if (e.key === 'Escape') {
          clearSelection();
          return;
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [
    activeTabId,
    tabs.length,
    currentPath,
    directoryItems,
    selectedIds,
    openCommandPalette,
    createTab,
    closeTab,
    setIsEditingAddress,
    toggleSidebar,
    toggleInspector,
    openNewFolderModal,
    openRenameModal,
    openDeleteModal,
    setViewMode,
    goBack,
    goForward,
    selectAll,
    copySelection,
    cutSelection,
    pasteToPath,
    clearSelection,
  ]);

  // Determine main view rendering
  const renderMainView = () => {
    if (isSearching) {
      return <SearchResultsView />;
    }

    if (currentPath === '/Home') {
      return <HomeView />;
    }

    if (currentPath === '/Home/Storage') {
      return <StorageView />;
    }

    if (currentPath === '/Home/Recent') {
      return <RecentView />;
    }

    if (currentPath === '/Home/Starred') {
      return <StarredView />;
    }

    switch (currentViewMode) {
      case 'miller':
        return <MillerColumnsView />;
      case 'grid':
        return <GridView />;
      case 'list':
        return <ListView />;
      case 'compact':
        return <CompactView />;
      default:
        return <MillerColumnsView />;
    }
  };

  return (
    <div className="w-screen h-screen flex flex-col bg-[var(--bg-app)] text-[var(--text-primary)] overflow-hidden font-sans select-none antialiased transition-colors duration-150">
      {/* 1. Window Frame & Tabs Header */}
      <WindowHeader />

      {/* 2. Navigation & Breadcrumb Toolbar */}
      <NavigationToolbar />

      {/* 3. Main Workspace Area: Sidebar + Active View + Inspector */}
      <div className="flex-1 flex overflow-hidden relative">
        <Sidebar />
        <main className="flex-1 flex flex-col min-w-0 overflow-hidden relative bg-[var(--bg-app)]">
          {renderMainView()}
        </main>
        <FileInspector />
      </div>

      {/* 4. Desktop Status Bar */}
      <StatusBar />

      {/* 5. Modals & Overlays */}
      <CommandPalette />
      <ContextMenu />
      <ActionDialogs />
      <OperationQueueModal />
      <SettingsModal />
    </div>
  );
}
