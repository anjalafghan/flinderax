import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { clearSettlements, deleteSettlement, getCouple, getSettle, getSummary, inviteCode, keys, leaveCouple, recordSettlement, resetPartnerShare, setPartnerShare, type Invite } from "../api";
import { CodeCard } from "../components/CodeCard";
import { useStateFrom } from "../components/useStateFrom";
import { ErrorState } from "../components/ErrorState";
import { errText, useBusy } from "../components/useBusy";
import { rupees, standing, toPaise } from "../money";

export default function SettingsPage() {
  const couple = useQuery({ queryKey: keys.couple, queryFn: getCouple });
  if (couple.isPending) return <p className="loading" role="status">Opening settings…</p>;
  if (couple.isError) return <ErrorState message={errText(couple.error)} onRetry={() => void couple.refetch()} />;
  const c = couple.data;
  return (
    <div className="set-page">
      <h1>Settings</h1>
      <PartnerCard name={c.partner_name} waiting={c.waiting} />
      <CoversCard partner={c.partner_name} currency={c.currency} />
      <SettleCard partner={c.partner_name} currency={c.currency} />
      <LeaveCard partner={c.partner_name} />
    </div>
  );
}

function PartnerCard({ name, waiting }: { name: string | null; waiting: boolean }) {
  const [invite, setInvite] = useState<Invite | null>(null);
  const [err, setErr] = useState("");
  const { busy, run } = useBusy();
  // A code is issued only on a tap: each one replaces the last, so never on load.
  const getCode = () =>
    run(async () => {
      setErr("");
      try {
        setInvite(await inviteCode());
      } catch (e) {
        setErr(errText(e));
      }
    });
  return (
    <section className="set-card" aria-labelledby="partner-h">
      <h2 id="partner-h">Your partner</h2>
      {waiting ? (
        <>
          <p>Waiting for your partner to join.</p>
          {invite ? (
            <CodeCard invite={invite} />
          ) : (
            <div className="set-row">
              <button type="button" className="btn btn-stamp" disabled={busy} onClick={() => void getCode()}>
                Get a code to share
              </button>
            </div>
          )}
          <p className="set-msg">A new code stops older ones from working.</p>
        </>
      ) : (
        <p>
          Dates is shared with <strong>{name}</strong>.
        </p>
      )}
      {err && <p className="set-msg set-err" role="alert">{err}</p>}
    </section>
  );
}

function CoversCard({ partner, currency }: { partner: string | null; currency: string }) {
  const qc = useQueryClient();
  const q = useQuery({ queryKey: keys.summary, queryFn: getSummary });
  const loaded = q.data?.partner_paise;
  const [val, setVal] = useStateFrom(loaded !== undefined ? String(Math.round(loaded / 100)) : "", [loaded]);
  const [msg, setMsg] = useState("");
  const [err, setErr] = useState("");
  const { busy, run } = useBusy();

  const save = (e: React.FormEvent) => {
    e.preventDefault();
    const paise = toPaise(val);
    if (paise === null) {
      setErr("Enter a whole amount, like 2000.");
      setMsg("");
      return;
    }
    void run(async () => {
      setErr("");
      setMsg("");
      try {
        await setPartnerShare(paise);
        await Promise.all([qc.invalidateQueries({ queryKey: keys.summary }), qc.invalidateQueries({ queryKey: keys.settle })]);
        setMsg("Saved.");
      } catch (er) {
        setErr(errText(er));
      }
    });
  };

  // Back to the default: half the monthly budget when there are two of you.
  const useDefault = () => {
    void run(async () => {
      setErr("");
      setMsg("");
      try {
        await resetPartnerShare();
        await Promise.all([qc.invalidateQueries({ queryKey: keys.summary }), qc.invalidateQueries({ queryKey: keys.settle })]);
        setMsg("Back to the default: half the monthly budget.");
      } catch (er) {
        setErr(errText(er));
      }
    });
  };

  return (
    <form className="set-card" onSubmit={save} aria-labelledby="covers-h">
      <h2 id="covers-h">What {partner ?? "your partner"} covers each month</h2>
      <div className="field">
        <label htmlFor="covers">Amount each month</label>
        <div className="money">
          <span className="money-sign" aria-hidden="true">{currency}</span>
          <input
            id="covers"
            className="set-input"
            type="number"
            inputMode="numeric"
            min={0}
            step={1}
            value={val}
            onChange={(e) => setVal(e.target.value)}
            disabled={q.isPending}
            aria-describedby="covers-hint"
          />
        </div>
        <p className="hint soft" id="covers-hint">The rest of the monthly budget is yours. This sets your share when you settle up.</p>
      </div>
      <div className="set-row">
        <button type="submit" className="btn btn-stamp" disabled={busy || q.isPending}>Save</button>
        <button type="button" className="btn" disabled={busy || q.isPending} onClick={useDefault}>
          Use the default
        </button>
        <button
          type="button"
          className="btn"
          disabled={busy || q.isPending || loaded === undefined}
          onClick={() => {
            setVal(loaded !== undefined ? String(Math.round(loaded / 100)) : "");
            setMsg("");
            setErr("");
          }}
        >
          Revert
        </button>
      </div>
      <p className="set-msg" role="status">{msg}</p>
      {err && <p className="set-msg set-err" role="alert">{err}</p>}
    </form>
  );
}

function SettleCard({ partner, currency }: { partner: string | null; currency: string }) {
  const qc = useQueryClient();
  const q = useQuery({ queryKey: keys.settle, queryFn: getSettle });
  const balance = q.data?.balance_paise;
  const [amount, setAmount] = useStateFrom(balance !== undefined && balance !== 0 ? String(Math.abs(balance) / 100) : "", [balance]);
  const [err, setErr] = useState("");
  const [msg, setMsg] = useState("");
  const { busy, run } = useBusy();

  const record = (youPaid: boolean) => {
    const paise = toPaise(amount);
    if (!paise) {
      setErr("Enter how much, like 500.");
      setMsg("");
      return;
    }
    void run(async () => {
      setErr("");
      setMsg("");
      try {
        await recordSettlement(youPaid, paise);
        await qc.invalidateQueries({ queryKey: keys.settle });
        setMsg(youPaid ? `Noted: you paid ${partner ?? "them"} back ${rupees(paise, currency)}.` : `Noted: ${partner ?? "they"} paid you back ${rupees(paise, currency)}.`);
      } catch (e) {
        setErr(errText(e));
      }
    });
  };

  const removeOne = (id: number) => {
    void run(async () => {
      setErr("");
      setMsg("");
      try {
        await deleteSettlement(id);
        await qc.invalidateQueries({ queryKey: keys.settle });
        setMsg("Removed.");
      } catch (e) {
        setErr(errText(e));
      }
    });
  };

  const clearAll = () => {
    if (!window.confirm("Start settling up from scratch? Every repayment you have noted will be removed.")) return;
    void run(async () => {
      setErr("");
      setMsg("");
      try {
        await clearSettlements();
        await qc.invalidateQueries({ queryKey: keys.settle });
        setMsg("Cleared. You are settling up from scratch.");
      } catch (e) {
        setErr(errText(e));
      }
    });
  };

  return (
    <section className="set-card" aria-labelledby="settle-h">
      <h2 id="settle-h">Settle up</h2>
      {q.isPending && <p className="soft" role="status">Counting…</p>}
      {q.isError && <ErrorState message={errText(q.error)} onRetry={() => void q.refetch()} />}
      {q.data && (
        <>
          <p className="settle-big" role="status">{standing(q.data.balance_paise, currency).text}</p>
          <p className="soft">
            You paid {rupees(q.data.you_paid_paise, currency)}, {partner ?? "your partner"} paid {rupees(q.data.partner_paid_paise, currency)}. Your share is {Math.round(q.data.your_share_bps / 100)}%.
          </p>
          <div className="field">
            <label htmlFor="settle-amt">Amount paid back</label>
            <div className="money">
              <span className="money-sign" aria-hidden="true">{currency}</span>
              <input id="settle-amt" className="set-input" type="number" inputMode="decimal" min={0} step="any" value={amount} onChange={(e) => setAmount(e.target.value)} />
            </div>
          </div>
          <div className="set-row">
            <button type="button" className="btn" disabled={busy} onClick={() => record(false)}>They paid me back</button>
            <button type="button" className="btn" disabled={busy} onClick={() => record(true)}>I paid them back</button>
          </div>
          <p className="set-msg" role="status">{msg}</p>
          {err && <p className="set-msg set-err" role="alert">{err}</p>}
          {q.data.settlements.length > 0 && (
            <>
              <h3 className="soft" style={{ fontSize: "var(--step--1)" }}>Paid back so far</h3>
              <ul className="settle-list">
                {q.data.settlements.map((s) => (
                  <li key={s.id}>
                    <span>
                      {s.payer_is_you ? "You paid back" : `${partner ?? "They"} paid back`} {rupees(s.amount_paise, currency)}
                      {s.note ? `, ${s.note}` : ""}
                    </span>
                    <span className="soft">{s.on_day}</span>
                    <button type="button" className="btn" disabled={busy} onClick={() => window.confirm("Remove this repayment?") && removeOne(s.id)} aria-label={`Remove the repayment of ${rupees(s.amount_paise, currency)} on ${s.on_day}`}>
                      Remove
                    </button>
                  </li>
                ))}
              </ul>
              <div className="set-row">
                <button type="button" className="btn" disabled={busy} onClick={clearAll}>
                  Reset: clear all repayments
                </button>
              </div>
            </>
          )}
        </>
      )}
    </section>
  );
}

function LeaveCard({ partner }: { partner: string | null }) {
  const qc = useQueryClient();
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [err, setErr] = useState("");
  const { busy, run } = useBusy();
  const leave = () =>
    run(async () => {
      setErr("");
      try {
        await leaveCouple();
        qc.removeQueries({ queryKey: ["joy"] });
        await qc.invalidateQueries({ queryKey: keys.couple });
        navigate("/joy", { replace: true });
      } catch (e) {
        setErr(errText(e));
      }
    });
  return (
    <section className="set-card set-leave" aria-labelledby="leave-h" style={{ marginTop: "var(--space-8, 3rem)", borderTop: "2px dashed currentColor" }}>
      <h2 id="leave-h">Leave</h2>
      <p className="soft">Not something you need often. Leaving asks you to confirm first.</p>
      {!open ? (
        <div className="set-row">
          <button type="button" className="btn" aria-expanded="false" aria-controls="leave-box" onClick={() => setOpen(true)}>
            Leave our jar…
          </button>
        </div>
      ) : (
        <div className="leave-box" id="leave-box" role="group" aria-labelledby="leave-h">
          <p>
            <strong>This can’t be undone from here.</strong> Your place in the jar empties and you lose access to the slips, kept dates and settle-up history.
            {partner ? ` ${partner} keeps everything.` : ""} If nobody is left, Dates and everything in it is deleted for good. To come back you will need a new code from your partner.
          </p>
          <div className="set-row">
            <button type="button" className="btn btn-danger" disabled={busy} onClick={() => void leave()}>
              {partner ? `Leave and give ${partner} the jar` : "Yes, leave Dates"}
            </button>
            <button type="button" className="btn btn-quiet" disabled={busy} onClick={() => setOpen(false)}>
              Keep it
            </button>
          </div>
        </div>
      )}
      {err && <p className="set-msg set-err" role="alert">{err}</p>}
    </section>
  );
}
