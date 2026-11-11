import React from 'react';
import {
  Folder,
  FileCode,
  FileText,
  FileImage,
  FileVideo,
  FileAudio,
  FileArchive,
  FileSpreadsheet,
  FileBox,
  File as FileDefault,
  FileCheck,
} from 'lucide-react';
import { FileItem } from '../types/filesystem';

export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function formatDate(isoString: string): string {
  try {
    const d = new Date(isoString);
    return d.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return isoString;
  }
}

export function getFileIcon(item: FileItem, className = 'w-4 h-4'): React.ReactElement {
  if (item.type === 'folder') {
    return <Folder className={`${className} text-[var(--accent)] fill-[var(--accent)]/15`} />;
  }

  const ext = (item.extension || '').toLowerCase();

  switch (item.previewKind) {
    case 'code':
      if (['rs', 'c', 'cpp'].includes(ext)) {
        return <FileCode className={`${className} text-amber-400`} />;
      }
      if (['ts', 'tsx', 'js', 'jsx'].includes(ext)) {
        return <FileCode className={`${className} text-blue-400`} />;
      }
      return <FileCode className={`${className} text-emerald-400`} />;
    case 'image':
      return <FileImage className={`${className} text-purple-400`} />;
    case 'video':
      return <FileVideo className={`${className} text-rose-400`} />;
    case 'audio':
      return <FileAudio className={`${className} text-teal-400`} />;
    case 'archive':
      return <FileArchive className={`${className} text-amber-500`} />;
    case 'pdf':
      return <FileText className={`${className} text-red-400`} />;
    case 'markdown':
      return <FileText className={`${className} text-sky-300`} />;
    case 'document':
      if (['xlsx', 'csv', 'ods'].includes(ext)) {
        return <FileSpreadsheet className={`${className} text-emerald-500`} />;
      }
      return <FileText className={`${className} text-slate-300`} />;
    case 'binary':
      return <FileBox className={`${className} text-indigo-400`} />;
    default:
      return <FileDefault className={`${className} text-slate-400`} />;
  }
}

export function getFileTypeLabel(item: FileItem): string {
  if (item.type === 'folder') return 'Folder';
  const ext = (item.extension || '').toUpperCase();
  if (ext) return `${ext} File`;
  return 'File';
}
