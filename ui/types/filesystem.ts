export type FileType = 'file' | 'folder';

export type PreviewKind =
  | 'image'
  | 'video'
  | 'audio'
  | 'code'
  | 'markdown'
  | 'document'
  | 'archive'
  | 'pdf'
  | 'binary';

export interface FileItem {
  id: string;
  name: string;
  path: string;
  parentPath: string;
  type: FileType;
  extension?: string;
  size: number; // in bytes
  createdAt: string;
  modifiedAt: string;
  mimeType?: string;
  isStarred?: boolean;
  isHidden?: boolean;
  isSystem?: boolean;
  previewKind?: PreviewKind;
  previewUrl?: string;
  codeLanguage?: string;
  textContent?: string;
  dimensions?: { width: number; height: number };
  duration?: string; // e.g. "03:42" for audio/video
  itemCount?: number; // for folders: number of direct children
  childrenIds?: string[];
}

export type ViewMode = 'miller' | 'grid' | 'list' | 'compact';

export type SortField = 'name' | 'size' | 'modifiedAt' | 'type';
export type SortOrder = 'asc' | 'desc';

export interface TabState {
  id: string;
  title: string;
  path: string;
  history: string[];
  historyIndex: number;
  viewMode: ViewMode;
  selectedIds: string[];
  sortField: SortField;
  sortOrder: SortOrder;
}

export interface DriveInfo {
  id: string;
  name: string;
  mountPoint: string;
  fileSystem: string;
  totalBytes: number;
  usedBytes: number;
  freeBytes: number;
  iconType: 'internal' | 'external' | 'network';
}

export interface StorageCategoryBreakdown {
  category: string;
  bytes: number;
  color: string;
  fileCount: number;
}

export type OperationType = 'copy' | 'move' | 'delete' | 'archive' | 'duplicate';

export interface FileOperationJob {
  id: string;
  type: OperationType;
  title: string;
  sourcePaths: string[];
  destinationPath?: string;
  progress: number; // 0 to 100
  totalBytes: number;
  processedBytes: number;
  speed: string; // e.g. "128 MB/s"
  status: 'running' | 'completed' | 'paused' | 'cancelled' | 'error';
  startedAt: string;
}

export interface ContextMenuPosition {
  x: number;
  y: number;
  type: 'file' | 'folder' | 'background' | 'tab' | 'sidebar';
  targetItem?: FileItem;
  targetPath?: string;
}

export interface SearchFilterState {
  query: string;
  extensions: string[];
  types: PreviewKind[];
  sizeRange: 'all' | 'small' | 'medium' | 'large'; // <1MB, 1-50MB, >50MB
  modifiedWithin: 'all' | 'today' | 'thisWeek' | 'thisMonth' | 'thisYear';
}

export type ThemeId =
  | 'obsidian'
  | 'midnight'
  | 'graphite'
  | 'pure-black'
  | 'snow'
  | 'warm-sand';

export interface ThemeDefinition {
  id: ThemeId;
  name: string;
  description: string;
  isDark: boolean;
  previewColors: {
    bg: string;
    sidebar: string;
    surface: string;
    border: string;
    accent: string;
    text: string;
  };
}

