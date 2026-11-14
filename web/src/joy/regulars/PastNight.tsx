import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useBusy } from "../components/useBusy";
import { regKeys, wentTo } from "./api";
import { apiMessage, parseRupees } from "./days";

/** Log a night we went to on any day that has passed: it becomes a kept date on that day. */
export function PastNight({ regularId, usualRaw }: { regularId: number; usualRaw: number }) {
  const qc = useQueryClient();
  const { busy, run } = useBusy();
  const [day, setDay] = useState("");
  const [amount, setAmount] = useState(usualRaw > 0 ? String(usualRaw) : "");
  const [error, setError] = useState("");
  const today = new Date().toLocaleDateString("en-CA");
  const reset = () => { setDay(""); setAmount(usualRaw > 0 ? String(usualRaw) : ""); setError(""); };
  return (
    <form
      className="rl-form"
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        const a = parseRupees(amount);
        if (!day) return setError("Pick the day we went.");
        if (a === null) return setError("Spent should be a number, like 300.");
        void run(async () => {
          setError("");
          try {
            await wentTo(regularId, { day, amount: a, place: "", back: "" });
            await Promise.all([qc.invalidateQueries({ queryKey: ["joy"] }), qc.invalidateQueries({ queryKey: regKeys.spread(String(regularId)) })]);
            reset();
          } catch (err) {
            setError(apiMessage(err));
          }
        });
      }}
    >
      <h2 className="reg-h">Log a night we missed</h2>
      {error && <p className="rs-error" role="alert">{error}</p>}
      <label>
        The day
        <input type="date" min="2000-01-01" max={today} value={day} onChange={(e) => setDay(e.target.value)} />
      </label>
      <label>
        Spent
        <input type="text" inputMode="numeric" autoComplete="off" value={amount} onChange={(e) => setAmount(e.target.value)} />
      </label>
      <button type="submit" className="btn btn-ink" disabled={busy}>Add to Kept</button>{" "}
      <button type="button" className="btn btn-quiet" onClick={reset}>Reset</button>
    </form>
  );
}
