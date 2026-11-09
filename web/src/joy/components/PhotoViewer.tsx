import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent, type RefObject } from "react";
import { stepTo } from "./viewer";
import "./photo-viewer.css";

export interface ViewerPhoto {
  url: string;
  /** Read aloud and shown when the image can't load. */
  alt: string;
  /** Under the photo, e.g. "Picnic at the lake · 14 Mar". */
  caption?: string;
}

/** A swipe counts once the finger moves this far sideways. */
const SWIPE_PX = 40;

/**
 * Full-screen photos, one at a time: arrows, the arrow keys and a sideways swipe move through
 * them, Escape or the close button shuts it. Native <dialog>, so focus stays inside while open.
 * `start` null means closed.
 */
export function PhotoViewer({ photos, start, onClose }: { photos: ViewerPhoto[]; start: number | null; onClose: () => void }) {
  const dlg = useRef<HTMLDialogElement>(null);
  const open = start !== null && photos.length > 0;

  useEffect(() => {
    const d = dlg.current;
    if (!d) return;
    if (open && !d.open) {
      if (typeof d.showModal === "function") d.showModal();
      else d.setAttribute("open", "");
    } else if (!open && d.open) {
      d.close();
    }
  }, [open]);

  return (
    <dialog ref={dlg} className="pv" aria-label="Photos" onClose={onClose}>
      {open && <Body key={start} photos={photos} start={Math.min(start, photos.length - 1)} dlg={dlg} />}
    </dialog>
  );
}

function Body({ photos, start, dlg }: { photos: ViewerPhoto[]; start: number; dlg: RefObject<HTMLDialogElement | null> }) {
  const [i, setI] = useState(start);
  const closeBtn = useRef<HTMLButtonElement>(null);
  const touch = useRef<number | null>(null);
  const n = photos.length;

  useEffect(() => closeBtn.current?.focus({ preventScroll: true }), []);

  // Load the neighbours early, so stepping feels instant.
  useEffect(() => {
    if (n < 2) return;
    for (const j of [stepTo(i, n, 1), stepTo(i, n, -1)]) {
      const img = new Image();
      img.src = photos[j].url;
    }
  }, [i, n, photos]);

  const go = (by: 1 | -1) => setI((cur) => stepTo(cur, n, by));
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "ArrowRight") {
      e.preventDefault();
      go(1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      go(-1);
    }
  };
  const down = (e: PointerEvent) => {
    touch.current = e.clientX;
  };
  const up = (e: PointerEvent) => {
    const from = touch.current;
    touch.current = null;
    if (from === null || n < 2) return;
    const dx = e.clientX - from;
    if (Math.abs(dx) >= SWIPE_PX) go(dx < 0 ? 1 : -1);
  };

  const p = photos[i];
  return (
    <div className="pv-body" onKeyDown={onKey}>
      <div className="pv-top">
        <span className="pv-count" aria-live="polite">
          {n > 1 ? `${i + 1} of ${n}` : ""}
        </span>
        <button ref={closeBtn} type="button" className="pv-btn" onClick={() => dlg.current?.close()} aria-label="Close photos">
          ✕
        </button>
      </div>
      <figure className="pv-fig" onPointerDown={down} onPointerUp={up} onPointerCancel={() => (touch.current = null)}>
        <img src={p.url} alt={p.alt} className="pv-img" draggable={false} />
        {p.caption && <figcaption className="pv-cap">{p.caption}</figcaption>}
      </figure>
      {n > 1 && (
        <div className="pv-nav">
          <button type="button" className="pv-btn" onClick={() => go(-1)} aria-label="Previous photo">
            ‹
          </button>
          <button type="button" className="pv-btn" onClick={() => go(1)} aria-label="Next photo">
            ›
          </button>
        </div>
      )}
    </div>
  );
}
