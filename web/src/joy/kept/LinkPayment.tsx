import { useState } from "react";
import { request, post } from "../api";
import { formatINR } from "@/utils/money";

export interface LinkedTxn {
  txn_id: string;
  merchant: string;
  amount_paise: number;
  day: string;
}

/**
 * "Which payment was this?": your own bank debits around the day of the date. Linking one makes it
 * the date's real cost and stops the app guessing a match from day and amount.
 */
export function LinkPayment({
  historyId,
  linked,
  onDone,
}: {
  historyId: number;
  linked: LinkedTxn | null;
  onDone: () => void;
}) {
  const [open, setOpen] = useState(false);
  const [list, setList] = useState<LinkedTxn[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const load = async () => {
    setError("");
    try {
      setList(
        (
          await request<{ candidates: LinkedTxn[] }>(
            "GET",
            `/api/dates/history/${historyId}/candidates`,
          )
        ).candidates,
      );
    } catch {
      setError("Couldn't load your payments. Try again.");
    }
  };
  const run = async (fn: () => Promise<unknown>) => {
    setBusy(true);
    setError("");
    try {
      await fn();
      onDone();
    } catch (e) {
      setError(e instanceof Error ? e.message : "That didn't work.");
    } finally {
      setBusy(false);
    }
  };

  if (linked) {
    return (
      <p className="k-link">
        Paid with {linked.merchant || "a payment"} ·{" "}
        {formatINR(linked.amount_paise)}{" "}
        <button
          type="button"
          className="btn-why"
          disabled={busy}
          onClick={() =>
            void run(() => post(`/api/dates/history/${historyId}/unlink`))
          }
        >
          Unlink
        </button>
        {error && <span role="alert"> {error}</span>}
      </p>
    );
  }
  return (
    <details
      className="k-link"
      open={open}
      onToggle={(e) => {
        const now = (e.currentTarget as HTMLDetailsElement).open;
        setOpen(now);
        if (now && list === null) void load();
      }}
    >
      <summary
        onClick={() => {
          if (!open && list === null) void load();
        }}
      >
        Which payment was this?
      </summary>
      {list && list.length === 0 && (
        <p>No payments of yours around that day.</p>
      )}
      <ul className="k-link-list">
        {(list ?? []).map((t) => (
          <li key={t.txn_id}>
            <span>
              {t.merchant || "Payment"} · {formatINR(t.amount_paise)} · {t.day}
            </span>
            <button
              type="button"
              className="btn-why"
              disabled={busy}
              onClick={() =>
                void run(() =>
                  post(`/api/dates/history/${historyId}/link`, {
                    txn_id: t.txn_id,
                  }),
                )
              }
            >
              Link<span className="visually-hidden"> {t.merchant}</span>
            </button>
          </li>
        ))}
      </ul>
      {error && <p role="alert">{error}</p>}
    </details>
  );
}
