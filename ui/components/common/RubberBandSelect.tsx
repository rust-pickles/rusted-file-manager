import React, { useCallback, useEffect, useRef, useState } from 'react';
import { useAppStore } from '../../store/useAppStore';

/**
 * Rectangular ("rubber band") selection for the file area.
 *
 * Drag on empty space to draw a box; every row or tile it touches is selected.
 * The box is measured against the rectangles of the elements carrying
 * `data-file-id`, so list, grid, compact and Miller views all support it without
 * knowing this exists.
 *
 * Why it previously did nothing: the trigger element was a zero-opacity overlay
 * rendered *underneath* an opaque view container, so it could never receive a
 * mousedown. The handler now lives on the scroll container itself and only
 * starts a drag when the press did not land on a row, a control or a menu. That
 * also means no extra element sits in the layout at all.
 *
 * Ctrl, Shift or Cmd held during the drag extends the selection; a plain click
 * on empty space clears it.
 */

interface Box {
  left: number;
  top: number;
  width: number;
  height: number;
}

const MIN_DRAG = 3; // px. Below this the gesture is a click, not a drag.

export const RubberBandSelect: React.FC = () => {
  const startRef = useRef<{ x: number; y: number } | null>(null);
  const additiveRef = useRef(false);
  const [box, setBox] = useState<Box | null>(null);

  const { clearSelection, setIsRubberBanding, setRubberBandIds } = useAppStore();

  const collect = (rect: Box): string[] => {
    const hits: string[] = [];
    document.querySelectorAll<HTMLElement>('[data-file-id]').forEach((node) => {
      const r = node.getBoundingClientRect();
      // Touching counts, so a thin drag across a row still selects it.
      const overlaps =
        r.left < rect.left + rect.width &&
        r.right > rect.left &&
        r.top < rect.top + rect.height &&
        r.bottom > rect.top;
      const id = node.dataset.fileId;
      if (overlaps && id) hits.push(id);
    });
    return hits;
  };

  const onMouseDown = useCallback(
    (event: React.MouseEvent) => {
      if (event.button !== 0) return;
      const target = event.target as HTMLElement;
      // A press on a row, a control or an overlay keeps its own behaviour and is
      // never converted into a selection drag.
      if (target.closest('[data-file-id]')) return;
      if (target.closest('button, input, a, textarea, select, [role="button"], [role="menuitem"], [data-no-rubber-band]')) return;

      startRef.current = { x: event.clientX, y: event.clientY };
      additiveRef.current = event.ctrlKey || event.metaKey || event.shiftKey;
      setIsRubberBanding(true);
    },
    [setIsRubberBanding],
  );

  useEffect(() => {
    const onMove = (event: MouseEvent) => {
      const start = startRef.current;
      if (!start) return;
      const dx = event.clientX - start.x;
      const dy = event.clientY - start.y;
      if (Math.abs(dx) < MIN_DRAG && Math.abs(dy) < MIN_DRAG) return;
      setBox({
        left: Math.min(start.x, event.clientX),
        top: Math.min(start.y, event.clientY),
        width: Math.abs(dx),
        height: Math.abs(dy),
      });
    };

    const onUp = (event: MouseEvent) => {
      const start = startRef.current;
      startRef.current = null;
      setIsRubberBanding(false);

      const moved = start
        ? Math.abs(event.clientX - start.x) >= MIN_DRAG || Math.abs(event.clientY - start.y) >= MIN_DRAG
        : false;

      if (!moved) {
        setBox(null);
        // A click on empty space clears the selection, like Explorer.
        if (start && !additiveRef.current) clearSelection();
        return;
      }

      const rect: Box = {
        left: Math.min(start!.x, event.clientX),
        top: Math.min(start!.y, event.clientY),
        width: Math.abs(event.clientX - start!.x),
        height: Math.abs(event.clientY - start!.y),
      };
      setBox(null);
      setRubberBandIds(collect(rect), additiveRef.current);
    };

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    return () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
  }, [clearSelection, setIsRubberBanding, setRubberBandIds]);

  return (
    <div
      onMouseDown={onMouseDown}
      className="contents"
      /* `contents` keeps this wrapper out of layout: the handler is attached for
         event delegation only, so nothing can cover the views. */
      aria-hidden="true"
    >
      {box && (
        <div
          className="fixed pointer-events-none z-[60] rounded-[2px] border border-[var(--accent)] bg-[var(--accent)]/15"
          style={{ left: box.left, top: box.top, width: box.width, height: box.height }}
        />
      )}
    </div>
  );
};
