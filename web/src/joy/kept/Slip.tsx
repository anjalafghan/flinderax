import { Fragment, useEffect, useRef, useState, type AnimationEvent, type ChangeEvent, type CSSProperties, type FormEvent, type KeyboardEvent } from "react";
import { Link } from "react-router-dom";
import { ApiError } from "../api";
import { deleteDate, deletePhoto, rate, reveal, setCost, setLine, setWhen, uploadPhoto } from "./api";
import { CalLink } from "./CalLink";
import { Icon } from "./Defs";
import { useKept } from "./ctx";
import { photoOf, rotOf, tapeOf, uploadLabel } from "./helpers";
import { buzz, reduced, useFit } from "./motion";
import { useStateFrom } from "../components/useStateFrom";
import { shrinkImage } from "./photo";
import { PhotoViewer } from "../components/PhotoViewer";
import type { KeptView, PlannedView } from "./types";
import { useAction } from "./useAction";
import { useFold } from "./useFold";

const slipVars = (h: KeptView, i: number) => ({ "--r": rotOf(h.id), "--i": i }) as CSSProperties;
const sparkVars = (dx: number, dr: number, delay: number) => ({ "--dx": `${dx}px`, "--dr": `${dr}deg`, animationDelay: `${delay}ms` }) as CSSProperties;

function Err({ text }: { text: string }) {
  return text ? (
    <p className="k-form-err" role="status" aria-live="polite">
      {text}
    </p>
  ) : null;
}

// ── a surprise, sealed for you: hints only ──────────────────────────────────
export function SealedSlip({ h, index }: { h: KeptView; index: number }) {
  return (
    <li className="k-slip k-env grain" id={`k${h.id}`} tabIndex={-1} data-setting={h.setting} style={slipVars(h, index)}>
      <article aria-labelledby={`k-t${h.id}`}>
        <span className="k-env-flap" aria-hidden="true" />
        <span className="k-wax" aria-hidden="true">
          <Icon id="k-heart" size={22} />
        </span>
        <div className="k-post" aria-hidden="true">
          <span>sealed</span>
          <b>{h.day_label}</b>
        </div>
        <p className="k-env-eyebrow">A sealed slip</p>
        <h3 className="k-env-title" id={`k-t${h.id}`}>
          {h.from_name ? `A surprise from ${h.from_name}` : "A surprise for you"}
        </h3>
        <p className="visually-hidden">Sealed on {h.day_label}.</p>
        <p className="k-env-aside hand" aria-hidden="true">
          no peeking… well, a little
        </p>
        {h.hints.length > 0 && (
          <>
            <h4 className="visually-hidden">What you can know</h4>
            <ul className="k-hints" role="list">
              {h.hints.map((hint, i) => (
                <li key={i}>{hint}</li>
              ))}
            </ul>
          </>
        )}
        <div className="k-env-foot">
          {h.ics_url && (
            <CalLink className="k-tool" href={h.ics_url}>
              <Icon id="k-cal" size={18} />
              Add to calendar
            </CalLink>
          )}
        </div>
      </article>
    </li>
  );
}

// ── the planner's own "open it" ─────────────────────────────────────────────
function RevealControl({ h }: { h: KeptView }) {
  const { finish, setRevealed } = useKept();
  const { busy, err, act } = useAction();
  const fold = useFold();
  return (
    <details className="k-reveal" {...fold.props}>
      <summary className="k-reveal-btn">
        <span className="k-r-a">Reveal it</span>
        <span className="k-r-b">Not yet, keep it sealed</span>
      </summary>
      <form
        className="k-reveal-form"
        onSubmit={(e) => {
          e.preventDefault();
          void act(async () => {
            const go = await reveal(h.id);
            setRevealed(h.id);
            await finish(go, { focus: `#k${h.id}` });
          });
        }}
      >
        <p>It’s more fun on the day. Open it anyway?</p>
        <div className="k-inline">
          <button className="k-btn k-btn-stamp" type="submit" disabled={busy}>
            Yes, open it
          </button>
        </div>
        <Err text={err} />
      </form>
    </details>
  );
}

/** The flap that lifts off a slip that has just been opened, and "surprise!" in the margin. */
function Tada() {
  const [cover, setCover] = useState(!reduced());
  useEffect(() => {
    buzz(12);
    if (!cover) return;
    const done = () => setCover(false);
    const t = setTimeout(done, 1400);
    addEventListener("keydown", done, { once: true });
    return () => {
      clearTimeout(t);
      removeEventListener("keydown", done);
    };
  }, [cover]);
  return (
    <>
      <span className="k-tada hand" aria-hidden="true">
        surprise!
      </span>
      {cover && (
        <div className="k-cover" aria-hidden="true" onPointerDown={() => setCover(false)} onAnimationEnd={(e: AnimationEvent) => e.target === e.currentTarget && setCover(false)}>
          <span className="k-env-flap" />
          <span className="k-wax">
            <Icon id="k-heart" size={22} />
          </span>
        </div>
      )}
    </>
  );
}

type Phase = { k: "idle" } | { k: "shrink" } | { k: "upload"; label: string } | { k: "dev" };

// ── a kept date ─────────────────────────────────────────────────────────────
export function Slip({ h, index }: { h: KeptView; index: number }) {
  const { currency, finish, refresh, revealed } = useKept();

  // rating + note: saves itself, newest wins (htmx's `hx-sync="this:queue last"`)
  const [rating, setRating] = useState<number | null>(h.rating);
  const [note, setNote] = useState(h.note);
  const saved = useRef({ rating: h.rating, note: h.note });
  const inflight = useRef(false);
  const queued = useRef<{ rating: number | null; note: string } | null>(null);
  const [status, setStatus] = useState<{ text: string; tone: "" | "ok" | "err" }>({ text: "", tone: "" });
  const statusTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const say = (text: string, tone: "" | "ok" | "err" = "") => {
    clearTimeout(statusTimer.current);
    setStatus({ text, tone });
    if (tone === "ok") statusTimer.current = setTimeout(() => setStatus({ text: "", tone: "" }), 2200);
  };
  useEffect(() => () => clearTimeout(statusTimer.current), []);

  const save = async (next: { rating: number | null; note: string }) => {
    queued.current = next;
    if (inflight.current) return;
    inflight.current = true;
    try {
      while (queued.current) {
        const b = queued.current;
        queued.current = null;
        await rate(h.id, b.rating, b.note);
        saved.current = b;
      }
      say("Saved", "ok");
    } catch (e) {
      queued.current = null;
      say(e instanceof TypeError ? "offline? didn’t save" : "didn’t save, try again", "err");
    } finally {
      inflight.current = false;
    }
  };

  const noteEl = useRef<HTMLTextAreaElement>(null);
  const lineEl = useRef<HTMLTextAreaElement>(null);
  useFit(noteEl, note);

  // hearts: a pop, and five hearts lift a few tiny ink hearts off
  const [popped, setPopped] = useState(0);
  const [sparks, setSparks] = useState<number[]>([]);
  const pick = (n: number | null) => {
    setRating(n);
    if (n && !reduced()) {
      setPopped(n);
      if (n === 5) setSparks([0, 1, 2]);
    }
    if (n) buzz(8);
    void save({ rating: n, note });
  };
  const SPARK = [
    [-14, -18],
    [12, 14],
    [2, -6],
  ];
  const onHeartEnd = (e: AnimationEvent<HTMLLabelElement>, n: number) => {
    const t = e.target as Element;
    if (t.classList.contains("k-spark")) setSparks((s) => s.filter((i) => i !== Number(t.getAttribute("data-i"))));
    else if (popped === n) setPopped(0);
  };

  const blurNote = () => {
    if (note !== saved.current.note) void save({ rating, note });
  };
  const noteKey = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      e.currentTarget.blur();
    }
  };

  // your line: sealed until both are written; theirs unfolds when it unseals
  const [line, setLine_] = useStateFrom(h.lines.mine, [h.lines.mine]);
  useFit(lineEl, line);
  const lineAct = useAction();
  const theirsEl = useRef<HTMLQuoteElement>(null);
  const hadTheirs = useRef(!!h.lines.theirs);
  useEffect(() => {
    if (!h.lines.theirs || hadTheirs.current) {
      hadTheirs.current = !!h.lines.theirs;
      return;
    }
    hadTheirs.current = true;
    buzz(10);
    if (!reduced()) {
      theirsEl.current?.animate(
        [
          { opacity: 0, transform: "translateY(-12px) rotate(-2.5deg) scale(.94)" },
          { opacity: 1, transform: "none" },
        ],
        { duration: 520, delay: 200, easing: "cubic-bezier(.34,1.4,.64,1)", fill: "backwards" },
      );
    }
  }, [h.lines.theirs]);
  const sealLine = (e: FormEvent) => {
    e.preventDefault();
    void lineAct.act(async () => {
      await setLine(h.id, line);
      await refresh();
    });
  };

  // photo: pick -> shrink -> upload with progress -> developing -> in
  const [phase, setPhase] = useState<Phase>({ k: "idle" });
  const [photoMsg, setPhotoMsg] = useState<{ text: string; tone: "" | "err" }>({ text: "", tone: "" });
  const [fresh, setFresh] = useState(false);
  const photoAct = useAction();
  const [viewing, setViewing] = useState<number | null>(null);
  const fileEl = useRef<HTMLInputElement>(null);
  const figEl = useRef<HTMLElement>(null);
  // Several files go up one after another; the first one that can't stops the rest.
  const onFile = (e: ChangeEvent<HTMLInputElement>) => {
    const input = e.currentTarget;
    const files = Array.from(input.files ?? []);
    if (files.length === 0) return;
    void photoAct.act(async () => {
      let added = 0;
      try {
        for (const [i, file] of files.entries()) {
          const tell = (label: string) => setPhotoMsg({ text: photoOf(i + 1, files.length, label), tone: "" });
          tell("Getting it ready…");
          setPhase({ k: "shrink" });
          const blob = await shrinkImage(file);
          await uploadPhoto(h.id, blob, (l, t) => {
            const label = uploadLabel(l, t);
            tell(label);
            setPhase(l < t ? { k: "upload", label } : { k: "dev" });
          });
          added += 1;
          setPhase({ k: "dev" });
          tell("Developing…");
          await refresh();
        }
        setFresh(true);
        setPhotoMsg({ text: added > 1 ? `${added} photos added.` : "Photo added.", tone: "" });
        figEl.current?.scrollIntoView({ block: "nearest", behavior: reduced() ? "auto" : "smooth" });
      } catch (x) {
        const why = x instanceof TypeError ? "Offline? The photo didn’t go up. Try again in a moment." : x instanceof ApiError ? x.message : "That didn’t work. Try again?";
        setPhotoMsg({ text: added > 0 ? `${added} added, then: ${why}` : why, tone: "err" });
      } finally {
        input.value = "";
        setPhase({ k: "idle" });
        await refresh(`#p${h.id}`);
      }
    });
  };
  const developing = phase.k !== "idle";

  // when is it?
  const whenFold = useFold();
  const whenSummary = useRef<HTMLElement>(null);
  const whenAct = useAction();
  const [at, setAt] = useStateFrom(h.planned_input, [h.planned_input]);
  const [planned, setPlanned] = useState<PlannedView | null>(null);
  const saveWhen = (e: FormEvent) => {
    e.preventDefault();
    void whenAct.act(async () => {
      const p = await setWhen(h.pair_ids, at);
      setPlanned(p);
      whenFold.setOpen(false);
      say(p.label ? "time saved ♥" : "time cleared", "ok");
      await refresh(`#k${h.id} .k-when-edit > summary`);
    });
  };

  // what did it cost?
  const costFold = useFold();
  const costAct = useAction();
  const cabKey = h.cabs.map((c) => c.fare).join("|");
  const [actual, setActual] = useStateFrom(h.cost_actual_raw, [h.cost_actual_raw, cabKey]);
  const [alsoEstimate, setAlsoEstimate] = useState(true);
  const [cabs, setCabs] = useStateFrom(
    h.cabs.map((c) => c.fare),
    [h.cost_actual_raw, cabKey],
  );
  const saveCost = (e: FormEvent) => {
    e.preventDefault();
    void costAct.act(async () => {
      const go = await setCost(h.id, actual, alsoEstimate, cabs);
      costFold.setOpen(false);
      say("cost saved ♥", "ok");
      await finish(go, { focus: `#k${h.id} .k-cost-edit > summary` });
    });
  };

  const photoDel = useAction();
  const photoFold = useFold();
  const dateDel = useAction();
  const dateFold = useFold();

  const pics = h.photos ?? [];
  const cover = pics[0];
  const hasPhoto = cover !== undefined;
  // A plain click opens the viewer on that photo; a modified click still opens the file itself.
  const view = (at: number) => (e: React.MouseEvent) => {
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    setViewing(at);
  };
  return (
    <li
      className={`k-slip grain${hasPhoto ? " has-photo" : ""}`}
      id={`k${h.id}`}
      tabIndex={-1}
      data-setting={h.setting}
      data-tape={tapeOf(h.id)}
      style={slipVars(h, index)}
    >
      <article aria-labelledby={`k-t${h.id}`}>
        <div className="k-stamp" aria-hidden="true">
          <span className="k-stamp-top">It’s a date</span>
          <span className="k-stamp-day">{h.day_label}</span>
        </div>
        <h3 className="k-slip-title" id={`k-t${h.id}`}>
          {h.title}
        </h3>
        <p className="k-meta">
          <time className="visually-hidden" dateTime={h.day}>
            Stamped {h.day_label}
          </time>
          {h.place && (
            <span className="k-place">
              <Icon id="k-pin" size={14} className="k-pin" />
              {h.place}
            </span>
          )}
        </p>
        {h.surprise_for && (
          <p className="k-secret">
            <span className="k-secret-dot" aria-hidden="true" />
            surprise for {h.surprise_for} · sealed
          </p>
        )}
        {h.can_reveal && <RevealControl h={h} />}
        {h.planned_label && (
          <p className="k-planned">
            <Icon id="k-clock" size={18} />
            <span>
              Planned for <strong>{h.planned_label}</strong>
            </span>
            {h.ics_url && (
              <CalLink className="k-cal" href={h.ics_url}>
                <Icon id="k-cal" size={16} />
                Add to calendar
              </CalLink>
            )}
          </p>
        )}
        {planned?.send_url && h.planned_label && (
          <p className="k-hint">
            <a className="k-cal" href={planned.send_url} download>
              Send {planned.partner || "them"} the invite
            </a>
          </p>
        )}
        <p className="k-cost">
          <span className="k-cost-sign" aria-hidden="true">
            {currency}
          </span>
          {h.cost_actual ? (
            <span>
              Estimated {h.cost_est} · actually <strong>{h.cost_actual}</strong>{" "}
              <span className={`k-cost-diff${h.cost_over ? " is-over" : ""}`}>({h.cost_diff})</span>
            </span>
          ) : (
            <span>
              Estimated <strong>{h.cost_est}</strong>
            </span>
          )}
        </p>

        {hasPhoto ? (
          <>
            <figure ref={figEl} className={`k-polaroid${fresh ? " k-fresh" : ""}`} id={`kp${h.id}`} tabIndex={-1}>
              <a
                href={cover.url}
                className="k-polaroid-frame"
                onClick={view(0)}
                aria-label={pics.length > 1 ? `Open the ${pics.length} photos from ${h.title}` : `Open the photo from ${h.title}`}
              >
                <img src={cover.url} alt={`Photo from ${h.title}`} width={480} height={360} loading={fresh ? "eager" : "lazy"} decoding="async" />
              </a>
              <figcaption className="hand" aria-hidden="true">
                {h.day_label}
              </figcaption>
            </figure>
            <p className="k-photo-tip">
              {pics.length > 1 ? "Tap a photo to see them all." : "Tap the photo to see it bigger."}{" "}
              <Link to="/joy/year#wall">See all our photos</Link>
            </p>
            {(pics.length > 1 || developing) && (
              <ul className="k-photo-strip" aria-label={`More photos from ${h.title}`}>
                {pics.slice(1).map((p, i) => (
                  <li key={p.id}>
                    <a href={p.url} onClick={view(i + 1)} aria-label={`Open photo ${i + 2} of ${pics.length} from ${h.title}`}>
                      <img src={p.url} alt="" width={160} height={120} loading="lazy" decoding="async" />
                    </a>
                  </li>
                ))}
                {developing && (
                  <li className="k-pending" aria-hidden="true">
                    <div className="k-dev">developing…</div>
                  </li>
                )}
              </ul>
            )}
            <PhotoViewer
              photos={pics.map((p, i) => ({ url: p.url, alt: `Photo ${i + 1} of ${pics.length} from ${h.title}`, caption: `${h.title} · ${h.day_label}` }))}
              start={viewing}
              onClose={() => setViewing(null)}
            />
          </>
        ) : (
          developing && (
            <figure ref={figEl} className="k-polaroid k-pending" aria-hidden="true">
              <div className="k-dev">developing…</div>
              <figcaption className="hand">&nbsp;</figcaption>
            </figure>
          )
        )}

        <form
          className="k-form"
          onSubmit={(e) => {
            e.preventDefault();
            void save({ rating, note });
          }}
        >
          <fieldset className="k-hearts">
            <legend className="visually-hidden">How was {h.title}?</legend>
            <span className="k-hearts-cap hand" aria-hidden="true" />
            <div className="k-hearts-row">
              {[1, 2, 3, 4, 5].map((n) => (
                <Fragment key={n}>
                  <input className="k-heart-in visually-hidden" type="radio" name={`rating-${h.id}`} value={n} id={`r${h.id}-${n}`} checked={rating === n} onChange={() => pick(n)} />
                  <label className={`k-heart${popped === n ? " k-pop" : ""}`} htmlFor={`r${h.id}-${n}`} data-n={n} onAnimationEnd={(e) => onHeartEnd(e, n)}>
                    <svg width="30" height="28" aria-hidden="true">
                      <use href="#k-heart" />
                    </svg>
                    <span className="visually-hidden">
                      {n} {n === 1 ? "heart" : "hearts"}
                    </span>
                    {n === 5 &&
                      sparks.map((i) => (
                        <svg key={i} data-i={i} className="k-spark" viewBox="0 0 32 30" aria-hidden="true" style={sparkVars(SPARK[i][0], SPARK[i][1], i * 60)}>
                          <use href="#k-heart" />
                        </svg>
                      ))}
                  </label>
                </Fragment>
              ))}
              <input className="k-heart-in k-clear-in visually-hidden" type="radio" name={`rating-${h.id}`} value="" id={`r${h.id}-0`} checked={rating === null} onChange={() => pick(null)} />
              <label className="k-clear" htmlFor={`r${h.id}-0`}>
                <span className="k-clear-x" aria-hidden="true">
                  ×
                </span>
                <span className="visually-hidden">No hearts</span>
              </label>
            </div>
          </fieldset>

          <div className="k-note">
            <label className="k-field-cap" htmlFor={`n${h.id}`}>
              Our note<span className="visually-hidden"> about {h.title}</span> <span className="k-opt">saved as you go</span>
            </label>
            <textarea
              ref={noteEl}
              className="k-note-in"
              id={`n${h.id}`}
              name="note"
              rows={1}
              maxLength={500}
              placeholder="Add a line to remember it by…"
              value={note}
              onChange={(e) => setNote(e.target.value)}
              onFocus={() => say("")}
              onBlur={blurNote}
              onKeyDown={noteKey}
            />
          </div>

          <div className="k-actions">
            <button className="k-save" type="submit">
              Save
            </button>
            <span className="k-status hand" role="status" aria-live="polite" data-tone={status.tone}>
              {status.text}
            </span>
          </div>
        </form>

        <section className="k-lines" aria-labelledby={`kl${h.id}`}>
          <h4 className="k-lines-h hand" id={`kl${h.id}`}>
            our lines
          </h4>
          {h.lines.theirs && (
            <blockquote ref={theirsEl} className="k-line k-line-theirs">
              <p>{h.lines.theirs}</p>
              <footer>{h.lines.partner}</footer>
            </blockquote>
          )}
          {h.lines.can_write ? (
            <form className="k-line-form" onSubmit={sealLine}>
              <label className="k-field-cap" htmlFor={`l${h.id}`}>
                My private line<span className="visually-hidden"> about {h.title}</span>{" "}
                <span className="k-opt">you’ll both see them once you’ve each written one</span>
              </label>
              <textarea
                ref={lineEl}
                className="k-note-in k-line-in"
                id={`l${h.id}`}
                name="line"
                rows={1}
                maxLength={280}
                placeholder="Only yours until they write theirs…"
                value={line}
                onChange={(e) => setLine_(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    e.currentTarget.form?.requestSubmit(); // your own line has no autosave: Enter seals it
                  }
                }}
              />
              <div className="k-inline">
                <button className="k-btn" type="submit" disabled={lineAct.busy}>
                  {h.lines.mine ? "Save my line" : "Seal my line"}
                </button>
                <span className="k-hint">
                  {h.lines.theirs_waiting
                    ? `${h.lines.partner} wrote a line. Write yours to read it.`
                    : !h.lines.mine
                      ? "You’ll each read the other’s once you’ve both written one."
                      : !h.lines.theirs
                        ? `Sealed until ${h.lines.partner} writes one too.`
                        : ""}
                </span>
              </div>
              <Err text={lineAct.err} />
            </form>
          ) : (
            <p className="k-hint">Sign in again to write your line.</p>
          )}
        </section>

        <div className="k-tools">
          <form className={`k-photo-form${photoAct.busy ? " is-busy" : ""}`} onSubmit={(e) => e.preventDefault()}>
            {h.can_add_photo !== false && (
              <>
                <input ref={fileEl} className="k-file" type="file" name="photo" id={`p${h.id}`} accept="image/*" multiple onChange={onFile} />
                <label className="k-tool" htmlFor={`p${h.id}`}>
                  <Icon id="k-cam" size={18} />
                  {hasPhoto ? "Add more photos" : "Add photos"}
                  <span className="visually-hidden"> for {h.title}</span>
                </label>
              </>
            )}
            <span className="k-photo-status" role="status" aria-live="polite" data-tone={photoMsg.tone}>
              {photoMsg.text}
            </span>
            {photoAct.err && <span className="k-photo-status" data-tone="err">{photoAct.err}</span>}
          </form>
          <details className="k-more">
            <summary className="k-tool k-tool-quiet">More</summary>
          <details className="k-when-edit" {...whenFold.props}>
            <summary ref={whenSummary} className="k-tool">
              <Icon id="k-clock" size={18} />
              {h.planned_label ? "Change time" : "Set a time"}
            </summary>
            <form className="k-when-form" onSubmit={saveWhen}>
              <label className="k-label" htmlFor={`w${h.id}`}>
                When is it?
              </label>
              <div className="k-inline">
                <input className="k-field" id={`w${h.id}`} type="datetime-local" name="at" value={at} onChange={(e) => setAt(e.target.value)} />
                <button className="k-btn" type="submit" disabled={whenAct.busy}>
                  Save time
                </button>
              </div>
              <p className="k-hint">Empty it and save to clear the time.</p>
              <Err text={whenAct.err} />
            </form>
          </details>
          <details className="k-when-edit k-cost-edit" {...costFold.props}>
            <summary className="k-tool">
              <span className="k-cost-ico" aria-hidden="true">
                {currency}
              </span>
              {h.cost_actual ? "Change cost" : "What did it cost?"}
            </summary>
            <form className="k-when-form k-cost-form" onSubmit={saveCost}>
              <label className="k-label" htmlFor={`c${h.id}`}>
                What did the plan itself cost, for both of you? <span className="k-opt">not counting travel</span>
              </label>
              <span className="k-money">
                <span aria-hidden="true">{currency}</span>
                <input className="k-field" id={`c${h.id}`} type="number" name="actual" inputMode="numeric" min={0} step={1} value={actual} onChange={(e) => setActual(e.target.value)} placeholder="e.g. 5000" />
              </span>
              <label className="k-check">
                <input type="checkbox" name="update_estimate" checked={alsoEstimate} onChange={(e) => setAlsoEstimate(e.target.checked)} /> Also update this slip’s estimate in the jar
              </label>
              <p className="k-hint">So next time the jar knows what it really costs.</p>
              <fieldset className="k-cabs">
                <legend className="k-label">
                  Took a cab? <span className="k-opt">one way, optional</span>
                </legend>
                {h.cabs.map((c, i) => (
                  <label className="k-cab" key={i}>
                    <span>{c.name}</span>
                    <span className="k-money">
                      <span aria-hidden="true">{currency}</span>
                      <input
                        className="k-field"
                        type="number"
                        name={`cab_${i}`}
                        inputMode="numeric"
                        min={0}
                        step={1}
                        value={cabs[i] ?? ""}
                        onChange={(e) => setCabs((cur) => cur.map((v, j) => (j === i ? e.target.value : v)))}
                        placeholder="—"
                        aria-label={`${c.name}’s cab fare, one way`}
                      />
                    </span>
                  </label>
                ))}
                <p className="k-hint">Helps the jar’s cab estimates match what you really pay.</p>
              </fieldset>
              <div className="k-cost-actions">
                <button className="k-btn" type="submit" disabled={costAct.busy}>
                  Save
                </button>
                <span className="k-hint">Empty a field and save to clear it.</span>
              </div>
              <Err text={costAct.err} />
            </form>
          </details>
          {hasPhoto && (
            <details className="k-photo-del" {...photoFold.props}>
              <summary className="k-tool k-tool-quiet">{pics.length > 1 ? "Remove a photo" : "Remove photo"}</summary>
              {pics.length > 1 ? (
                <div>
                  <p>Which one should come off this page? It can’t be undone.</p>
                  <ul className="k-photo-pick">
                    {pics.map((p, i) => (
                      <li key={p.id}>
                        <img src={p.url} alt="" width={96} height={72} loading="lazy" decoding="async" />
                        <button
                          className="k-btn k-btn-stamp"
                          type="button"
                          disabled={photoDel.busy}
                          aria-label={`Remove photo ${i + 1} of ${pics.length}`}
                          onClick={() => window.confirm("Remove this photo? This can't be undone.") && void photoDel.act(async () => finish(await deletePhoto(h.id, p.id), { focus: `#k${h.id} .k-photo-del > summary` }))}
                        >
                          Remove
                        </button>
                      </li>
                    ))}
                  </ul>
                  <Err text={photoDel.err} />
                </div>
              ) : (
                <form
                  onSubmit={(e) => {
                    e.preventDefault();
                    if (!window.confirm("Remove this photo? This can't be undone.")) return;
                    void photoDel.act(async () => finish(await deletePhoto(h.id, cover.id), { focus: `#p${h.id}` }));
                  }}
                >
                  <p>Take the photo off this page? It can’t be undone.</p>
                  <button className="k-btn k-btn-stamp" type="submit" disabled={photoDel.busy}>
                    Yes, remove it
                  </button>
                  <Err text={photoDel.err} />
                </form>
              )}
            </details>
          )}
          <details className="k-photo-del k-date-del" {...dateFold.props}>
            <summary className="k-tool k-tool-quiet">
              Remove from Kept<span className="visually-hidden">: {h.title}</span>
            </summary>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (!window.confirm("Remove this date from Kept, with its photos? This can't be undone.")) return;
                void dateDel.act(async () => finish(await deleteDate(h.id), { focus: "#k-title, #k-empty-h" }));
              }}
            >
              <p>
                Take “{h.title}” out of Kept? Its rating{hasPhoto ? `, note and ${pics.length > 1 ? "photos" : "photo"}` : " and note"} go too. The slip stays in the jar.
              </p>
              <button className="k-btn k-btn-stamp" type="submit" disabled={dateDel.busy}>
                Yes, remove it
              </button>
              <Err text={dateDel.err} />
            </form>
          </details>
          </details>
        </div>
      </article>
      {revealed === h.id && <Tada />}
    </li>
  );
}

