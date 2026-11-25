// A kept date's illustrated keepsake: a die-cut object with one moment of its own (the coin tosses,
// the cat wakes, the card gets punched) that plays when it's touched and rewinds on a second touch.
// Its photo, when there is one, is an instant print tucked in behind it that can be pulled out.
// The drawings are files of their own (/keepsakes/<id>-<hash>.svg), fetched once a keepsake is
// near the screen and then kept by the browser for good.
import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type RefObject,
} from "react";
import type { SceneDef } from "./scene-types";

export type { SceneDef };

const ratio = (s: SceneDef) =>
  ({
    "--ar": `${s.vb[2]} / ${s.vb[3]}`,
    "--arn": s.vb[2] / s.vb[3],
  }) as CSSProperties;

const files = new Map<string, Promise<string | null>>();
/** The drawing's markup (our own static file), fetched once per page load. */
function loadScene(s: SceneDef): Promise<string | null> {
  let p = files.get(s.file);
  if (!p) {
    p =
      typeof fetch === "function"
        ? fetch(`/keepsakes/${s.file}`)
            .then((r) => (r.ok ? r.text() : null))
            .catch(() => null)
        : Promise.resolve(null);
    files.set(s.file, p);
  }
  return p;
}

/** True once the element has come within a screen of view (and stays true). */
function useNear(ref: RefObject<HTMLElement | null>, eager: boolean) {
  const [near, setNear] = useState(eager);
  useEffect(() => {
    if (near) return;
    const el = ref.current;
    if (!el || typeof IntersectionObserver !== "function") return setNear(true);
    const io = new IntersectionObserver(
      (seen) => seen.some((s) => s.isIntersecting) && setNear(true),
      { rootMargin: "400px" },
    );
    io.observe(el);
    return () => io.disconnect();
  }, [near, ref]);
  return near;
}

function useSvg(scene: SceneDef, near: boolean) {
  const [svg, setSvg] = useState<{ file: string; text: string } | null>(null);
  useEffect(() => {
    if (!near) return;
    let live = true;
    void loadScene(scene).then(
      (t) => live && t && setSvg({ file: scene.file, text: t }),
    );
    return () => {
      live = false;
    };
  }, [scene, near]);
  return svg?.file === scene.file ? svg.text : null;
}

function Drawing({ text }: { text: string | null }) {
  return text ? (
    <span className="ksc-svg" dangerouslySetInnerHTML={{ __html: text }} />
  ) : (
    <span className="ksc-svg ksc-wait" />
  );
}

/** Just the drawing. `open` plays its moment (the Year shelf opens it while hovered); `ghost` shows
 *  a slip's keepsake before it's ours: the shape of it, in grey. */
export function SceneArt({
  scene,
  open = false,
  ghost = false,
  eager = false,
  className = "",
}: {
  scene: SceneDef;
  open?: boolean;
  ghost?: boolean;
  eager?: boolean;
  className?: string;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const text = useSvg(scene, useNear(ref, eager));
  return (
    <span
      ref={ref}
      className={`ksc ksc-${scene.id}${open ? " open" : ""}${ghost ? " is-ghost" : ""} ${className}`}
      style={ratio(scene)}
      aria-hidden="true"
    >
      <Drawing text={text} />
    </span>
  );
}

/** The keepsake you can touch. `what` names the date for a screen reader: "Cat café, 9 Nov".
 *  `arrive`: the first time it's seen it plays its moment by itself, marked new, then calls `onArrived`. */
export function SceneButton({
  scene,
  what,
  className = "",
  arrive = false,
  onArrived,
}: {
  scene: SceneDef;
  what: string;
  className?: string;
  arrive?: boolean;
  onArrived?: () => void;
}) {
  const ref = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const [fresh, setFresh] = useState(arrive);
  const text = useSvg(scene, useNear(ref, false));
  const arrived = useRef(onArrived);
  useEffect(() => {
    arrived.current = onArrived;
  }, [onArrived]);
  useEffect(() => {
    if (!fresh || !text) return;
    const el = ref.current;
    if (!el || typeof IntersectionObserver !== "function") return;
    let t: ReturnType<typeof setTimeout> | undefined;
    const io = new IntersectionObserver(
      (seen) => {
        if (!seen.some((s) => s.isIntersecting && s.intersectionRatio > 0.5))
          return;
        io.disconnect();
        t = setTimeout(() => {
          setOpen(true);
          arrived.current?.();
        }, 700);
      },
      { threshold: [0.5, 0.8] },
    );
    io.observe(el);
    return () => {
      io.disconnect();
      clearTimeout(t);
    };
  }, [fresh, text]);
  return (
    <button
      ref={ref}
      type="button"
      className={`ksc ksc-btn ksc-${scene.id}${open ? " open" : ""}${fresh ? " is-new" : ""} ${className}`}
      style={ratio(scene)}
      aria-expanded={open}
      aria-label={`${scene.name}, from ${what}. ${open ? scene.close : scene.open}.`}
      onClick={() => {
        setFresh(false);
        setOpen((o) => !o);
      }}
    >
      <Drawing text={text} />
      {fresh && (
        <span className="ksc-new" aria-hidden="true">
          new in the museum
        </span>
      )}
    </button>
  );
}

/** An instant print tucked behind a keepsake: touch it to pull it out, again to tuck it back. */
export function TuckedPrint({
  src,
  alt,
  className = "",
}: {
  src: string;
  alt: string;
  className?: string;
}) {
  const [up, setUp] = useState(false);
  return (
    <button
      type="button"
      className={`ksc-print${up ? " up" : ""} ${className}`}
      aria-expanded={up}
      aria-label={`${alt}. ${up ? "Tuck it back" : "Pull it out"}.`}
      onClick={() => setUp((u) => !u)}
    >
      <img
        src={src}
        alt=""
        loading="lazy"
        decoding="async"
        width={240}
        height={300}
      />
    </button>
  );
}
