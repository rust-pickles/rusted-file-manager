import { FileItem } from '../types/filesystem';
import { fileSystemService } from './FileSystemService';

/**
 * Opens an item the way a desktop file manager should: folders navigate, files
 * go to their associated application.
 *
 * Returns an error message instead of throwing, because every caller is a UI
 * event handler that has nowhere to rethrow to, and a silent failure is what
 * made double-click feel broken.
 */
export async function openItem(
  item: FileItem,
  navigate: (path: string) => void,
): Promise<string | null> {
  if (item.type === 'folder') {
    navigate(item.path);
    return null;
  }
  try {
    await fileSystemService.openPath(item.path);
    return null;
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}
