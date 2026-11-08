import { useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { archiveSlip, getSlipForm, keys, saveSlip, type FormView } from "../api";
import { errorField, formToState, tidyTitle, toPayload, type FormState } from "../form";
import { mapGoTo } from "../paths";
import { errText, useBusy } from "../components/useBusy";

const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

export default function SlipFormPage() {
  const { id = null } = useParams();
  const q = useQuery({
    queryKey: keys.slipForm(id ?? "new"),
    queryFn: () => getSlipForm(id),
    gcTime: 0,
    staleTime: 0,
    retry: false,
    refetchOnWindowFocus: false,
  });

  if (q.isPending) return <p className="loading" role="status">Opening the slip…</p>;
  if (q.isError) {
    return (
      <div className="write-page">
        <Link className="back" to="/joy/slips">All slips</Link>
        <p className="kind-error" role="alert">{errText(q.error, "That slip isn’t here.")}</p>
      </div>
    );
  }
  // Keyed, so opening another slip starts from its values rather than the last form's.
  return <SlipForm key={id ?? "new"} id={id} view={q.data} />;
}

const Back = () => (
  <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M14.6 6.2 8.8 12l5.8 5.8" />
  </svg>
);

function SlipForm({ id, view }: { id: string | null; view: FormView }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [s, setS] = useState<FormState>(() => formToState(view));
  const [error, setError] = useState("");
  const [fieldErr, setFieldErr] = useState<ReturnType<typeof errorField>>(null);
  const [more, setMore] = useState(view.details_open);
  const [folding, setFolding] = useState(false);
  const { busy, run } = useBusy();
  const archiving = useBusy();
  const errBox = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLTextAreaElement>(null);
  const estCostRef = useRef<HTMLInputElement>(null);
  const locationRef = useRef<HTMLInputElement>(null);
  const hoursRef = useRef<HTMLInputElement>(null);

  const set = <K extends keyof FormState>(k: K, v: FormState[K]) => setS((p) => ({ ...p, [k]: v }));
  const toggle = (k: "closed" | "pairs", v: string) =>
    setS((p) => ({ ...p, [k]: p[k].includes(v) ? p[k].filter((x) => x !== v) : [...p[k], v] }));

  // A refused save puts focus on the field it is about, else on the message.
  useEffect(() => {
    if (!error) return;
    const fieldRefs = { title: titleRef, est_cost: estCostRef, location: locationRef, hours: hoursRef };
    const el = fieldErr ? fieldRefs[fieldErr].current : null;
    (el ?? errBox.current)?.focus();
  }, [error, fieldErr]);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    void run(async () => {
      setError("");
      setFieldErr(null);
      const clean = tidyTitle(s.title);
      setS((p) => ({ ...p, title: clean }));
      setFolding(true);
      try {
        await wait(reduced() ? 0 : 360);
        const res = await saveSlip(id, toPayload({ ...s, title: clean }));
        if (res.ok) {
          void qc.invalidateQueries({ queryKey: keys.slips });
          navigate(mapGoTo(res.go.to));
          return;
        }
        setFolding(false);
        const field = errorField(res.form.error);
        if (field === "hours") setMore(true);
        setFieldErr(field);
        setError(res.form.error);
      } catch (er) {
        setFolding(false);
        setError(errText(er));
      }
    });
  };

  const takeOut = () =>
    void archiving.run(async () => {
      try {
        const g = await archiveSlip(Number(id));
        void qc.invalidateQueries({ queryKey: keys.slips });
        navigate(mapGoTo(g.to));
      } catch (er) {
        setError(errText(er));
      }
    });

  return (
    <div className="write-page">
      <Link className="back" to="/joy/slips"><Back />All slips</Link>
      <h1 className="write-h">{id ? "Edit this slip" : "Write a slip"}</h1>

      {error && (
        <div className="kind-error" role="alert" id="form-error" tabIndex={-1} ref={errBox}>
          <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 3.6c4.8 0 8.5 3.6 8.5 8.4s-3.8 8.4-8.5 8.4-8.4-3.6-8.4-8.4 3.6-8.4 8.4-8.4z" />
            <path d="M12 7.6v5.6" />
            <path d="M12 16.3v.3" />
          </svg>
          <p><strong>Almost.</strong> {error}</p>
        </div>
      )}

      <form className={`write-form${folding ? " sending" : ""}`} onSubmit={submit} noValidate={false} aria-describedby={error ? "form-error" : undefined}>
        <div className={`paper-slip grain tint-${s.setting}${folding && !reduced() ? " folding" : ""}`} id="paper-slip">
          <span className="fold" aria-hidden="true"></span>
          <label className="slip-label" htmlFor="f-title">The idea</label>
          <textarea
            ref={titleRef}
            className="slip-input"
            id="f-title"
            name="title"
            rows={2}
            maxLength={120}
            required
            autoComplete="off"
            placeholder="Sunset at Marine Drive"
            aria-invalid={fieldErr === "title" ? true : undefined}
            value={s.title}
            onChange={(e) => set("title", e.target.value.replace(/\s*[\r\n]+\s*/g, " "))}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.nativeEvent.isComposing) {
                e.preventDefault();
                e.currentTarget.form?.requestSubmit();
              }
            }}
          />
          <p className="slip-sign" aria-hidden="true"><span className="hand-note">for {view.couple}</span></p>
        </div>

        <fieldset className="chips chips-setting">
          <legend>Where does it happen?</legend>
          <div className="chip-row">
            {view.settings.map((o) => (
              <label key={o.value} className={`chip tint-${o.value}`}>
                <input type="radio" name="setting" value={o.value} checked={s.setting === o.value} onChange={() => set("setting", o.value)} required />
                <span><i className="dot" aria-hidden="true"></i>{o.label}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <div className="pair">
          <div className="field">
            <label htmlFor="f-cost">Cost for both of us</label>
            <div className="money">
              <span className="money-sign" aria-hidden="true">{view.currency}</span>
              <input
                ref={estCostRef}
                id="f-cost"
                type="number"
                name="est_cost"
                value={s.est_cost}
                onChange={(e) => set("est_cost", e.target.value)}
                inputMode="numeric"
                min={0}
                step={1}
                required
                placeholder="e.g. 800"
                aria-describedby="cost-hint"
                aria-invalid={fieldErr === "est_cost" ? true : undefined}
              />
            </div>
            <p className="hint" id="cost-hint">Roughly, in {view.currency}. Type 0 if it’s free.</p>
          </div>
          <div className="field">
            <label htmlFor="f-dur">How long</label>
            <div className="select">
              <select id="f-dur" name="duration" value={s.duration} onChange={(e) => set("duration", e.target.value)}>
                {view.durations.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
              </select>
            </div>
          </div>
        </div>

        <fieldset className="chips">
          <legend>Energy</legend>
          <div className="chip-row">
            {view.energies.map((o) => (
              <label key={o.value} className="chip">
                <input type="radio" name="energy" value={o.value} checked={s.energy === o.value} onChange={() => set("energy", o.value)} />
                <span>{o.label}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <fieldset className="group-box">
          <legend>The place <span className="opt">optional</span></legend>
          <div className="pair">
            <div className="field">
              <label htmlFor="f-venue">Venue</label>
              <input id="f-venue" name="venue" value={s.venue} onChange={(e) => set("venue", e.target.value)} autoComplete="off" placeholder="Carter Road promenade" />
            </div>
            <div className="field">
              <label htmlFor="f-area">Area</label>
              <input id="f-area" name="area" value={s.area} onChange={(e) => set("area", e.target.value)} autoComplete="off" placeholder="Bandra" />
            </div>
          </div>
          <div className="field">
            <label htmlFor="f-loc">Location <span className="opt">lat, lng</span></label>
            <input
              ref={locationRef}
              id="f-loc"
              name="location"
              value={s.location}
              onChange={(e) => set("location", e.target.value)}
              autoComplete="off"
              inputMode="text"
              spellCheck={false}
              placeholder="19.13, 72.81"
              aria-describedby="loc-hint"
              aria-invalid={fieldErr === "location" ? true : undefined}
            />
            <p className="hint" id="loc-hint">In Google Maps, long-press the spot and copy the numbers at the top. Leave it blank for at-home plans.</p>
          </div>
        </fieldset>

        <div className="field">
          <label htmlFor="f-tags">Vibes</label>
          <input id="f-tags" name="tags" value={s.tags} onChange={(e) => set("tags", e.target.value)} autoComplete="off" placeholder="romantic, foodie" aria-describedby="tags-hint" />
          <p className="hint" id="tags-hint">A few words, separated by commas.</p>
        </div>

        <div className="field">
          <label htmlFor="f-notes">Notes</label>
          <textarea id="f-notes" className="notes" name="notes" rows={3} value={s.notes} onChange={(e) => set("notes", e.target.value)} placeholder="Go before 6, grab cutting chai on the way." />
        </div>

        <fieldset className="group-box when-open">
          <legend>When it’s on <span className="opt">optional</span></legend>
          <fieldset className="chips">
            <legend className="sub-legend">Closed on</legend>
            <div className="chip-row">
              {view.closed_opts.map((o) => (
                <label key={o.value} className="chip">
                  <input type="checkbox" name="closed" value={o.value} checked={s.closed.includes(o.value)} onChange={() => toggle("closed", o.value)} />
                  <span>{o.label}</span>
                </label>
              ))}
            </div>
          </fieldset>
          <div className="pair season-pair">
            <div className="field">
              <label htmlFor="f-season-from">In season from</label>
              <div className="select">
                <select id="f-season-from" name="season_from" value={s.season_from} onChange={(e) => set("season_from", e.target.value)}>
                  <option value="">All year</option>
                  {view.season_from_opts.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                </select>
              </div>
            </div>
            <div className="field">
              <label htmlFor="f-season-to">until</label>
              <div className="select">
                <select id="f-season-to" name="season_to" value={s.season_to} onChange={(e) => set("season_to", e.target.value)}>
                  <option value="">All year</option>
                  {view.season_to_opts.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                </select>
              </div>
            </div>
          </div>
          <p className="hint">The jar won’t suggest it on closed days or out of season. Seasons can wrap the new year (Nov until May).</p>
        </fieldset>

        <details className="group-box more-details" open={more} onToggle={(e) => setMore(e.currentTarget.open)}>
          <summary>More details <span className="opt">for the day-of plan</span></summary>
          <p className="hint">The jar uses these to work out when to leave, whether to book, and what to bring. Leave any of them blank.</p>
          <div className="field">
            <label htmlFor="f-hours">Opening hours</label>
            <input
              ref={hoursRef}
              id="f-hours"
              name="hours"
              value={s.hours}
              onChange={(e) => set("hours", e.target.value)}
              autoComplete="off"
              spellCheck={false}
              placeholder="tue-sun 12:00-23:30; mon 18:00-23:00"
              aria-describedby="hours-hint"
              aria-invalid={fieldErr === "hours" ? true : undefined}
            />
            <p className="hint" id="hours-hint">
              {view.hours_today ? `Today: ${view.hours_today}` : "Days, then 24-hour times. A close before the open runs past midnight."}
            </p>
          </div>
          <div className="pair">
            <div className="field">
              <label htmlFor="f-best">Best time</label>
              <div className="select">
                <select id="f-best" name="best_time" value={s.best_time} onChange={(e) => set("best_time", e.target.value)}>
                  <option value="">Any time</option>
                  {view.best_times.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                </select>
              </div>
            </div>
            <div className="field">
              <label htmlFor="f-min">How long, in minutes</label>
              <input id="f-min" type="number" name="minutes" value={s.minutes} onChange={(e) => set("minutes", e.target.value)} inputMode="numeric" min={1} step={5} placeholder="90" />
            </div>
          </div>
          <div className="pair">
            <div className="field">
              <label htmlFor="f-book">Booking</label>
              <div className="select">
                <select id="f-book" name="booking" value={s.booking} onChange={(e) => set("booking", e.target.value)}>
                  <option value="">Not sure</option>
                  {view.bookings.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
                </select>
              </div>
            </div>
            <div className="field">
              <label htmlFor="f-ahead">Days ahead</label>
              <input id="f-ahead" type="number" name="book_ahead_days" value={s.book_ahead_days} onChange={(e) => set("book_ahead_days", e.target.value)} inputMode="numeric" min={1} max={90} placeholder="2" />
            </div>
          </div>
          <fieldset className="chips">
            <legend className="sub-legend">If it rains</legend>
            <div className="chip-row">
              <label className="chip">
                <input type="radio" name="cover" value="" checked={s.cover === ""} onChange={() => set("cover", "")} />
                <span>Not sure</span>
              </label>
              {view.covers.map((o) => (
                <label key={o.value} className="chip">
                  <input type="radio" name="cover" value={o.value} checked={s.cover === o.value} onChange={() => set("cover", o.value)} />
                  <span>{o.label}</span>
                </label>
              ))}
            </div>
          </fieldset>
          <div className="field">
            <label htmlFor="f-bring">Bring</label>
            <input id="f-bring" name="bring" value={s.bring} onChange={(e) => set("bring", e.target.value)} autoComplete="off" placeholder="closed shoes, cash" />
          </div>
          <div className="pair">
            <div className="field">
              <label htmlFor="f-station">Nearest station</label>
              <input id="f-station" name="station" value={s.station} onChange={(e) => set("station", e.target.value)} autoComplete="off" placeholder="Andheri" />
            </div>
            <div className="field">
              <label htmlFor="f-walk">Walk, in minutes</label>
              <input id="f-walk" type="number" name="station_walk_min" value={s.station_walk_min} onChange={(e) => set("station_walk_min", e.target.value)} inputMode="numeric" min={1} max={120} placeholder="10" />
            </div>
          </div>
        </details>

        {view.pair_opts.length > 0 && (
          <fieldset className="group-box pairs-box">
            <legend>Pairs well with <span className="opt">optional</span></legend>
            <p className="hint" id="pairs-hint">Tick slips that make a good double plan with this one. When either is drawn, the jar offers the other.</p>
            <div className="pair-list" role="group" aria-describedby="pairs-hint">
              {view.pair_opts.map((o) => (
                <label key={o.value} className="pair-opt">
                  <input type="checkbox" name="pairs" value={o.value} checked={s.pairs.includes(o.value)} onChange={() => toggle("pairs", o.value)} />
                  <span>{o.label}</span>
                </label>
              ))}
            </div>
          </fieldset>
        )}

        <fieldset className="chips">
          <legend>Written by</legend>
          <div className="chip-row">
            {view.people.map((o) => (
              <label key={o.value} className="chip">
                <input type="radio" name="added_by" value={o.value} checked={s.added_by === o.value} onChange={() => set("added_by", o.value)} />
                <span>{o.label}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <div className="write-actions">
          <button type="submit" className="btn btn-stamp" id="save-btn" disabled={busy}>
            <svg className="ink-ico" width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M5 9.5h14l-1.4 9.4c-.2 1-1 1.7-2 1.7H8.4c-1 0-1.8-.7-2-1.7z" />
              <path d="M7 6.2h10" />
              <path d="M12 3.4v8.2M9.4 9l2.6 2.7L14.6 9" />
            </svg>
            {id ? "Save the slip" : "Fold it into the jar"}
          </button>
          <Link className="btn btn-quiet" to="/joy/slips">Cancel</Link>
        </div>
      </form>

      {id && (
        <div className="write-takeout">
          <p className="soft">Not feeling this one for now?</p>
          <button type="button" className="takeout-btn" disabled={archiving.busy || busy} onClick={takeOut}>Take it out of the jar</button>
        </div>
      )}
    </div>
  );
}
