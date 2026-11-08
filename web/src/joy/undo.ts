// The "Took out ... Undo" toast: it lasts 6.5 s, and while the pointer is over it or focus is
// inside it, the clock waits (rechecking every 1.5 s) so a slow tap on Undo isn't missed.
// A small state machine with an injected scheduler, so it can be tested without real time.

export const UNDO_MS = 6500;
export const RECHECK_MS = 1500;

export interface Scheduler {
  set(fn: () => void, ms: number): unknown;
  clear(handle: unknown): void;
}

export const realScheduler: Scheduler = {
  set: (fn, ms) => setTimeout(fn, ms),
  clear: (h) => clearTimeout(h as ReturnType<typeof setTimeout>),
};

export interface UndoTimer {
  start(): void;
  cancel(): void;
  readonly done: boolean;
}

export function createUndoTimer(opts: {
  onExpire: () => void;
  /** True while the toast is hovered or focused. */
  isHeld: () => boolean;
  scheduler?: Scheduler;
  ms?: number;
  recheckMs?: number;
}): UndoTimer {
  const s = opts.scheduler ?? realScheduler;
  const ms = opts.ms ?? UNDO_MS;
  const recheck = opts.recheckMs ?? RECHECK_MS;
  let handle: unknown = null;
  let done = false;
  const fire = () => {
    handle = null;
    if (done) return;
    if (opts.isHeld()) {
      handle = s.set(fire, recheck);
      return;
    }
    done = true;
    opts.onExpire();
  };
  return {
    start() {
      if (handle !== null) s.clear(handle);
      done = false;
      handle = s.set(fire, ms);
    },
    cancel() {
      if (handle !== null) s.clear(handle);
      handle = null;
      done = true;
    },
    get done() {
      return done;
    },
  };
}
