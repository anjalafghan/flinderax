import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getCouple, keys } from "../api";
import { regKeys, updateRegular, type SpreadView } from "./api";
import { apiMessage, newRegularBody } from "./days";
import { useBusy } from "../components/useBusy";

const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/** Change a regular's name, place, link, days and usual spend. Nights already stamped stay as they are. */
export function EditRegular({ regular, onDone }: { regular: SpreadView; onDone: () => void }) {
  const qc = useQueryClient();
  const { busy, run } = useBusy();
  const currency = useQuery({ queryKey: keys.couple, queryFn: getCouple }).data?.currency ?? "";
  const [title, setTitle] = useState(regular.title);
  const [place, setPlace] = useState(regular.place);
  const [maps, setMaps] = useState(regular.maps_url);
  const [picked, setPicked] = useState<number[]>(regular.day_values);
  const [cost, setCost] = useState(regular.usual_raw > 0 ? String(regular.usual_raw) : "");
  const [error, setError] = useState("");

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const { body, error: msg } = newRegularBody({ title, place, maps_url: maps, days: picked, cost, since: "" });
    if (!body) {
      setError(msg);
      return;
    }
    void run(async () => {
      setError("");
      try {
        await updateRegular(regular.regular_id, body);
        await Promise.all([
          qc.invalidateQueries({ queryKey: regKeys.spread(String(regular.regular_id)) }),
          qc.invalidateQueries({ queryKey: regKeys.list }),
        ]);
        onDone();
      } catch (err) {
        setError(apiMessage(err));
      }
    });
  };

  return (
    <form className="rl-form" onSubmit={submit} noValidate aria-labelledby="edit-reg-h">
      <h2 className="reg-h" id="edit-reg-h">Edit this regular</h2>
      {error && <p className="rs-error" role="alert">{error}</p>}
      <label>
        Name
        <input type="text" maxLength={60} required autoComplete="off" value={title} onChange={(e) => setTitle(e.target.value)} />
      </label>
      <label>
        Place
        <input type="text" maxLength={120} autoComplete="off" value={place} onChange={(e) => setPlace(e.target.value)} />
      </label>
      <label>
        Maps link
        <input type="url" autoComplete="off" placeholder="https://maps.app.goo.gl/…" value={maps} onChange={(e) => setMaps(e.target.value)} />
      </label>
      <fieldset className="rl-days">
        <legend>Which days</legend>
        {DAY_LABELS.map((l, v) => (
          <label className="rl-day" key={v}>
            <input
              type="checkbox"
              checked={picked.includes(v)}
              onChange={(e) => setPicked((p) => (e.target.checked ? [...p, v] : p.filter((x) => x !== v)))}
            />
            <span>{l}</span>
          </label>
        ))}
      </fieldset>
      <label>
        Usual spend a night {currency && <span className="opt">in {currency}</span>}
        <input type="text" inputMode="numeric" autoComplete="off" placeholder="e.g. 300" value={cost} onChange={(e) => setCost(e.target.value)} />
      </label>
      <div className="rs-actions">
        <button className="btn btn-stamp" type="submit" disabled={busy}>Save changes</button>
        <button className="btn btn-quiet" type="button" disabled={busy} onClick={onDone}>Cancel</button>
      </div>
    </form>
  );
}
