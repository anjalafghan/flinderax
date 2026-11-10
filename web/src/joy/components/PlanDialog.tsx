import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { jarApi } from "../jar/api";
import { planSlip, type ActivityView, type SlipPlanned } from "../api";
import { mapGoTo } from "../paths";
import { quickPick, toLocalInput, type WhenKey } from "../plan";
import { errText, useBusy } from "./useBusy";

const QUICK: { key: WhenKey; label: string }[] = [
  { key: "tonight", label: "Tonight" },
  { key: "tomorrow", label: "Tomorrow evening" },
  { key: "saturday", label: "Saturday" },
];

/** A date that is already kept (just stamped "It's a date"): put a day on it instead of keeping the slip again. */
export interface KeptDate {
  /** Comma-separated history ids (two for a pair). */
  ids: string;
  partner: string;
  wa_url: string;
  share_text: string;
}

/** "Plan it for a day": pick a time, then send the plan on WhatsApp. Native <dialog>, so focus is trapped for free. */
export function PlanDialog({
  slip,
  kept,
  onPlanned,
  onClose,
}: {
  slip: Pick<ActivityView, "id" | "title"> | null;
  /** Given when the date is already kept: the day is set on it rather than creating a second date. */
  kept?: KeptDate;
  onPlanned?: (r: SlipPlanned) => void;
  onClose: (planned: boolean) => void;
}) {
  const dlg = useRef<HTMLDialogElement>(null);
  const at = useRef<HTMLInputElement>(null);
  const done = useRef<HTMLDivElement>(null);
  const [when, setWhen] = useState(() => (slip ? toLocalInput(quickPick("tonight")) : ""));
  const [picked, setPicked] = useState<WhenKey | null>("tonight");
  const [err, setErr] = useState("");
  const [result, setResult] = useState<SlipPlanned | null>(null);
  const [seenSlip, setSeenSlip] = useState(slip);
  const { busy, run } = useBusy();

  // A new slip starts the form afresh. Done during render rather than in an effect.
  if (slip !== seenSlip) {
    setSeenSlip(slip);
    if (slip) {
      setResult(null);
      setErr("");
      setPicked("tonight");
      setWhen(toLocalInput(quickPick("tonight")));
    }
  }

  useEffect(() => {
    const d = dlg.current;
    if (!d) return;
    if (slip) {
      if (!d.open) d.showModal();
      at.current?.focus({ preventScroll: true });
    } else if (d.open) {
      d.close();
    }
  }, [slip]);

  useEffect(() => {
    if (result) done.current?.focus();
  }, [result]);

  const choose = (k: WhenKey) => {
    setWhen(toLocalInput(quickPick(k)));
    setPicked(k);
    setErr("");
  };

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!slip) return;
    if (!when) {
      setErr("Pick a day and a time.");
      return;
    }
    void run(async () => {
      setErr("");
      try {
        let r: SlipPlanned;
        if (kept) {
          const w = await jarApi.when(kept.ids, when);
          r = {
            title: slip.title,
            label: w.label,
            ics_url: w.ics_url,
            partner: kept.partner,
            wa_url: kept.wa_url,
            share_text: kept.share_text,
            history_id: Number(kept.ids.split(",")[0]),
            to: `/joy/kept#k${kept.ids.split(",")[0]}`,
          };
        } else {
          r = await planSlip(slip.id, when);
        }
        setResult(r);
        onPlanned?.(r);
      } catch (er) {
        setErr(errText(er, "That didn’t save. Try again."));
      }
    });
  };

  const share = (e: React.MouseEvent<HTMLAnchorElement>) => {
    if (!result || !navigator.share) return;
    e.preventDefault();
    navigator.share({ text: result.share_text }).catch((er: unknown) => {
      if (er && (er as { name?: string }).name === "AbortError") return;
      window.open(result.wa_url, "_blank", "noopener");
    });
  };

  const close = () => dlg.current?.close();

  return (
    <dialog
      ref={dlg}
      className="plan-dlg"
      id="plan-dlg"
      aria-labelledby="plan-h"
      onClose={() => onClose(result !== null)}
      onClick={(e) => {
        if (e.target === dlg.current) close();
      }}
    >
      <div className="plan-body">
        {result ? (
          <div className="plan-done" tabIndex={-1} ref={done}>
            <p className="plan-done-label" role="status">
              <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="3.6" y="5.2" width="16.8" height="15" rx="2" />
                <path d="M3.8 9.8h16.4M8 3.4v3.6M16 3.4v3.6" />
                <path d="M9 14.6l2 2 4-4.2" />
              </svg>
              <span>
                <strong>{result.title}</strong> is planned for <strong>{result.label}</strong>.
              </span>
            </p>
            <div className="plan-done-acts">
              <a className="btn btn-stamp plan-send" href={result.wa_url} onClick={share} target="_blank" rel="noopener">
                <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M20.6 3.6 3.4 10.4c-.7.3-.6 1.2.1 1.4l6.3 1.9 1.9 6.3c.2.7 1.1.8 1.4.1z" />
                  <path d="M9.8 13.7 20.6 3.6" />
                </svg>
                Send {result.partner} the plan on WhatsApp
              </a>
              <a className="btn" href={result.ics_url} download>
                Add to calendar
              </a>
            </div>
            <p className="plan-see">
              <Link to={mapGoTo(result.to)} onClick={close}>See it with our dates</Link>
            </p>
            <button className="btn btn-quiet plan-close" type="button" onClick={close}>
              Done
            </button>
          </div>
        ) : (
          <form className="plan-form" onSubmit={submit}>
            <p className="plan-kicker">Plan it for a day</p>
            <h2 className="plan-h" id="plan-h">{slip?.title}</h2>
            <div className="when-row">
              <label className="when-l">
                <span className="when-q">When is it?</span>
                <input
                  ref={at}
                  className="when-at"
                  type="datetime-local"
                  name="at"
                  required
                  value={when}
                  onChange={(e) => {
                    setWhen(e.target.value);
                    setPicked(null);
                  }}
                />
              </label>
            </div>
            <div className="plan-quick" role="group" aria-label="Quick picks">
              {QUICK.map((c) => (
                <button key={c.key} type="button" className="plan-chip" aria-pressed={picked === c.key} onClick={() => choose(c.key)}>
                  {c.label}
                </button>
              ))}
            </div>
            <p className="when-err" role="status">{err}</p>
            <div className="plan-acts">
              <button className="btn btn-stamp" type="submit" disabled={busy}>
                Plan it
              </button>
              <button className="btn btn-quiet plan-close" type="button" onClick={close}>
                Not now
              </button>
            </div>
          </form>
        )}
      </div>
    </dialog>
  );
}
