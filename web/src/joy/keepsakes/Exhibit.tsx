// One kept date drawn as its own keepsake from the catalogue, the way it would sit in a museum of
// us: real text on a physical object, with a small life of its own (the chopsticks clink, the
// perfume sprays) when she touches it, and once when it first comes into view.
// Everything inside is a <span>, so the exhibit can live in a button (Month) or a link (Year).
import { useEffect } from "react";
import type { Spec } from "./spec";
import { Motif } from "./motifs";
import { exhibitName, usePlay, type ExhibitDate } from "./play";

export type { ExhibitDate };

const stars = (n: number) => "★".repeat(Math.max(0, Math.min(5, n)));

/** The object itself: shape, drawing, words. Wrap it in `ExhibitButton` or a link. */
export function ExhibitBody({ spec, d }: { spec: Spec; d: ExhibitDate }) {
  const place = d.place && d.place.toLowerCase() !== d.title.toLowerCase() && !spec.head.toLowerCase().includes(d.place.toLowerCase()) ? d.place : "";
  return (
    <>
      <span className="ex-d1" aria-hidden="true"></span>
      <span className="ex-d2" aria-hidden="true"></span>
      <span className="ex-body">
        <span className="ex-d3" aria-hidden="true"></span>
        <span className="ex-d4" aria-hidden="true"></span>
        <span className="ex-m" aria-hidden="true">
          <Motif name={spec.motif} />
          <span className="ex-fx">
            <i></i><i></i><i></i><i></i><i></i><i></i><i></i>
          </span>
        </span>
        {spec.head && <span className="ex-head">{spec.head}</span>}
        <span className="ex-title">{d.title}</span>
        {place && <span className="ex-place">{place}</span>}
        {spec.lines.map((l, i) => <span className="ex-line" key={i}>{l}</span>)}
        <span className="ex-day">{d.day}</span>
        {d.rating > 0 && <span className="ex-stars" aria-hidden="true">{stars(d.rating)}</span>}
      </span>
    </>
  );
}

const styleOf = (spec: Spec) => ({ "--p": spec.paper, "--i": spec.ink, "--a": spec.accent }) as React.CSSProperties;

/** An exhibit she can touch to bring to life (the Month page). */
export function ExhibitButton({ spec, d }: { spec: Spec; d: ExhibitDate }) {
  const { ref, play } = usePlay<HTMLButtonElement>();
  return (
    <button
      ref={ref}
      type="button"
      className={`ex ex-${spec.form}`}
      data-anim={spec.anim}
      style={styleOf(spec)}
      aria-label={exhibitName(spec, d)}
      onClick={play}
      onPointerEnter={(e) => e.pointerType === "mouse" && play()}
    >
      <ExhibitBody spec={spec} d={d} />
    </button>
  );
}

/** An exhibit inside something else that is interactive (a link on the Year page's shelf). */
export function ExhibitFigure({ spec, d }: { spec: Spec; d: ExhibitDate }) {
  const { ref, play } = usePlay<HTMLSpanElement>();
  // Hovering or focusing the link it sits in plays it too.
  useEffect(() => {
    const host = ref.current?.closest("a, button");
    if (!host) return;
    const onEnter = (e: Event) => (e as PointerEvent).pointerType === "mouse" && play();
    host.addEventListener("pointerenter", onEnter);
    host.addEventListener("focus", play);
    return () => {
      host.removeEventListener("pointerenter", onEnter);
      host.removeEventListener("focus", play);
    };
  }, [ref, play]);
  return (
    <span ref={ref} className={`ex ex-${spec.form}`} data-anim={spec.anim} style={styleOf(spec)}>
      <ExhibitBody spec={spec} d={d} />
    </span>
  );
}
