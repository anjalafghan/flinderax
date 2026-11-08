import { useEffect, useRef } from "react";
import { Link, Navigate, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { getMonth, yearKeys, type MonthView, type Nav } from "../year/api";
import { pressStickers, turnIn, turnOut, type Turn } from "../year/motion";
import { ApiError } from "../api";
import { errText } from "../components/useBusy";

const plural = (n: number, one: string, many: string) => (n === 1 ? one : many);
const stars = (n: number) => "★".repeat(Math.max(0, n));

// Which way the last sheet turned, so the next month can swing in from the other side.
let pendingTurn: Turn | null = null;

export default function MonthPage() {
  const { y = "", m = "" } = useParams();
  const q = useQuery({ queryKey: yearKeys.month(y, m), queryFn: () => getMonth(y, m), retry: false });
  if (q.isPending) return <p className="loading" role="status">Turning to the page…</p>;
  if (q.isError) {
    if (q.error instanceof ApiError && q.error.status === 404) return <Navigate to={`/joy/year/${y}`} replace />;
    return (
      <div>
        <p className="kind-error" role="alert">{errText(q.error)}</p>
        <button type="button" className="btn" onClick={() => void q.refetch()}>Try again</button>
      </div>
    );
  }
  return <Month key={`${y}/${m}`} v={q.data} />;
}

function Month({ v }: { v: MonthView }) {
  const { year, month_name: name, entries, others, nights, best } = v;
  const sheet = useRef<HTMLDivElement>(null);
  const head = useRef<HTMLElement>(null);
  const turning = useRef(false);
  const navigate = useNavigate();

  useEffect(() => {
    if (head.current) pressStickers(head.current);
    if (sheet.current && pendingTurn) turnIn(sheet.current, pendingTurn);
    pendingTurn = null;
  }, []);

  const turn = (e: React.MouseEvent<HTMLAnchorElement>, to: Nav, dir: Turn) => {
    // let new-tab / modified clicks behave like links
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    if (turning.current) return;
    turning.current = true;
    pendingTurn = dir;
    const done = () => {
      turning.current = false;
      navigate(to.href);
    };
    const el = sheet.current;
    if (!el) return done();
    // always navigate, even if the animation never finishes
    let went = false;
    const go = () => {
      if (!went) {
        went = true;
        done();
      }
    };
    setTimeout(go, 600);
    void turnOut(el, dir).then(go);
  };

  const empty = entries.length === 0 && others.length === 0 && nights.length === 0;

  return (
    <>
      <svg className="m-defs" width="0" height="0" aria-hidden="true" focusable="false">
        <defs>
          <symbol id="m-star" viewBox="0 0 24 24"><path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3 6.1 20.6l1.3-6.6L2.5 9.4l6.6-.8z" /></symbol>
          <symbol id="m-arrow" viewBox="0 0 40 44"><path d="M33 41C34 28 26 16 9 9M9 9l9-1M9 9l3 8" /></symbol>
        </defs>
      </svg>

      <nav className="m-top" aria-label="Back to the year">
        <Link className="m-back" to={`/joy/year/${year}`}><span aria-hidden="true">←</span> Our {year}</Link>
      </nav>

      <header className="m-head" ref={head}>
        <div className="m-shadow m-patch-wrap">
          <div className="m-patch" style={{ "--n": v.month_letters.length } as React.CSSProperties}>
            <h1 className="visually-hidden">Our {name} {year}</h1>
            <p className="m-stickers" aria-hidden="true">{v.month_letters.map((c, i) => <span key={i}>{c}</span>)}</p>
            <p className="m-stickers m-stickers-year" aria-hidden="true">{v.year_letters.map((c, i) => <span key={i}>{c}</span>)}</p>
          </div>
        </div>
        {entries.length > 0 && (
          <p className="m-summary">
            {entries.length} {plural(entries.length, "date", "dates")}
            {v.spent && ` · ${v.spent} spent`}
            {v.avg_rating && (
              <> · {v.avg_rating} <span className="m-star-s" aria-hidden="true">★</span><span className="visually-hidden">stars</span> on average</>
            )}
          </p>
        )}
        {v.to_come > 0 && <p className="m-soon">…and {v.to_come} more still to come this month</p>}
      </header>

      <div className="m-sheet" ref={sheet}>
        {empty ? (
          <div className="m-empty">
            <p className="m-scribble">nothing kept in {name} {v.to_come > 0 && "yet"} ♡</p>
            <p className="m-empty-p">This page fills up with every date we keep in {name}: a print or a ticket for each, and the receipts.</p>
          </div>
        ) : (
          <div className="m-collage">
            {best && entries.length > 1 && (
              <div className="m-piece m-sticker-wrap">
                <div className="m-sticker">
                  <svg className="m-sticker-arc" viewBox="0 0 240 60" aria-hidden="true">
                    <defs><path id="m-arc" d="M20,56 Q120,-6 220,56" /></defs>
                    <text><textPath href="#m-arc" startOffset="50%" textAnchor="middle">★ OUR BEST NIGHT ★</textPath></text>
                  </svg>
                  <p className="m-sticker-title">{best.title}</p>
                  <p className="m-sticker-stars" role="img" aria-label={`Our best night: ${best.rating} out of 5 stars`}>
                    {[1, 2, 3, 4, 5].map((n) => <svg key={n} className={n <= best.rating ? "is-on" : ""} aria-hidden="true"><use href="#m-star" /></svg>)}
                  </p>
                  <p className="m-sticker-day">{best.day} {year}</p>
                </div>
              </div>
            )}

            {entries.map((e, i) => (
              <article className="m-piece m-entry" key={i} style={{ "--t": e.tilt } as React.CSSProperties} aria-label={`${e.title}, ${e.day}`}>
                {e.photo ? (
                  <figure className="m-print">
                    <span className="m-tape" aria-hidden="true"></span>
                    <span className="m-print-stamp" aria-hidden="true">{e.stamp}</span>
                    <span className="m-print-photo"><img src={e.photo} alt={`Photo from ${e.title}`} loading="lazy" decoding="async" width={480} height={600} /></span>
                    <figcaption>
                      {e.title}
                      {e.rating > 0 && <> <span className="m-print-stars" role="img" aria-label={`${e.rating} out of 5 stars`}>{stars(e.rating)}</span></>}
                    </figcaption>
                  </figure>
                ) : (
                  <div className="m-shadow">
                    <div className="m-ticket" data-kind={e.kind}>
                      <p className="m-ticket-brand">
                        <svg viewBox="0 0 24 14" aria-hidden="true"><path d="M2 13a10 10 0 0 1 20 0M12 1v3M4.5 4.5l2 2M19.5 4.5l-2 2" /></svg>
                        <span>{e.kind_label}</span>
                      </p>
                      <div className="m-ticket-body">
                        <div className="m-ticket-text">
                          <p className="m-ticket-title">{e.title}</p>
                          <dl className="m-ticket-meta">
                            {e.place && <div><dt>Ticket issued at:</dt><dd>{e.place}</dd></div>}
                            <div><dt>Date:</dt><dd>{e.day}</dd></div>
                            {e.rating > 0 && <div><dt>Stars:</dt><dd><span aria-hidden="true">{stars(e.rating)}</span><span className="visually-hidden">{e.rating} out of 5</span></dd></div>}
                          </dl>
                        </div>
                        <svg className="m-qr" viewBox="0 0 21 21" aria-hidden="true"><path d="M0 0h7v7H0zM1 1v5h5V1zM2 2h3v3H2zM14 0h7v7h-7zM15 1v5h5V1zM16 2h3v3h-3zM0 14h7v7H0zM1 15v5h5v-5zM2 16h3v3H2zM8 0h2v2H8zM11 1h2v3h-2zM8 3h2v2H8zM9 6h3v2H9zM8 9h4v2H8zM0 8h2v2H0zM3 9h3v2H3zM13 8h3v2h-3zM17 9h4v2h-4zM14 12h2v3h-2zM17 12h2v2h-2zM8 13h3v2H8zM12 15h2v2h-2zM16 16h3v2h-3zM9 17h2v4H9zM12 19h3v2h-3zM19 19h2v2h-2zM18 14h3v1h-3z" /></svg>
                      </div>
                    </div>
                  </div>
                )}

                {e.actual && (
                  <div className="m-shadow m-receipt-wrap">
                    <div className="m-receipt">
                      <p className="m-r-head">{e.place || e.title}</p>
                      <p className="m-r-sub">{e.day} {year} · no. {i + 1}</p>
                      <p className="m-r-line"><span>Our guess</span><span>{e.est}</span></p>
                      {e.cabs && <p className="m-r-line"><span>Cabs</span><span>{e.cabs}</span></p>}
                      <p className="m-r-total"><span>Amount</span><span>{e.actual}</span></p>
                      {e.diff && <p className={`m-r-foot${e.over ? " is-over" : ""}`}>{e.diff}</p>}
                    </div>
                  </div>
                )}

                {e.note && (
                  <p className="m-scribble m-note">
                    <svg className="m-arrow" aria-hidden="true"><use href="#m-arrow" /></svg>
                    {e.note}
                  </p>
                )}
              </article>
            ))}

            {nights.map((g) => (
              <div className="m-piece m-nights" key={g.href}>
                <p className="m-scribble"><Link to={g.href}>{g.title}: {g.count} {plural(g.count, "night", "nights")} · {g.total}</Link></p>
                <ul className="reg-strip">
                  {g.nights.map((n, i) => (
                    <li key={i} className={`reg-strip-n${n.photo ? " has-photo" : ""}`} title={n.day}>
                      {n.photo ? <img src={n.photo} alt={`${g.title}, ${n.day}`} loading="lazy" width={64} height={80} /> : <span className="reg-ink">{n.day}</span>}
                    </li>
                  ))}
                </ul>
              </div>
            ))}

            {others.map((o, i) => (
              <div className="m-piece m-other" key={i}>
                <div className="m-shadow m-receipt-wrap">
                  <div className="m-receipt">
                    <p className="m-r-head">{o.note || "Somewhere we went anyway"}</p>
                    <p className="m-r-sub">{o.day} {year} · not a slip</p>
                    <p className="m-r-total"><span>Amount</span><span>{o.amount}</span></p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {(v.prev || v.next) && (
          <nav className="m-pages" aria-label="Other months">
            {v.prev && <Link className="m-page-a m-prev" to={v.prev.href} onClick={(e) => v.prev && turn(e, v.prev, "prev")}><span aria-hidden="true">←</span> {v.prev.label}</Link>}
            {v.next && <Link className="m-page-a m-next" to={v.next.href} onClick={(e) => v.next && turn(e, v.next, "next")}>{v.next.label} <span aria-hidden="true">→</span></Link>}
          </nav>
        )}
      </div>
    </>
  );
}
