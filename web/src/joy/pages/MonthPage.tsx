import { useEffect, useRef } from "react";
import { Link, Navigate, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { getMonth, yearKeys, type MonthView, type Nav } from "../year/api";
import { pressStickers, turnIn, turnOut, type Turn } from "../year/motion";
import { ApiError } from "../api";
import { errText } from "../components/useBusy";
import { Keepsake, KeepsakeDefs } from "../keepsakes/Keepsake";
import { ExhibitButton } from "../keepsakes/Exhibit";
import { specFor } from "../keepsakes/find";
import { SceneButton, TuckedPrint } from "../keepsakes/Scene";
import { sceneFor } from "../keepsakes/scene-find";
import { keepsakeFor } from "../keepsakes/pick";

const plural = (n: number, one: string, many: string) => (n === 1 ? one : many);
/** Which roll of tape a piece gets: four rolls in turn, so neighbours never match. */
const roll = (i: number) => String(i % 4);

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
  const hasBest = best && entries.length > 1;
  // settle-in order: the sticker first, then entries, the list, nights, others and the keepsakes
  const first = hasBest ? 1 : 0;
  const at = (i: number) => ({ "--i": first + i }) as React.CSSProperties;
  const afterEntries = entries.length + 1;

  return (
    <>
      <MonthDefs />
      <KeepsakeDefs />

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
        <div className="m-wrap" aria-hidden="true"></div>
        <div className="m-leaf" aria-hidden="true"></div>
        {empty ? (
          <div className="m-empty">
            <p className="m-scribble">nothing kept in {name} {v.to_come > 0 && "yet"} ♡</p>
            <p className="m-empty-p">This page fills up with every date we keep in {name}: a print or a ticket for each, and the receipts.</p>
            <p className="m-empty-p"><Link to="/joy/kept#add">Add a date we had</Link></p>
          </div>
        ) : (
          <div className="m-collage" data-snug={entries.length >= 5 ? "" : undefined}>
            {hasBest && (
              <div className="m-piece m-sticker-wrap" style={{ "--i": 0 } as React.CSSProperties}>
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

            {entries.map((e, i) => {
              const scene = sceneFor(e.slip_key, e.visit);
              return scene ? (
              <article
                className="m-piece m-entry m-exhibit"
                key={i}
                data-photo={e.photo ? "" : undefined}
                style={{ "--t": e.tilt, ...at(i) } as React.CSSProperties}
                aria-label={`${e.title}, ${e.day}`}
              >
                <div className="m-ex">
                  {e.photo && <TuckedPrint className="m-ex-print" src={e.photo} alt={`Our photo from ${e.title}, ${e.day}`} />}
                  <SceneButton className="m-ex-obj" scene={scene} what={`${e.title}, ${e.day}`} />
                  <p className="m-ex-tag" aria-hidden="true">{e.title} <span>{e.day}</span></p>
                </div>
                {e.actual && (
                  <div className="m-shadow m-receipt-wrap">
                    <div className="m-receipt">
                      <p className="m-r-head">{e.place || e.title}</p>
                      <p className="m-r-sub">{e.day} {year} · no. {i + 1}</p>
                      <p className="m-r-line"><span>Our guess</span><span>{e.est}</span></p>
                      {e.cabs && <p className="m-r-line"><span>Cabs</span><span>{e.cabs}</span></p>}
                      <p className="m-r-total"><span>Amount</span><span>{e.actual}</span></p>
                      {e.diff && <p className={`m-r-foot${e.over ? " is-over" : ""}`}>{e.diff}</p>}
                      <Stamp day={e.day} />
                    </div>
                  </div>
                )}
              </article>
              ) : (
              <article
                className="m-piece m-entry"
                key={i}
                data-photo={e.photo ? "" : undefined}
                style={{ "--t": e.tilt, ...at(i) } as React.CSSProperties}
                aria-label={`${e.title}, ${e.day}`}
              >
                {e.photo && (
                  <figure className="m-print">
                    <span className="m-tape" data-roll={roll(i)} aria-hidden="true"></span>
                    <span className="m-print-photo"><img src={e.photo} alt={`Photo from ${e.title}`} loading="lazy" decoding="async" width={480} height={600} /></span>
                    <figcaption>{e.title}</figcaption>
                    <span className="m-print-stamp" aria-hidden="true">{e.stamp}</span>
                  </figure>
                )}
                <div className="m-keep">
                  {!e.photo && <span className="m-tape m-tape-corner" data-roll={roll(i + 1)} aria-hidden="true"></span>}
                  {(() => {
                    const spec = specFor(e.slip_key, e.visit);
                    return spec ? <ExhibitButton spec={spec} d={e} /> : <Keepsake d={e} no={i + 1} kind={keepsakeFor(e.kind, e.tags)} />;
                  })()}
                </div>

                {e.actual && (
                  <div className="m-shadow m-receipt-wrap">
                    <div className="m-receipt">
                      <p className="m-r-head">{e.place || e.title}</p>
                      <p className="m-r-sub">{e.day} {year} · no. {i + 1}</p>
                      <p className="m-r-line"><span>Our guess</span><span>{e.est}</span></p>
                      {e.cabs && <p className="m-r-line"><span>Cabs</span><span>{e.cabs}</span></p>}
                      <p className="m-r-total">
                        <span>Amount</span><span>{e.actual}</span>
                        <svg className="m-ring" viewBox="0 0 90 34" preserveAspectRatio="none" aria-hidden="true"><path d="M10 20C12 8 52 3 76 9s10 20-20 22S4 30 10 15" /></svg>
                      </p>
                      {e.diff && <p className={`m-r-foot${e.over ? " is-over" : ""}`}>{e.diff}</p>}
                      <svg className="m-bar" viewBox="0 0 119 30" aria-hidden="true"><use href="#m-bar" /></svg>
                      <Stamp day={e.day} />
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
              );
            })}

            {entries.length > 0 && (
              <div className="m-piece m-list-wrap" style={at(entries.length)}>
                <div className="m-list">
                  <span className="m-tape" data-roll="2" aria-hidden="true"></span>
                  <svg className="m-list-edge" viewBox="0 0 300 398" preserveAspectRatio="none" aria-hidden="true"><use href="#m-wave" /></svg>
                  <svg className="m-list-swirls" viewBox="0 0 268 140" preserveAspectRatio="xMidYMid meet" aria-hidden="true"><use href="#m-swirls" /></svg>
                  <h2>Our {name}</h2>
                  <ul>
                    {entries.map((e, i) => <li key={i}>{e.day} ~ {e.title}</li>)}
                    {v.to_come > 0 && <><li className="is-blank" aria-hidden="true">~</li><li className="is-blank" aria-hidden="true">~</li></>}
                  </ul>
                </div>
              </div>
            )}

            {nights.map((g, gi) => (
              <div className="m-piece m-nights" key={g.href} style={at(afterEntries + gi)}>
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
              <div className="m-piece m-other" key={i} style={at(afterEntries + nights.length + i)}>
                <div className="m-shadow m-receipt-wrap">
                  <div className="m-receipt">
                    <p className="m-r-head">{o.note || "Somewhere we went anyway"}</p>
                    <p className="m-r-sub">{o.day} {year} · not a slip</p>
                    <p className="m-r-total"><span>Amount</span><span>{o.amount}</span></p>
                  </div>
                </div>
              </div>
            ))}

            <div className="m-piece m-keepsakes" aria-hidden="true" style={at(afterEntries + nights.length + others.length)}>
              <svg className="m-pin" viewBox="0 0 78 78"><use href="#m-pin" /></svg>
            </div>
          </div>
        )}
      </div>

      {(v.prev || v.next) && (
        <nav className="m-pages" aria-label="Other months">
          {v.prev && <Link className="m-page-a m-prev" to={v.prev.href} onClick={(e) => v.prev && turn(e, v.prev, "prev")}><span aria-hidden="true">←</span> {v.prev.label}</Link>}
          {v.next && <Link className="m-page-a m-next" to={v.next.href} onClick={(e) => v.next && turn(e, v.next, "next")}>{v.next.label} <span aria-hidden="true">→</span></Link>}
        </nav>
      )}
    </>
  );
}

/** The IT'S A DATE rubber stamp, inked unevenly, with the day in the middle. */
function Stamp({ day }: { day: string }) {
  return (
    <svg className="m-stamp" viewBox="0 0 100 100" aria-hidden="true">
      <g filter="url(#m-wear)">
        <circle cx="50" cy="50" r="46" fill="none" stroke="#b9301b" strokeWidth="3" />
        <circle cx="50" cy="50" r="31" fill="none" stroke="#b9301b" strokeWidth="1.5" />
        <text fontSize="9.5" letterSpacing="2.1"><textPath href="#m-ring-path">★ IT&apos;S A DATE ★ IT&apos;S A DATE</textPath></text>
        <text className="m-stamp-day" x="50" y="51" textAnchor="middle" fontSize="15">{day.toUpperCase()}</text>
        <text x="50" y="65" textAnchor="middle" fontSize="9" letterSpacing="2">KEPT</text>
      </g>
    </svg>
  );
}

const STAR = "M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3 6.1 20.6l1.3-6.6L2.5 9.4l6.6-.8z";
const BAR =
  "M0 0h1v30H0zM3 0h1v30H3zM6 0h2v30H6zM9 0h1v30H9zM11 0h1v30h-1zM14 0h2v30h-2zM18 0h1v30h-1zM20 0h2v30h-2zM25 0h1v30h-1zM28 0h1v30h-1zM30 0h1v30h-1zM32 0h1v30h-1zM35 0h1v30h-1zM37 0h1v30h-1zM40 0h1v30h-1zM43 0h1v30h-1zM47 0h1v30h-1z" +
  "M50 0h2v30h-2zM53 0h1v30h-1zM55 0h2v30h-2zM59 0h1v30h-1zM63 0h1v30h-1zM65 0h1v30h-1zM69 0h3v30h-3zM74 0h2v30h-2zM77 0h2v30h-2zM81 0h2v30h-2zM85 0h2v30h-2zM89 0h1v30h-1zM91 0h1v30h-1zM94 0h1v30h-1zM96 0h1v30h-1zM99 0h3v30h-3zM104 0h2v30h-2zM109 0h3v30h-3zM114 0h3v30h-3z";
/** A hand-drawn wave along each edge of the list card (300 × 398). */
const WAVE = [
  "M12 12q5.5-3.5 11 0" + "t11 0".repeat(24),
  "M288 12q3.5 5.5 0 11" + "t0 11".repeat(33),
  "M288 386q-5.5 3.5-11 0" + "t-11 0".repeat(24),
  "M12 386q-3.5-5.5 0-11" + "t0-11".repeat(33),
].join("");

/** Shared drawings for the page: stars, the pen arrow, the stub's code, the stamp's ink, the list card and the pin. */
function MonthDefs() {
  return (
    <svg className="m-defs" width="0" height="0" aria-hidden="true" focusable="false">
      <defs>
        <symbol id="m-star" viewBox="0 0 24 24"><path d={STAR} /></symbol>
        <symbol id="m-arrow" viewBox="0 0 40 44"><path d="M33 41C34 28 26 16 9 9M9 9l9-1M9 9l3 8" /></symbol>
        <symbol id="m-bar" viewBox="0 0 119 30"><path d={BAR} /></symbol>
        <symbol id="m-wave" viewBox="0 0 300 398" preserveAspectRatio="none"><path d={WAVE} vectorEffect="non-scaling-stroke" /></symbol>
        <symbol id="m-swirls" viewBox="0 0 268 140">
          <g fill="none" stroke="#2b59a6" strokeWidth="2.2" strokeDasharray="5 4" strokeLinecap="round">
            <path d="M62 76a5 5 0 1 1 5 5a10 10 0 1 1-10-10a15 15 0 1 1 15 15a21 21 0 1 1-21-21a27 27 0 1 1 27 27" />
            <path d="M160 58a5 5 0 1 1 5 5a10 10 0 1 1-10-10a15 15 0 1 1 15 15a21 21 0 1 1-21-21" />
            <path d="M206 112a4 4 0 1 1 4 4a8 8 0 1 1-8-8a13 13 0 1 1 13 13" />
          </g>
          <path fill="none" stroke="#e3b52b" strokeWidth="2.2" strokeDasharray="3 5" strokeLinecap="round" d="M42 112c20 14 50 18 74 6M104 36c18-8 40-8 56 4M180 92c16 6 30 4 48-6" />
          <g fill="#e9b928">
            <path d={STAR} transform="translate(20 24) scale(.66)" />
            <path d={STAR} transform="translate(232 40) scale(.58)" />
            <path d={STAR} transform="translate(96 88) scale(.46)" />
            <path d={STAR} transform="translate(238 118) scale(.6)" />
          </g>
        </symbol>
        <path id="m-ring-path" d="M50 50 m-38.5 0 a38.5 38.5 0 1 1 77 0 a38.5 38.5 0 1 1 -77 0" />
        <filter id="m-wear" x="-5%" y="-5%" width="110%" height="110%">
          <feTurbulence type="fractalNoise" baseFrequency="1.15" numOctaves="2" seed="7" result="n" />
          <feColorMatrix in="n" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  -1.9 0 0 0 1.6" result="m" />
          <feComposite in="SourceGraphic" in2="m" operator="in" />
        </filter>
        <linearGradient id="m-rim" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stopColor="#f4efe6" /><stop offset="1" stopColor="#a9a194" /></linearGradient>
        <radialGradient id="m-enamel" cx="40%" cy="35%" r="70%"><stop offset="0" stopColor="#4f8fd0" /><stop offset="1" stopColor="#245a95" /></radialGradient>
        <symbol id="m-pin" viewBox="0 0 78 78">
          <circle cx="39" cy="39" r="38" fill="url(#m-rim)" />
          <circle cx="39" cy="39" r="33" fill="url(#m-enamel)" />
          <path d="M30 24h18M32 24v4c-3 2-5 4-5 8v13c0 3 2 5 5 5h14c3 0 5-2 5-5V36c0-4-2-6-5-8v-4" fill="none" stroke="#fbf3df" strokeWidth="2.6" strokeLinecap="round" strokeLinejoin="round" />
          <path d={STAR} fill="#f1c84b" transform="translate(33 36) scale(.5)" />
          <path d={STAR} fill="#f1c84b" transform="translate(16 16) scale(.38)" />
          <path d={STAR} fill="#f1c84b" transform="translate(54 50) scale(.34)" />
          <ellipse cx="28" cy="22" rx="14" ry="7" fill="rgba(255,255,255,.28)" transform="rotate(-30 28 22)" />
        </symbol>
      </defs>
    </svg>
  );
}
