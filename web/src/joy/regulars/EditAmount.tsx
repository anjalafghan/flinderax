import { useState } from "react";
import { useBusy } from "../components/useBusy";
import { setNightAmount } from "./api";
import { apiMessage, parseRupees } from "./days";

/** Fix what a night we went to cost. `onSaved` runs after the server has it. */
export function EditAmount({ visitId, initial, onSaved, onCancel }: { visitId: number; initial: number; onSaved: () => void; onCancel: () => void }) {
  const { busy, run } = useBusy();
  const [value, setValue] = useState(String(initial));
  const [error, setError] = useState("");
  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const n = parseRupees(value);
    if (n === null || n === "") return setError("Spent should be a number, like 300.");
    void run(async () => {
      setError("");
      try {
        await setNightAmount(visitId, n);
        onSaved();
      } catch (err) {
        setError(apiMessage(err));
      }
    });
  };
  return (
    <form className="reg-else-form" onSubmit={submit} noValidate>
      {error && <p className="np-err" role="alert">{error}</p>}
      <label>
        Spent
        <input type="text" inputMode="numeric" autoComplete="off" value={value} onChange={(e) => setValue(e.target.value)} />
      </label>
      <button type="submit" className="btn btn-ink" disabled={busy}>Save</button>
      <button type="button" className="btn btn-quiet" onClick={() => { setValue(String(initial)); setError(""); onCancel(); }}>Cancel</button>
    </form>
  );
}
