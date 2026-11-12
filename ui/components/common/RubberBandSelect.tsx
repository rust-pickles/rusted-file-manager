import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useAppStore } from '../../store/useAppStore';

/**
 * Rectangular ("rubber band") selection for the file area.
 *
 * Drag on empty space to draw a box; every row or tile it touches is selected.
 * The box is measured against the real rectangles of elements carrying
 * `data-file-id`, so this works for the list, grid, compact and Miller views
 * without any of them knowing it exists.
 *
 * Ctrl, Shift or Cmd held at the end of a drag adds to the existing selection
 * instead of replacing it, matching Explorer. A plain click on empty space
 * clears the selection.
 */

interface Point {
  x: number;
  y: number;
}

const MIN_DRAG = 4; // px: below this a drag is treated as a click on empty space

export const RubberBandSelect: React.FC = () => {
  const originRef = useRef<Point | null>(null);
  const additiveRef = useRef(false);
  const [origin, setOrigin] = useState<Point | null>(null);
  const [current, setCurrent] = useState<Point | null>(null);

  const { clearSelection, setIsRubberBanding, setRubberBandIds } = useAppStore();

  const finish = useCallback(() => {
    const started = originRef.current;
    const ended = current;
    originRef.current = null;
    setOrigin(null);
    setCurrent(null);
    setIsRubberBanding(false);

    if (!started || !ended) return;
    const moved =
      Math.abs(ended.x - started.x) > MIN_DRAG || Math.abs(ended.y - started.y) > MIN_DRAG;
    if (!moved) {
      if (!additiveRef.current) clearSelection();
      return;
    }

    const left = Math.min(started.x, ended.x);
    const right = Math.max(started.x, ended.x);
    const top = Math.min(started.y, ended.y);
    const bottom = Math.max(started.y, ended.y);

    const hits: string[] = [];
    document.querySelectorAll<HTMLElement>('[data-file-id]').forEach((node) => {
      const rect = node.getBoundingClientRect();
      // Touching counts, so a thin drag across a row still selects it.
      const overlaps =
        rect.left < right && rect.right > left && rect.top < bottom && rect.bottom > top;
      const id = node.dataset.fileId;
      if (overlaps && id) hits.push(id);
    });

    setRubberBandIds(hits, additiveRef.current);
  }, [current, clearSelection, setIsRubberBanding, setRubberBandIds]);

  useEffect(() => {
    if (!origin) return;

    // Registered once per drag, not per mousemove: the position lives in state
    // for rendering, so re-subscribing on every pixel would be wasteful.
    const onMove = (event: MouseEvent) => setCurrent({ x: event.clientX, y: event.clientY });
    const onUp = () => finish();

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    return () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
  }, [origin, finish]);

  const onMouseDown = (event: React.MouseEvent) => {
    // Left button only, and only on empty space: a press on a row must keep
    // doing normal row selection.
    if (event.button !== 0) return;
    const target = event.target as HTMLElement;
    if (target.closest('[data-file-id]')) return;
    if (target.closest('button, input, a, [role="button"], [role="menuitem"]')) return;

    additiveRef.current = event.ctrlKey || event.metaKey || event.shiftKey;
    const point = { x: event.clientX, y: event.clientY };
    originRef.current = point;
    setOrigin(point);
    setCurrent(point);
    setIsRubberBanding(true);
  };

  const box =
    origin && current
      ? {
          left: Math.min(origin.x, current.x),
          top: Math.min(origin.y, current.y),
          width: Math.abs(current.x - origin.x),
          height: Math.abs(current.y - origin.y),
        }
      : null;

  return (
    <>
      {/* Transparent hit area covering the file region. It sits below the rows,
          so presses on a row still reach the row. */}
      <div onMouseDown={onMouseDown} className="absolute inset-0 z-0" aria-hidden="true" />
      {box && (
        <div
          className="fixed pointer-events-none z-50 rounded-[2px] border border-[var(--accent)] bg-[var(--accent)]/15"
          style={{ left: box.left, top: box.top, width: box.width, height: box.height }}
        />
      )}
    </>
  );
};
