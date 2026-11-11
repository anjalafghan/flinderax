import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { clearSkips, getSpread, regKeys, stopTracking } from "../regulars/api";
import { EditRegular } from "../regulars/EditRegular";
import { apiMessage } from "../regulars/days";
import { nightsWord } from "../regulars/days";
import { NightPhoto } from "../regulars/NightPhoto";
import { ApiError } from "../api";
import { ErrorState } from "../components/ErrorState";
import { errText, useBusy } from "../components/useBusy";
import "../regulars/regulars.css";

export default function RegularPage() {
  const { id = "" } = useParams();
  const qc = useQueryClient();
  const q = useQuery({ queryKey: regKeys.spread(id), queryFn: () => getSpread(id), retry: false });
  const navigate = useNavigate();
  const [editing, setEditing] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [stopErr, setStopErr] = useState("");
  const stopping = useBusy();
  const stop = () =>
    void stopping.run(async () => {
      setStopErr("");
      try {
        await stopTracking(id);
        await qc.invalidateQueries({ queryKey: regKeys.list });
        qc.removeQueries({ queryKey: regKeys.spread(id) });
        navigate("/joy/regulars", { replace: true });
      } catch (e) {
        setStopErr(apiMessage(e));
      }
    });
  const refresh = () => void qc.invalidateQueries({ queryKey: regKeys.spread(id) });
  const forgetSkips = () =>
    void stopping.run(async () => {
      setStopErr("");
      try {
        await clearSkips(id);
        await qc.invalidateQueries({ queryKey: regKeys.list });
        refresh();
      } catch (e) {
        setStopErr(apiMessage(e));
      }
    });

  if (q.isPending) return <p className="loading" role="status">Opening the stamps…</p>;
  if (q.isError) {
    const gone = q.error instanceof ApiError && q.error.status === 404;
    return (
      <div>
        <nav className="m-top" aria-label="Back">
          <Link className="m-back" to="/joy/regulars"><span aria-hidden="true">←</span> Our regular nights</Link>
        </nav>
        <ErrorState message={gone ? "No such regular night." : errText(q.error)} onRetry={gone ? undefined : () => void q.refetch()} />
      </div>
    );
  }
  const v = q.data;
  return (
    <>
      <nav className="m-top" aria-label="Back">
        <Link className="m-back" to="/joy/regulars"><span aria-hidden="true">←</span> Our regular nights</Link>
      </nav>
      <header className="rs-head">
        <h1 className="rs-title">{v.title}</h1>
        <p className="rs-sub">
          {v.days}
          {v.place && ` · ${v.place}`}
          {v.maps_url && (
            <>
              {" · "}
              <a href={v.maps_url} rel="noopener" target="_blank">Maps</a>
            </>
          )}
        </p>
        {v.count > 0 && (
          <p className="rs-sum">
            {v.count} {nightsWord(v.count)} · {v.total} spent · about {v.average} a night
            {v.elsewhere > 0 && ` · ${v.elsewhere} somewhere else`}
          </p>
        )}
      </header>

      {editing ? (
        <EditRegular regular={v} onDone={() => setEditing(false)} />
      ) : (
        <div className="rs-actions">
          <button type="button" className="btn" onClick={() => setEditing(true)}>Edit</button>
          <button
            type="button"
            className="btn"
            disabled={stopping.busy}
            onClick={() => window.confirm("Forget every night you skipped here? Those days will be open again.") && forgetSkips()}
          >
            Reset: forget skipped nights
          </button>
        </div>
      )}

      {v.stamps.length === 0 ? (
        <p className="rs-empty">Nothing stamped yet. On {v.days} the jar page asks if we went.</p>
      ) : (
        <ul className="rs-grid">
          {v.stamps.map((n) => (
            <li className="rs-night" id={`n${n.id}`} key={n.id} style={{ "--t": n.tilt } as React.CSSProperties}>
              {n.photo ? (
                <figure className="rs-print">
                  <img src={n.photo} alt={`${v.title}, ${n.day}`} loading="lazy" width={240} height={300} />
                  <figcaption>{n.day} {n.year}</figcaption>
                </figure>
              ) : (
                <div className="rs-stamp">
                  <span className="rs-stamp-d">{n.day}</span>
                  <span className="rs-stamp-y">{n.year}</span>
                </div>
              )}
              <p className="rs-meta">
                {n.amount}
                {n.elsewhere && ` · ${n.elsewhere}`}
              </p>
              <div className="rs-actions">
                <NightPhoto visitId={n.id} hasPhoto={!!n.photo} onChange={refresh} />
              </div>
            </li>
          ))}
        </ul>
      )}

      <section className="rs-stop" aria-labelledby="stop-h">
        <h2 id="stop-h" className="reg-h">Stop tracking</h2>
        {!confirming ? (
          <button type="button" className="btn btn-quiet" aria-expanded="false" onClick={() => setConfirming(true)}>
            Stop tracking this regular
          </button>
        ) : (
          <div role="group" aria-labelledby="stop-h">
            <p>
              {v.title} leaves the list and the jar stops asking about it.
              {v.count > 0 ? ` The ${v.count} ${nightsWord(v.count)} you stamped stay in your year.` : ""}
            </p>
            <div className="rs-actions">
              <button type="button" className="btn btn-danger" disabled={stopping.busy} onClick={stop}>Yes, stop tracking</button>
              <button type="button" className="btn btn-quiet" disabled={stopping.busy} onClick={() => setConfirming(false)}>Keep it</button>
            </div>
          </div>
        )}
        {stopErr && <p className="rs-error" role="alert">{stopErr}</p>}
      </section>
    </>
  );
}
