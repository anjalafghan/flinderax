// The next date's plan (Joy's dayplan.html): the day as a timeline, Plan B, and the little jobs.
// Your own jobs tick off (POST /tasks/{id}/done); your partner's are shown but can't be ticked from here.
// Swapping to Plan B posts to /history/{id}/plan-b. A failure of either is a quiet line, not a crash.
import { useRef, useState } from "react";
import type { DayPlan, StepView } from "./api";
import { jarApi } from "./api";

export function Route({ steps, className = "dp-route" }: { steps: StepView[]; className?: string }) {
  return (
    <ol className={className}>
      {steps.map((st, i) => (
        <li key={`${st.minutes}-${i}`} className={`dp-step${st.mine ? " is-mine" : ""}`} style={{ "--i": i } as React.CSSProperties}>
          <span className="dp-at">{st.at}</span>
          <span className="dp-what">{st.what}</span>
        </li>
      ))}
    </ol>
  );
}

const Rain = () => (
  <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M7.2 15.4a4.3 4.3 0 0 1-.4-8.5 5.4 5.4 0 0 1 10.3 1.4 3.6 3.6 0 0 1 .3 7.1" />
    <path d="M9 18.2l-.9 2.2M13 17.6l-.9 2.6M16.6 18.2l-.8 2" />
  </svg>
);

export default function DayPlanView({ plan, onSwapped }: { plan: DayPlan; onSwapped?: () => void }) {
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const [note, setNote] = useState("");
  const [ticks, setTicks] = useState<Record<number, boolean>>({});
  const [jobNote, setJobNote] = useState("");
  const id = `plan-${plan.history_id}`;

  const swap = async () => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setNote("");
    try {
      await jarApi.planB(plan.history_id);
      setNote("Swapped. Plan B is on.");
      onSwapped?.();
    } catch {
      setNote("Couldn't swap to Plan B. Try again.");
    } finally {
      lock.current = false;
      setBusy(false);
    }
  };

  const tick = async (taskId: number, done: boolean) => {
    setJobNote("");
    setTicks((t) => ({ ...t, [taskId]: done })); // show it at once; undone below if it didn't save
    try {
      await jarApi.taskDone(taskId, done);
      onSwapped?.(); // the page refreshes the plan (and the "things for you to do" count)
    } catch {
      setTicks((t) => Object.fromEntries(Object.entries(t).filter(([k]) => Number(k) !== taskId)));
      setJobNote("Couldn't tick that off. Try again.");
    }
  };

  return (
    <section className={`dayplan grain${plan.sealed ? " is-sealed" : ""}`} id={id} aria-labelledby={`${id}-h`}>
      <p className="dp-when">{plan.when}</p>
      <h3 className="dp-title" id={`${id}-h`}>
        {plan.title}
      </h3>
      {plan.rain != null && (
        <p className="dp-rain" role="status">
          <Rain />
          <span>Rain likely ({plan.rain}%) and this one's outdoors.</span>
        </p>
      )}
      {plan.steps.length > 0 && <Route steps={plan.steps} />}
      {plan.tight && <p className="dp-note">A short one: the jar trimmed it so you both get home in time.</p>}
      {plan.hours_note && <p className="dp-note">{plan.hours_note}</p>}
      {plan.after && (
        <p className="dp-after">
          <span className="dp-hand">after, if you're hungry:</span> {plan.after}
        </p>
      )}

      {plan.tasks.length > 0 && (
        <div className="dp-jobs">
          <p className="dp-hand">{plan.sealed ? "your part of the surprise" : "dealt to you two"}</p>
          <ul>
            {plan.tasks.map((t) => {
              const done = ticks[t.id] ?? t.done;
              return (
                <li key={t.id} className={`dp-job${done ? " is-done" : ""}${t.mine ? " is-mine" : ""}`}>
                  <label className="dp-theirs dp-tick" title={t.mine ? undefined : `That one is ${t.whose}'s`}>
                    <input type="checkbox" checked={done} disabled={!t.mine} onChange={(e) => void tick(t.id, e.target.checked)} />
                    <span className="dp-theirs-text">
                      <span className="dp-job-t">{t.task}</span>
                      <span className="dp-due">
                        {[t.mine ? "you" : t.whose, t.due, done ? "done" : ""].filter(Boolean).join(" · ")}
                      </span>
                    </span>
                  </label>
                </li>
              );
            })}
          </ul>
          {jobNote && (
            <p className="dp-note" role="status">
              {jobNote}
            </p>
          )}
        </div>
      )}

      {plan.plan_b && (
        <div className="dp-planb">
          <p>
            <span className="dp-hand">plan B, folded in:</span> {plan.plan_b.title}
            {plan.plan_b.place && <span className="dp-soft"> · {plan.plan_b.place}</span>}
          </p>
          <button className={`btn ${plan.rain != null ? "btn-stamp" : "btn-quiet"}`} type="button" disabled={busy} onClick={swap}>
            Swap to Plan B
          </button>
          {note && (
            <p className="dp-note" role="status">
              {note}
            </p>
          )}
        </div>
      )}
      {plan.ics_url && (
        <p className="dp-links">
          <a href={plan.ics_url}>
            Add to calendar<span className="visually-hidden"> (with a reminder when it's time to leave)</span>
          </a>
        </p>
      )}
    </section>
  );
}
