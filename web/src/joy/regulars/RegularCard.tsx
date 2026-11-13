// "Tonight?": went / not tonight in one tap, with the somewhere-else form folded away. Ready for the jar page to mount
// (the jar currently renders its own copy of this card).
import { useState } from "react";
import { Link } from "react-router-dom";
import { useBusy } from "../components/useBusy";
import { deleteNight, skipTo, wentTo, type RegularCardData } from "./api";
import { apiMessage, skipBody, wentElse, wentUsual } from "./days";
import { NightPhoto } from "./NightPhoto";
import { EditAmount } from "./EditAmount";
import "./regulars.css";

export function RegularCard({ card, onChange }: { card: RegularCardData; onChange: () => void }) {
  const { busy, run } = useBusy();
  const [error, setError] = useState("");
  const [editing, setEditing] = useState<number | null>(null);
  const [elseFor, setElseFor] = useState<Record<string, { place: string; amount: string }>>({});

  if (!card.show) return null;
  const first = card.open[0];
  const heading = first ? (first.today ? "Tonight?" : "Did we go?") : "Tonight, kept";

  const act = (fn: () => Promise<unknown>) =>
    run(async () => {
      setError("");
      try {
        await fn();
        onChange();
      } catch (e) {
        setError(apiMessage(e));
      }
    });

  return (
    <section className="reg" aria-labelledby="reg-h">
      <h2 className="reg-h" id="reg-h">
        {heading}
      </h2>
      {error && (
        <p className="np-err" role="alert">
          {error}
        </p>
      )}
      {card.went.map((w) => (
        <div className="reg-done" key={w.visit_id}>
          <p className="reg-line">
            <span className="reg-tick" aria-hidden="true">✓</span> <strong>{w.place || w.title}</strong> · {w.amount}
          </p>
          {w.photo && <img className="reg-thumb" src={w.photo} alt="Photo from tonight" width={96} height={120} loading="lazy" />}
          <NightPhoto visitId={w.visit_id} hasPhoto={!!w.photo} addLabel="Got a photo?" removeLabel="Remove photo" onChange={onChange} />
          {editing === w.visit_id ? (
            <EditAmount visitId={w.visit_id} initial={w.amount_raw} onCancel={() => setEditing(null)} onSaved={() => { setEditing(null); onChange(); }} />
          ) : (
            <div>
              <button type="button" className="btn btn-quiet" disabled={busy} onClick={() => setEditing(w.visit_id)}>
                Edit price
              </button>{" "}
              <button type="button" className="btn btn-quiet" disabled={busy} onClick={() => void act(() => deleteNight(w.visit_id))}>
                Undo
              </button>
            </div>
          )}
        </div>
      ))}
      {card.open.map((n) => {
        const k = `${n.regular_id}-${n.day}`;
        const f = elseFor[k] ?? { place: "", amount: String(n.usual_raw) };
        const set = (p: Partial<typeof f>) => setElseFor((s) => ({ ...s, [k]: { ...f, ...p } }));
        return (
          <div className="reg-night" key={k}>
            <p className="reg-line">
              <strong>{n.title}</strong>
              {!n.today && <> · {n.label}</>} · {n.usual}
              {n.maps_url && (
                <>
                  {" "}· <a href={n.maps_url} rel="noopener" target="_blank">Maps</a>
                </>
              )}
            </p>
            <div className="reg-actions">
              <button type="button" className="btn btn-stamp" disabled={busy} onClick={() => void act(() => wentTo(n.regular_id, wentUsual(n.day)))}>
                Went
              </button>
              <button type="button" className="btn btn-ink" disabled={busy} onClick={() => void act(() => skipTo(n.regular_id, skipBody(n.day)))}>
                Not {n.today ? "tonight" : "that day"}
              </button>
            </div>
            <details className="reg-else">
              <summary>Somewhere else, or a different amount</summary>
              <form
                className="reg-else-form"
                onSubmit={(e) => {
                  e.preventDefault();
                  const r = wentElse(n.day, f.place, f.amount);
                  if (!r.body) return setError(r.error);
                  const body = r.body;
                  void act(() => wentTo(n.regular_id, body));
                }}
              >
                <label>
                  Where
                  <input name="place" type="text" maxLength={80} autoComplete="off" placeholder={n.title} value={f.place} onChange={(e) => set({ place: e.target.value })} />
                </label>
                <label>
                  Spent
                  <input name="amount" type="text" inputMode="numeric" autoComplete="off" value={f.amount} onChange={(e) => set({ amount: e.target.value })} />
                </label>
                <button type="submit" className="btn btn-ink" disabled={busy}>
                  Save
                </button>
              </form>
            </details>
          </div>
        );
      })}
      {card.ceiling && <p className="reg-ceiling">Up to {card.ceiling} more this month if we go to every one. Not taken off what’s left.</p>}
      <p className="reg-more">
        <Link to="/joy/regulars">Our regular nights</Link>
      </p>
    </section>
  );
}

