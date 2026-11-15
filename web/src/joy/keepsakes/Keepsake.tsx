// One kept date as the thing we'd have brought home from it: a ticket stub, a boarding pass, a
// postcard, a recipe card… Real text throughout (titles, places and days stay readable and
// selectable); paper, ink and print are CSS and inline SVG. The look lives in styles/keepsakes.css.
// Physical objects: their colours are fixed and never follow light or dark.
import { codeOf, keepsakeFor, type Keepsake as Kind } from "./pick";

export interface KeepsakeDate {
  title: string;
  place: string;
  /** The slip's setting ("local", "home", …). */
  kind: string;
  /** "Around town", "Travel", … */
  kind_label: string;
  tags: string[];
  /** "16 Oct" */
  day: string;
  /** 0 when not rated. */
  rating: number;
}

const stars = (n: number) => "★".repeat(Math.max(0, Math.min(5, n)));

/** Stars a screen reader hears as a number. */
function Stars({ n, className = "ks-stars" }: { n: number; className?: string }) {
  if (n <= 0) return null;
  return <span className={className} role="img" aria-label={`${n} out of 5 stars`}>{stars(n)}</span>;
}

/** The QR and barcode a ticket carries (drawn once by `KeepsakeDefs`). */
const Qr = () => <svg className="ks-qr" viewBox="0 0 25 25" aria-hidden="true"><use href="#ks-qr" /></svg>;
const Bar = ({ v = false }: { v?: boolean }) => (
  <svg className={v ? "ks-bar ks-bar-v" : "ks-bar"} viewBox="0 0 119 30" preserveAspectRatio="none" aria-hidden="true"><use href="#ks-bar" /></svg>
);

export function Keepsake({ d, no, kind: forced }: { d: KeepsakeDate; no: number; kind?: Kind }) {
  const kind = forced ?? keepsakeFor(d.kind, d.tags);
  const serial = `no. ${String(no).padStart(3, "0")}`;
  const place = d.place && d.place !== d.title ? d.place : "";
  switch (kind) {
    case "boarding":
      return (
        <div className="ks ks-boarding" data-ks={kind}>
          <div className="ks-body">
            <div className="ks-main">
              <p className="ks-band"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M2 14l20-7-3 6-9 2-4 5-1-5z" /></svg><span>Boarding pass</span></p>
              <div className="ks-pad">
                <p className="ks-code" aria-hidden="true">{codeOf(d.place, d.title)}</p>
                <p className="ks-title">{d.title}</p>
                <dl className="ks-grid">
                  <div><dt>Passengers</dt><dd>2</dd></div>
                  <div><dt>Date</dt><dd>{d.day}</dd></div>
                  {place && <div className="ks-wide"><dt>To</dt><dd>{place}</dd></div>}
                </dl>
                <Stars n={d.rating} />
              </div>
            </div>
            <div className="ks-stub" aria-hidden="true"><Bar v /><p className="ks-admit">{serial}</p></div>
          </div>
        </div>
      );
    case "postcard":
      return (
        <div className="ks ks-postcard" data-ks={kind}>
          <div className="ks-body">
            <p className="ks-eyebrow" aria-hidden="true">Post card</p>
            <div className="ks-msg">
              <p className="ks-title">{d.title}</p>
              {place && <p className="ks-place">{place}</p>}
              <Stars n={d.rating} />
              <p className="ks-sign">wish you were… oh, you were ♡</p>
            </div>
            <div className="ks-addr">
              <div className="ks-postage" aria-hidden="true">
                <span className="ks-stampbox"><svg viewBox="0 0 40 46"><circle cx="28" cy="13" r="6" /><path d="M0 46V30l10-10 9 9 7-6 14 13v10z" /></svg></span>
                <svg className="ks-postmark" viewBox="0 0 120 64"><circle cx="34" cy="32" r="26" /><circle cx="34" cy="32" r="20" /><path d="M64 22c10-6 20 6 30 0s20 6 26 0M64 32c10-6 20 6 30 0s20 6 26 0M64 42c10-6 20 6 30 0s20 6 26 0" /></svg>
              </div>
              <p className="ks-day">{d.day}</p>
              <span className="ks-lines" aria-hidden="true"></span>
            </div>
          </div>
        </div>
      );
    case "airmail":
      return (
        <div className="ks ks-airmail" data-ks={kind}>
          <div className="ks-body">
            <div className="ks-env">
              <p className="ks-avion" aria-hidden="true">Par avion · By air mail</p>
              <span className="ks-stampbox" aria-hidden="true"><svg viewBox="0 0 40 46"><path d="M20 38s-14-8-14-19a8 8 0 0 1 14-5 8 8 0 0 1 14 5c0 11-14 19-14 19z" /></svg></span>
              <p className="ks-to">to us, from far apart</p>
              <p className="ks-title">{d.title}</p>
              <p className="ks-day">{d.day}</p>
              <Stars n={d.rating} />
            </div>
          </div>
        </div>
      );
    case "recipe":
      return (
        <div className="ks ks-recipe" data-ks={kind}>
          <div className="ks-body">
            <p className="ks-eyebrow">From our kitchen</p>
            <p className="ks-title">{d.title}</p>
            <ul className="ks-ruled">
              <li>serves 2 · {d.day}</li>
              {place && <li>at {place}</li>}
              <li>{d.rating > 0 ? <>would make again <Stars n={d.rating} /></> : "a night in, just us"}</li>
            </ul>
          </div>
        </div>
      );
    case "gig":
      return (
        <div className="ks ks-gig ks-stubbed" data-ks={kind}>
          <div className="ks-body">
            <div className="ks-main">
              <p className="ks-eyebrow">Live · one night only</p>
              <p className="ks-title">{d.title}</p>
              {place && <p className="ks-place">{place}</p>}
              <p className="ks-day">{d.day} · doors at dusk</p>
              <Stars n={d.rating} />
            </div>
            <div className="ks-stub" aria-hidden="true"><Qr /><p className="ks-admit">ADMIT 2<span>{serial}</span></p></div>
          </div>
        </div>
      );
    case "arcade":
      return (
        <div className="ks ks-arcade" data-ks={kind}>
          <div className="ks-body">
            <p className="ks-eyebrow" aria-hidden="true">★ Prize ticket ★</p>
            <p className="ks-title">{d.title}</p>
            <p className="ks-day">{[d.day, place].filter(Boolean).join(" · ")}</p>
            <p className="ks-serial"><span aria-hidden="true">{serial}</span><Stars n={d.rating} /></p>
          </div>
        </div>
      );
    case "admission":
      return (
        <div className="ks ks-admission" data-ks={kind}>
          <div className="ks-body">
            <p className="ks-eyebrow">Admission</p>
            <p className="ks-orn" aria-hidden="true">— ✦ —</p>
            <p className="ks-title">{d.title}</p>
            {place && <p className="ks-place">{place}</p>}
            <Stars n={d.rating} />
            <p className="ks-foot"><span>{d.day}</span><span aria-hidden="true">{serial}</span></p>
            <p className="ks-tear" aria-hidden="true">Admit two</p>
          </div>
        </div>
      );
    case "swingtag":
      return (
        <div className="ks ks-swingtag" data-ks={kind}>
          <svg className="ks-string" viewBox="0 0 60 50" aria-hidden="true"><path d="M30 46C24 30 10 22 4 4M30 46C36 30 50 24 58 8" /></svg>
          <div className="ks-body">
            <p className="ks-hand">made by us</p>
            <p className="ks-title">{d.title}</p>
            {place && <p className="ks-place">{place}</p>}
            <p className="ks-day">{d.day}</p>
            <Stars n={d.rating} />
          </div>
        </div>
      );
    case "trail":
      return (
        <div className="ks ks-trail" data-ks={kind}>
          <svg className="ks-string" viewBox="0 0 50 40" aria-hidden="true"><path d="M48 20C34 10 18 30 2 14M48 20C34 26 20 36 4 30" /></svg>
          <div className="ks-body">
            <svg className="ks-topo" viewBox="0 0 200 120" preserveAspectRatio="none" aria-hidden="true">
              <path d="M-10 90C30 70 50 100 90 80s60-40 120-20M-10 70C40 50 60 80 100 60s60-40 110-26M-10 50C30 36 70 56 110 40s50-30 100-20M40 120c10-20 40-24 60-14s40 4 50-6" />
            </svg>
            <p className="ks-band">Trail pass</p>
            <div className="ks-pad">
              <p className="ks-title">{d.title}</p>
              {place && <p className="ks-place">{place}</p>}
              <p className="ks-day">{d.day} · party of 2</p>
              <Stars n={d.rating} />
            </div>
          </div>
        </div>
      );
    case "coaster":
      return (
        <div className="ks ks-coaster" data-ks={kind}>
          <div className="ks-body">
            <svg className="ks-ringstain" viewBox="0 0 100 100" aria-hidden="true"><circle cx="58" cy="44" r="30" /></svg>
            {place && <p className="ks-eyebrow">{place}</p>}
            <p className="ks-title">{d.title}</p>
            <p className="ks-day">{d.day}</p>
            <Stars n={d.rating} />
          </div>
        </div>
      );
    case "ticket":
    default:
      return (
        <div className="ks ks-ticket ks-stubbed" data-ks="ticket" data-setting={d.kind}>
          <div className="ks-body">
            <div className="ks-main">
              <p className="ks-band"><svg viewBox="0 0 24 14" aria-hidden="true"><path d="M2 13a10 10 0 0 1 20 0M12 1v3M4.5 4.5l2 2M19.5 4.5l-2 2" /></svg><span>{d.kind_label}</span></p>
              <div className="ks-pad">
                <p className="ks-title">{d.title}</p>
                <dl className="ks-meta">
                  {place && <div><dt>Ticket issued at:</dt><dd>{place}</dd></div>}
                  <div><dt>Date:</dt><dd>{d.day}</dd></div>
                </dl>
                <Stars n={d.rating} />
              </div>
            </div>
            <div className="ks-stub" aria-hidden="true"><Qr /><p className="ks-admit">ADMIT 2<span>{serial}</span></p></div>
          </div>
        </div>
      );
  }
}

const QR =
  "M0 0h7v7H0zM1 1v5h5V1zM2 2h3v3H2zM18 0h7v7h-7zM19 1v5h5V1zM20 2h3v3h-3zM0 18h7v7H0zM1 19v5h5v-5zM2 20h3v3H2zM16 16h5v5h-5zM17 17v3h3v-3zM18 18h1v1h-1z" +
  "M8 0h1v1H8zM11 0h1v1h-1zM14 0h1v2h-1zM9 1h3v1H9zM10 2h2v1h-2zM13 2h4v1h-4zM8 3h2v1H8zM15 3h2v1h-2zM12 4h4v1h-4zM8 5h1v1H8zM10 5h1v1h-1zM13 5h2v1h-2zM8 6h1v1H8zM10 6h1v1h-1zM12 6h1v1h-1zM14 6h1v1h-1zM16 6h1v1h-1zM9 7h2v1H9zM13 7h3v1h-3z" +
  "M1 8h4v1H1zM6 8h2v1H6zM9 8h2v1H9zM12 8h1v1h-1zM14 8h4v1h-4zM20 8h1v1h-1zM22 8h2v1h-2zM1 9h1v1H1zM3 9h2v1H3zM7 9h4v1H7zM14 9h2v1h-2zM20 9h2v1h-2zM24 9h1v1h-1zM0 10h1v1H0zM3 10h1v1H3zM8 10h3v1H8zM12 10h1v1h-1zM17 10h2v1h-2zM20 10h2v1h-2z" +
  "M0 11h1v1H0zM2 11h4v1H2zM7 11h3v1H7zM11 11h2v1h-2zM18 11h3v1h-3zM24 11h1v1h-1zM3 12h1v1H3zM5 12h2v1H5zM14 12h1v1h-1zM18 12h1v1h-1zM23 12h1v1h-1zM1 13h1v1H1zM7 13h1v1H7zM10 13h1v1h-1zM12 13h1v1h-1zM17 13h1v1h-1zM22 13h3v1h-3z" +
  "M1 14h2v1H1zM5 14h2v1H5zM8 14h1v1H8zM10 14h2v1h-2zM15 14h1v1h-1zM24 14h1v1h-1zM0 15h5v1H0zM8 15h1v1H8zM11 15h3v1h-3zM22 15h2v1h-2zM11 16h1v1h-1zM13 16h2v1h-2zM22 16h3v1h-3zM12 17h1v1h-1zM23 17h2v1h-2zM10 18h1v1h-1zM22 18h1v1h-1z" +
  "M10 19h1v1h-1zM22 19h2v1h-2zM8 20h2v1H8zM13 20h1v1h-1zM22 20h2v1h-2zM8 21h2v1H8zM17 22h2v1h-2zM9 22h2v1H9zM13 22h1v1h-1zM22 22h1v1h-1zM8 23h1v1H8zM13 23h1v1h-1zM15 23h1v1h-1zM20 23h2v1h-2zM8 24h2v1H8zM12 24h1v1h-1zM14 24h3v1h-3zM18 24h2v1h-2zM23 24h1v1h-1z";
const BAR =
  "M0 0h1v30H0zM3 0h1v30H3zM6 0h2v30H6zM9 0h1v30H9zM11 0h1v30h-1zM14 0h2v30h-2zM18 0h1v30h-1zM20 0h2v30h-2zM25 0h1v30h-1zM28 0h1v30h-1zM30 0h1v30h-1zM32 0h1v30h-1zM35 0h1v30h-1zM37 0h1v30h-1zM40 0h1v30h-1zM43 0h1v30h-1zM47 0h1v30h-1z" +
  "M50 0h2v30h-2zM53 0h1v30h-1zM55 0h2v30h-2zM59 0h1v30h-1zM63 0h1v30h-1zM65 0h1v30h-1zM69 0h3v30h-3zM74 0h2v30h-2zM77 0h2v30h-2zM81 0h2v30h-2zM85 0h2v30h-2zM89 0h1v30h-1zM91 0h1v30h-1zM94 0h1v30h-1zM96 0h1v30h-1zM99 0h3v30h-3zM104 0h2v30h-2zM109 0h3v30h-3zM114 0h3v30h-3z";

/** Drawings every keepsake shares. Render once on a page that shows keepsakes. */
export function KeepsakeDefs() {
  return (
    <svg className="ks-defs" width="0" height="0" aria-hidden="true" focusable="false">
      <defs>
        <symbol id="ks-qr" viewBox="0 0 25 25"><path d={QR} fillRule="evenodd" /></symbol>
        <symbol id="ks-bar" viewBox="0 0 119 30" preserveAspectRatio="none"><path d={BAR} /></symbol>
      </defs>
    </svg>
  );
}
