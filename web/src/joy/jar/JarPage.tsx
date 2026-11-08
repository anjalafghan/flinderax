/* eslint-disable react-hooks/refs, react-hooks/set-state-in-effect --
   an imperative page: the latest-value refs, the sim state and the one-time capability probes are deliberate. */
// The jar page: Joy's decide.html / pick_card.html / planned.html as React.
//
// This file owns the requests and the choreography; the pieces are elsewhere:
//   physics.ts / JarCanvas.tsx  the slip simulation and the glass
//   Sentence.tsx / form.ts      the filter sentence and the forms it becomes
//   PickCard.tsx / peeks.ts     the drawn slip, the stamp, the note, peek counting
//   swipe.ts / drag.ts          the swipe rules and the pointer plumbing
//   DayPlanView.tsx             the next date's plan
//   motion.ts / HeartLayer.tsx  Joy's spring presets and the hearts
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import "./jar-scope.css";
import { jarApi, type DecideView, type PickCard } from "./api";
import { createChoreo } from "./choreo";
import DayPlanView from "./DayPlanView";
import { attachDrag, attachPress } from "./drag";
import { RegularCard } from "../regulars/RegularCard";
import { applyRelax, EMPTY_FILTERS, filtersToForm, putBackForm, stampDate, vetoForm, type Filters, type PickForm } from "./form";
import HeartLayer, { type HeartHandle } from "./HeartLayer";
import JarCanvas, { type JarHandle } from "./JarCanvas";
import { buzz, finish, pose, reduced, set, springs, to } from "./motion";
import { NoteView, SlipView, type CardActions } from "./PickCard";
import { peekView, spentDot } from "./peeks";
import Sentence from "./Sentence";
import { magnitude, newShake, shakeStep } from "./shake";
import { dragPose, MOST, swipeOutcome } from "./swipe";

type View =
  | { kind: "none"; key: number }
  | { kind: "card" | "done" | "note"; key: number; card: PickCard; putBack: boolean };

const blankCard = (message: string): PickCard => ({
  activity: null, message, relax_label: "", relax: null, done: false, stamp_date: "", rerolls_left: 0, max_rerolls: 0,
  free_putback: false, pairs: [], paired: null, state: null, bonus_peek: false, can_surprise: false, can_veto: false,
  partner: "", surprise: false, history_ids: "", share_text: "", wa_url: "", middle: false, when: "", start: "",
  when_label: "", plan_line: "", route: [], route_note: "", plan: null,
});

const TAUGHT = "joy.swipe-taught";
const taught = () => {
  try {
    return Number(localStorage.getItem(TAUGHT) || 0);
  } catch {
    return 2;
  }
};
const finished = (a: Animation) => a.finished.then(() => undefined, () => undefined);

export default function JarPage() {
  const [data, setData] = useState<DecideView | null>(null);
  const [error, setError] = useState(false);
  const [filters, setFilters] = useState<Filters>(EMPTY_FILTERS);
  const [matchIds, setMatchIds] = useState<number[] | null>(null);
  const [counter, setCounter] = useState<{ count: number; total: number } | null>(null);
  const [view, setView] = useState<View>({ kind: "none", key: 0 });
  const [busy, setBusy] = useState(false);
  const [arriving, setArriving] = useState(false);
  const [shakeOn, setShakeOn] = useState(false);
  const [shakeLabel, setShakeLabel] = useState("or shake your phone");
  const [shakeAvailable, setShakeAvailable] = useState(false);

  const rootRef = useRef<HTMLDivElement>(null);
  const pickRef = useRef<HTMLDivElement>(null);
  const jarEl = useRef<HTMLDivElement>(null);
  const jar = useRef<JarHandle>(null);
  const hearts = useRef<HeartHandle>(null);
  const ghosts = useRef<HTMLDivElement>(null);
  const tagRef = useRef<HTMLParagraphElement>(null);
  const tagN = useRef<HTMLSpanElement>(null);

  const choreo = useMemo(() => createChoreo(), []);
  const lock = useRef(false); // set synchronously: state is too late for a double tap
  const keyRef = useRef(0);
  const viewRef = useRef(view);
  viewRef.current = view;
  const filtersRef = useRef(filters);
  filtersRef.current = filters;
  const touched = useRef(false);
  const matchDelay = useRef(60);
  const seq = useRef(0);
  const timers = useRef<ReturnType<typeof setTimeout>[]>([]);
  const alive = useRef(true);

  useEffect(() => {
    alive.current = true;
    const pending = timers.current;
    return () => {
      alive.current = false;
      choreo.dispose();
      pending.forEach(clearTimeout);
    };
  }, [choreo]);

  /* ── load the jar ──────────────────────────────────────── */
  const load = useCallback(() => {
    setError(false);
    jarApi
      .jar()
      .then((d) => {
        if (!alive.current) return;
        setData(d);
        setFilters((f) => (f === EMPTY_FILTERS ? { ...EMPTY_FILTERS, when: d.whens.find((w) => w.selected)?.value ?? d.whens[0]?.value ?? "" } : f));
      })
      .catch(() => alive.current && setError(true));
  }, []);
  useEffect(load, [load]);

  /** After a Plan B swap: fetch the plan again, but keep the pile in the jar as it is. */
  const reload = useCallback(() => {
    jarApi
      .jar()
      .then((fresh) => {
        if (!alive.current) return;
        setData((prev) => (prev ? { ...fresh, slips: prev.slips } : fresh));
        setView((v) => (v.kind === "done" && fresh.next_plan && v.card.plan?.history_id === fresh.next_plan.history_id ? { ...v, card: { ...v.card, plan: fresh.next_plan } } : v));
      })
      .catch(() => undefined);
  }, []);

  /* ── the live "N slips could happen" counter ───────────── */
  useEffect(() => {
    if (!data || !touched.current) return;
    const t = setTimeout(async () => {
      const mine = ++seq.current;
      try {
        const j = await jarApi.match(filtersToForm(filters, false));
        if (mine !== seq.current || !alive.current) return; // a newer answer is on its way
        setMatchIds(j.ids);
        setCounter({ count: j.count, total: j.total });
      } catch {
        /* session expired or offline: the jar just stays as it is */
      }
    }, matchDelay.current);
    return () => clearTimeout(t);
  }, [filters, data]);

  const shownCount = counter ? counter.count : (data?.count ?? 0);
  const prevCount = useRef(shownCount);
  useEffect(() => {
    if (prevCount.current === shownCount) return;
    prevCount.current = shownCount;
    if (reduced()) return;
    tagRef.current?.animate([{ transform: "rotate(4deg)" }, { transform: "rotate(10deg)" }, { transform: "rotate(1deg)" }, { transform: "rotate(5deg)" }, { transform: "rotate(4deg)" }], { duration: 700, easing: "ease-out" });
    tagN.current?.animate([{ transform: "translateY(-6px)", opacity: 0 }, { transform: "none", opacity: 1 }], { duration: 220, easing: "cubic-bezier(.2,.8,.2,1)" });
  }, [shownCount]);

  const change = useCallback((patch: Partial<Filters>, slow = false) => {
    touched.current = true;
    matchDelay.current = slow ? 200 : 60;
    setFilters((f) => ({ ...f, ...patch }));
  }, []);

  /* ── choreography ──────────────────────────────────────── */
  const pickEl = () => pickRef.current;
  const hold = () => {
    const p = pickEl();
    if (!p) return;
    p.style.setProperty("--hold", `${p.offsetHeight}px`);
    p.classList.add("is-holding");
  };
  const release = () => {
    const p = pickEl();
    if (!p) return;
    p.classList.remove("is-holding");
    p.style.removeProperty("--hold");
  };
  const unlock = () => {
    release();
    lock.current = false;
    setArriving(false);
    setBusy(false);
  };
  const focusTitle = () => pickEl()?.querySelector<HTMLElement>(".j-slip-title")?.focus({ preventScroll: true });

  /** The slip folds and drops back into the jar. */
  const foldAway = (old: HTMLElement): Promise<void> => {
    const host = ghosts.current;
    const h = jar.current;
    const id = Number(old.dataset.id) || 0;
    if (!host || !h) return Promise.resolve();
    const r = old.getBoundingClientRect();
    const g = old.cloneNode(true) as HTMLElement;
    g.querySelectorAll("[id]").forEach((n) => n.removeAttribute("id"));
    g.removeAttribute("aria-labelledby");
    g.classList.add("ghost");
    g.setAttribute("aria-hidden", "true");
    g.inert = true;
    Object.assign(g.style, { left: `${r.left}px`, top: `${r.top}px`, width: `${r.width}px`, height: `${r.height}px` });
    host.appendChild(g);
    old.classList.add("is-leaving");
    old.inert = true;
    const done = () => {
      g.remove();
      if (id) h.dropIn(id);
    };
    if (reduced()) return finished(choreo.track(g.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 150, fill: "forwards" }))).then(done);
    const cover = document.createElement("div");
    cover.className = "slip-cover";
    g.appendChild(cover);
    const m = h.mouth();
    if (!m) {
      done();
      return Promise.resolve();
    }
    const dx = m.x - (r.left + r.width / 2);
    const dy = m.y - r.top;
    const base = getComputedStyle(old).transform;
    const start = base && base !== "none" ? base : "none";
    choreo.track(cover.animate([{ opacity: 0 }, { opacity: 1, offset: 0.35 }, { opacity: 1 }], { duration: 480, fill: "forwards" }));
    const a = choreo.track(
      g.animate(
        [
          { transform: start, opacity: 1 },
          { transform: "translateY(-6px) scale(1, .07) rotate(-1.5deg)", offset: 0.38, opacity: 1 },
          { transform: `translate(${dx}px, ${dy - 46}px) scale(${34 / r.width}, ${14 / r.height}) rotate(10deg)`, offset: 0.78, opacity: 1 },
          { transform: `translate(${dx}px, ${dy + 14}px) scale(${30 / r.width}, ${12 / r.height}) rotate(24deg)`, opacity: 0 },
        ],
        { duration: 480, easing: "cubic-bezier(.45,0,.3,1)", fill: "forwards" },
      ),
    );
    choreo.later(() => h.jostle(0.35), 420);
    return finished(a).then(done);
  };

  /** Once a slip has arrived: let go of the lock, move focus to the title, teach the swipe. */
  const ready = (el: HTMLElement, teach: boolean) => {
    jar.current?.openLid(false);
    unlock();
    focusTitle();
    if (!teach || reduced() || taught() >= 2 || el.dataset.kind !== "card") return;
    try {
      localStorage.setItem(TAUGHT, String(taught() + 1));
    } catch {
      /* fine */
    }
    const canBack = !!el.querySelector(".slip-actions .btn-ink");
    timers.current.push(
      setTimeout(async () => {
        if (!el.isConnected || rootRef.current?.classList.contains("m-dragging") || lock.current) return;
        await to(el, { x: 20, r: 0.5, config: { tension: 200, friction: 16 } });
        await to(el, { x: canBack ? -16 : 0, r: canBack ? -0.4 : 0, config: { tension: 200, friction: 16 } });
        to(el, { x: 0, r: 0, config: springs.snap });
      }, 650),
    );
  };

  /** One slip rises out of the mouth and unfolds into the card. */
  const arrive = (el: HTMLElement, card: PickCard) => {
    const h = jar.current;
    const p = pickEl();
    const cover = el.querySelector<HTMLElement>(".slip-cover");
    if (!h || !p) return ready(el, true);
    const id = card.activity?.id ?? 0;
    const setting = card.activity?.setting_key ?? "local";
    const from = h.takeOut(id, setting) ?? { x: 120, y: 80, a: 0, setting };
    if (reduced()) {
      release();
      if (cover) choreo.track(cover.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 150, fill: "forwards" }));
      void finished(choreo.track(el.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 150 }))).then(() => ready(el, true));
      return;
    }
    const p0 = h.at(from.x, from.y);
    const pm = h.mouth();
    const r = el.getBoundingClientRect();
    const cx = r.left + r.width / 2;
    const cy = r.top + 10;
    // the region grows as the slip unfolds, so the sentence below glides instead of jumping
    const held = p.classList.contains("is-holding") ? parseFloat(p.style.getPropertyValue("--hold")) || 0 : 0;
    release();
    const overlap = -parseFloat(getComputedStyle(p).marginTop) || 0;
    const startH = held || Math.max(0, overlap - 14);
    const endH = p.offsetHeight;
    const delay = 440;
    const fl = document.createElement("div");
    fl.className = "flyer";
    fl.style.setProperty("--tint", `var(--tint-${from.setting || "local"})`);
    if (p0 && pm) {
      ghosts.current?.appendChild(fl);
      const at = (x: number, y: number, rot: number, s: number) => `translate(${x - 17}px, ${y - 7}px) rotate(${rot}rad) scale(${s})`;
      const k = (p0.s * 30) / 34;
      choreo.track(
        fl.animate(
          [
            { transform: at(p0.x, p0.y, from.a, k) },
            { transform: at(pm.x, pm.y + 6, -0.15, k * 1.05), offset: 0.34 },
            { transform: at(pm.x + 6, pm.y - 46 * pm.s, -0.3, k * 1.7), offset: 0.58 },
            { transform: at(cx, cy, 0, Math.min(3, (r.width / 34) * 0.3)), opacity: 1, offset: 0.92 },
            { transform: at(cx, cy, 0, Math.min(3, (r.width / 34) * 0.3)), opacity: 0 },
          ],
          { duration: 560, easing: "cubic-bezier(.33,.7,.25,1)", fill: "forwards" },
        ),
      );
      choreo.later(() => fl.remove(), 620);
    }
    choreo.track(p.animate([{ height: `${startH}px` }, { height: `${startH}px`, offset: delay / (delay + 480), easing: "cubic-bezier(.3,.7,.2,1)" }, { height: `${endH}px` }], { duration: delay + 480 }));
    if (cover) choreo.track(cover.animate([{ opacity: 1 }, { opacity: 1, offset: 0.5 }, { opacity: 0 }], { duration: 480, delay, fill: "both" }));
    const a = choreo.track(
      el.animate(
        [
          { transform: "translateY(-4px) scale(.3, .05) rotate(-3deg)", opacity: 0 },
          { opacity: 1, offset: 0.08 },
          { transform: "translateY(-4px) scale(1, .06) rotate(-1.6deg)", offset: 0.38 },
          { transform: "perspective(1200px) rotateX(-28deg) scale(1, .7) rotate(-1deg)", offset: 0.66 },
          { transform: "rotate(-.8deg)", opacity: 1 },
        ],
        { duration: 480, delay, easing: "cubic-bezier(.2,.8,.2,1)", fill: "backwards" },
      ),
    );
    void finished(a).then(() => {
      fl.remove();
      ready(el, true);
    });
  };

  const noteArrives = (el: HTMLElement) => {
    jar.current?.openLid(false);
    release();
    if (reduced()) return void finished(choreo.track(el.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 150 }))).then(unlock);
    void finished(
      choreo.track(
        el.animate(
          [
            { transform: "translateY(-14px) rotate(-2deg)", opacity: 0 },
            { transform: "rotate(1.2deg)", opacity: 1 },
          ],
          { duration: 420, easing: "cubic-bezier(.34,1.56,.64,1)" },
        ),
      ),
    ).then(unlock);
  };

  /** The vermilion stamp thumps on, the phone buzzes, a few hearts fly. */
  const stampIt = (el: HTMLElement) => {
    const stamp = el.querySelector<HTMLElement>(".stamp");
    const end = () => {
      unlock();
      focusTitle();
    };
    if (!stamp) return end();
    const land = () => {
      buzz(12);
      const sr = stamp.getBoundingClientRect();
      hearts.current?.burst(sr.left + sr.width / 2, sr.top + sr.height / 2);
      if (!reduced()) {
        set(el, { y: 4 });
        choreo.trackSpring(to(el, { y: 0, config: springs.boing }), () => finish(el));
      }
    };
    if (reduced()) {
      land();
      void finished(choreo.track(stamp.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 150 }))).then(end);
      return;
    }
    set(stamp, { x: 14, y: -46, s: 1.9, r: -8, o: 0 });
    const thump = to(stamp, { x: 0, y: 0, s: 1, r: 0, o: 1, config: springs.boing });
    choreo.trackSpring(thump, () => finish(stamp));
    choreo.later(land, 320);
    // The boing rings for a while: don't hold the buttons for all of it.
    void Promise.race([thump, choreo.wait(700)]).then(() => choreo.wait(300)).then(end);
  };

  useLayoutEffect(() => {
    const v = viewRef.current;
    if (v.kind === "none") return;
    const el = pickEl()?.firstElementChild as HTMLElement | null;
    if (!el) return unlock();
    if (v.kind === "card") arrive(el, v.card);
    else if (v.kind === "done") stampIt(el);
    else noteArrives(el);
  }, [view.key]); // eslint-disable-line react-hooks/exhaustive-deps

  /* ── requests ──────────────────────────────────────────── */
  const show = (kind: "card" | "done" | "note", card: PickCard, putBack = false) => {
    setArriving(true);
    setView({ kind, key: ++keyRef.current, card, putBack });
  };

  /** Draw, put back, pass, or relax: all post a form to /pick and bring the answer out of the jar. */
  const draw = async (form: PickForm, how: "draw" | "putback" | "veto" | "relax", relaxTo?: Filters) => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    const h = jar.current;
    if (relaxTo) {
      touched.current = true;
      matchDelay.current = 0;
      setFilters(relaxTo);
    }
    const stage = jarEl.current;
    if (stage) {
      const top = stage.getBoundingClientRect().top;
      if (top < 40 || top > innerHeight * 0.5) stage.scrollIntoView({ block: "start", behavior: reduced() ? "auto" : "smooth" });
    }
    const old = pickEl()?.firstElementChild as HTMLElement | null;
    if (old) {
      hold();
      void foldAway(old);
    }
    h?.openLid(true);
    h?.shake();
    try {
      const [card] = await Promise.all([jarApi.pick(form), choreo.wait(reduced() ? 0 : 340)]);
      if (!alive.current) return;
      if (card.activity) show("card", card, how === "putback");
      else show("note", card);
    } catch {
      if (!alive.current) return;
      show("note", blankCard("The lid stuck for a second. Try that again?"));
    }
  };

  const accept = async (o: { with?: string; surprise?: boolean }) => {
    const v = viewRef.current;
    if (lock.current || v.kind !== "card" || !v.card.activity) return;
    lock.current = true;
    setBusy(true);
    try {
      const card = await jarApi.accept(v.card.activity.id, { ...o, when: v.card.when, start: v.card.start });
      if (!alive.current) return;
      show("done", { ...card, stamp_date: stampDate(card.stamp_date) });
    } catch {
      if (!alive.current) return;
      show("note", blankCard("The stamp stuck for a second. Try that again?"));
    }
  };

  const backToJar = () => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    const old = pickEl()?.firstElementChild as HTMLElement | null;
    hold();
    jar.current?.openLid(true);
    void (old ? foldAway(old) : Promise.resolve()).then(() => {
      if (!alive.current) return;
      jar.current?.openLid(false);
      setView({ kind: "none", key: ++keyRef.current });
      unlock();
      rootRef.current?.querySelector<HTMLElement>("#draw")?.focus({ preventScroll: true });
    });
  };

  const drawNow = (middle = false) => draw(filtersToForm(filtersRef.current, middle), "draw");

  const actions: CardActions = {
    accept,
    putBack: () => {
      const v = viewRef.current;
      if (v.kind === "card" && v.card.state) void draw(putBackForm(v.card.state), "putback");
    },
    veto: () => {
      const v = viewRef.current;
      if (v.kind === "card" && v.card.state) void draw(vetoForm(v.card.state), "veto");
    },
    relax: () => {
      const v = viewRef.current;
      if (v.kind === "note" && v.card.relax) void draw(v.card.relax, "relax", applyRelax(filtersRef.current, v.card.relax));
    },
    back: backToJar,
    planSwapped: reload,
  };
  const actionsRef = useRef(actions);
  actionsRef.current = actions;
  const drawRef = useRef(drawNow);
  drawRef.current = drawNow;

  /* ── pointer plumbing: press squash, swipe, shake the jar, tap to skip ── */
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const offs: (() => void)[] = [];

    offs.push(attachPress(root, ".btn, .j-chip, .share", (el, down) => void to(el, down ? { s: 0.93, config: springs.press } : { s: 1, config: springs.boing })));

    // The drawn slip: swipe right for "It's a date", left to put it back.
    offs.push(
      attachDrag(root, {
        sel: '#pick > .j-slip[data-kind="card"]',
        ok: (el) => !lock.current && !el.classList.contains("is-arriving"),
        start: (d) => pose(d.el).stop(),
        move: (d) => {
          const v = viewRef.current;
          const can = v.kind === "card" && (d.dx > 0 ? true : peekView(v.card).canPutBack);
          const p = dragPose(d.dx, can);
          set(d.el, p);
        },
        end: (d, cancelled) => {
          const v = viewRef.current;
          if (v.kind !== "card") return void to(d.el, { x: 0, y: 0, r: 0, s: 1, config: springs.snap });
          const out = swipeOutcome({ dx: d.dx, vx: d.vx, canYes: true, canBack: peekView(v.card).canPutBack, cancelled });
          if (out === "yes") {
            void to(d.el, { x: 0, y: 0, r: 0, s: 1, config: springs.snap });
            void actionsRef.current.accept({});
          } else if (out === "back") {
            void to(d.el, { x: -MOST, r: -0.6, s: 0.94, config: springs.quick });
            actionsRef.current.putBack();
          } else void to(d.el, { x: 0, y: 0, r: 0, s: 1, config: springs.snap });
        },
      }),
    );

    // Grab the jar and shake it side to side: three quick back-and-forths and it gives up a slip.
    let flips = 0;
    let dir = 0;
    let last = 0;
    offs.push(
      attachDrag(root, {
        sel: "#jar",
        ok: () => !lock.current,
        start: () => {
          flips = 0;
          dir = 0;
          last = performance.now();
        },
        move: (d) => {
          const el = jarEl.current;
          if (!el) return;
          const x = Math.max(-38, Math.min(38, d.dx * 0.45));
          set(el, { x, r: x / 3 });
          const now = performance.now();
          const s = Math.sign(d.vx);
          if (Math.abs(d.vx) > 0.35 && s !== 0 && s !== dir) {
            if (dir !== 0) {
              flips = now - last < 600 ? flips + 1 : 1;
              last = now;
              jar.current?.jostle(0.8);
              buzz(6);
            }
            dir = s;
          }
        },
        end: () => {
          const el = jarEl.current;
          if (el) void to(el, { x: 0, r: 0, config: springs.boing });
          if (flips >= 3 && !lock.current) void drawRef.current(false);
        },
      }),
    );

    // A tap or a key skips any hero animation to its end state; the click after it is swallowed.
    let swallowUntil = 0;
    const down = () => {
      if (lock.current && choreo.animating()) {
        choreo.skip();
        swallowUntil = performance.now() + 700;
      }
    };
    const click = (e: MouseEvent) => {
      if (performance.now() < swallowUntil) {
        e.preventDefault();
        e.stopPropagation();
        swallowUntil = 0;
      }
    };
    const key = (e: KeyboardEvent) => {
      if (lock.current && choreo.animating() && !e.metaKey && !e.ctrlKey) {
        choreo.skip();
        if (e.key === "Enter" || e.key === " ") e.preventDefault();
      }
    };
    root.addEventListener("pointerdown", down, true);
    root.addEventListener("click", click, true);
    root.addEventListener("keydown", key, true);
    return () => {
      offs.forEach((f) => f());
      root.removeEventListener("pointerdown", down, true);
      root.removeEventListener("click", click, true);
      root.removeEventListener("keydown", key, true);
    };
  }, [choreo]);

  /* ── shake your phone to draw (progressive extra) ──────── */
  useEffect(() => {
    const coarse = typeof matchMedia === "function" && matchMedia("(pointer: coarse)").matches;
    // iOS only delivers motion events on secure origins (https or localhost).
    if (!coarse || !window.isSecureContext || !("DeviceMotionEvent" in window)) return;
    setShakeAvailable(true);
    let saved = false;
    try {
      saved = localStorage.getItem("joy.shake") === "1";
    } catch {
      /* private mode */
    }
    const needsPermission = typeof (DeviceMotionEvent as unknown as { requestPermission?: unknown }).requestPermission === "function";
    if (saved && !needsPermission) setShakeOn(true);
  }, []);

  useEffect(() => {
    if (!shakeOn) {
      setShakeLabel((l) => (l === "shake to draw is on" ? "or shake your phone" : l));
      return;
    }
    setShakeLabel("shake to draw is on");
    let s = newShake();
    const onMotion = (ev: DeviceMotionEvent) => {
      const r = shakeStep(s, magnitude(ev), performance.now(), lock.current);
      s = r.state;
      if (r.jiggle) jar.current?.jostle(r.jiggle);
      if (r.draw) void drawRef.current(false);
    };
    addEventListener("devicemotion", onMotion);
    return () => removeEventListener("devicemotion", onMotion);
  }, [shakeOn]);

  const toggleShake = async () => {
    if (shakeOn) {
      setShakeOn(false);
      try {
        localStorage.removeItem("joy.shake");
      } catch {
        /* fine */
      }
      return;
    }
    try {
      const req = (DeviceMotionEvent as unknown as { requestPermission?: () => Promise<string> }).requestPermission;
      if (typeof req === "function" && (await req()) !== "granted") return void setShakeLabel("motion is off, tap Draw instead");
      setShakeOn(true);
      try {
        localStorage.setItem("joy.shake", "1");
      } catch {
        /* fine */
      }
    } catch {
      setShakeAvailable(false);
    }
  };

  /* ── render ────────────────────────────────────────────── */
  const rootProps = { ref: rootRef, className: "m-on", style: { overflowX: "clip" as const } };

  if (!data) {
    return (
      <div {...rootProps}>
        <div className="jar-main">
          {error ? (
            <div className="note" role="alert">
              <p className="note-text">The jar wouldn't open.</p>
              <button className="btn btn-stamp" type="button" onClick={load}>
                Try again
              </button>
            </div>
          ) : (
            <p role="status" className="visually-hidden">
              Opening the jar
            </p>
          )}
        </div>
      </div>
    );
  }

  const v = data;
  const empty = v.count === 0;
  const plan = v.next_plan;
  const cardNode =
    view.kind === "none" ? null : view.kind === "note" ? (
      <NoteView card={view.card} busy={busy} actions={actions} />
    ) : view.card.activity ? (
      <SlipView
        key={view.key}
        card={view.card}
        busy={busy}
        actions={actions}
        arriving={arriving}
        spent={arriving ? spentDot(view.card.rerolls_left, view.putBack, view.card.free_putback) : -1}
      />
    ) : null;

  return (
    <div {...rootProps}>
      <div className="jar-main">
        <div className={`jar-page${empty ? " is-empty" : ""}`}>
          <section className="stage" aria-labelledby="jar-h">
            <h1 id="jar-h" className="visually-hidden">
              {v.couple}: the date jar
            </h1>
            <JarCanvas slips={v.slips} matchIds={matchIds} total={v.slips.length} rainy={v.rain != null} couple={v.couple} busy={busy} elRef={jarEl} handle={jar} />
            <svg className="twine" viewBox="0 0 100 40" preserveAspectRatio="none" aria-hidden="true" focusable="false">
              <path d="M2 2C26 30 62 42 99 38" />
            </svg>
            <p className="tag" id="tag" role="status" aria-live="polite" ref={tagRef}>
              <span className="tag-n" id="tag-n" ref={tagN}>
                {shownCount}
              </span>{" "}
              <span className="tag-unit" id="tag-unit">
                {shownCount === 1 ? "slip" : "slips"}
              </span>
              <span className="j-tag-text" id="tag-text">
                {shownCount === 0 ? (empty ? "in the jar so far" : "fit tonight, yet") : "could happen tonight"}
              </span>
            </p>
          </section>

          {empty && (
            <div className="empty-note grain">
              <p className="empty-hand">The jar is empty.</p>
              <p>Every date starts as a slip of paper. Write the first one and it goes straight in.</p>
              <Link className="btn btn-stamp" to="/joy/slips/new">
                Write the first slip
              </Link>
            </div>
          )}

          {/* #pick stays empty until a slip is drawn: Joy's CSS keys off :empty and sits .tonight right after it. */}
          <div id="pick" ref={pickRef} aria-live="polite" aria-busy={busy}>
            {cardNode}
          </div>

          {!empty && plan && (
            <div className="next-plan">
              <p className="next-plan-k">
                {plan.my_open_tasks > 0 ? `Your next date · ${plan.my_open_tasks} thing${plan.my_open_tasks > 1 ? "s" : ""} for you to do` : "Your next date"}
              </p>
              <DayPlanView plan={plan} onSwapped={reload} />
            </div>
          )}

          {!empty && (
            <Sentence
              view={v}
              filters={filters}
              onChange={change}
              busy={busy}
              onDraw={() => void drawNow(false)}
              onDrawMiddle={() => void drawNow(true)}
              shake={{ available: shakeAvailable, on: shakeOn, label: shakeLabel, toggle: () => void toggleShake() }}
            />
          )}

          <aside className="shelf-notes" aria-label="Notes from the shelf">
            <RegularCard card={v.regulars} onChange={reload} />
            {v.memory && (
              <div className="keepsake grain">
                <span className="keepsake-k">This week, a year ago</span>
                <span className="keepsake-t">{v.memory.title}</span>
                <span className="keepsake-d">{v.memory.when}</span>
              </div>
            )}
            {!v.budget.budget ? (
              <div className="budget-line">
                <p>
                  <strong>{v.budget.spent}</strong> on dates this month
                </p>
              </div>
            ) : (
              <div className={`budget-line${v.budget.over ? " is-over" : ""}`}>
                <p>
                  <strong>{v.budget.spent}</strong> of {v.budget.budget} on dates this month
                  {v.budget.over && <span className="visually-hidden">, over budget</span>}
                </p>
                <span className="budget-bar" style={{ "--pct": v.budget.pct } as React.CSSProperties} aria-hidden="true" />
              </div>
            )}
          </aside>
        </div>
      </div>
      {/* Things that fly or fold live inside the page, never on <body>, so they keep the .joy tokens. */}
      <div ref={ghosts} aria-hidden="true" />
      <HeartLayer handle={hearts} />
    </div>
  );
}
