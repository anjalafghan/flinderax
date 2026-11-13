import { useEffect, useRef, useState, type CSSProperties, type FormEvent } from "react";
import { addCustomDate, addSaving, addSpend, deleteSaving, deleteSpend, keepGoal, rollover, setBudget, setGoal } from "./api";
import { useKept } from "./ctx";
import { useStateFrom } from "../components/useStateFrom";
import { isBudgetHash, type RolloverChoice } from "./helpers";
import { buzz, reduced } from "./motion";
import type { HistoryView, SavingsState } from "./types";
import { useAction } from "./useAction";
import { useFold } from "./useFold";

/** The bar starts empty and fills to its value; later changes tween through the CSS transition. */
function useFill(pct: number): number {
  const [shown, setShown] = useState(reduced() ? pct : 0);
  useEffect(() => {
    const id = requestAnimationFrame(() => setShown(pct));
    return () => cancelAnimationFrame(id);
  }, [pct]);
  return shown;
}

const bar = (pct: number) => ({ "--pct": pct }) as CSSProperties;
const MONEY = { inputMode: "numeric", pattern: "[0-9, ]*", autoComplete: "off" } as const;

function Err({ text }: { text: string }) {
  return text ? (
    <p className="k-form-err" role="alert">
      {text}
    </p>
  ) : null;
}

function BudgetBar({ v }: { v: HistoryView }) {
  const { budget } = v;
  const fill = useFill(budget.pct);
  if (!budget.budget) {
    return (
      <div className="budget-line">
        <p>
          <strong>{budget.spent}</strong> spent on dates this month
        </p>
      </div>
    );
  }
  return (
    <div className={`budget-line${budget.over ? " is-over" : ""}`}>
      <p>
        <strong>{budget.spent}</strong> of {budget.budget} spent on dates this month
        {budget.over && <span className="k-worth hand"> · over budget, worth it</span>}
      </p>
      <span className="budget-bar" style={bar(fill)} aria-hidden="true" />
    </div>
  );
}

function RolloverForm({ v }: { v: HistoryView }) {
  const { finish } = useKept();
  const { busy, err, act } = useAction();
  const r = v.budget.rollover;
  if (!r) return null;
  const choose = (choice: RolloverChoice) => act(async () => finish(await rollover(choice), { focus: "#k-budget" }));
  return (
    <div className="k-budget-form">
      <p>
        {r.month} is over: {r.spent} of {r.budget}, so {r.left} left.
      </p>
      <p className="k-hint">Left over from {r.month}: choose what happens to it.</p>
      <div className="k-inline">
        <button className="k-btn" type="button" disabled={busy} onClick={() => void choose("roll")}>
          Roll {r.left} over
        </button>
        {v.savings.goal && (
          <button className="k-btn" type="button" disabled={busy} onClick={() => void choose("save")}>
            Put it in the savings jar
          </button>
        )}
        <button className="k-btn" type="button" disabled={busy} onClick={() => void choose("fresh")}>
          Start fresh
        </button>
      </div>
      <Err text={err} />
    </div>
  );
}

function BudgetEdit({ v, hash }: { v: HistoryView; hash: string }) {
  const { finish } = useKept();
  const { busy, err, act } = useAction();
  const input = useRef<HTMLInputElement>(null);
  const summary = useRef<HTMLElement>(null);
  const fold = useFold(false, () => input.current?.focus());
  const [amount, setAmount] = useStateFrom(v.budget.budget_raw, [v.budget.budget_raw]);

  // Landing on #budget opens the editor and puts the cursor in it.
  const { setOpen } = fold;
  useEffect(() => {
    if (!isBudgetHash(hash)) return;
    setOpen(true);
    requestAnimationFrame(() => {
      input.current?.focus({ preventScroll: true });
      input.current?.closest("details")?.scrollIntoView({ block: "center", behavior: reduced() ? "auto" : "smooth" });
    });
  }, [hash, setOpen]);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    void act(async () => {
      const go = await setBudget(amount);
      fold.setOpen(false);
      await finish(go, { focus: "#k-budget-edit-summary" });
    });
  };
  return (
    <details className="k-budget-edit" id="k-budget-edit" {...fold.props}>
      <summary ref={summary} id="k-budget-edit-summary">
        {v.budget.budget ? "Change budget" : "Set a monthly budget"}
      </summary>
      <form className="k-budget-form" onSubmit={submit}>
        <label className="k-label" htmlFor="budget">
          Monthly budget for dates
        </label>
        <div className="k-inline">
          <input
            ref={input}
            className="k-field"
            id="budget"
            name="amount"
            type="text"
            {...MONEY}
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="e.g. 6000"
            aria-describedby="budget-hint"
          />
          <button className="k-btn" type="submit" disabled={busy}>
            Save
          </button>
          <button className="k-btn" type="button" disabled={busy} onClick={() => setAmount(v.budget.budget_raw)}>
            Revert
          </button>
        </div>
        <p className="k-hint" id="budget-hint">
          Leave it empty for no budget. It’s a guide, not a rule.
        </p>
        <Err text={err} />
      </form>
    </details>
  );
}

/** The kinds a custom date can be, as the jar names them. */
const KINDS: [string, string][] = [
  ["local", "Around town"],
  ["home", "At home, together"],
  ["day_trip", "Day trip"],
  ["travel", "Travel"],
  ["apart", "Long distance"],
];

/** A date we had that never was a slip. It lands in Kept, ready for photos, a rating and lines. */
export function CustomDateFold({ fold }: { fold: ReturnType<typeof useFold> }) {
  const { finish } = useKept();
  const { busy, err, act } = useAction();
  const [title, setTitle] = useState("");
  const [place, setPlace] = useState("");
  const [setting, setSetting] = useState("local");
  const [on, setOn] = useState("");
  const [cost, setCost] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    void act(async () => {
      const go = await addCustomDate({ title, place, setting, on, cost });
      setTitle("");
      setPlace("");
      setOn("");
      setCost("");
      fold.setOpen(false);
      await finish(go);
    });
  };
  return (
    <details className="k-budget-edit" id="k-add" {...fold.props}>
      <summary>Add a date we had</summary>
      <form className="k-budget-form" onSubmit={submit}>
        <label className="k-label" htmlFor="custom-title">
          What did you do?
        </label>
        <input className="k-field" id="custom-title" name="title" type="text" maxLength={80} required autoComplete="off" value={title} onChange={(e) => setTitle(e.target.value)} placeholder="e.g. Picnic at the lake" />
        <div className="k-inline">
          <input className="k-field" name="place" type="text" maxLength={80} autoComplete="off" value={place} onChange={(e) => setPlace(e.target.value)} placeholder="Where? (optional)" aria-label="Where" />
          <select className="k-field" name="setting" value={setting} onChange={(e) => setSetting(e.target.value)} aria-label="Kind of date">
            {KINDS.map(([k, l]) => (
              <option key={k} value={k}>
                {l}
              </option>
            ))}
          </select>
        </div>
        <div className="k-inline">
          <input className="k-field" name="on" type="date" value={on} onChange={(e) => setOn(e.target.value)} aria-label="When (blank is today)" />
          <input className="k-field" name="cost" type="text" {...MONEY} value={cost} onChange={(e) => setCost(e.target.value)} placeholder="Cost (optional)" aria-label="What it cost" />
          <button className="k-btn" type="submit" disabled={busy}>
            Keep it
          </button>
          <button
            className="k-btn"
            type="button"
            disabled={busy}
            onClick={() => {
              setTitle("");
              setPlace("");
              setSetting("local");
              setOn("");
              setCost("");
            }}
          >
            Clear
          </button>
        </div>
        <p className="k-hint">It goes straight into Kept for photos, a rating and your lines. It won’t be added to the jar.</p>
        <Err text={err} />
      </form>
    </details>
  );
}

function OtherSpendFold({ v }: { v: HistoryView }) {
  const { finish } = useKept();
  const { busy, err, act } = useAction();
  const fold = useFold(v.other_spend.length > 0);
  const [amount, setAmount] = useState("");
  const [note, setNote] = useState("");
  const [on, setOn] = useState("");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    void act(async () => {
      const go = await addSpend(amount, note, on);
      setAmount("");
      setNote("");
      setOn("");
      await finish(go, { focus: "#k-spend-list" });
    });
  };
  return (
    <details className="k-budget-edit" {...fold.props}>
      <summary>Add spending that wasn’t a slip</summary>
      <form className="k-budget-form" onSubmit={submit}>
        <label className="k-label" htmlFor="spend-amount">
          What we spent
        </label>
        <div className="k-inline">
          <input className="k-field" id="spend-amount" name="amount" type="text" {...MONEY} required value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="e.g. 1800" />
          <input className="k-field" name="note" type="text" maxLength={80} autoComplete="off" value={note} onChange={(e) => setNote(e.target.value)} placeholder="Where? (optional)" aria-label="Where" />
          <input className="k-field" name="on" type="date" value={on} onChange={(e) => setOn(e.target.value)} aria-label="Date (blank is today)" />
          <button className="k-btn" type="submit" disabled={busy}>
            Add
          </button>
          <button
            className="k-btn"
            type="button"
            disabled={busy}
            onClick={() => {
              setAmount("");
              setNote("");
              setOn("");
            }}
          >
            Clear
          </button>
        </div>
        <p className="k-hint">Counts toward this month’s budget. Leave the date blank for today.</p>
        <Err text={err} />
      </form>
      {v.other_spend.length > 0 && (
        <ul className="k-hint" id="k-spend-list" tabIndex={-1}>
          {v.other_spend.map((o) => (
            <li key={o.id}>
              {o.on} · {o.amount}
              {o.note && ` · ${o.note}`}{" "}
              <button
                className="k-btn"
                type="button"
                disabled={busy}
                aria-label={`Remove ${o.amount}`}
                onClick={() => window.confirm("Delete this spend?") && void act(async () => finish(await deleteSpend(o.id), { focus: "#k-spend-list, #k-budget" }))}
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}
    </details>
  );
}

function Savings({ s }: { s: SavingsState }) {
  const { finish } = useKept();
  const goalAct = useAction();
  const putAct = useAction();
  const keepAct = useAction();
  const goalFold = useFold();
  const putFold = useFold();
  const [goal, setGoalId] = useStateFrom(s.choices.find((c) => c.selected)?.value ?? "", [s.choices]);
  const [amount, setAmount] = useState("");
  const [note, setNote] = useState("");
  const g = s.goal;
  const fill = useFill(g?.pct ?? 0);

  const keep = () =>
    keepAct.act(async () => {
      const go = await keepGoal();
      // the stamp thumps down onto the slip it just made
      await finish(go, {
        after: () => {
          const hit = /#(k\d+)$/.exec(go.to);
          const stamp = hit ? document.querySelector<HTMLElement>(`#${hit[1]} .k-stamp`) : null;
          buzz(12);
          if (stamp && !reduced() && "animate" in stamp) {
            stamp.animate(
              [
                { opacity: 0, transform: "scale(1.8) rotate(-10deg)" },
                { opacity: 1, transform: "scale(1) rotate(0)" },
              ],
              { duration: 480, delay: 200, easing: "cubic-bezier(.34,1.5,.64,1)", fill: "backwards" },
            );
          }
        },
      });
    });

  return (
    <section className="k-savings" id="k-savings" aria-labelledby="k-savings-h">
      <h2 className="k-savings-h" id="k-savings-h" tabIndex={-1}>
        The savings jar
      </h2>
      {g ? (
        <>
          <div className={`budget-line${g.ready ? " is-ready" : ""}`}>
            <p>
              Saving for <strong>{g.title}</strong>: {s.saved} of {g.cost}
            </p>
            <span className="budget-bar" style={bar(fill)} aria-hidden="true" />
          </div>
          {g.ready ? (
            <div className="k-budget-form">
              <p className="k-insight">It’s saved up. Time to go.</p>
              <button className="k-btn k-btn-stamp" type="button" disabled={keepAct.busy} onClick={() => void keep()}>
                It’s a date
              </button>
              <p className="k-hint">Stamps it into Kept and pays {g.cost} from the jar. That part won’t count against the month it happens in.</p>
              <Err text={keepAct.err} />
            </div>
          ) : (
            <p className="k-hint">{g.to_go}. A month’s leftover can go in here too.</p>
          )}
        </>
      ) : (
        <p className="k-hint">{s.saved_raw > 0 && `${s.saved} put aside. `}Pick something to save for, then put money aside until it’s covered.</p>
      )}
      <p className="k-hint">The savings jar is for a slip too big for one month, like a trip. A month’s leftover can go in too.</p>
      <details className="k-budget-edit" {...goalFold.props}>
        <summary>{g ? "Save for something else" : "Choose what to save for"}</summary>
        <form
          className="k-budget-form"
          onSubmit={(e) => {
            e.preventDefault();
            void goalAct.act(async () => {
              const go = await setGoal(goal);
              goalFold.setOpen(false);
              await finish(go);
            });
          }}
        >
          <label className="k-label" htmlFor="goal">
            Saving for
          </label>
          <div className="k-inline">
            <select className="k-field k-goal" id="goal" name="id" value={goal} onChange={(e) => setGoalId(e.target.value)}>
              <option value="">Nothing for now</option>
              {s.choices.map((c) => (
                <option key={c.value} value={c.value}>
                  {c.label}
                </option>
              ))}
            </select>
            <button className="k-btn" type="submit" disabled={goalAct.busy}>
              Save
            </button>
          </div>
          <p className="k-hint">Changing it keeps what’s in the jar.</p>
          <Err text={goalAct.err} />
        </form>
      </details>
      <details className="k-budget-edit" {...putFold.props}>
        <summary>Put money in</summary>
        <form
          className="k-budget-form"
          onSubmit={(e) => {
            e.preventDefault();
            void putAct.act(async () => {
              const go = await addSaving(amount, note);
              setAmount("");
              setNote("");
              await finish(go, { focus: "#k-saving-list, #k-savings-h" });
            });
          }}
        >
          <label className="k-label" htmlFor="save-amount">
            How much
          </label>
          <div className="k-inline">
            <input className="k-field" id="save-amount" name="amount" type="text" {...MONEY} required value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="e.g. 2000" />
            <input className="k-field" name="note" type="text" maxLength={80} autoComplete="off" value={note} onChange={(e) => setNote(e.target.value)} placeholder="Note (optional)" aria-label="Note" />
            <button className="k-btn" type="submit" disabled={putAct.busy}>
              Add
            </button>
          </div>
          <p className="k-hint">It doesn’t count as spending this month.</p>
          <Err text={putAct.err} />
        </form>
        {s.recent.length > 0 && (
          <ul className="k-hint" id="k-saving-list" tabIndex={-1}>
            {s.recent.map((m) => (
              <li key={m.id}>
                {m.on} · {m.amount}
                {m.note && ` · ${m.note}`}{" "}
                {m.removable && (
                  <button
                    className="k-btn"
                    type="button"
                    disabled={putAct.busy}
                    aria-label={`Take back ${m.amount}`}
                    onClick={() => window.confirm("Delete this saving?") && void putAct.act(async () => finish(await deleteSaving(m.id), { focus: "#k-saving-list, #k-savings-h" }))}
                  >
                    Take back
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}
      </details>
    </section>
  );
}

/** `.k-budget`: this month's budget, its editor, past months, other spending, and the savings jar. */
export function Budget({ v, hash }: { v: HistoryView; hash: string }) {
  return (
    <div className="k-budget" id="k-budget" tabIndex={-1}>
      <BudgetBar v={v} />
      {v.budget.carry_note && <p className="k-hint">{v.budget.carry_note}</p>}
      <RolloverForm v={v} />
      {v.cost_insight && <p className="k-insight">{v.cost_insight}</p>}
      <BudgetEdit v={v} hash={hash} />
      {v.past.length > 0 && (
        <details className="k-budget-edit">
          <summary>Past months</summary>
          <ul className="k-hint">
            {v.past.map((m) => (
              <li key={m.label}>
                {m.label} · {m.spent}
                {m.budget && ` of ${m.budget}`}
              </li>
            ))}
          </ul>
        </details>
      )}
      <OtherSpendFold v={v} />
      <Savings s={v.savings} />
    </div>
  );
}
