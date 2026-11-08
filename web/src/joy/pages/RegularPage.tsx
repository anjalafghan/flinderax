import { Link, useParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getSpread, regKeys } from "../regulars/api";
import { nightsWord } from "../regulars/days";
import { NightPhoto } from "../regulars/NightPhoto";
import { ApiError } from "../api";
import { errText } from "../components/useBusy";
import "../regulars/regulars.css";

export default function RegularPage() {
  const { id = "" } = useParams();
  const qc = useQueryClient();
  const q = useQuery({ queryKey: regKeys.spread(id), queryFn: () => getSpread(id), retry: false });
  const refresh = () => void qc.invalidateQueries({ queryKey: regKeys.spread(id) });

  if (q.isPending) return <p className="loading" role="status">Opening the stamps…</p>;
  if (q.isError) {
    const gone = q.error instanceof ApiError && q.error.status === 404;
    return (
      <div>
        <nav className="m-top" aria-label="Back">
          <Link className="m-back" to="/joy/regulars"><span aria-hidden="true">←</span> Our regular nights</Link>
        </nav>
        <p className="kind-error" role="alert">{gone ? "No such regular night." : errText(q.error)}</p>
        {!gone && <button type="button" className="btn" onClick={() => void q.refetch()}>Try again</button>}
      </div>
    );
  }
  const v = q.data;
  return (
    <>
      <nav className="m-top" aria-label="Back">
        <Link className="m-back" to="/joy/kept"><span aria-hidden="true">←</span> Kept</Link>
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
    </>
  );
}
