import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getCouple, keys } from "../api";
import { createRegular, getRegulars, regKeys, restoreRegular, type DayBox, type StoppedItem } from "../regulars/api";
import { apiMessage, newRegularBody, nightsWord } from "../regulars/days";
import { mapGoTo } from "../paths";
import { ErrorState } from "../components/ErrorState";
import { errText, useBusy } from "../components/useBusy";
import "../regulars/regulars.css";

export default function RegularsPage() {
  const q = useQuery({ queryKey: regKeys.list, queryFn: getRegulars });
  if (q.isPending) return <p className="loading" role="status">Opening our regular nights…</p>;
  if (q.isError) {
    return (
      <ErrorState message={errText(q.error)} onRetry={() => void q.refetch()} />
    );
  }
  const { items, days, stopped } = q.data;
  return (
    <>
      <nav className="m-top" aria-label="Back">
        <Link className="m-back" to="/joy/kept"><span aria-hidden="true">←</span> Kept</Link>
      </nav>
      <header className="rs-head">
        <h1 className="rs-title">Our regular nights</h1>
        <p className="rs-sub">Places we go on the same days. On those evenings the jar asks if we went.</p>
      </header>

      {items.length === 0 ? (
        <p className="rs-empty">No regulars yet. Add the first one below.</p>
      ) : (
        <ul className="rl-list">
          {items.map((i) => (
            <li key={i.id}>
              <Link className="rl-item" to={`/joy/regulars/${i.id}`}>
                <strong>{i.title}</strong>
                <span>
                  {i.days} · {i.usual} a night · {i.count} {nightsWord(i.count)} so far{i.place && ` · ${i.place}`}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}

      {stopped.length > 0 && <StoppedList stopped={stopped} />}

      <AddForm days={days} />
    </>
  );
}

/** Regulars you stopped tracking. Their nights are kept; restoring one starts the prompts again. */
function StoppedList({ stopped }: { stopped: StoppedItem[] }) {
  const qc = useQueryClient();
  const { busy, run } = useBusy();
  const [err, setErr] = useState("");
  const restore = (id: number) =>
    void run(async () => {
      setErr("");
      try {
        await restoreRegular(id);
        await qc.invalidateQueries({ queryKey: regKeys.list });
      } catch (e) {
        setErr(errText(e));
      }
    });
  return (
    <details className="rs-stopped">
      <summary>Stopped ({stopped.length})</summary>
      <ul className="rl-list">
        {stopped.map((s) => (
          <li key={s.id}>
            <span className="rl-item">
              <strong>{s.title}</strong>
              <button type="button" className="btn" disabled={busy} onClick={() => restore(s.id)}>
                Reset: track it again
              </button>
            </span>
          </li>
        ))}
      </ul>
      {err && <p role="alert">{err}</p>}
    </details>
  );
}

function AddForm({ days }: { days: DayBox[] }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { busy, run } = useBusy();
  const [title, setTitle] = useState("");
  const [place, setPlace] = useState("");
  const [maps, setMaps] = useState("");
  const [picked, setPicked] = useState<number[]>(() => days.filter((d) => d.checked).map((d) => d.value));
  const [cost, setCost] = useState("");
  const [since, setSince] = useState("");
  const [error, setError] = useState("");
  const currency = useQuery({ queryKey: keys.couple, queryFn: getCouple }).data?.currency ?? "";

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const { body, error: msg } = newRegularBody({ title, place, maps_url: maps, days: picked, cost, since });
    if (!body) {
      setError(msg);
      return;
    }
    void run(async () => {
      setError("");
      try {
        const go = await createRegular(body);
        await qc.invalidateQueries({ queryKey: regKeys.list });
        navigate(mapGoTo(go.to));
      } catch (err) {
        setError(apiMessage(err));
      }
    });
  };

  return (
    <form className="rl-form" onSubmit={submit} noValidate>
      <h2 className="reg-h">Add a regular</h2>
      {error && <p className="rs-error" role="alert">{error}</p>}
      <label>
        Name
        <input name="title" type="text" maxLength={60} required autoComplete="off" placeholder="Blue Tokai" value={title} onChange={(e) => setTitle(e.target.value)} />
      </label>
      <label>
        Place
        <input name="place" type="text" maxLength={120} autoComplete="off" placeholder="Blue Tokai Coffee Roasters, Goregaon East" value={place} onChange={(e) => setPlace(e.target.value)} />
      </label>
      <label>
        Maps link
        <input name="maps_url" type="url" autoComplete="off" placeholder="https://maps.app.goo.gl/…" value={maps} onChange={(e) => setMaps(e.target.value)} />
      </label>
      <fieldset className="rl-days">
        <legend>Which days</legend>
        {days.map((d) => (
          <label className="rl-day" key={d.value}>
            <input
              type="checkbox"
              name="days"
              value={d.value}
              checked={picked.includes(d.value)}
              onChange={(e) => setPicked((p) => (e.target.checked ? [...p, d.value] : p.filter((v) => v !== d.value)))}
            />
            <span>{d.label}</span>
          </label>
        ))}
      </fieldset>
      <label>
        Usual spend a night {currency && <span className="opt">in {currency}</span>}
        <input name="cost" type="text" inputMode="numeric" autoComplete="off" placeholder="e.g. 300" value={cost} onChange={(e) => setCost(e.target.value)} />
        <small>Optional. It fills in when you tap Went.</small>
      </label>
      <label>
        Counting from
        <input name="since" type="date" value={since} onChange={(e) => setSince(e.target.value)} />
        <small>Blank is today. Pick an earlier day to be asked about nights you've missed this week.</small>
      </label>
      <button className="btn btn-stamp" type="submit" disabled={busy}>Add it</button>
    </form>
  );
}
