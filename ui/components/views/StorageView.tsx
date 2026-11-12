import React, { useEffect, useState } from 'react';
import { HardDrive, Database, Folder, AlertTriangle, CheckCircle2, RefreshCw } from 'lucide-react';
import { fileSystemService, DriveHealthEntry, FolderUsage } from '../../services/FileSystemService';
import { DriveInfo } from '../../types/filesystem';
import { formatBytes } from '../../utils/formatters';

/**
 * Storage telemetry: what each drive holds and what is taking the space.
 *
 * Two-stage load on purpose. Capacities come from `list_drives`, which reads
 * volume metadata that Windows already has cached, so the cards paint the moment
 * the view opens. The per-folder breakdown needs to walk directories, so it runs
 * only for the selected drive and only when asked.
 *
 * The previous version invented its numbers: it asked for an "overview" that
 * scanned the home folder and then displayed that total as if it belonged to
 * whichever drive was selected. The breakdown below is measured per drive.
 */
export const StorageView: React.FC = () => {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [health, setHealth] = useState<DriveHealthEntry[]>([]);
  const [activeDriveId, setActiveDriveId] = useState<string>('');
  const [usage, setUsage] = useState<FolderUsage[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [scannedRoot, setScannedRoot] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  // Stage one: capacities, immediately.
  useEffect(() => {
    let cancelled = false;
    fileSystemService
      .listDrives()
      .then((found) => {
        if (cancelled) return;
        setDrives(found);
        if (found.length > 0) setActiveDriveId((current) => current || found[0].id);
      })
      .catch((problem: unknown) =>
        setError(problem instanceof Error ? problem.message : String(problem)),
      );

    // Health is a write probe per volume, so it is allowed to arrive later.
    fileSystemService
      .getDriveHealth()
      .then((found) => {
        if (!cancelled) setHealth(found);
      })
      .catch(() => {
        if (!cancelled) setHealth([]);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const activeDrive = drives.find((drive) => drive.id === activeDriveId) ?? drives[0];
  const activeHealth = health.find((entry) => entry.root === activeDrive?.mountPoint);

  /** Walks the selected drive one level deep; only runs on request. */
  const scan = async (root: string) => {
    setIsScanning(true);
    setError(null);
    try {
      const rows = await fileSystemService.getFolderUsage(root);
      setUsage(rows);
      setScannedRoot(root);
    } catch (problem) {
      setUsage([]);
      setError(problem instanceof Error ? problem.message : String(problem));
    } finally {
      setIsScanning(false);
    }
  };

  const totalCapacity = drives.reduce((sum, drive) => sum + drive.totalBytes, 0);
  const totalUsed = drives.reduce((sum, drive) => sum + drive.usedBytes, 0);
  const totalFree = drives.reduce((sum, drive) => sum + drive.freeBytes, 0);
  const poolPct = totalCapacity > 0 ? Math.round((totalUsed / totalCapacity) * 100) : 0;

  const usageTotal = usage.reduce((sum, row) => sum + row.bytes, 0);
  const problems = health.filter((entry) => !entry.writable);

  return (
    <div className="flex-1 overflow-y-auto p-6 bg-[var(--bg-main)] text-xs select-none transition-colors duration-150">
      <div className="max-w-5xl mx-auto space-y-6">
        <div className="pb-4 border-b border-[var(--border-subtle)] flex items-center justify-between gap-4">
          <div>
            <h1 className="text-xl font-bold text-[var(--text-primary)] tracking-tight">Storage</h1>
            <p className="text-[var(--text-secondary)] mt-0.5">
              Drive capacity and what is taking the space
            </p>
          </div>
          <div className="flex items-center gap-2 font-mono text-[11px] text-[var(--text-secondary)] bg-[var(--bg-surface)] px-3 py-1.5 rounded-[10px] border border-[var(--border-subtle)]">
            <span>All drives:</span>
            <span className="text-[var(--text-primary)] font-semibold">{poolPct}%</span>
            <span className="text-[var(--text-muted)]">·</span>
            <span>{formatBytes(totalFree)} free</span>
          </div>
        </div>

        {problems.length > 0 && (
          <div className="flex items-start gap-2.5 p-3 rounded-[12px] border border-amber-500/40 bg-amber-500/10 text-amber-200">
            <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5" />
            <div className="space-y-1">
              {problems.map((entry) => (
                <p key={entry.root}>
                  <span className="font-semibold">{entry.name}</span> cannot be written to:{' '}
                  {entry.problem ?? 'unknown reason'}
                </p>
              ))}
            </div>
          </div>
        )}

        {error && (
          <div className="p-3 rounded-[12px] border border-rose-500/40 bg-rose-500/10 text-rose-200">
            {error}
          </div>
        )}

        {drives.length === 0 && !error && (
          <p className="text-[var(--text-secondary)]">
            No drives were reported. This view needs the desktop app.
          </p>
        )}

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {drives.map((drive) => {
            const usedPct =
              drive.totalBytes > 0 ? Math.round((drive.usedBytes / drive.totalBytes) * 100) : 0;
            const isSelected = drive.id === activeDrive?.id;
            const driveHealth = health.find((entry) => entry.root === drive.mountPoint);

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
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div className="w-8 h-8 rounded-[10px] bg-[var(--bg-app)] flex items-center justify-center text-[var(--accent)] shrink-0">
                      <HardDrive className="w-4 h-4" />
                    </div>
                    <div className="min-w-0">
                      <p className="font-semibold text-[var(--text-primary)] truncate">{drive.name}</p>
                      <p className="text-[10px] text-[var(--text-muted)] font-mono">
                        {drive.mountPoint} · {drive.fileSystem || 'unknown'}
                      </p>
                    </div>
                  </div>
                  <span className="font-mono text-xs font-semibold text-[var(--text-primary)] shrink-0">
                    {usedPct}%
                  </span>
                </div>

                <div className="w-full h-2 bg-[var(--bg-hover)] rounded-full overflow-hidden mb-3">
                  <div
                    className={`h-full rounded-full transition-all ${
                      usedPct > 85 ? 'bg-amber-500' : 'bg-[var(--accent)]'
                    }`}
                    style={{ width: `${usedPct}%` }}
                  />
                </div>

                <div className="flex justify-between items-center text-[11px] font-mono text-[var(--text-muted)]">
                  <span>Used {formatBytes(drive.usedBytes)}</span>
                  <span>Free {formatBytes(drive.freeBytes)}</span>
                </div>

                {driveHealth && (
                  <div
                    className={`mt-2 flex items-center gap-1.5 text-[10px] ${
                      driveHealth.writable ? 'text-emerald-400' : 'text-amber-400'
                    }`}
                  >
                    {driveHealth.writable ? (
                      <>
                        <CheckCircle2 className="w-3 h-3" /> writable
                      </>
                    ) : (
                      <>
                        <AlertTriangle className="w-3 h-3" /> read-only or failing
                      </>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {activeDrive && (
          <div className="bg-[var(--bg-surface)] border border-[var(--border-subtle)] rounded-[14px] p-5 space-y-4 shadow-xs">
            <div className="flex items-center justify-between pb-3 border-b border-[var(--border-subtle)] gap-3">
              <div className="flex items-center gap-2 min-w-0">
                <Database className="w-4 h-4 text-[var(--accent)] shrink-0" />
                <h3 className="font-semibold text-[var(--text-primary)] text-sm truncate">
                  What is using {activeDrive.name}
                </h3>
              </div>
              <button
                onClick={() => scan(activeDrive.mountPoint)}
                disabled={isScanning}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-[8px] bg-[var(--bg-app)] border border-[var(--border-subtle)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--border-strong)] disabled:opacity-50 transition-colors shrink-0"
              >
                <RefreshCw className={`w-3 h-3 ${isScanning ? 'animate-spin' : ''}`} />
                {isScanning ? 'Measuring…' : scannedRoot === activeDrive.mountPoint ? 'Measure again' : 'Measure'}
              </button>
            </div>

            <p className="text-[11px] text-[var(--text-muted)]">
              {isScanning
                ? `Walking ${activeDrive.mountPoint} — this reads the disk, so it can take a while on a large drive.`
                : scannedRoot === activeDrive.mountPoint
                  ? `Measured ${usage.length} top-level entries, ${formatBytes(usageTotal)} accounted for.`
                  : 'Measuring walks the drive and reads file sizes. Nothing is written, and it runs only when you ask.'}
            </p>

            {scannedRoot === activeDrive.mountPoint && usage.length > 0 && (
              <div className="space-y-1.5">
                {usage.map((row) => {
                  const share = usageTotal > 0 ? (row.bytes / usageTotal) * 100 : 0;
                  return (
                    <div
                      key={row.path}
                      className="flex items-center gap-3 px-2 py-1.5 rounded-[8px] hover:bg-[var(--bg-hover)]"
                      title={row.path}
                    >
                      <Folder className="w-3.5 h-3.5 text-[var(--accent)] shrink-0" />
                      <span className="w-40 truncate text-[var(--text-primary)]">{row.name}</span>
                      <div className="flex-1 h-1.5 bg-[var(--bg-hover)] rounded-full overflow-hidden">
                        <div
                          className="h-full bg-[var(--accent)] rounded-full"
                          style={{ width: `${Math.max(1, share)}%` }}
                        />
                      </div>
                      <span className="w-20 text-right font-mono text-[var(--text-secondary)]">
                        {formatBytes(row.bytes)}
                      </span>
                      <span className="w-24 text-right font-mono text-[var(--text-muted)]">
                        {row.truncated ? `${row.fileCount}+ files` : `${row.fileCount} files`}
                      </span>
                    </div>
                  );
                })}
              </div>
            )}

            {scannedRoot === activeDrive.mountPoint && usage.length === 0 && !isScanning && (
              <p className="text-[var(--text-secondary)]">
                Nothing to report. The drive root may be unreadable, or it is empty.
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
