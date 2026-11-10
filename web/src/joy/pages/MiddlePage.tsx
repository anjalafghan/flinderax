import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { checkIn, getJarMiddle, keys, type MiddleBody, type MiddleView } from "../api";
import { errText, useBusy } from "../components/useBusy";

export default function MiddlePage() {
  const q = useQuery({ queryKey: [...keys.jar, "middle"], queryFn: getJarMiddle, retry: false });
  if (q.isPending) return <p className="loading" role="status">Opening the jar…</p>;
  if (q.isError) return <p className="kind-error" role="alert">{errText(q.error)}</p>;
  const m = q.data.middle;
  if (!m.enabled) {
    return (
      <section className="middle grain" aria-labelledby="middle-h">
        <h2 className="middle-h" id="middle-h">Check in together</h2>
        <p className="middle-p">Check-ins aren’t available yet.</p>
      </section>
    );
  }
  return <Middle m={m} />;
}

function Middle({ m }: { m: MiddleView }) {
  const qc = useQueryClient();
  const pick = (opts: { value: string; selected: boolean }[], fallback = "") => opts.find((o) => o.selected)?.value ?? fallback;
  const [energy, setEnergy] = useState(pick(m.energies, "med"));
  const [budget, setBudget] = useState(pick(m.budgets));
  const [time, setTime] = useState(pick(m.times));
  const [reach, setReach] = useState(pick(m.reaches));
  const [vibe, setVibe] = useState(pick(m.vibes));
  const [open, setOpen] = useState(!m.mine && m.partner_in);
  const [err, setErr] = useState("");
  const [said, setSaid] = useState("");
  const { busy, run } = useBusy();

  const send = (body: MiddleBody, saidText: string) =>
    run(async () => {
      setErr("");
      try {
        await checkIn(body);
        await qc.invalidateQueries({ queryKey: keys.jar });
        setSaid(saidText);
      } catch (e) {
        setErr(errText(e));
      }
    });

  return (
    <section className={`middle grain${m.both_in ? " is-met" : ""}`} id="middle" aria-labelledby="middle-h">
      <svg className="middle-mark" viewBox="0 0 64 28" aria-hidden="true" focusable="false">
        <circle cx="22" cy="14" r="11" />
        <circle cx="42" cy="14" r="11" />
        <path className="middle-lens" d="M32 5.6a11 11 0 0 1 0 16.8 11 11 0 0 1 0-16.8z" />
      </svg>
      <h2 className="middle-h" id="middle-h">Check in together</h2>
      {m.both_in ? (
        <p className="middle-p">You've both told the jar how tonight feels. Use "Draw for both of us" on the Jar page and it picks only what fits you both. Neither of you sees the other's answers.</p>
      ) : m.mine ? (
        <>
          <p className="middle-p">You're checked in. Waiting for {m.partner}: when they open the jar, they'll see it's their turn. Your answers stay yours.</p>
          <p className="middle-ask">
            <a href={m.wa_url} target="_blank" rel="noopener noreferrer">
              Ask {m.partner} to check in<span className="visually-hidden"> (opens WhatsApp)</span>
            </a>
          </p>
        </>
      ) : m.partner_in ? (
        <p className="middle-p"><strong>{m.partner} has checked in for tonight.</strong> Add yours and the jar draws from where you meet. You won't see their answers, and they won't see yours.</p>
      ) : (
        <p className="middle-p">Can't decide together? Each of you tells the jar how tonight feels, on your own phone. It draws from the overlap, and neither of you sees the other's answers.</p>
      )}

      <details className="middle-form-wrap" open={open} onToggle={(e) => setOpen(e.currentTarget.open)}>
        <summary className="middle-toggle">{m.mine ? "Change my answers" : "Check in for tonight"}</summary>
        <form
          className="middle-form"
          onSubmit={(e) => {
            e.preventDefault();
            void send({ energy, budget, max_duration: time, reach, vibe }, m.mine ? "Saved your answers." : "You’re checked in.");
          }}
        >
          <fieldset className="middle-q">
            <legend>Energy</legend>
            {m.energies.map((o) => (
              <label key={o.value} className="pill"><input type="radio" name="energy" value={o.value} checked={energy === o.value} onChange={() => setEnergy(o.value)} /><span>{o.label}</span></label>
            ))}
          </fieldset>
          <fieldset className="middle-q">
            <legend>Spend, for both of us</legend>
            {m.budgets.map((o) => (
              <label key={o.value} className="pill"><input type="radio" name="budget" value={o.value} checked={budget === o.value} onChange={() => setBudget(o.value)} /><span>{o.value === "" || o.value === "0" ? o.label : `up to ${o.label}`}</span></label>
            ))}
          </fieldset>
          <fieldset className="middle-q">
            <legend>Time</legend>
            {m.times.map((o) => (
              <label key={o.value} className="pill"><input type="radio" name="max_duration" value={o.value} checked={time === o.value} onChange={() => setTime(o.value)} /><span>{o.label}</span></label>
            ))}
          </fieldset>
          <fieldset className="middle-q">
            <legend>How far</legend>
            {m.reaches.map((o) => (
              <label key={o.value} className="pill"><input type="radio" name="reach" value={o.value} checked={reach === o.value} onChange={() => setReach(o.value)} /><span>{o.label}</span></label>
            ))}
          </fieldset>
          <fieldset className="middle-q">
            <legend>Mood</legend>
            <label className="pill"><input type="radio" name="vibe" value="" checked={vibe === ""} onChange={() => setVibe("")} /><span>any</span></label>
            {m.vibes.map((o) => (
              <label key={o.value} className="pill"><input type="radio" name="vibe" value={o.value} checked={vibe === o.value} onChange={() => setVibe(o.value)} /><span>{o.label}</span></label>
            ))}
          </fieldset>
          <div className="middle-actions">
            <button className="btn btn-ink" type="submit" disabled={busy}>{m.mine ? "Save my answers" : "Check in"}</button>
            {m.mine && (
              <button className="middle-clear" type="button" disabled={busy} onClick={() => void send({ clear: "1" }, "Took your answers back.")}>
                Take mine back
              </button>
            )}
          </div>
          <p className="middle-fine">Kept until tonight ends, then deleted.</p>
          <p className="set-msg" role="status">{said}</p>
          {err && <p className="set-msg set-err" role="alert">{err}</p>}
        </form>
      </details>
    </section>
  );
}
