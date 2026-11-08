/* eslint-disable react-hooks/refs, react-hooks/immutability, react-hooks/purity --
   the canvas simulation lives in a mutable ref and is stepped by requestAnimationFrame, outside React's render. */
// The jar: Joy's SVG glass with a canvas of slips inside it. The simulation itself is physics.ts
// (pure); this component owns the canvas, the animation loop and the colours. The loop only runs
// while something moves and the tab is visible.
import { useEffect, useImperativeHandle, useRef, type Ref, type RefObject } from "react";
import {
  assign,
  dropIn as dropPart,
  jostle as jostleParts,
  makeParts,
  makeSlots,
  MOUTH,
  mix,
  parseColor,
  rgba,
  sample,
  step,
  SW,
  SH,
  takeOut as takePart,
  W,
  type Part,
  type RGBA,
  type Slot,
  type SlipInfo,
} from "./physics";
import { reduced } from "./motion";

export interface JarHandle {
  jostle: (strength: number) => void;
  /** Lift a slip out of the pile; says where it was (jar units) so it can fly from there. */
  takeOut: (id: number, setting: string) => { x: number; y: number; a: number; setting: string } | null;
  dropIn: (id: number) => void;
  /** The jar's mouth and scale, in viewport pixels. */
  mouth: () => { x: number; y: number; s: number } | null;
  /** A position inside the jar (jar units) in viewport pixels. */
  at: (x: number, y: number) => { x: number; y: number; s: number } | null;
  /** The glass shakes (a draw is coming). */
  shake: () => void;
  openLid: (open: boolean) => void;
}

interface Colors {
  dark: boolean;
  paper: RGBA;
  paper2: RGBA;
  slip: RGBA;
  ink: RGBA;
  tint: Record<string, RGBA>;
}

interface Props {
  slips: SlipInfo[];
  /** Ids that fit the sentence right now; null = everything fits. */
  matchIds: number[] | null;
  /** Slips in the jar in all (for the lantern's brightness). */
  total: number;
  rainy: boolean;
  couple: string;
  /** A request is in flight: a tap on the glass does nothing. */
  busy: boolean;
  elRef: RefObject<HTMLDivElement | null>;
  handle: Ref<JarHandle>;
}

// styles/login.css also defines a bare `.jar { width: 100% }` (for the login page's SVG jar), which
// would stretch this one past its 240:300 aspect ratio. This page's jar sizes itself by ratio.
const JAR_STYLE = { width: "auto" } as const;

const TINTS = ["home", "apart", "local", "day_trip", "travel"] as const;

export default function JarCanvas({ slips, matchIds, total, rainy, couple, busy, elRef, handle }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sim = useRef({
    parts: [] as Part[],
    shown: [] as SlipInfo[],
    slots: [] as Slot[],
    matchSet: null as Set<number> | null,
    raf: 0,
    last: 0,
    scale: 1,
    colors: null as Colors | null,
    first: true,
  });
  const busyRef = useRef(busy);
  busyRef.current = busy;
  const rainyRef = useRef(rainy);
  rainyRef.current = rainy;

  /* ── drawing ───────────────────────────────────────────── */
  const readColors = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    // The tokens live on .joy (not :root), and dark mode is Flinderax's .dark class.
    const cs = getComputedStyle(canvas);
    const g = (n: string) => parseColor(cs.getPropertyValue(n));
    sim.current.colors = {
      dark: !!canvas.closest(".dark"),
      paper: g("--paper"),
      paper2: g("--paper-2"),
      slip: g("--slip"),
      ink: g("--ink"),
      tint: Object.fromEntries(TINTS.map((t) => [t, g(`--tint-${t}`)])),
    };
  };

  const render = () => {
    const canvas = canvasRef.current;
    const c = sim.current.colors;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || !c) return;
    const { parts, scale } = sim.current;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.setTransform(scale, 0, 0, scale, 0, 0);
    ctx.save();
    ctx.clip(INSIDE());
    // greyed-out slips sit behind, the ones that fit come forward
    const order = parts.filter((p) => !p.out).sort((a, b) => Number(a.dim < 0.5) - Number(b.dim < 0.5) || a.ty - b.ty);
    for (const p of order) drawSlip(ctx, p, c);
    ctx.restore();
  };

  const kick = () => {
    const s = sim.current;
    if (s.raf || document.hidden) return;
    s.last = performance.now();
    s.raf = requestAnimationFrame(frame);
  };
  function frame(now: number) {
    const s = sim.current;
    s.raf = 0;
    const dt = Math.min(0.034, (now - s.last) / 1000);
    s.last = now;
    const busyParts = step(s.parts, dt, reduced());
    render();
    if (busyParts) kick();
  }

  const settle = (animate: boolean) => {
    const s = sim.current;
    assign(s.parts, s.slots, { matchSet: s.matchSet, rainy: rainyRef.current, animate, reduced: reduced() });
    kick();
  };

  const at = (x: number, y: number) => {
    const el = elRef.current;
    if (!el) return null;
    const r = el.getBoundingClientRect();
    const s = r.width / W;
    return { x: r.left + x * s, y: r.top + y * s, s };
  };

  useImperativeHandle(handle, () => ({
    jostle(k) {
      if (reduced()) return;
      jostleParts(sim.current.parts, k);
      kick();
    },
    takeOut(id, setting) {
      const at = takePart(sim.current.parts, id, setting);
      if (at) settle(true);
      return at;
    },
    dropIn(id) {
      const s = sim.current;
      if (dropPart(s.parts, id, s.shown, reduced())) settle(true);
    },
    mouth: () => at(MOUTH.x, MOUTH.y),
    at,
    shake() {
      const el = elRef.current;
      if (!el || reduced()) return;
      el.classList.remove("is-shaking", "is-nudged");
      void el.offsetWidth;
      el.classList.add("is-shaking");
      setTimeout(() => el.classList.remove("is-shaking"), 400);
      jostleParts(sim.current.parts, 1);
      kick();
      setTimeout(() => {
        jostleParts(sim.current.parts, 0.5);
        kick();
      }, 170);
    },
    openLid(open) {
      elRef.current?.classList.toggle("is-open", open && !reduced());
    },
  }), []); // eslint-disable-line react-hooks/exhaustive-deps

  /* ── build the pile when the slips arrive ──────────────── */
  useEffect(() => {
    const s = sim.current;
    s.shown = sample(slips);
    s.parts = makeParts(s.shown);
    s.slots = makeSlots(s.parts.length + 1);
    s.matchSet = null;
    s.first = true;
    readColors();
    assign(s.parts, s.slots, { matchSet: null, rainy: rainyRef.current, animate: false, reduced: reduced() });
    // initial: slips settle in with a small drop, top ones last
    if (!reduced()) {
      for (const p of s.parts) {
        p.y -= 10 + Math.random() * 14;
        p.falling = true;
        p.vy = 0;
        p.delay = ((270 - p.ty) / 260) * 0.35 + Math.random() * 0.08;
      }
    }
    render();
    kick();
    return () => {
      cancelAnimationFrame(s.raf);
      s.raf = 0;
    };
  }, [slips]); // eslint-disable-line react-hooks/exhaustive-deps

  /* ── dim what doesn't fit ──────────────────────────────── */
  useEffect(() => {
    const s = sim.current;
    s.matchSet = matchIds ? new Set(matchIds) : null;
    elRef.current?.style.setProperty("--lit", String(matchIds && total ? (matchIds.length / total).toFixed(3) : 1));
    if (s.first) {
      s.first = false;
      if (!matchIds) return;
    }
    settle(true);
  }, [matchIds, total]); // eslint-disable-line react-hooks/exhaustive-deps

  /* ── size, visibility, theme ───────────────────────────── */
  useEffect(() => {
    const el = elRef.current;
    const canvas = canvasRef.current;
    if (!el || !canvas) return;
    const size = () => {
      const r = el.getBoundingClientRect();
      if (!r.width) return;
      const dpr = Math.min(window.devicePixelRatio || 1, 2.5);
      canvas.width = Math.round(r.width * dpr);
      canvas.height = Math.round(r.height * dpr);
      sim.current.scale = (r.width * dpr) / W;
      render();
    };
    const ro = new ResizeObserver(size);
    ro.observe(el);
    size();
    // Stop the loop when the tab is hidden; pick it up again when it's back.
    const onVisible = () => {
      const s = sim.current;
      if (document.hidden) {
        cancelAnimationFrame(s.raf);
        s.raf = 0;
      } else kick();
    };
    document.addEventListener("visibilitychange", onVisible);
    // Dark mode is a class on <html> (or any ancestor): re-read the colours when it flips.
    const mo = new MutationObserver(() => {
      readColors();
      render();
    });
    mo.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    const rm = matchMedia("(prefers-reduced-motion: reduce)");
    const onRm = () => settle(false);
    rm.addEventListener("change", onRm);
    return () => {
      ro.disconnect();
      mo.disconnect();
      document.removeEventListener("visibilitychange", onVisible);
      rm.removeEventListener("change", onRm);
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  /* ── a tap on the glass jostles the slips (decorative) ─── */
  const nudge = () => {
    const el = elRef.current;
    if (!el || busyRef.current || reduced()) return;
    el.classList.remove("is-nudged");
    void el.offsetWidth;
    el.classList.add("is-nudged");
    jostleParts(sim.current.parts, 0.55);
    kick();
  };

  return (
    <div className={`jar${rainy ? " is-rainy" : ""}`} id="jar" ref={elRef} aria-hidden="true" data-total={total} onPointerDown={nudge} style={JAR_STYLE}>
      <svg className="jar-back" viewBox="0 0 240 300" focusable="false">
        <defs>
          <radialGradient id="lantern" cx="50%" cy="56%" r="50%">
            <stop offset="0" className="glow-stop" />
            <stop offset="1" className="glow-stop glow-stop--out" />
          </radialGradient>
        </defs>
        <ellipse className="shelf-shadow" cx="122" cy="291" rx="104" ry="7" />
        <g className="lantern">
          <ellipse cx="120" cy="196" rx="118" ry="104" fill="url(#lantern)" />
        </g>
        <path className="glass-fill" d="M72 56v14c0 10-8 14-22 18-20 6-28 16-28 34v140c0 18 10 28 28 28h140c18 0 28-10 28-28V122c0-18-8-28-28-34-14-4-22-8-22-18V56z" />
        <path className="glass-back-rim" d="M28 276q92 9 184 0" />
      </svg>

      <canvas className="jar-slips" ref={canvasRef} />

      <svg className="jar-front" viewBox="0 0 240 300" focusable="false">
        <defs>
          <clipPath id="inside">
            <path d="M76 60v10c0 12-10 17-25 21-18 5-25 14-25 31v140c0 15 8 24 24 24h140c16 0 24-9 24-24V122c0-17-7-26-25-31-15-4-25-9-25-21V60z" />
          </clipPath>
          <radialGradient id="ff-glow">
            <stop offset="0" className="ff-core" />
            <stop offset=".4" className="ff-mid" />
            <stop offset="1" className="ff-out" />
          </radialGradient>
        </defs>
        <g className="fireflies" clipPath="url(#inside)">
          <circle cx="70" cy="200" r="5.4" style={ff("5.2s", "-.4s")} />
          <circle cx="150" cy="160" r="5.0" style={ff("6.4s", "-2.1s")} />
          <circle cx="112" cy="232" r="5.2" style={ff("4.6s", "-3.3s")} />
          <circle cx="186" cy="214" r="4.8" style={ff("7.1s", "-1.2s")} />
          <circle cx="96" cy="140" r="4.7" style={ff("5.8s", "-4.4s")} />
          <circle cx="168" cy="250" r="5.1" style={ff("6.8s", "-.9s")} />
        </g>
        <path className="glass-base" d="M24 268c2 12 11 20 26 20h140c15 0 24-8 26-20q-96 12-192 0z" />
        <path className="glass-floor" d="M26 274q94 10 188 0" />
        <path className="glass-edge" d="M72 56v14c0 10-8 14-22 18-20 6-28 16-28 34v140c0 18 10 28 28 28h140c18 0 28-10 28-28V122c0-18-8-28-28-34-14-4-22-8-22-18V56" />
        <path className="hl hl-1" d="M33 132c-1 40-1 86 1 122" />
        <path className="hl hl-2" d="M42 148v34" />
        <path className="hl hl-3" d="M58 96c-12 4-19 10-22 20" />
        <path className="hl hl-4" d="M209 140c1 30 1 70-1 104" />
        {rainy && (
          <g className="rain-glass">
            <path className="trickle" d="M188 132c.6 14-.8 26 .4 40" style={ff("9s", "-2s")} />
            <path className="trickle" d="M52 176c-.5 12 .6 22-.2 34" style={ff("11s", "-6s")} />
            <g className="beads">
              <path d="M44 140c2 3 3 4.6 3 6a3 3 0 0 1-6 0c0-1.4 1-3 3-6z" />
              <path d="M196 196c1.6 2.4 2.4 3.6 2.4 4.8a2.4 2.4 0 0 1-4.8 0c0-1.2.8-2.4 2.4-4.8z" />
              <path d="M160 108c1.4 2 2 3 2 4a2 2 0 0 1-4 0c0-1 .6-2 2-4z" />
              <path d="M70 226c1.6 2.4 2.4 3.6 2.4 4.8a2.4 2.4 0 0 1-4.8 0c0-1.2.8-2.4 2.4-4.8z" />
              <path d="M138 246c1.4 2 2 3 2 4a2 2 0 0 1-4 0c0-1 .6-2 2-4z" />
              <path d="M98 104c1.2 1.8 1.8 2.7 1.8 3.6a1.8 1.8 0 0 1-3.6 0c0-.9.6-1.8 1.8-3.6z" />
              <path d="M184 170c1.2 1.8 1.8 2.7 1.8 3.6a1.8 1.8 0 0 1-3.6 0c0-.9.6-1.8 1.8-3.6z" />
            </g>
          </g>
        )}
        <rect className="rim" x="66" y="52" width="108" height="9" rx="4" />
        <g className="j-lid">
          <rect className="j-lid-body" x="62" y="28" width="116" height="26" rx="6" />
          <path className="j-lid-ridges" d="M74 33v17M84 33v17M94 33v17M104 33v17M114 33v17M124 33v17M134 33v17M144 33v17M154 33v17M164 33v17" />
          <path className="lid-shine" d="M68 32.5h104" />
        </g>
        <g className="label" transform="rotate(-4 120 128)">
          <path className="tape" d="M64 116l3 3-2 3 2 3-2 3 2 3-3 3h112l-2-3 3-3-3-3 3-3-3-3 2-3z" />
          <text className="label-text" x="120" y="132" textAnchor="middle">
            {couple}
          </text>
        </g>
      </svg>
    </div>
  );
}

// CSS custom properties on SVG children: React types them as a plain record.
function ff(d: string, o: string) {
  return { "--d": d, "--o": o } as React.CSSProperties;
}

let inside: Path2D | null = null;
const INSIDE = () =>
  (inside ??= new Path2D("M76 60v10c0 12-10 17-25 21-18 5-25 14-25 31v140c0 15 8 24 24 24h140c16 0 24-9 24-24V122c0-17-7-26-25-31-15-4-25-9-25-21V60z"));

function drawSlip(ctx: CanvasRenderingContext2D, p: Part, C: Colors) {
  const t = C.tint[p.setting] || C.tint.local;
  const grey = C.dark ? mix(t, C.paper, 0.62) : mix(mix(t, C.paper2, 0.82), C.ink, 0.04);
  let body = mix(t, grey, p.dim);
  if (C.dark) body = mix(body, [255, 200, 130], 0.2 * (1 - p.dim)); // slips catch the lantern light
  const flap = mix(body, C.dark ? C.ink : C.slip, (C.dark ? 0.12 : 0.5) * (1 - p.dim * 0.6));
  const alpha = 1 - 0.3 * p.dim;
  ctx.save();
  ctx.translate(p.x, p.y);
  ctx.rotate(p.a);
  ctx.beginPath();
  if (ctx.roundRect) ctx.roundRect(-SW / 2, -SH / 2, SW, SH, 1.6);
  else ctx.rect(-SW / 2, -SH / 2, SW, SH);
  ctx.fillStyle = rgba(body, alpha);
  ctx.fill();
  ctx.fillStyle = rgba(flap, alpha);
  ctx.fillRect(2, -SH / 2 + 0.3, SW / 2 - 2.3, SH - 0.6);
  ctx.lineWidth = 0.7;
  ctx.strokeStyle = rgba(C.ink, (C.dark ? 0.2 : 0.26) * (1 - p.dim * 0.5));
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(2, -SH / 2);
  ctx.lineTo(2, SH / 2);
  ctx.stroke();
  if (p.dim < 0.6) {
    // ink showing through the paper
    ctx.strokeStyle = rgba(C.ink, 0.3 * (1 - p.dim / 0.6));
    ctx.lineWidth = 0.8;
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(-11.5, -2);
    ctx.quadraticCurveTo(-7, -3 + p.scribble * 0.4, -1.5, -2.2);
    ctx.moveTo(-11.5, 1.6);
    ctx.lineTo(-5 - p.scribble * 1.5, 1.4);
    ctx.stroke();
  }
  ctx.restore();
}

