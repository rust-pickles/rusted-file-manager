import {
  FileItem,
  DriveInfo,
  StorageCategoryBreakdown,
  SearchFilterState,
  PreviewKind,
} from '../types/filesystem';
import { INITIAL_FILES, INITIAL_DRIVES } from './mockData';

export interface IFileSystemService {
  listDirectory(path: string, showHidden?: boolean): Promise<FileItem[]>;
  getItem(path: string): Promise<FileItem | undefined>;
  createFolder(parentPath: string, name: string): Promise<FileItem>;
  createFile(parentPath: string, name: string, content?: string): Promise<FileItem>;
  renameItem(path: string, newName: string): Promise<FileItem>;
  deleteItem(path: string, permanent?: boolean): Promise<void>;
  restoreFromTrash(path: string): Promise<void>;
  emptyTrash(): Promise<void>;
  copyItems(sources: string[], destFolder: string): Promise<FileItem[]>;
  moveItems(sources: string[], destFolder: string): Promise<FileItem[]>;
  duplicateItem(path: string): Promise<FileItem>;
  toggleStarred(path: string): Promise<FileItem>;
  search(query: string, filters?: Partial<SearchFilterState>): Promise<FileItem[]>;
  getStorageOverview(): Promise<{ drives: DriveInfo[]; categories: StorageCategoryBreakdown[] }>;
  getRecentFiles(limit?: number): Promise<FileItem[]>;
  getStarredItems(): Promise<FileItem[]>;
}

class MockFileSystemService implements IFileSystemService {
  private files: Map<string, FileItem> = new Map();
  private drives: DriveInfo[] = [...INITIAL_DRIVES];

  constructor() {
    this.reset();
  }

  public reset() {
    this.files.clear();
    INITIAL_FILES.forEach((f) => {
      this.files.set(f.path, { ...f });
    });
  }

  // Determine file preview kind from extension
  private detectPreviewKind(ext?: string): PreviewKind {
    if (!ext) return 'document';
    const lower = ext.toLowerCase();
    if (['jpg', 'jpeg', 'png', 'svg', 'gif', 'webp', 'avif'].includes(lower)) return 'image';
    if (['mp4', 'mov', 'webm', 'mkv', 'avi'].includes(lower)) return 'video';
    if (['mp3', 'wav', 'aac', 'ogg', 'flac'].includes(lower)) return 'audio';
    if (['ts', 'tsx', 'js', 'jsx', 'rs', 'toml', 'json', 'yaml', 'yml', 'css', 'html', 'wgsl', 'py', 'c', 'cpp'].includes(lower)) return 'code';
    if (['md', 'markdown', 'txt'].includes(lower)) return 'markdown';
    if (['pdf'].includes(lower)) return 'pdf';
    if (['zip', 'tar', 'gz', 'tar.gz', '7z', 'rar', 'deb'].includes(lower)) return 'archive';
    return 'document';
  }

  public async listDirectory(path: string, showHidden = false): Promise<FileItem[]> {
    const normalized = path === '/' ? '/' : path.replace(/\/$/, '');
    const results: FileItem[] = [];

    for (const item of this.files.values()) {
      if (item.parentPath === normalized) {
        if (!showHidden && item.isHidden) continue;
        results.push({ ...item });
      }
    }

    // Default sort: folders first, then alphabetical by name
    return results.sort((a, b) => {
      if (a.type !== b.type) {
        return a.type === 'folder' ? -1 : 1;
      }
      return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
    });
  }

  public async getItem(path: string): Promise<FileItem | undefined> {
    const item = this.files.get(path);
    return item ? { ...item } : undefined;
  }

  public async createFolder(parentPath: string, name: string): Promise<FileItem> {
    const cleanParent = parentPath === '/' ? '' : parentPath.replace(/\/$/, '');
    const cleanName = name.trim() || 'New Folder';
    let targetPath = `${cleanParent}/${cleanName}`;
    let counter = 1;

    while (this.files.has(targetPath)) {
      targetPath = `${cleanParent}/${cleanName} (${counter++})`;
    }

    const finalName = targetPath.split('/').pop() || cleanName;
    const newFolder: FileItem = {
      id: `folder-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      name: finalName,
      path: targetPath,
      parentPath: cleanParent || '/',
      type: 'folder',
      size: 4096,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
      itemCount: 0,
      isStarred: false,
    };

    this.files.set(targetPath, newFolder);
    this.updateParentCount(cleanParent || '/', 1);
    return { ...newFolder };
  }

  public async createFile(parentPath: string, name: string, content = ''): Promise<FileItem> {
    const cleanParent = parentPath === '/' ? '' : parentPath.replace(/\/$/, '');
    const cleanName = name.trim() || 'untitled.txt';
    let targetPath = `${cleanParent}/${cleanName}`;
    let counter = 1;

    while (this.files.has(targetPath)) {
      const parts = cleanName.split('.');
      if (parts.length > 1) {
        const ext = parts.pop();
        targetPath = `${cleanParent}/${parts.join('.')}_(${counter++}).${ext}`;
      } else {
        targetPath = `${cleanParent}/${cleanName}_(${counter++})`;
      }
    }

    const finalName = targetPath.split('/').pop() || cleanName;
    const ext = finalName.includes('.') ? finalName.split('.').pop() : undefined;
    const previewKind = this.detectPreviewKind(ext);

    const newFile: FileItem = {
      id: `file-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      name: finalName,
      path: targetPath,
      parentPath: cleanParent || '/',
      type: 'file',
      extension: ext,
      size: content.length || 1024,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
      previewKind,
      textContent: content,
      isStarred: false,
    };

    this.files.set(targetPath, newFile);
    this.updateParentCount(cleanParent || '/', 1);
    return { ...newFile };
  }

  public async renameItem(path: string, newName: string): Promise<FileItem> {
    const item = this.files.get(path);
    if (!item) throw new Error(`File or folder not found: ${path}`);

    const cleanNewName = newName.trim();
    if (!cleanNewName || cleanNewName === item.name) return { ...item };

    const parentPath = item.parentPath;
    const cleanParent = parentPath === '/' ? '' : parentPath;
    const newPath = `${cleanParent}/${cleanNewName}`;

    if (this.files.has(newPath)) {
      throw new Error(`An item named "${cleanNewName}" already exists in this folder.`);
    }

    const ext = cleanNewName.includes('.') ? cleanNewName.split('.').pop() : undefined;
    const updated: FileItem = {
      ...item,
      name: cleanNewName,
      path: newPath,
      extension: item.type === 'file' ? ext : undefined,
      previewKind: item.type === 'file' ? this.detectPreviewKind(ext) : undefined,
      modifiedAt: new Date().toISOString(),
    };

    this.files.delete(path);
    this.files.set(newPath, updated);

    // If folder, update paths of all children recursively
    if (item.type === 'folder') {
      const oldPrefix = `${path}/`;
      const newPrefix = `${newPath}/`;
      for (const [childPath, child] of Array.from(this.files.entries())) {
        if (childPath.startsWith(oldPrefix)) {
          const updatedChildPath = childPath.replace(oldPrefix, newPrefix);
          const updatedParentPath = child.parentPath.startsWith(path)
            ? child.parentPath.replace(path, newPath)
            : child.parentPath;
          this.files.delete(childPath);
          this.files.set(updatedChildPath, {
            ...child,
            path: updatedChildPath,
            parentPath: updatedParentPath,
          });
        }
      }
    }

    return { ...updated };
  }

  public async deleteItem(path: string, permanent = false): Promise<void> {
    const item = this.files.get(path);
    if (!item) return;

    if (permanent || item.parentPath === '/Trash') {
      // Complete deletion
      this.files.delete(path);
      if (item.type === 'folder') {
        const prefix = `${path}/`;
        for (const [childPath] of Array.from(this.files.entries())) {
          if (childPath.startsWith(prefix)) {
            this.files.delete(childPath);
          }
        }
      }
      this.updateParentCount(item.parentPath, -1);
    } else {
      // Move to Trash
      await this.moveItems([path], '/Trash');
    }
  }

  public async restoreFromTrash(path: string): Promise<void> {
    const item = this.files.get(path);
    if (!item) return;
    await this.moveItems([path], '/Home/Desktop');
  }

  public async emptyTrash(): Promise<void> {
    const trashItems = await this.listDirectory('/Trash', true);
    for (const item of trashItems) {
      await this.deleteItem(item.path, true);
    }
  }

  public async copyItems(sources: string[], destFolder: string): Promise<FileItem[]> {
    const results: FileItem[] = [];
    const cleanDest = destFolder === '/' ? '' : destFolder.replace(/\/$/, '');

    for (const src of sources) {
      const item = this.files.get(src);
      if (!item) continue;

      let targetName = item.name;
      let targetPath = `${cleanDest}/${targetName}`;
      let counter = 1;

      while (this.files.has(targetPath)) {
        if (item.type === 'file' && item.extension) {
          const base = item.name.substring(0, item.name.lastIndexOf('.'));
          targetName = `${base} (copy ${counter++}).${item.extension}`;
        } else {
          targetName = `${item.name} (copy ${counter++})`;
        }
        targetPath = `${cleanDest}/${targetName}`;
      }

      const copy: FileItem = {
        ...item,
        id: `item-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
        name: targetName,
        path: targetPath,
        parentPath: cleanDest || '/',
        createdAt: new Date().toISOString(),
        modifiedAt: new Date().toISOString(),
      };

      this.files.set(targetPath, copy);
      results.push({ ...copy });

      if (item.type === 'folder') {
        // Deep copy sub-tree
        const prefix = `${src}/`;
        for (const [childPath, child] of Array.from(this.files.entries())) {
          if (childPath.startsWith(prefix)) {
            const relPath = childPath.substring(prefix.length);
            const newChildPath = `${targetPath}/${relPath}`;
            const newChildParent = newChildPath.substring(0, newChildPath.lastIndexOf('/')) || targetPath;
            this.files.set(newChildPath, {
              ...child,
              id: `item-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
              path: newChildPath,
              parentPath: newChildParent,
            });
          }
        }
      }
    }

    this.updateParentCount(cleanDest || '/', results.length);
    return results;
  }

  public async moveItems(sources: string[], destFolder: string): Promise<FileItem[]> {
    const results: FileItem[] = [];
    const cleanDest = destFolder === '/' ? '' : destFolder.replace(/\/$/, '');

    for (const src of sources) {
      const item = this.files.get(src);
      if (!item) continue;

      // Prevent moving a folder into itself or its own subfolder
      if (item.type === 'folder' && (destFolder === src || destFolder.startsWith(`${src}/`))) {
        continue;
      }

      let targetName = item.name;
      let targetPath = `${cleanDest}/${targetName}`;
      let counter = 1;

      while (this.files.has(targetPath) && targetPath !== src) {
        if (item.type === 'file' && item.extension) {
          const base = item.name.substring(0, item.name.lastIndexOf('.'));
          targetName = `${base} (${counter++}).${item.extension}`;
        } else {
          targetName = `${item.name} (${counter++})`;
        }
        targetPath = `${cleanDest}/${targetName}`;
      }

      if (targetPath === src) {
        results.push({ ...item });
        continue;
      }

      const moved: FileItem = {
        ...item,
        name: targetName,
        path: targetPath,
        parentPath: cleanDest || '/',
        modifiedAt: new Date().toISOString(),
      };

      this.files.delete(src);
      this.files.set(targetPath, moved);
      results.push({ ...moved });

      if (item.type === 'folder') {
        const oldPrefix = `${src}/`;
        const newPrefix = `${targetPath}/`;
        for (const [childPath, child] of Array.from(this.files.entries())) {
          if (childPath.startsWith(oldPrefix)) {
            const updatedChildPath = childPath.replace(oldPrefix, newPrefix);
            const updatedParentPath = child.parentPath.startsWith(src)
              ? child.parentPath.replace(src, targetPath)
              : child.parentPath;
            this.files.delete(childPath);
            this.files.set(updatedChildPath, {
              ...child,
              path: updatedChildPath,
              parentPath: updatedParentPath,
            });
          }
        }
      }

      this.updateParentCount(item.parentPath, -1);
      this.updateParentCount(cleanDest || '/', 1);
    }

    return results;
  }

  public async duplicateItem(path: string): Promise<FileItem> {
    const item = this.files.get(path);
    if (!item) throw new Error(`Item not found: ${path}`);
    const copied = await this.copyItems([path], item.parentPath);
    return copied[0];
  }

  public async toggleStarred(path: string): Promise<FileItem> {
    const item = this.files.get(path);
    if (!item) throw new Error(`Item not found: ${path}`);
    const updated: FileItem = {
      ...item,
      isStarred: !item.isStarred,
    };
    this.files.set(path, updated);
    return { ...updated };
  }

  public async search(query: string, filters?: Partial<SearchFilterState>): Promise<FileItem[]> {
    const q = (query || '').trim().toLowerCase();
    const results: FileItem[] = [];

    for (const item of this.files.values()) {
      if (item.parentPath === '/Trash' && q !== 'trash') continue;

      // Filter by query substring
      if (q && !item.name.toLowerCase().includes(q)) {
        continue;
      }

      // Filter by extension
      if (filters?.extensions && filters.extensions.length > 0) {
        if (!item.extension || !filters.extensions.includes(item.extension.toLowerCase())) {
          continue;
        }
      }

      // Filter by preview type
      if (filters?.types && filters.types.length > 0) {
        if (!item.previewKind || !filters.types.includes(item.previewKind)) {
          continue;
        }
      }

      // Filter by size range
      if (filters?.sizeRange && filters.sizeRange !== 'all') {
        const mb = item.size / (1024 * 1024);
        if (filters.sizeRange === 'small' && mb > 1) continue;
        if (filters.sizeRange === 'medium' && (mb <= 1 || mb > 50)) continue;
        if (filters.sizeRange === 'large' && mb <= 50) continue;
      }

      results.push({ ...item });
    }

    return results.sort((a, b) => b.modifiedAt.localeCompare(a.modifiedAt));
  }

  public async getStorageOverview(): Promise<{
    drives: DriveInfo[];
    categories: StorageCategoryBreakdown[];
  }> {
    let codeBytes = 0, mediaBytes = 0, docBytes = 0, archiveBytes = 0, otherBytes = 0;
    let codeCount = 0, mediaCount = 0, docCount = 0, archiveCount = 0, otherCount = 0;

    for (const item of this.files.values()) {
      if (item.type === 'folder') continue;
      const size = item.size;

      switch (item.previewKind) {
        case 'code':
          codeBytes += size;
          codeCount++;
          break;
        case 'image':
        case 'video':
        case 'audio':
          mediaBytes += size;
          mediaCount++;
          break;
        case 'pdf':
        case 'document':
        case 'markdown':
          docBytes += size;
          docCount++;
          break;
        case 'archive':
        case 'binary':
          archiveBytes += size;
          archiveCount++;
          break;
        default:
          otherBytes += size;
          otherCount++;
          break;
      }
    }

    const categories: StorageCategoryBreakdown[] = [
      { category: 'Media & Videos', bytes: mediaBytes, color: '#38BDF8', fileCount: mediaCount },
      { category: 'Archives & Binaries', bytes: archiveBytes, color: '#818CF8', fileCount: archiveCount },
      { category: 'Documents & PDFs', bytes: docBytes, color: '#34D399', fileCount: docCount },
      { category: 'Code & Repositories', bytes: codeBytes, color: '#F59E0B', fileCount: codeCount },
      { category: 'Other Assets', bytes: otherBytes, color: '#94A3B8', fileCount: otherCount },
    ];

    return {
      drives: [...this.drives],
      categories,
    };
  }

  public async getRecentFiles(limit = 12): Promise<FileItem[]> {
    const fileList: FileItem[] = [];
    for (const item of this.files.values()) {
      if (item.type === 'file' && !item.parentPath.startsWith('/Trash')) {
        fileList.push({ ...item });
      }
    }
    return fileList
      .sort((a, b) => b.modifiedAt.localeCompare(a.modifiedAt))
      .slice(0, limit);
  }

  public async getStarredItems(): Promise<FileItem[]> {
    const starred: FileItem[] = [];
    for (const item of this.files.values()) {
      if (item.isStarred && !item.parentPath.startsWith('/Trash')) {
        starred.push({ ...item });
      }
    }
    return starred.sort((a, b) => a.name.localeCompare(b.name));
  }

  private updateParentCount(parentPath: string, delta: number) {
    const parent = this.files.get(parentPath);
    if (parent && parent.type === 'folder') {
      const current = parent.itemCount || 0;
      parent.itemCount = Math.max(0, current + delta);
    }
  }
}

export const fileSystemService = new MockFileSystemService();
