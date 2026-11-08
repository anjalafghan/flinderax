// What #pick shows: the drawn slip (Joy's pick_card.html), the stamped "it's a date" card, and the
// handwritten note for "nothing fits". Pure rendering: JarPage owns the requests and the motion.
import { useState, type CSSProperties } from "react";
import { Link } from "react-router-dom";
import { animated, type SpringValue } from "@react-spring/web";
import type { ActivityView, PickCard } from "./api";
import DayPlanView, { Route } from "./DayPlanView";
import { peekView } from "./peeks";
import { FOLLOW, YES } from "./swipe";
import { pose } from "./motion";

export interface CardActions {
  accept: (o: { with?: string; surprise?: boolean }) => void;
  putBack: () => void;
  veto: () => void;
  relax: () => void;
  back: () => void;
  planSwapped: () => void;
}

const I = ({ d, cls = "ico" }: { d: string[]; cls?: string }) => (
  <svg className={cls} viewBox="0 0 24 24" aria-hidden="true">
    {d.map((p) => (
      <path key={p} d={p} />
    ))}
  </svg>
);
const ENVELOPE = ["M3.6 7.2c0-.9.7-1.6 1.6-1.6h13.6c.9 0 1.6.7 1.6 1.6v9.6c0 .9-.7 1.6-1.6 1.6H5.2c-.9 0-1.6-.7-1.6-1.6z", "M4 6.6l8 6.4 8-6.4"];
const CAL = ["M3.6 5.2h16.8v15H3.6z", "M3.8 9.8h16.4M8 3.4v3.6M16 3.4v3.6"];
const PIN = ["M12 21.2c-3.9-4.6-6.4-8.2-6.4-11.2a6.4 6.4 0 0 1 12.8-.2c.1 3.1-2.5 6.7-6.4 11.4z", "M9.7 9.8a2.3 2.3 0 1 0 4.6 0 2.3 2.3 0 1 0-4.6 0"];
const WALK = ["M4 18.5c3.2-.4 4.4-2.6 5.6-5.4 1.3-3.1 3.2-6.2 7-6.9", "M15.4 3.9l2.4 2.3-2.2 2.5"];
const SEND = ["M20.6 3.6 3.4 10.4c-.7.3-.6 1.2.1 1.4l6.3 1.9 1.9 6.3c.2.7 1.1.8 1.4.1z", "M9.8 13.7 20.6 3.6"];
const NOPE = ["M3.6 12a8.4 8.4 0 1 0 16.8 0 8.4 8.4 0 1 0-16.8 0", "M6.2 17.8 17.8 6.2"];

/** The two stamps that say what letting go of a swipe will do, bound to the slip's own spring. */
function Hints({ x, yes, back }: { x: SpringValue<number>; yes: boolean; back: boolean }) {
  return (
    <span className="m-hints" aria-hidden="true">
      {yes && (
        <animated.span className="m-hint m-hint-yes" style={{ opacity: x.to([20 * FOLLOW, YES * FOLLOW], [0, 1], "clamp"), scale: x.to([20 * FOLLOW, YES * FOLLOW], [0.7, 1], "clamp") }}>
          it's a date
        </animated.span>
      )}
      {back && (
        <animated.span className="m-hint m-hint-back" style={{ opacity: x.to([-YES * FOLLOW, -20 * FOLLOW], [1, 0], "clamp"), scale: x.to([-YES * FOLLOW, -20 * FOLLOW], [1, 0.7], "clamp") }}>
          put it back
        </animated.span>
      )}
    </span>
  );
}

function Peeks({ card, spent }: { card: PickCard; spent: number }) {
  const v = peekView(card);
  return (
    <div className="peeks" data-left={card.rerolls_left}>
      <span className="peek-dots" aria-hidden="true">
        {v.dots.map((d, i) =>
          d.bonus ? (
            <svg key={i} className={`j-peek is-bonus${d.full || i === spent ? " is-full" : ""}`} viewBox="0 0 20 20">
              <path className="peek-ring" d="M10 1.6l2.3 5.3 5.8.6-4.4 3.8 1.3 5.7L10 14l-5 3 1.3-5.7-4.4-3.8 5.8-.6z" />
              <path className="peek-fill" d="M10 5.4l1.3 3 3.3.3-2.5 2.2.8 3.2L10 12.4l-2.9 1.7.8-3.2-2.5-2.2 3.3-.3z" />
            </svg>
          ) : (
            <svg key={i} className={`j-peek${d.full || i === spent ? " is-full" : ""}`} viewBox="0 0 20 20">
              <path className="peek-ring" d="M10 2.6c4.3-.2 7.4 3.1 7.3 7.3-.1 4.2-3.3 7.4-7.5 7.4-4.1 0-7.2-3.3-7.1-7.5.1-4 3.2-7.1 7.3-7.2z" />
              <circle className="peek-fill" cx="10" cy="10" r="4.6" />
            </svg>
          ),
        )}
      </span>
      <span className="peek-label">
        {v.label}
        {v.bonusNote && <span className="peek-bonus"> · one's a bonus</span>}
      </span>
    </div>
  );
}

function ShareLink({ card, children, busy }: { card: PickCard; children: React.ReactNode; busy: boolean }) {
  return (
    <a
      className="share"
      href={card.wa_url}
      target="_blank"
      rel="noopener noreferrer"
      aria-disabled={busy || undefined}
      onClick={(e) => {
        if (busy) return e.preventDefault();
        // The native share sheet when there is one; otherwise the wa.me link does its job.
        if (!navigator.share) return;
        e.preventDefault();
        navigator.share({ text: card.share_text }).catch((err: unknown) => {
          if (err instanceof DOMException && err.name === "AbortError") return; // they changed their mind
          window.open(card.wa_url, "_blank", "noopener");
        });
      }}
    >
      {children}
    </a>
  );
}

function Travel({ a }: { a: ActivityView }) {
  return (
    <section className="travel" aria-labelledby="travel-h">
      <p className="travel-h" id="travel-h">
        getting there <span>· one way, roughly</span>
      </p>
      <table className="travel-t">
        <thead>
          <tr>
            <th scope="col">
              <span className="visually-hidden">Who</span>
            </th>
            <th scope="col">Cab</th>
            <th scope="col">Train</th>
          </tr>
        </thead>
        <tbody>
          {a.travel.map((t) => (
            <tr key={t.name}>
              <th scope="row">{t.name}</th>
              <td>
                <strong>{t.cab}</strong>
                <span>{t.cab_time}</span>
              </td>
              <td>
                <strong>{t.train}</strong>
                <span>{t.train_time}</span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {a.travel_note && <p className="travel-note">cab fares {a.travel_note}</p>}
      {a.uber_url && (
        <p className="travel-link">
          <a href={a.uber_url} target="_blank" rel="noopener noreferrer">
            Check the real fare in Uber<span className="visually-hidden"> (opens the Uber app or site)</span>
          </a>
        </p>
      )}
    </section>
  );
}

function Stamp({ date }: { date: string }) {
  return (
    <div className="stamp" role="img" aria-label={`Stamped: it's a date, ${date}`}>
      <svg viewBox="0 0 240 120" aria-hidden="true" focusable="false">
        <defs>
          <filter id="stamp-rough" x="-5%" y="-5%" width="110%" height="110%">
            <feTurbulence type="fractalNoise" baseFrequency=".045" numOctaves="2" seed="7" result="w" />
            <feDisplacementMap in="SourceGraphic" in2="w" scale="3.2" xChannelSelector="R" yChannelSelector="G" />
          </filter>
          <filter id="stamp-wear-f" x="0" y="0" width="100%" height="100%">
            <feTurbulence type="fractalNoise" baseFrequency=".7" numOctaves="2" seed="11" />
            <feColorMatrix type="matrix" values="0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  0 0 0 -3 2.05" />
          </filter>
          <mask id="stamp-wear" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="120">
            <rect width="240" height="120" fill="#fff" filter="url(#stamp-wear-f)" />
          </mask>
        </defs>
        <g className="stamp-ink" filter="url(#stamp-rough)" mask="url(#stamp-wear)">
          <rect x="6" y="6" width="228" height="108" rx="14" fill="none" strokeWidth="6" />
          <rect x="16" y="16" width="208" height="88" rx="8" fill="none" strokeWidth="1.6" />
          <text className="stamp-big" x="120" y="66" textAnchor="middle">
            IT'S A DATE
          </text>
          <text className="stamp-small" x="120" y="91" textAnchor="middle">
            {date}
          </text>
        </g>
      </svg>
    </div>
  );
}

interface SlipProps {
  card: PickCard;
  busy: boolean;
  actions: CardActions;
  /** The slip is still unfolding: blank paper over the text, no taps. */
  arriving: boolean;
  /** Index of the peek the put-back just spent (shown full, then empties). */
  spent: number;
}

export function SlipView({ card, busy, actions, arriving, spent }: SlipProps) {
  const a = card.activity!;
  const done = card.done;
  const [el, setEl] = useState<HTMLElement | null>(null);
  const peeks = peekView(card);
  const canYes = !done;
  const canBack = !done && peeks.canPutBack;

  return (
    <article
      className={`j-slip grain${done ? " is-done" : ""}${card.surprise ? " is-surprise" : ""}${arriving ? " is-arriving" : ""}`}
      data-kind={done ? "done" : "card"}
      data-id={a.id}
      data-setting={a.setting_key}
      style={{ "--tint": `var(--tint-${a.setting_key})` } as CSSProperties}
      aria-labelledby="slip-title"
      ref={setEl}
    >
      <span className="slip-crease" aria-hidden="true" />
      <p className="j-slip-meta">
        <span className="slip-setting">{a.setting}</span>
        <span aria-hidden="true"> · </span>
        {a.duration}
        {a.door && <span className="door"> ({a.door})</span>}
        <span aria-hidden="true"> · </span>
        <strong>{a.cost}</strong>
      </p>
      {a.cost_note && <p className="cost-note">{a.cost_note}</p>}
      {a.closed_label && (
        <p className="closed-note">
          <I d={CAL} />
          <span>{a.closed_label}</span>
        </p>
      )}
      {card.middle && !done && (
        <p className="middle-line">
          <svg viewBox="0 0 64 28" aria-hidden="true" focusable="false">
            <circle cx="22" cy="14" r="11" />
            <circle cx="42" cy="14" r="11" />
          </svg>
          where you both meet tonight
        </p>
      )}
      <h2 id="slip-title" className="j-slip-title" tabIndex={-1}>
        {a.title}
      </h2>
      {!done && card.plan_line && card.route.length === 0 && (
        <p className="plan-line">
          <span className="plan-line-k">{card.when_label}:</span> {card.plan_line}
        </p>
      )}
      {done && card.paired && (
        <p className="paired-line">
          <span aria-hidden="true">+ </span>
          <span className="visually-hidden">paired with </span>
          {card.paired.title}
        </p>
      )}
      {a.generated && (
        <p className="j-gen-tag">
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M8 1.2l1.5 4.3 4.3 1.5-4.3 1.5L8 12.8 6.5 8.5 2.2 7l4.3-1.5z" />
          </svg>
          generated for you
        </p>
      )}
      {a.times_done === 0 && (
        <p className="never" aria-label="Never done before">
          never done!
        </p>
      )}

      {(a.place || a.distances.length > 0) && (
        <div className="slip-where">
          {a.place && (
            <p>
              <I d={PIN} />
              {a.maps_url ? (
                <a className="place-link" href={a.maps_url} target="_blank" rel="noopener noreferrer">
                  {a.place}
                  <span className="visually-hidden"> (opens the map)</span>
                </a>
              ) : (
                <span>{a.place}</span>
              )}
            </p>
          )}
          {a.distances.length > 0 && (
            <p className="soft">
              <I d={WALK} />
              <span>{a.distances.join(" · ")}</span>
            </p>
          )}
        </div>
      )}
      {!done && card.route.length > 0 && (
        <section className="card-route" aria-label={`The plan for ${card.when_label}`}>
          <p className="dp-hand">{card.when_label}, if it's a date:</p>
          <Route steps={card.route} />
          {card.route_note && <p className="dp-note">{card.route_note}</p>}
        </section>
      )}
      {a.travel.length > 0 && !card.plan_line && !card.plan && <Travel a={a} />}

      {a.notes && <p className="slip-notes">{a.notes}</p>}
      {a.tags.length > 0 && (
        <p className="slip-tags">
          {a.tags.map((t) => (
            <span key={t}>#{t}</span>
          ))}
        </p>
      )}

      {done ? (
        <>
          <Stamp date={card.stamp_date} />
          {card.surprise ? (
            <p className="sealed-line">
              <I d={ENVELOPE} />
              <span>Sealed. {card.partner} will only see hints until it's revealed.</span>
            </p>
          ) : (
            <p className="kept-line">In the keepsake box. Rate it after you go.</p>
          )}
          {card.share_text && (
            <p className="share-row">
              <ShareLink card={card} busy={busy}>
                <I d={ENVELOPE} />
                <span>{card.surprise ? `Send ${card.partner} the sealed envelope` : `Send ${card.partner} the envelope`}</span>
                <span className="visually-hidden"> (opens WhatsApp)</span>
              </ShareLink>
            </p>
          )}
          {card.plan && <DayPlanView plan={card.plan} onSwapped={actions.planSwapped} />}
          <div className="slip-actions">
            <Link className="btn btn-quiet" to="/joy/kept">
              See it in Kept
            </Link>
            <button className="btn btn-quiet" type="button" disabled={busy} onClick={actions.back}>
              Back to the jar
            </button>
          </div>
        </>
      ) : (
        <>
          {card.pairs.length > 0 && (
            <section className="pairs" aria-labelledby="pairs-h">
              <p className="pairs-h" id="pairs-h">
                pairs nicely with…
              </p>
              {card.pairs.map((p) => (
                <div className="f-pair" key={p.id}>
                  <p className="pair-title">
                    <span className="pair-plus" aria-hidden="true">
                      +
                    </span>
                    {p.title}
                  </p>
                  <p className="pair-meta">
                    {p.cost} · {p.duration}
                    <span aria-hidden="true"> · </span>
                    <span className="pair-total">together {p.together}</span>
                  </p>
                  {p.over && <p className="pair-over">{p.over}</p>}
                  <button className="btn btn-ink" type="button" disabled={busy} onClick={() => actions.accept({ with: String(p.id) })}>
                    Make it a pair<span className="visually-hidden">: {a.title} and {p.title}</span>
                  </button>
                </div>
              ))}
            </section>
          )}
          <div className="slip-actions">
            <button className="btn btn-stamp" type="button" disabled={busy} onClick={() => actions.accept({})}>
              It's a date
            </button>
            {peeks.canPutBack && (
              <button className="btn btn-ink" type="button" disabled={busy} onClick={actions.putBack}>
                Put it back
              </button>
            )}
          </div>
          <Peeks card={card} spent={spent} />
          {(card.can_surprise || card.can_veto || card.share_text) && (
            <div className="slip-more">
              {card.can_surprise && (
                <span className="f-surprise" style={{ display: "contents" }}>
                  <button className="share" type="button" disabled={busy} onClick={() => actions.accept({ surprise: true })}>
                    <I d={ENVELOPE} />
                    <span>Keep it a surprise for {card.partner}</span>
                  </button>
                </span>
              )}
              {card.share_text && (
                <ShareLink card={card} busy={busy}>
                  <I d={SEND} />
                  <span>Send to {card.partner}</span>
                  <span className="visually-hidden"> (opens WhatsApp)</span>
                </ShareLink>
              )}
              {card.can_veto && (
                <button className="share f-veto" type="button" disabled={busy} onClick={actions.veto}>
                  <I d={NOPE} />
                  <span>
                    Not for me<span className="visually-hidden">: take “{a.title}” out of the draw</span>
                  </span>
                </button>
              )}
            </div>
          )}
        </>
      )}
      {arriving && !done && <div className="slip-cover" />}
      {el && !done && !arriving && (canYes || canBack) && <Hints x={pose(el).springs.x} yes={canYes} back={canBack} />}
    </article>
  );
}

/** Nothing fits: a handwritten note with the hint and one button that loosens the sentence. */
export function NoteView({ card, busy, actions }: { card: PickCard; busy: boolean; actions: CardActions }) {
  return (
    <article className="note" data-kind="note">
      <p className="note-text">{card.message}</p>
      {card.relax_label && card.relax ? (
        <button className="btn btn-stamp" type="button" disabled={busy} onClick={actions.relax}>
          {card.relax_label}
        </button>
      ) : null}
    </article>
  );
}
