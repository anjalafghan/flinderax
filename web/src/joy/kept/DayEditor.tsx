import { useState } from "react";
import { useAction } from "./useAction";
import { setDay } from "./api";
import { useKept } from "./ctx";

/** Change the day a kept date happened, like dating a journal entry. Never a day to come. */
export function DayEditor({ id, day }: { id: number; day: string }) {
  const { finish } = useKept();
  const { busy, err, act } = useAction();
  const [on, setOn] = useState(day);
  const today = new Date().toLocaleDateString("en-CA");
  return (
    <details className="k-when-edit">
      <summary className="k-tool">Change the day</summary>
      <form
        className="k-when-form"
        onSubmit={(e) => {
          e.preventDefault();
          if (!on || on === day) return;
          void act(async () => finish(await setDay(id, on), { focus: `#k${id}` }));
        }}
      >
        <input className="k-field" type="date" min="2000-01-01" max={today} value={on} onChange={(e) => setOn(e.target.value)} aria-label="The day it happened" />
        <button type="submit" className="k-btn" disabled={busy}>Save day</button>
        <button type="button" className="k-btn" onClick={() => setOn(day)}>Reset</button>
        {err && <span className="k-photo-status" data-tone="err">{err}</span>}
      </form>
    </details>
  );
}
