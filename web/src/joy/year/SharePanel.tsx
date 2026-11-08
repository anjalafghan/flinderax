// "Make our picture": two taps. Tap 1 does the slow part (fonts, photos, canvas, encode) and opens a preview;
// tap 2 calls navigator.share({files}) as the first thing it does, so the tap's activation is still fresh.
import { useEffect, useRef, useState } from "react";
import { drawData, MAX_PHOTOS, shareControls, type ShareData } from "./layout";
import { makePicture } from "./paint";

const canDraw = () => typeof HTMLCanvasElement !== "undefined" && "toBlob" in HTMLCanvasElement.prototype;

interface Props {
  data: ShareData;
  /** Photo ids ticked on the wall, in tick order. */
  ticked: number[];
  hasWall: boolean;
  /** Reports a status line (the page reads it to say the 9-photo cap). */
  say: string;
}

export function SharePanel({ data, ticked, hasWall, say }: Props) {
  const ctl = shareControls(data);
  const [places, setPlaces] = useState(false);
  const [spent, setSpent] = useState(false);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState("");
  const [fileName, setFileName] = useState("");
  const [canShare, setCanShare] = useState(false);
  const [showSave, setShowSave] = useState(false);
  const file = useRef<File | null>(null);
  const url = useRef("");
  const dialog = useRef<HTMLDialogElement>(null);
  const makeBtn = useRef<HTMLButtonElement>(null);
  const sendBtn = useRef<HTMLButtonElement>(null);
  const saveLink = useRef<HTMLAnchorElement>(null);
  const lock = useRef(false);

  useEffect(() => () => { if (url.current) URL.revokeObjectURL(url.current); }, []);

  if (!canDraw()) return null; // no picture without toBlob; the page stays readable

  const make = async () => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setStatus("Making our picture…");
    try {
      const f = await makePicture(drawData(data, { places, spent, photos: ticked }));
      if (url.current) URL.revokeObjectURL(url.current);
      url.current = URL.createObjectURL(f);
      file.current = f;
      let can = false;
      try {
        can = typeof navigator.share === "function" && typeof navigator.canShare === "function" && navigator.canShare({ files: [f] });
      } catch {
        can = false;
      }
      setPreview(url.current);
      setFileName(f.name);
      setCanShare(can);
      setShowSave(!can);
      setStatus("");
      const d = dialog.current;
      if (d) {
        if (typeof d.showModal === "function") d.showModal();
        else d.setAttribute("open", "");
      }
      requestAnimationFrame(() => (can ? sendBtn.current : saveLink.current)?.focus());
    } catch {
      setStatus("The picture didn't come out this time. Try once more?");
    } finally {
      lock.current = false;
      setBusy(false);
    }
  };

  const fallback = () => {
    setCanShare(false);
    setShowSave(true);
    requestAnimationFrame(() => saveLink.current?.focus());
  };

  // synchronously, first thing: no await before navigator.share
  const send = () => {
    const f = file.current;
    if (!f) return;
    let p: Promise<void>;
    try {
      p = navigator.share({ files: [f] });
    } catch {
      fallback();
      return;
    }
    p.then(
      () => close(),
      (err: unknown) => {
        if (!(err && (err as { name?: string }).name === "AbortError")) fallback();
      },
    );
  };

  const close = () => {
    const d = dialog.current;
    if (d?.close) d.close();
    else {
      d?.removeAttribute("open");
      cleanup();
    }
  };
  const cleanup = () => {
    if (url.current) URL.revokeObjectURL(url.current);
    url.current = "";
    file.current = null;
    setPreview("");
    makeBtn.current?.focus();
  };

  return (
    <>
      <section className="y-share" id="y-share" aria-labelledby="y-share-h">
        <div className="y-postcard">
          <div className="y-postcard-msg">
            <p className="y-postcard-k">Post card · a keepsake</p>
            <h2 id="y-share-h">Make our picture</h2>
            <p className="y-share-lead">
              A keepsake of our {data.year} to share or keep. It says “us” and never our names, notes or homes. Places, money and photos only go in if you tick them.
            </p>
          </div>
          <div className="y-postcard-addr">
            <fieldset className="y-opts">
              <legend className="visually-hidden">What goes in the picture</legend>
              {ctl.places && (
                <label className="y-opt">
                  <input type="checkbox" id="y-opt-places" checked={places} onChange={(e) => setPlaces(e.target.checked)} /> Show where we went
                </label>
              )}
              {ctl.spent && (
                <label className="y-opt">
                  <input type="checkbox" id="y-opt-spent" checked={spent} onChange={(e) => setSpent(e.target.checked)} /> Show what we spent
                </label>
              )}
              {hasWall && <p className="y-opt-note">Photos: tick “include in picture” on the wall above (up to {MAX_PHOTOS}).</p>}
            </fieldset>
            <div className="y-make-row">
              <button ref={makeBtn} type="button" className="y-make" id="y-make" disabled={busy} onClick={() => void make()}>
                Make our picture
              </button>
              <p className="y-status" id="y-status" role="status" aria-live="polite">{status || say}</p>
            </div>
          </div>
        </div>
      </section>

      <dialog
        ref={dialog}
        className="y-sheet"
        id="y-sheet"
        aria-labelledby="y-sheet-h"
        onClose={cleanup}
        onClick={(e) => {
          if (e.target === e.currentTarget) close(); // tap the backdrop
        }}
      >
        <div className="y-sheet-in">
          <h2 id="y-sheet-h">Our {data.year}, ready</h2>
          <div className="y-preview-wrap">
            {preview && <img className="y-preview" id="y-preview" src={preview} alt={`Preview of the picture of our ${data.year}`} />}
          </div>
          <div className="y-sheet-acts">
            <button ref={sendBtn} type="button" className="btn btn-stamp" id="y-send" hidden={!canShare} onClick={send}>
              Share
            </button>
            <p className="y-save" id="y-save" hidden={!showSave}>
              <a ref={saveLink} className="btn" id="y-save-link" download={fileName || `our-${data.year}.png`} href={preview || "#"}>
                Save image
              </a>{" "}
              <span className="soft">or long-press the picture to save it</span>
            </p>
            <button type="button" className="btn btn-quiet" id="y-close" onClick={close}>
              Close
            </button>
          </div>
        </div>
      </dialog>
    </>
  );
}
