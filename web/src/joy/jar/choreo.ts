// Tracks the hero animations of one draw so a tap or a key can skip straight to the end state.
// Everything it starts is cancellable; dispose() clears timers so nothing fires after unmount.

interface Item {
  finish: () => void;
  playing: () => boolean;
}

export interface Choreo {
  /** A Web Animation. */
  track: (a: Animation) => Animation;
  /** Anything with a promise (a spring) and a way to put it at its end state. */
  trackSpring: (done: Promise<unknown>, finish: () => void) => void;
  later: (fn: () => void, ms: number) => void;
  wait: (ms: number) => Promise<void>;
  skip: () => void;
  animating: () => boolean;
  dispose: () => void;
}

export function createChoreo(): Choreo {
  let items: Item[] = [];
  let pending: { fn: () => void; t: ReturnType<typeof setTimeout> }[] = [];

  const later = (fn: () => void, ms: number) => {
    const p = {
      fn,
      t: setTimeout(() => {
        pending = pending.filter((x) => x !== p);
        fn();
      }, ms),
    };
    pending.push(p);
  };

  return {
    track(a) {
      items.push({
        finish: () => {
          try {
            a.finish();
          } catch {
            a.cancel();
          }
        },
        playing: () => a.playState === "running" || a.pending,
      });
      return a;
    },
    trackSpring(done, finish) {
      let over = false;
      done.then(() => (over = true), () => (over = true));
      items.push({ finish: () => ((over = true), finish()), playing: () => !over });
    },
    later,
    wait: (ms) => new Promise<void>((r) => later(r, ms)),
    skip() {
      for (let i = 0; i < 3 && (items.length || pending.length); i++) {
        const ps = pending;
        pending = [];
        ps.forEach((p) => {
          clearTimeout(p.t);
          p.fn();
        });
        const as = items;
        items = [];
        as.forEach((a) => a.finish());
      }
    },
    animating: () => items.some((a) => a.playing()) || pending.length > 0,
    dispose() {
      pending.forEach((p) => clearTimeout(p.t));
      pending = [];
      items = [];
    },
  };
}
