import { useCallback, useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { archiveSlip, deleteSlip, getSlips, keys, restoreSlip, unvetoSlip, type ActivityView } from "../api";
import { filterGroups, slipsWord } from "../slips";
import { createUndoTimer } from "../undo";
import { ErrorState } from "../components/ErrorState";
import { errText, useBusy } from "../components/useBusy";
import { PlanDialog } from "../components/PlanDialog";

const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

interface ToastState {
  id: number;
  msg: string;
  undo?: () => void;
}

const PenIcon = () => (
  <svg className="ink-ico" width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M4.6 19.6c1.7-.3 3-.9 4.1-1.9l9.6-9.9c.9-.9.9-2.2.1-3s-2.1-.8-3 .1L5.8 14.6c-.9 1.1-1.5 2.6-1.8 4.3" />
    <path d="M13.6 6.7l3.4 3.3" />
  </svg>
);

export default function SlipsPage() {
  const qc = useQueryClient();
  const q = useQuery({ queryKey: keys.slips, queryFn: getSlips });
  const { busy, run } = useBusy();
  const [query, setQuery] = useState("");
  const [leaving, setLeaving] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [error, setError] = useState("");
  const [planFor, setPlanFor] = useState<ActivityView | null>(null);
  const [fab, setFab] = useState(false);
  const toastEl = useRef<HTMLDivElement>(null);
  const searchEl = useRef<HTMLInputElement>(null);
  const writeBtn = useRef<HTMLAnchorElement>(null);
  const focusAfter = useRef<{ id: number; kind: "slip" } | null>(null);
  const planOpener = useRef<HTMLElement | null>(null);
  const toastId = useRef(0);

  const refresh = useCallback(() => qc.invalidateQueries({ queryKey: keys.slips }), [qc]);

  const show = useCallback((msg: string, undo?: () => void) => {
    toastId.current += 1;
    setToast({ id: toastId.current, msg, undo });
  }, []);

  // The toast lasts 6.5 s; hover or focus on it pauses the clock.
  useEffect(() => {
    if (!toast) return;
    const t = createUndoTimer({
      onExpire: () => setToast((cur) => (cur && cur.id === toast.id ? null : cur)),
      isHeld: () => {
        const el = toastEl.current;
        return !!el && (el.contains(document.activeElement) || el.matches(":hover"));
      },
    });
    t.start();
    return () => t.cancel();
  }, [toast]);

  // The toast is a polite live region: it is announced, never focused. Undo stays a normal
  // tab stop, and the clock above pauses while focus is inside it.

  // After a list change, put focus back on the slip that returned.
  useEffect(() => {
    const f = focusAfter.current;
    if (!f || !q.data) return;
    const el = document.querySelector<HTMLElement>(`.slip[data-id="${f.id}"] .slip-link`);
    if (el) {
      focusAfter.current = null;
      el.focus({ preventScroll: true });
      el.scrollIntoView({ block: "center", behavior: reduced() ? "auto" : "smooth" });
    }
  }, [q.data]);

  // "/" jumps to the search box unless focus is already in a field.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "/" || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey || e.isComposing) return;
      const t = e.target;
      if (t instanceof HTMLElement && (t.matches("input, textarea, select") || t.isContentEditable)) return;
      if (!searchEl.current) return;
      e.preventDefault();
      searchEl.current.focus({ preventScroll: true });
    };
    addEventListener("keydown", onKey);
    return () => removeEventListener("keydown", onKey);
  }, []);

  // The floating "Write a slip" once the header button has scrolled away.
  const hasList = !!q.data && q.data.total > 0;
  useEffect(() => {
    const el = writeBtn.current;
    if (!el || typeof IntersectionObserver === "undefined") return;
    const io = new IntersectionObserver(([e]) => setFab(!e.isIntersecting && e.boundingClientRect.bottom < 0));
    io.observe(el);
    return () => io.disconnect();
  }, [hasList]);

  const restore = (a: ActivityView, say = true) =>
    run(async () => {
      setError("");
      try {
        await restoreSlip(a.id);
        focusAfter.current = { id: a.id, kind: "slip" };
        await refresh();
        if (say) show(`“${a.title}” is back in the jar.`);
      } catch (e) {
        setError(errText(e));
      }
    });

  const takeOut = (a: ActivityView) =>
    run(async () => {
      setError("");
      try {
        if (!reduced()) setLeaving(a.id);
        await Promise.all([archiveSlip(a.id), wait(reduced() ? 0 : 220)]);
        await refresh();
        show(`Took out “${a.title}”. It’s in the drawer.`, () => void restore(a));
      } catch (e) {
        setError(errText(e));
      } finally {
        setLeaving(null);
      }
    });

  const simple = (fn: () => Promise<unknown>) =>
    run(async () => {
      setError("");
      try {
        await fn();
        await refresh();
      } catch (e) {
        setError(errText(e));
      }
    });

  if (q.isPending) return <p className="loading" role="status">Opening the slips…</p>;
  if (q.isError) {
    return (
      <div className="slips-page">
        <ErrorState message={errText(q.error)} onRetry={() => void q.refetch()} />
      </div>
    );
  }

  const { groups, archived, total } = q.data;
  const searching = query.trim() !== "";
  const { shown, total: matched } = filterGroups(groups, query);
  const status = !searching ? "" : matched === 0 ? `Nothing in the jar matches “${query.trim()}”.` : matched === 1 ? "1 slip matches" : `${matched} slips match`;

  return (
    <div className={`can-plan${toast ? " has-toast" : ""}`}>
      <div className={`slips-page${searching ? " searching" : ""}`} data-total={total}>
        <header className="slips-head">
          <div className="slips-title">
            <h1>Slips</h1>
            {total > 0 && (
              <p className="slips-count">
                <span className="hand-note" aria-hidden="true">{total} in the jar</span>
                <span className="visually-hidden">{total} slips in the jar</span>
              </p>
            )}
          </div>
          {total > 0 && (
            <Link ref={writeBtn} className="btn btn-stamp write-btn" to="/joy/slips/new">
              <PenIcon />
              Write a slip
            </Link>
          )}
        </header>

        {error && (
          <p className="kind-error" role="alert">
            {error}
          </p>
        )}

        {total === 0 ? (
          <section className="empty-jar" aria-labelledby="empty-h">
            <svg className="empty-jar-art" viewBox="0 0 200 220" aria-hidden="true">
              <path className="lid" d="M62 30.5c24-2.2 52-2 76 .4 2.2.3 3.4 1.6 3.4 3.8v10.2c0 2-1.3 3.2-3.4 3.3-25 1.4-51 1.5-76-.2-2.1-.1-3.4-1.4-3.4-3.4V34.2c0-2.1 1.3-3.4 3.4-3.7z" />
              <path className="glass" d="M66 50c-3 8-11 12-19 19-9 8-12 19-12 33v72c0 17 10 27 27 28 26 1.6 50 1.7 76 .2 17-1 27-11 27-28v-72c0-14-3-25-12-33-8-7-16-11-19-19" />
              <path className="shine" d="M52 104c0-10 3-17 9-22M50 122v30" />
              <path className="floor" d="M48 186c30 3.2 74 3.4 104 .3" />
              <path className="lone-slip" d="M118 170.5l22-6.2 3.4 11.6-22 6.4z" />
            </svg>
            <h2 id="empty-h">The jar is empty.</h2>
            <p className="soft">
              Every date starts as a little idea. Write one down and drop it in.
              {archived.length > 0 && " Or put one back from the drawer below."}
            </p>
            <Link className="btn btn-stamp" to="/joy/slips/new">Write the first slip</Link>
          </section>
        ) : (
          <>
            <nav className="group-index" aria-label="Jump to a group">
              <ul>
                {shown.map(({ group, items }) => (
                  <li key={group.key}>
                    <a href={`#g-${group.key}`} className={`tint-${group.key}`}>
                      <span className="dot" aria-hidden="true"></span>
                      {group.label}{" "}
                      <span className="n">{searching ? `${items.length} of ${group.items.length}` : group.items.length}</span>
                    </a>
                  </li>
                ))}
              </ul>
            </nav>

            <div className="slip-search" role="search">
              <label htmlFor="slip-q" className="visually-hidden">Search slips</label>
              <svg className="search-ico" width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="10.6" cy="10.4" r="6.4" />
                <path d="M15.4 15.3l4.8 4.9" />
              </svg>
              <input
                ref={searchEl}
                id="slip-q"
                className="search-input"
                type="search"
                placeholder="Search slips, places, tags…"
                autoComplete="off"
                autoCapitalize="off"
                spellCheck={false}
                enterKeyHint="search"
                aria-describedby="search-status"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Escape" && query) {
                    e.preventDefault();
                    setQuery("");
                  }
                }}
              />
              <p className="search-status soft" id="search-status" aria-live="polite">{status}</p>
            </div>

            {!searching && (
              <p className="slips-hint soft">
                Tap a slip to change it. <span className="nowrap">“Plan it”</span> puts it on a day, <span className="nowrap">“Take out”</span> sets one aside in the drawer below.
              </p>
            )}

            {shown.map(({ group, items }) => (
              <section key={group.key} className={`group tint-${group.key}`} id={`g-${group.key}`} aria-labelledby={`h-${group.key}`}>
                <h2 className="group-h" id={`h-${group.key}`}>
                  <span className="group-label">{group.label}</span>{" "}
                  <span className="group-n">
                    {searching
                      ? `${items.length} of ${group.items.length} ${slipsWord(group.items.length)}`
                      : `${group.items.length} ${slipsWord(group.items.length)}`}
                  </span>
                </h2>
                <ul
                  className="stack"
                  role="list"
                  style={{ containIntrinsicSize: `auto ${group.items.length * (group.key === "home" || group.key === "apart" ? 104 : 138) + 32}px` }}
                >
                  {items.map((a, i) => (
                    <li
                      key={a.id}
                      className={`slip grain${leaving === a.id ? " leaving" : ""}`}
                      style={{ "--i": i } as React.CSSProperties}
                      data-id={a.id}
                    >
                      <span className="fold" aria-hidden="true"></span>
                      <Link className="slip-link" to={`/joy/slips/${a.id}`}>
                        <span className="slip-title">{a.title}</span>
                      </Link>
                      <p className="slip-meta">
                        <span>{a.cost}</span>
                        <span className="sep" aria-hidden="true"> · </span>
                        <span>{a.duration}</span>
                        {a.times_done > 0 && (
                          <span className="done">
                            <svg width="12" height="12" viewBox="0 0 24 24" aria-hidden="true">
                              <path d="M12 20.3c-4.6-3.2-8.3-6.4-8.3-10.4 0-2.6 2-4.6 4.4-4.6 1.6 0 3 .9 3.9 2.2.9-1.3 2.3-2.2 3.9-2.2 2.4 0 4.4 2 4.4 4.6 0 4-3.7 7.2-8.3 10.4z" />
                            </svg>
                            {a.times_done === 1 ? "done once" : `done ${a.times_done}×`}
                          </span>
                        )}
                      </p>
                      {a.closed_label && <p className="slip-closed">{a.closed_label}</p>}
                      {a.vetoed_by && (
                        <div className="slip-veto">
                          <span>{a.vetoed_by} passed on this, so it’s out of the draw.</span>
                          <button type="button" className="takeout-btn" disabled={busy} onClick={() => void simple(() => unvetoSlip(a.id))}>
                            Bring it back<span className="visually-hidden">: {a.title}</span>
                          </button>
                        </div>
                      )}
                      {(a.generated || a.pairs.length > 0) && (
                        <p className="slip-extra">
                          {a.generated && (
                            <span className="gen-tag">
                              <svg viewBox="0 0 16 16" aria-hidden="true">
                                <path d="M8 1.2l1.5 4.3 4.3 1.5-4.3 1.5L8 12.8 6.5 8.5 2.2 7l4.3-1.5z" />
                              </svg>
                              generated for you
                            </span>
                          )}
                          {a.pairs.length > 0 && <span className="slip-pairs">pairs with {a.pairs.join(", ")}</span>}
                        </p>
                      )}
                      {a.distances.length > 0 && (
                        <p className="slip-far">
                          <svg width="12" height="12" viewBox="0 0 24 24" aria-hidden="true">
                            <path d="M12 21.2c-3.6-4.3-6.4-7.9-6.4-11.4a6.4 6.4 0 0 1 12.8 0c0 3.5-2.8 7.1-6.4 11.4z" />
                            <circle cx="12" cy="9.8" r="2.2" />
                          </svg>
                          {a.distances.join(" · ")}
                        </p>
                      )}
                      <div className="slip-acts">
                        <button
                          type="button"
                          className="plan-btn"
                          aria-label={`Plan “${a.title}” for a day`}
                          onClick={(e) => {
                            planOpener.current = e.currentTarget;
                            setPlanFor(a);
                          }}
                        >
                          Plan it
                        </button>
                        <button type="button" className="takeout-btn" disabled={busy} aria-label={`Take out “${a.title}”`} onClick={() => void takeOut(a)}>
                          Take out
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              </section>
            ))}
          </>
        )}

        {archived.length > 0 && (
          <details className="drawer" id="drawer">
            <summary>
              <svg className="drawer-ico" width="22" height="22" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M3.6 9.4h16.8l-1.2 10.1c-.1.7-.7 1.2-1.4 1.2H6.2c-.7 0-1.3-.5-1.4-1.2z" />
                <path d="M5 9.2 6.8 4.4c.2-.5.7-.8 1.2-.8h8c.5 0 1 .3 1.2.8L19 9.2" />
                <path d="M9.4 13.3c1.7.8 3.5.8 5.2 0" />
              </svg>
              <span>The drawer</span> <span className="drawer-n">{archived.length} taken out</span>
              <svg className="chev" width="16" height="16" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M6.5 9.5 12 15l5.5-5.5" />
              </svg>
            </summary>
            <p className="drawer-note soft">Slips set aside. They won’t be drawn until you put them back.</p>
            <ul className="drawer-list" role="list">
              {archived.map((a) => (
                <DrawerSlip key={a.id} a={a} busy={busy} onRestore={() => void restore(a)} onDelete={() => void simple(() => deleteSlip(a.id))} />
              ))}
            </ul>
          </details>
        )}
      </div>

      {total > 0 && (
        <Link className={`btn btn-stamp fab${fab ? " show" : ""}`} to="/joy/slips/new" tabIndex={-1} aria-hidden="true">
          <PenIcon />
          Write a slip
        </Link>
      )}

      <div className="toast-region" role="status" aria-live="polite" id="toast">
        {toast && (
          <div className="toast" ref={toastEl} key={toast.id}>
            <span className="toast-msg">{toast.msg}</span>
            {toast.undo && (
              <button
                type="button"
                disabled={busy}
                onClick={() => {
                  const u = toast.undo;
                  setToast(null);
                  u?.();
                }}
              >
                Undo
              </button>
            )}
          </div>
        )}
      </div>

      <PlanDialog
        slip={planFor}
        onClose={(planned) => {
          setPlanFor(null);
          if (planned) void refresh();
          planOpener.current?.focus({ preventScroll: true });
        }}
      />
    </div>
  );
}

function DrawerSlip({ a, busy, onRestore, onDelete }: { a: ActivityView; busy: boolean; onRestore: () => void; onDelete: () => void }) {
  const [confirming, setConfirming] = useState(false);
  return (
    <li className={`drawer-slip tint-${a.setting_key}`} data-id={a.id}>
      <span className="drawer-tab" aria-hidden="true"></span>
      <span className="drawer-title">{a.title}</span>
      <div className="drawer-actions">
        <button type="button" className="btn btn-small" disabled={busy} aria-label={`Put “${a.title}” back in the jar`} onClick={onRestore}>
          Put back
        </button>
        <details className="del" open={confirming} onToggle={(e) => setConfirming(e.currentTarget.open)}>
          <summary className="del-sum">
            <span className="when-closed">Delete…<span className="visually-hidden"> “{a.title}”</span></span>
            <span className="when-open">Keep it</span>
          </summary>
          <div className="del-form">
            <span>Gone for good?</span>
            <button type="button" className="del-yes" disabled={busy} onClick={onDelete}>
              Delete forever
            </button>
          </div>
        </details>
      </div>
    </li>
  );
}
