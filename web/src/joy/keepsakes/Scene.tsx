// A kept date's illustrated keepsake: a die-cut object with one moment of its own (the coin tosses,
// the cat wakes, the card gets punched) that plays when it's touched and rewinds on a second touch.
// Its photo, when there is one, is an instant print tucked in behind it that can be pulled out.
import { useState, type CSSProperties } from "react";
import type { SceneDef } from "./scene-types";

export type { SceneDef };

const ratio = (s: SceneDef) => ({ "--ar": `${s.vb[2]} / ${s.vb[3]}`, "--arn": s.vb[2] / s.vb[3] }) as CSSProperties;

/** Just the drawing. `open` plays its moment (the Year shelf opens it while hovered). */
export function SceneArt({ scene, open = false }: { scene: SceneDef; open?: boolean }) {
  return (
    <span className={`ksc ksc-${scene.id}${open ? " open" : ""}`} style={ratio(scene)} aria-hidden="true">
      <svg viewBox={scene.vb.join(" ")} dangerouslySetInnerHTML={{ __html: scene.svg }} />
    </span>
  );
}

/** The keepsake you can touch. `what` names the date for a screen reader: "Cat café, 9 Nov". */
export function SceneButton({ scene, what, className = "" }: { scene: SceneDef; what: string; className?: string }) {
  const [open, setOpen] = useState(false);
  return (
    <button
      type="button"
      className={`ksc ksc-btn ksc-${scene.id}${open ? " open" : ""} ${className}`}
      style={ratio(scene)}
      aria-expanded={open}
      aria-label={`${scene.name}, from ${what}. ${open ? scene.close : scene.open}.`}
      onClick={() => setOpen((o) => !o)}
    >
      <svg viewBox={scene.vb.join(" ")} aria-hidden="true" dangerouslySetInnerHTML={{ __html: scene.svg }} />
    </button>
  );
}

/** An instant print tucked behind a keepsake: touch it to pull it out, again to tuck it back. */
export function TuckedPrint({ src, alt, className = "" }: { src: string; alt: string; className?: string }) {
  const [up, setUp] = useState(false);
  return (
    <button
      type="button"
      className={`ksc-print${up ? " up" : ""} ${className}`}
      aria-expanded={up}
      aria-label={`${alt}. ${up ? "Tuck it back" : "Pull it out"}.`}
      onClick={() => setUp((u) => !u)}
    >
      <img src={src} alt="" loading="lazy" decoding="async" width={240} height={300} />
    </button>
  );
}
