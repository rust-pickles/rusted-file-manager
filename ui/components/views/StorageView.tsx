import React, { useEffect, useState } from 'react';
import {
  HardDrive,
  Database,
} from 'lucide-react';
import { fileSystemService } from '../../services/FileSystemService';
import { DriveInfo, StorageCategoryBreakdown } from '../../types/filesystem';
import { formatBytes } from '../../utils/formatters';

export const StorageView: React.FC = () => {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [categories, setCategories] = useState<StorageCategoryBreakdown[]>([]);
  const [activeDriveId, setActiveDriveId] = useState<string>('drive-main');

  useEffect(() => {
    async function load() {
      const data = await fileSystemService.getStorageOverview();
      setDrives(data.drives);
      setCategories(data.categories);
    }
    load();
  }, []);

  const totalCapacity = drives.reduce((acc, d) => acc + d.totalBytes, 0);
  const totalUsed = drives.reduce((acc, d) => acc + d.usedBytes, 0);
  const totalFree = drives.reduce((acc, d) => acc + d.freeBytes, 0);
  const totalUsedPct = totalCapacity > 0 ? Math.round((totalUsed / totalCapacity) * 100) : 0;

  const activeDrive = drives.find((d) => d.id === activeDriveId) || drives[0];

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150">
      <div className="max-w-5xl mx-auto space-y-6">
        {/* Header */}
        <div className="pb-4 border-b border-[var(--border-subtle)] flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-[var(--text-primary)] tracking-tight">Storage Telemetry</h1>
            <p className="text-[var(--text-secondary)] mt-0.5">Drives, capacity metrics, and partition breakdown</p>
          </div>
          <div className="flex items-center gap-2 font-mono text-[11px] text-[var(--text-secondary)] bg-[var(--bg-surface)] px-3 py-1.5 rounded-[10px] border border-[var(--border-subtle)]">
            <span>Pool Usage:</span>
            <span className="text-[var(--text-primary)] font-semibold">{totalUsedPct}%</span>
            <span className="text-[var(--text-muted)]">·</span>
            <span>{formatBytes(totalFree)} Free</span>
          </div>
        </div>

        {/* Drives Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {drives.map((drive) => {
            const isSelected = drive.id === activeDrive?.id;
            const usedPct = Math.round((drive.usedBytes / drive.totalBytes) * 100);

            return (
              <div
                key={drive.id}
                onClick={() => setActiveDriveId(drive.id)}
                className={`p-4 rounded-[14px] border transition-all cursor-pointer shadow-2xs ${
                  isSelected
                    ? 'bg-[var(--bg-surface)] border-[var(--accent)] ring-1 ring-[var(--accent)]/30'
                    : 'bg-[var(--bg-surface)] border-[var(--border-subtle)] hover:border-[var(--border-strong)] hover:bg-[var(--bg-hover)]'
                }`}
              >
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2.5">
                    <div className="w-8 h-8 rounded-[10px] bg-[var(--bg-app)] flex items-center justify-center text-[var(--accent)]">
                      <HardDrive className="w-4 h-4" />
                    </div>
                    <div>
                      <p className="font-semibold text-[var(--text-primary)]">{drive.name}</p>
                      <p className="text-[10px] text-[var(--text-muted)] font-mono">{drive.fileSystem}</p>
                    </div>
                  </div>
                  <span className="font-mono text-xs font-semibold text-[var(--text-primary)]">{usedPct}%</span>
                </div>

                <div className="w-full h-2 bg-[var(--bg-hover)] rounded-full overflow-hidden mb-3">
                  <div
                    className={`h-full rounded-full transition-all ${
                      usedPct > 80 ? 'bg-amber-500' : 'bg-[var(--accent)]'
                    }`}
                    style={{ width: `${usedPct}%` }}
                  />
                </div>

                <div className="flex justify-between items-center text-[11px] font-mono text-[var(--text-muted)]">
                  <span>Used: {formatBytes(drive.usedBytes)}</span>
                  <span>Total: {formatBytes(drive.totalBytes)}</span>
                </div>
              </div>
            );
          })}
        </div>

        {/* Active Drive Detail & Category Breakdown */}
        {activeDrive && (
          <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] p-5 space-y-5 shadow-xs">
            <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)]">
              <div className="flex items-center gap-2">
                <Database className="w-4 h-4 text-[var(--accent)]" />
                <h3 className="font-semibold text-[var(--text-primary)] text-sm">
                  {activeDrive.name} Distribution
                </h3>
              </div>
              <span className="text-[var(--text-muted)] font-mono text-[11px]">
                Mount: {activeDrive.mountPoint}
              </span>
            </div>

            {/* Visual Multi-Segment Bar */}
            <div>
              <div className="w-full h-3 bg-[var(--bg-app)] rounded-full overflow-hidden flex mb-3 border border-[var(--border-subtle)]">
                {categories.map((cat, idx) => {
                  const pct = Math.max(2, Math.round((cat.bytes / activeDrive.usedBytes) * 100));
                  return (
                    <div
                      key={idx}
                      title={`${cat.category}: ${formatBytes(cat.bytes)} (${pct}%)`}
                      style={{ width: `${pct}%`, backgroundColor: cat.color }}
                      className="h-full hover:opacity-90 transition-opacity"
                    />
                  );
                })}
              </div>

              {/* Categories Grid */}
              <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
                {categories.map((cat, idx) => (
                  <div
                    key={idx}
                    className="p-3 rounded-[10px] bg-[var(--bg-app)]/50 border border-[var(--border-subtle)] flex items-center justify-between"
                  >
                    <div className="flex items-center gap-2.5">
                      <div
                        className="w-3 h-3 rounded-full shrink-0"
                        style={{ backgroundColor: cat.color }}
                      />
                      <div>
                        <p className="font-medium text-[var(--text-primary)]">{cat.category}</p>
                        <p className="text-[10px] text-[var(--text-muted)] font-mono">{cat.fileCount} files</p>
                      </div>
                    </div>
                    <span className="font-mono font-semibold text-[var(--text-secondary)]">
                      {formatBytes(cat.bytes)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
