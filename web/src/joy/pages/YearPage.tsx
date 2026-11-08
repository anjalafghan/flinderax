import { useEffect, useRef, useState } from "react";
import { Link, Navigate, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { getYear, getYearGo, yearKeys, type YearView } from "../year/api";
import { Defs, Jar, Lights } from "../year/Decor";
import { DuskArt } from "../year/BestPhoto";
import { SharePanel } from "../year/SharePanel";
import { MAX_PHOTOS } from "../year/layout";
import { dealIn, dropSlips } from "../year/motion";
import { ApiError } from "../api";
import { mapGoTo } from "../paths";
import { errText } from "../components/useBusy";

const plural = (n: number, one: string, many: string) => (n === 1 ? one : many);

/** /joy/year goes to the right year (last year in January); /joy/year/:y is the page. */
export default function YearPage() {
  const { y } = useParams();
  if (!y) return <ToCurrentYear />;
  return <YearBody key={y} y={y} />;
}

function ToCurrentYear() {
  const navigate = useNavigate();
  const q = useQuery({ queryKey: yearKeys.current, queryFn: getYearGo, staleTime: 0, gcTime: 0, retry: false });
  useEffect(() => {
    if (q.data) navigate(mapGoTo(q.data.to), { replace: true });
  }, [q.data, navigate]);
  if (q.isError) return <p className="kind-error" role="alert">{errText(q.error)}</p>;
  return <p className="loading" role="status">Turning to this year…</p>;
}

function YearBody({ y }: { y: string }) {
  const q = useQuery({ queryKey: yearKeys.year(y), queryFn: () => getYear(y), retry: false });
  if (q.isPending) return <p className="loading" role="status">Opening our year…</p>;
  if (q.isError) {
    if (q.error instanceof ApiError && q.error.status === 404) return <Navigate to="/joy/year" replace />;
    return (
      <div>
        <p className="kind-error" role="alert">{errText(q.error)}</p>
        <button type="button" className="btn" onClick={() => void q.refetch()}>Try again</button>
      </div>
    );
  }
  return <Year v={q.data} />;
}

function KindsSec({ v }: { v: YearView }) {
  if (v.kinds.length === 0) return null;
  return (
    <section className="y-sec y-kinds" aria-labelledby="y-kinds-h">
      <h2 id="y-kinds-h" className="y-hand-h y-hand-h-s">{v.vibe_label ? `Mostly ${v.vibe_label.toLowerCase()}` : "Our kind of night"}</h2>
      <ul className="y-strip">
        {v.kinds.map((k) => (
          <li key={k.key} data-kind={k.key} style={{ "--w": k.pct } as React.CSSProperties}>
            <span className="y-washi" aria-hidden="true"></span>
            <span>{k.label} · {k.n}</span>
          </li>
        ))}
      </ul>
      {v.tags.length > 0 && (
        <div className="y-mostly">
          <p className="y-hand-label">mostly</p>
          <ul className="y-tags" aria-label="Top tags">{v.tags.map((t) => <li key={t}>{t}</li>)}</ul>
        </div>
      )}
      {v.best_kind && <p className="y-hand-sub">{v.best_kind} ♡</p>}
    </section>
  );
}

const tallyPath = (g: number) =>
  (g >= 1 ? "M6,4 L5,36" : "") + (g >= 2 ? "M14,5 L14,35" : "") + (g >= 3 ? "M22,4 L23,36" : "") + (g >= 4 ? "M30,5 L29,35" : "") + (g >= 5 ? "M1,28 L36,10" : "");

function Year({ v }: { v: YearView }) {
  const { year, dates, to_come: toCome } = v;
  const stage = useRef<HTMLDivElement>(null);
  const [ticked, setTicked] = useState<number[]>([]);
  const [cap, setCap] = useState("");

  useEffect(() => {
    if (!stage.current) return;
    dropSlips(stage.current);
    dealIn(stage.current, year);
  }, [year]);

  const shareIds = new Set(v.share.photos.map((p) => p.id));
  const wall = v.photos.filter((p) => shareIds.has(p.id));
  const tick = (id: number, on: boolean) => {
    if (!on) {
      setCap("");
      setTicked((t) => t.filter((x) => x !== id));
    } else if (ticked.length >= MAX_PHOTOS) {
      setCap(`Up to ${MAX_PHOTOS} photos fit in the picture.`);
    } else {
      setCap("");
      setTicked((t) => (t.includes(id) ? t : [...t, id]));
    }
  };

  const oldest = v.years[v.years.length - 1];
  const hasMoney = !!v.year_total || !!v.money;
  const showYears = v.years.length > 1 || (v.years.length === 1 && v.years[0] !== year);

  return (
    <div className="y-stage" ref={stage}>
      <Defs />
      <Lights />

      {showYears && (
        <nav className="y-years" aria-label="Other years">
          <ul>
            {v.years.map((yy) => (
              <li key={yy}>
                <Link to={`/joy/year/${yy}`} aria-current={yy === year ? "page" : undefined}>{yy}</Link>
              </li>
            ))}
          </ul>
        </nav>
      )}

      <header className="y-head">
        <div className="y-head-text">
          <p className="y-eyebrow">Our year in the jar</p>
          {dates > 0 ? (
            <>
              <h1 className="y-title">
                <span className="y-count">
                  {dates}
                  <svg className="y-count-ink" viewBox="0 0 150 34" aria-hidden="true"><path className="y-ink" d="M6,22 C44,34 100,32 144,8" /></svg>
                </span>
                <span className="y-count-word">{plural(dates, "date", "dates")} kept<br /> in {year}</span>
              </h1>
              {toCome > 0 && <p className="y-sub">…and {toCome} more on the calendar, still to come.</p>}
            </>
          ) : (
            <h1 className="y-title y-title-empty">Our {year}</h1>
          )}
          {v.sealed_mine > 0 && (
            <p className="y-sealed">
              <span className="y-wax" aria-hidden="true"></span>+{v.sealed_mine} {plural(v.sealed_mine, "surprise", "surprises")} still sealed — {v.sealed_mine === 1 ? "it'll join" : "they'll join"} once revealed
            </p>
          )}
        </div>
        <Jar year={year} dates={dates} slips={v.jar_slips} />
      </header>

      {dates === 0 ? (
        <section className="y-empty grain" aria-labelledby="y-empty-h">
          {oldest !== undefined && oldest - year > 0 ? (
            <>
              <h2 id="y-empty-h">Before the jar</h2>
              <p>No dates kept in {year}: this was before our first slip. Our story starts in <Link to={`/joy/year/${oldest}`}>{oldest}</Link>.</p>
              <p className="y-empty-hand hand" aria-hidden="true">every jar starts empty</p>
            </>
          ) : (
            <>
              <h2 id="y-empty-h">This page is still blank</h2>
              <p>
                No dates kept in {year}
                {toCome > 0 && ` yet: ${toCome} ${toCome === 1 ? "is" : "are"} still to come`}. Every slip we keep this year will land here, month by month.
              </p>
              <p className="y-empty-act"><Link className="btn btn-stamp" to="/joy">Draw a slip</Link></p>
              <p className="y-empty-hand hand" aria-hidden="true">the best pages start empty</p>
            </>
          )}
        </section>
      ) : (
        <>
          <div className="y-spread">
            <div className="y-postmark" aria-hidden="true">
              <svg viewBox="0 0 210 130" width="210" height="130">
                <defs><path id="y-ring" d="M150,65 m-46,0 a46,46 0 1,1 92,0 a46,46 0 1,1 -92,0" /></defs>
                <path className="y-pm-wave" d="M4,46 C18,36 30,56 44,46 S70,36 84,46 S98,52 106,46M4,64 C18,54 30,74 44,64 S70,54 84,64 S98,70 104,64M4,82 C18,72 30,92 44,82 S70,72 84,82 S98,88 106,82" />
                <circle cx="150" cy="65" r="56" className="y-pm-ring" />
                <circle cx="150" cy="65" r="34" className="y-pm-ring y-pm-inner" />
                <text className="y-pm-arc"><textPath href="#y-ring">KEPT IN THE JAR · JUST US · {year} ·</textPath></text>
                <text x="150" y="73" textAnchor="middle" className="y-pm-year">{year}</text>
              </svg>
            </div>

            <div className="y-album">
              <article className="y-page y-page-l">
                <section className="y-sec y-rhythm" aria-labelledby="y-months-h">
                  <div className="y-sec-head">
                    <h2 id="y-months-h" className="y-hand-h">{v.busiest ? `Our ${v.busiest}` : "Month by month"}</h2>
                    <svg className="y-dash" viewBox="0 0 400 10" preserveAspectRatio="none" aria-hidden="true"><path d="M2,5 Q50,1 100,5 T200,5 T300,5 T398,5" /></svg>
                    {v.busiest && (
                      <p className="y-busiest-note">
                        <svg className="y-arrow" viewBox="0 0 40 30" aria-hidden="true"><path d="M4,4 C9,17 19,24 34,24M26,18 L34,24 L26,29" /></svg>
                        ({v.busiest_n} {plural(v.busiest_n, "date", "dates")}, our busiest month)
                      </p>
                    )}
                  </div>
                  <ol className="y-dots">
                    {v.months.map((m, i) => (
                      <li key={m.name} className={`y-mo${m.pct === 100 && m.n > 0 ? " is-busiest" : ""}`}>
                        <span className="y-mo-dots" aria-hidden="true">
                          {Array.from({ length: Math.min(m.n, 9) }, (_, k) => <svg key={k}><use href="#y-heart" /></svg>)}
                        </span>
                        {m.n > 0 ? (
                          <Link className="y-mo-l y-mo-a" to={`/joy/year/${year}/${i + 1}`}>
                            <span aria-hidden="true">{m.letter}</span>
                            {m.pct === 100 && (
                              <svg className="y-circle" viewBox="0 0 40 36" aria-hidden="true"><path className="y-ink" d="M30,6 C20,1 5,6 4,18 C3,30 20,34 30,29 C38,25 39,12 26,5" /></svg>
                            )}
                            <span className="visually-hidden">{m.name}: {m.n} {plural(m.n, "date", "dates")}</span>
                          </Link>
                        ) : (
                          <>
                            <span className="y-mo-l" aria-hidden="true">{m.letter}</span>
                            <span className="visually-hidden">{m.name}: no dates</span>
                          </>
                        )}
                      </li>
                    ))}
                  </ol>
                  <div className="y-tally-row">
                    <div className="y-tally-col">
                      <p className="y-hand-label">our tally:</p>
                      <div className="y-tally" role="img" aria-label={`${dates} ${plural(dates, "date", "dates")} kept in ${year}`}>
                        {v.tally.map((g, i) => (
                          <svg key={i} viewBox="0 0 44 40" width="44" height="40" aria-hidden="true"><path d={tallyPath(g)} /></svg>
                        ))}
                        {v.tally_more > 0 && <span className="y-hand-label">+{v.tally_more} more</span>}
                      </div>
                      <p className="y-hand-big">{dates} {plural(dates, "date", "dates")} in {year}</p>
                      {toCome === 0 && <p className="y-hand-label">(room for more ♡)</p>}
                    </div>
                    {v.streak_weeks > 1 && (
                      <div className="y-note">
                        <span className="y-tape y-tape-dot" aria-hidden="true"></span>
                        <p className="y-note-card"><strong>{v.streak_weeks} weeks</strong> in a row ♡</p>
                      </div>
                    )}
                  </div>
                </section>

                {/* with no money page, the kinds of night go right so the spread stays balanced */}
                {hasMoney && <KindsSec v={v} />}

                <div className="y-doodles" aria-hidden="true">
                  <svg className="y-doodle-sparkle"><use href="#y-sparkle" /></svg>
                  <svg className="y-doodle-heart y-beat"><use href="#y-heart" /></svg>
                </div>
              </article>

              <article className="y-page y-page-r">
                {hasMoney && (
                  <section className="y-sec y-cost" aria-labelledby="y-cost-h">
                    <div className="y-sec-head">
                      <h2 id="y-cost-h" className="y-hand-h">What it cost us</h2>
                      <svg className="y-dash" viewBox="0 0 400 10" preserveAspectRatio="none" aria-hidden="true"><path d="M2,5 Q50,9 100,5 T200,5 T300,5 T398,5" /></svg>
                    </div>
                    <div className="y-ticket-wrap">
                      <span className="y-tape y-tape-stripe" aria-hidden="true"></span>
                      <div className="y-ticket">
                        <div className="y-ticket-stub" aria-hidden="true"><span>ADMIT TWO · {year}</span></div>
                        <div className="y-ticket-body">
                          {v.year_total ? (
                            <>
                              <p className="y-ticket-k">Spent in {year}</p>
                              <p className="y-ticket-big">{v.year_total}</p>
                              {v.budget_note && <p className="y-ticket-line">on a budget of {v.budget_note}</p>}
                            </>
                          ) : (
                            v.money && (
                              <>
                                <p className="y-ticket-k">Spent on our dates</p>
                                <p className="y-ticket-big">{v.money.spent}</p>
                              </>
                            )
                          )}
                          {v.money && (
                            <p className={`y-ticket-line${v.money.over ? " is-over" : ""}`}>
                              {v.money.diff} <span>(on the {v.money.n} {plural(v.money.n, "date", "dates")} we logged)</span>
                            </p>
                          )}
                          {v.cabs && <p className="y-ticket-line"><span>plus {v.cabs} getting there</span></p>}
                        </div>
                      </div>
                    </div>
                    {v.year_total && (
                      <div className="y-blot-chart">
                        <p className="y-hand-label">each blot is a month; bigger means we spent more{v.budget_note && ", coral when over"}</p>
                        <ol className="y-blots">
                          {v.month_blots.map((b, i) => (
                            <li key={i} className={`${b.over ? "is-over" : ""}${b.r === "0.0" ? " is-none" : ""}`} style={{ "--r": b.r } as React.CSSProperties}>
                              <span className="y-blot" aria-hidden="true"></span>
                              <span className="y-mo-l" aria-hidden="true">{b.letter}</span>
                              <span className="visually-hidden">{b.text}</span>
                            </li>
                          ))}
                        </ol>
                        <p className="y-hand-label">dinners that weren’t a slip count too</p>
                      </div>
                    )}
                  </section>
                )}

                {v.best && (
                  <section className="y-sec y-best" aria-labelledby="y-best-h">
                    <div className="y-print-wrap">
                      <div className="y-sway">
                        <span className="y-tape y-tape-stripe" aria-hidden="true"></span>
                        <figure className="y-print">
                          <span className="y-print-photo">
                            {v.best.photo ? <img src={v.best.photo} alt={`Photo from ${v.best.title}`} loading="lazy" decoding="async" width={480} height={600} /> : <DuskArt />}
                          </span>
                          <figcaption>
                            <span id="y-best-h" className="y-cap-big">our best night ♡</span>
                            <span className="y-cap-small">{v.best.when}</span>
                          </figcaption>
                        </figure>
                        <svg className="y-print-heart y-beat" aria-hidden="true"><use href="#y-heart" /></svg>
                      </div>
                    </div>
                    <div className="y-best-text">
                      <h3 className="y-hand-h y-hand-h-s">{v.best.title}</h3>
                      <p className="y-stars-big" role="img" aria-label={`${v.best.rating} out of 5 stars`}>
                        {[1, 2, 3, 4, 5].map((n) => <svg key={n} className={n <= (v.best?.rating ?? 0) ? "is-on" : ""} aria-hidden="true"><use href="#y-star" /></svg>)}
                      </p>
                      {v.best.note && <p className="y-hand-line">“{v.best.note}”</p>}
                      {v.best.more > 0 && (
                        <p className="y-hand-line">
                          and {v.best.more} more {v.best.rating === 5 ? "five-star" : `${v.best.rating}-star`} {plural(v.best.more, "night", "nights")}
                        </p>
                      )}
                      {v.rated > 0 && v.avg_rating && (
                        <div className="y-gave" role="group" aria-label={`Stars we gave: ${v.rating_line}`}>
                          <p className="y-hand-label">stars we gave:</p>
                          {v.rating_rows.map((rr) => (
                            <p className="y-gave-row" aria-hidden="true" key={rr.stars}>
                              <span className="y-stars-small">{Array.from({ length: rr.stars }, (_, k) => <svg key={k}><use href="#y-star" /></svg>)}</span> × {rr.n}
                            </p>
                          ))}
                          <p className="y-hand-label">{v.avg_rating} on average across {v.rated} rated {plural(v.rated, "night", "nights")}</p>
                        </div>
                      )}
                    </div>
                  </section>
                )}

                {!hasMoney && <KindsSec v={v} />}

                <div className="y-doodles" aria-hidden="true">
                  <svg className="y-doodle-sparkle"><use href="#y-sparkle" /></svg>
                </div>
              </article>

              {/* wire rings over the spine, wide screens only */}
              <div className="y-rings" aria-hidden="true">{Array.from({ length: 24 }, (_, i) => <span key={i}></span>)}</div>
            </div>
          </div>

          {v.photos.length > 0 && (
            <section className="y-wall y-page y-page-wall" aria-labelledby="y-wall-h">
              <div className="y-sec-head">
                <h2 id="y-wall-h" className="y-hand-h">The photo wall</h2>
                <p className="y-hand-label">{v.photos.length} {plural(v.photos.length, "photo", "photos")} ♡</p>
              </div>
              <ul className="y-photos">
                {v.photos.map((p) => (
                  <li className="y-polaroid" key={p.id} style={{ "--r": p.id % 5 } as React.CSSProperties}>
                    <figure>
                      <span className="y-tape" aria-hidden="true"></span>
                      <span className="y-frame"><img src={p.url} alt={`Photo from ${p.title}, ${p.when}`} loading="lazy" decoding="async" width={480} height={600} /></span>
                      <figcaption><span className="y-cap-title">{p.title}</span> <span className="y-cap-when">{p.when}</span></figcaption>
                    </figure>
                    {shareIds.has(p.id) && (
                      <label className="y-pick">
                        <input type="checkbox" className="y-pick-in" checked={ticked.includes(p.id)} onChange={(e) => tick(p.id, e.target.checked)} /> Include in picture
                      </label>
                    )}
                  </li>
                ))}
              </ul>
            </section>
          )}

          <SharePanel data={v.share} ticked={ticked} hasWall={wall.length > 0} say={cap} />
        </>
      )}
    </div>
  );
}
