import { useState } from "react";
import { post } from "../api";

export interface ToClose {
  id: number;
  title: string;
  when: string;
}

/** "Did it happen?": a kept date whose day has passed. Yes confirms it; no cancels it. */
export function CloseOut({
  item,
  onDone,
}: {
  item: ToClose;
  onDone: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const answer = async (happened: boolean) => {
    setBusy(true);
    setError("");
    try {
      await post(`/api/dates/history/${item.id}/happened`, {
        happened: happened ? "1" : "0",
      });
      onDone();
    } catch {
      setError("Couldn't save that. Try again.");
      setBusy(false);
    }
  };
  return (
    <div className="closeout grain" role="group" aria-label="Did it happen?">
      <span className="closeout-k">Did it happen?</span>
      <span className="closeout-t">
        {item.title} <span className="closeout-d">({item.when})</span>
      </span>
      <span className="closeout-actions">
        <button
          className="btn btn-ink"
          type="button"
          disabled={busy}
          onClick={() => void answer(true)}
        >
          Yes, we went
        </button>
        <button
          className="btn-why"
          type="button"
          disabled={busy}
          onClick={() => void answer(false)}
        >
          It didn't happen
        </button>
      </span>
      {error && (
        <span role="alert" className="closeout-err">
          {error}
        </span>
      )}
    </div>
  );
}
