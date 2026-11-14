import { useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { Go } from "../api";
import { errText } from "../components/useBusy";
import { mapGoTo } from "../paths";
import { clearPutbacks, getKept, keptKey } from "../kept/api";
import { Budget, CustomDateFold } from "../kept/Budget";
import { useFold } from "../kept/useFold";
import { KeptContext, type FinishOptions, type KeptCtx } from "../kept/ctx";
import { Defs } from "../kept/Defs";
import { dateWord, flatIds, moveFor, sinceLabel, splitTo, stepIndex } from "../kept/helpers";
import { behavior, nextFrame, useDealIn } from "../kept/motion";
import { SealedSlip, Slip } from "../kept/Slip";
import "../kept/kept.css";

const KEPT_PATH = "/joy/kept";

/** The id-or-selector a hash points at; the savings anchor lands on its heading. */
function hashSelector(hash: string): string {
  if (hash === "#k-savings") return "#k-savings-h";
  return /^#[\w-]+$/.test(hash) ? hash : "";
}

function focusTo(selector: string, center = false) {
  if (!selector) return;
  const el = document.querySelector<HTMLElement>(selector);
  if (!el) return;
  el.focus({ preventScroll: true });
  el.scrollIntoView({ block: center ? "center" : "nearest", behavior: behavior() });
}

export default function KeptPage() {
  const qc = useQueryClient();
  const navigate = useNavigate();
  const { hash } = useLocation();
  const q = useQuery({ queryKey: keptKey, queryFn: getKept });
  const [revealed, setRevealed] = useState<number | null>(null);
  const book = useRef<HTMLDivElement>(null);
  const handled = useRef("");
  const addFold = useFold(false, () => requestAnimationFrame(() => document.getElementById("custom-title")?.focus()));
  const { setOpen: setAddOpen } = addFold;
  const openAdd = useCallback(() => {
    setAddOpen(true);
    requestAnimationFrame(() => {
      document.getElementById("k-add")?.scrollIntoView({ block: "center", behavior: behavior() });
      document.getElementById("custom-title")?.focus({ preventScroll: true });
    });
  }, [setAddOpen]);

  // Joy's "no script" rules key off .k-js; this page is script all the way.
  useEffect(() => {
    const root = document.querySelector(".joy");
    root?.classList.add("k-js");
    return () => root?.classList.remove("k-js");
  }, []);

  const refresh = useCallback(
    async (focus?: string) => {
      await qc.invalidateQueries({ queryKey: keptKey });
      await nextFrame();
      if (focus) focusTo(focus, focus.startsWith("#k") && /^#k\d+$/.test(focus));
    },
    [qc],
  );

  const finish = useCallback(
    async (go: Go, opts: FinishOptions = {}) => {
      const dest = mapGoTo(go.to);
      const { path, hash: h } = splitTo(dest);
      if (path !== KEPT_PATH) {
        navigate(dest);
        return;
      }
      const target = opts.focus ?? hashSelector(h);
      await refresh();
      focusTo(target, /^#k\d+$/.test(target));
      opts.after?.();
    },
    [navigate, refresh],
  );

  const ctx = useMemo<KeptCtx>(
    () => ({ currency: q.data?.currency ?? "₹", finish, refresh, revealed, setRevealed }),
    [q.data?.currency, finish, refresh, revealed],
  );

  const months = q.data?.months;
  const dealt = useDealIn(book, months);

  // Landing on #k12 / #k-savings brings that into view once, after the data is there.
  useEffect(() => {
    if (!q.data || handled.current === hash) return;
    handled.current = hash;
    if (hash === "#add") {
      openAdd();
      return;
    }
    const sel = hashSelector(hash);
    if (sel && !/^#(k-)?budget$/.test(hash)) requestAnimationFrame(() => focusTo(sel, /^#k\d+$/.test(sel)));
  }, [q.data, hash, openAdd]);

  // The book is a list of slips: j/k anywhere in it, arrows and Home/End once a slip has focus.
  const ids = useMemo(() => flatIds(months ?? []), [months]);
  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.metaKey || e.ctrlKey || e.altKey || e.nativeEvent.isComposing) return;
    const t = e.target as HTMLElement;
    if (t.closest("input, textarea, select, summary, [contenteditable='true']")) return;
    const slip = t.closest<HTMLElement>(".k-slip");
    const move = moveFor(e.key, t.classList.contains("k-slip"));
    if (!move) return;
    const cur = slip ? ids.indexOf(Number(slip.id.slice(1))) : -1;
    const next = stepIndex(ids.length, cur, move);
    if (next < 0) return;
    e.preventDefault();
    focusTo(`#k${ids[next]}`, true);
  };

  if (q.isPending) return <p className="loading" role="status">Opening the keepsake box…</p>;
  if (q.isError) {
    return (
      <div className="k-page">
        <p className="kind-error" role="alert">{errText(q.error)}</p>
        <button type="button" className="btn" onClick={() => void q.refetch()}>Try again</button>
      </div>
    );
  }

  const v = q.data;
  const merged = v.months;
  const first = sinceLabel(v.months);
  const showMoney = v.kept > 0 || !!v.budget.budget;
  return (
    <KeptContext.Provider value={ctx}>
      <div className={`k-page k-js${dealt ? " m-reveal" : ""}`}>
        <Defs />

        <header className="k-head">
          <p className="k-eyebrow">The keepsake box</p>
          {v.kept > 0 && (
            <>
              <h1 className="k-title" id="k-title" tabIndex={-1}>
                <span className="k-count">{v.kept}</span> <span className="k-count-word">{dateWord(v.kept)} kept</span>
              </h1>
              {first && <p className="k-since">Together, since {first}.</p>}
              <p className="k-margin hand" aria-hidden="true">
                {v.kept === 1 ? "and so it begins" : "look at us go"}
              </p>
            </>
          )}
          {merged.length > 0 && (
          <p className="k-add">
            <button type="button" className="btn" onClick={openAdd}>
              + Add a date we had
            </button>
          </p>
          )}
          {v.kept > 0 && (
            <p className="k-year">
              <Link to="/joy/year">
                Our year so far <span aria-hidden="true">→</span>
              </Link>
              {" · "}
              <Link to="/joy/year#wall">See all our photos</Link>
              {showMoney && (
                <>
                  {" · "}
                  <a href="#k-budget">Budget and savings</a>
                </>
              )}
            </p>
          )}
          <CustomDateFold fold={addFold} />
        </header>

        {merged.length === 0 ? (
          <section className="k-empty" aria-labelledby="k-empty-h">
            <div className="k-empty-page grain">
              <span className="k-corner k-corner-tl" aria-hidden="true" />
              <span className="k-corner k-corner-tr" aria-hidden="true" />
              <span className="k-corner k-corner-bl" aria-hidden="true" />
              <span className="k-corner k-corner-br" aria-hidden="true" />
              <h1 id="k-empty-h" className="k-empty-h" tabIndex={-1}>
                Nothing kept yet.
              </h1>
              <p className="k-empty-p">The jar’s waiting. Draw a slip, say yes, and it lands here with its stamp.</p>
              <Link className="btn btn-stamp" to="/joy">
                Go to the jar
              </Link>
              <p className="k-empty-p">Already had some great nights? Add one.</p>
              <button type="button" className="btn" onClick={openAdd}>
                Add a date we had
              </button>
              <p className="k-empty-hand hand" aria-hidden="true">
                first page, saved for you
              </p>
            </div>
          </section>
        ) : (
          <>
            <p className="visually-hidden" id="k-keys">
              Press J or K to move between dates. With a date focused, the arrow keys, Home and End also move.
            </p>
            <div ref={book} className="k-book" id="k-book" role="group" aria-label="Kept dates" aria-describedby="k-keys" onKeyDown={onKey}>
              {merged.map((m, mi) => (
                <section key={m.label + mi} className="k-month" aria-labelledby={`k-m${mi + 1}`} style={{ "--m": mi } as React.CSSProperties}>
                  <h2 className="k-month-h" id={`k-m${mi + 1}`}>
                    <Link className="k-month-a" to={m.href}>
                      {m.label}
                    </Link>
                    <span className="k-month-n hand" aria-hidden="true">
                      {m.items.length} {dateWord(m.items.length)}
                    </span>
                    <Link className="k-month-open" to={m.href} tabIndex={-1} aria-hidden="true">
                      Open →
                    </Link>
                  </h2>
                  <ol className="k-spread" role="list">
                    {m.items.map((h, i) => (h.sealed ? <SealedSlip key={h.id} h={h} index={i} /> : <Slip key={h.id} h={h} index={i} />))}
                  </ol>
                </section>
              ))}
            </div>
          </>
        )}

        {v.skipped.length > 0 && (
          <aside className="k-foot" aria-label="Put back">
            <p>
              <span className="k-foot-mark" aria-hidden="true">
                *
              </span>
              {v.skipped.length === 1 ? "One slip was drawn and put back" : `${v.skipped.length} slips were drawn and put back`}, no harm done.
            </p>
            <p>
              <button
                type="button"
                className="btn"
                onClick={() =>
                  void clearPutbacks()
                    .then(() => refresh())
                    .catch(() => {})
                }
              >
                Reset: clear this list
              </button>
            </p>
            <details className="k-foot-more">
              <summary>Which {v.skipped.length === 1 ? "one" : "ones"}</summary>
              <ul role="list">
                {v.skipped.map((s, i) => (
                  <li key={i}>
                    {s.title} <span className="soft">· {s.day_label}</span>
                  </li>
                ))}
              </ul>
            </details>
          </aside>
        )}

        {showMoney && (
          <section className="k-money-sec" aria-labelledby="k-money-h">
            <h2 id="k-money-h">Money</h2>
            <Budget v={v} hash={hash} />
          </section>
        )}
      </div>
    </KeptContext.Provider>
  );
}
