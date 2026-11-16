import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";

import { QUEUED_NOTE } from "@/offline";
import { budgetKey, weddingApi, type Budget, type BudgetItem, type ItemInput } from "@/services/wedding";
import { formatDay } from "@/utils/dates";
import { formatINR, toPaise } from "@/utils/money";

const PANEL = "rounded-[28px] border border-border bg-card px-5 py-5 sm:px-8";
const FIELD = "rounded border border-border bg-background px-2 py-1";

const BANNER: Record<Budget["verdict"]["status"], string> = {
  on_track: "border-border bg-card",
  tight: "border-amber-500/60 bg-amber-500/10",
  short: "border-destructive/60 bg-destructive/10",
};

const label = (c: string) => c.charAt(0).toUpperCase() + c.slice(1);

/** One line at the top: whether what is set aside covers every payment by its date. */
function Verdict({ b }: { b: Budget }) {
  const v = b.verdict;
  return (
    <section role="status" aria-label="Budget verdict" className={`rounded-[28px] border px-5 py-4 sm:px-8 ${BANNER[v.status]}`}>
      <p className="m-0 text-lg font-medium">{v.line}</p>
      <p className="m-0 mt-1 text-sm text-muted-foreground">
        {formatINR(b.on_hand_paise)} set aside and not yet paid out
        {b.expected_monthly_paise > 0 ? `, and about ${formatINR(b.expected_monthly_paise)} a month going in` : ""}.
        {b.next_due ? ` Next payment: ${b.next_due.item_name}, ${formatINR(b.next_due.amount_paise)} on ${formatDay(b.next_due.due_on)}.` : ""}
      </p>
    </section>
  );
}

interface ItemFormProps {
  categories: string[];
  initial?: BudgetItem;
  busy: boolean;
  submitLabel: string;
  onSubmit: (i: ItemInput) => void;
  onError: (m: string) => void;
}

function ItemForm({ categories, initial, busy, submitLabel, onSubmit, onError }: ItemFormProps) {
  const [f, setF] = useState({
    name: initial?.name ?? "",
    category: initial?.category ?? "venue",
    estimate: initial?.estimate_input ?? "",
    quote: initial?.quote_input ?? "",
    vendor: initial?.vendor ?? "",
  });
  const go = () => {
    const estimate = f.estimate.trim() === "" ? 0 : toPaise(f.estimate);
    const quote = f.quote.trim() === "" ? null : toPaise(f.quote);
    if (!f.name.trim()) return onError("Give the item a name.");
    if (estimate === null || estimate < 0 || (quote !== null && quote < 0)) return onError("Enter amounts in rupees, like 50000.");
    onError("");
    onSubmit({ name: f.name.trim(), category: f.category, estimate_paise: estimate, quote_paise: quote, vendor: f.vendor.trim(), notes: initial?.notes ?? "" });
    if (!initial) setF({ ...f, name: "", estimate: "", quote: "", vendor: "" });
  };
  const id = initial?.item_id ?? "new";
  return (
    <div className="mt-2 grid gap-2 text-sm">
      <label>
        Item
        <input aria-label={`Item name ${id}`} className={`ml-2 ${FIELD}`} value={f.name} onChange={(e) => setF({ ...f, name: e.target.value })} />
      </label>
      <label>
        Category
        <select aria-label={`Category ${id}`} className={`ml-2 ${FIELD}`} value={f.category} onChange={(e) => setF({ ...f, category: e.target.value })}>
          {categories.map((c) => (
            <option key={c} value={c}>
              {label(c)}
            </option>
          ))}
        </select>
      </label>
      <label>
        Estimate (₹)
        <input aria-label={`Estimate ${id}`} inputMode="decimal" className={`ml-2 w-32 ${FIELD}`} value={f.estimate} onChange={(e) => setF({ ...f, estimate: e.target.value })} />
      </label>
      <label>
        Quote (₹, if you have one)
        <input aria-label={`Quote ${id}`} inputMode="decimal" className={`ml-2 w-32 ${FIELD}`} value={f.quote} onChange={(e) => setF({ ...f, quote: e.target.value })} />
      </label>
      <label>
        Vendor
        <input aria-label={`Vendor ${id}`} className={`ml-2 ${FIELD}`} value={f.vendor} onChange={(e) => setF({ ...f, vendor: e.target.value })} />
      </label>
      <div>
        <button type="button" className="btn-why" disabled={busy} onClick={go}>
          {submitLabel}
        </button>
      </div>
    </div>
  );
}

function AddPayment({ item, busy, onAdd, onError }: { item: BudgetItem; busy: boolean; onAdd: (m: { due_on: string | null; amount_paise: number }) => void; onError: (m: string) => void }) {
  const [due, setDue] = useState("");
  const [amount, setAmount] = useState("");
  const go = () => {
    const paise = toPaise(amount);
    if (!paise || paise <= 0) return onError("Enter the payment amount in rupees, like 25000.");
    onError("");
    onAdd({ due_on: due || null, amount_paise: paise });
    setAmount("");
    setDue("");
  };
  return (
    <span className="flex flex-wrap items-center gap-2">
      <input type="date" aria-label={`Due date for ${item.name}`} className={FIELD} value={due} onChange={(e) => setDue(e.target.value)} />
      <input aria-label={`Payment amount for ${item.name}`} inputMode="decimal" placeholder="₹" className={`w-28 ${FIELD}`} value={amount} onChange={(e) => setAmount(e.target.value)} />
      <button type="button" className="btn-why" disabled={busy} onClick={go}>
        Add payment
      </button>
    </span>
  );
}

/** The wedding budget under one couple goal: items by category, their payments, and the verdict. */
export function WeddingBudget({ goalId }: { goalId: string }) {
  const qc = useQueryClient();
  const { data, isError } = useQuery({ queryKey: budgetKey(goalId), queryFn: () => weddingApi.get(goalId), retry: 1 });
  const [error, setError] = useState("");
  const [note, setNote] = useState("");
  const refresh = () => {
    void qc.invalidateQueries({ queryKey: budgetKey(goalId) });
    void qc.invalidateQueries({ queryKey: ["couple-goals"] });
    void qc.invalidateQueries({ queryKey: ["coming-up"] });
  };
  const fail = (e: unknown) => setError(e instanceof Error ? e.message : "That didn't work.");
  const opts = { onSuccess: refresh, onError: fail };
  const add = useMutation({ mutationFn: (i: ItemInput) => weddingApi.addItem(goalId, i), ...opts });
  const save = useMutation({ mutationFn: ({ id, i }: { id: string; i: ItemInput }) => weddingApi.updateItem(goalId, id, i), ...opts });
  const drop = useMutation({ mutationFn: (id: string) => weddingApi.removeItem(goalId, id), ...opts });
  const plan = useMutation({ mutationFn: ({ id, m }: { id: string; m: { due_on: string | null; amount_paise: number } }) => weddingApi.addMilestone(goalId, id, m), ...opts });
  const unplan = useMutation({ mutationFn: (id: string) => weddingApi.removeMilestone(goalId, id), ...opts });
  const unpay = useMutation({ mutationFn: (id: string) => weddingApi.unpay(goalId, id), ...opts });
  const pay = useMutation({
    mutationFn: ({ id, what }: { id: string; what: string }) => weddingApi.pay(goalId, id, what),
    onSuccess: (r) => {
      setNote(r.queued ? QUEUED_NOTE : "");
      refresh();
    },
    onError: fail,
  });
  const busy = add.isPending || save.isPending || drop.isPending || plan.isPending || unplan.isPending || unpay.isPending || pay.isPending;

  if (isError && !data) return <p role="alert">Couldn't load the budget.</p>;
  if (!data) return <div className="h-48 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading the budget" />;

  const s = data.summary;
  return (
    <div className="flex flex-col gap-6">
      <Verdict b={data} />

      <section aria-labelledby="budget-totals" className={PANEL}>
        <h2 id="budget-totals" className="m-0 mb-2 text-[15px] font-medium text-muted-foreground">
          {data.name}: the numbers
        </h2>
        <p className="m-0 text-sm">
          Planned {formatINR(s.planned_paise)} ({formatINR(s.committed_paise)} quoted, {formatINR(s.estimated_paise)} still an estimate) · Paid {formatINR(s.paid_paise)} · Left to pay {formatINR(s.remaining_paise)}
        </p>
        <p className="m-0 mt-1 text-sm text-muted-foreground">
          Goal {formatINR(data.target_paise)}, {formatINR(data.saved_paise)} saved
          {s.overdue_paise > 0 ? ` · ${formatINR(s.overdue_paise)} overdue` : ""}
          {s.unscheduled_paise > 0 ? ` · ${formatINR(s.unscheduled_paise)} has no payment date yet` : ""}
        </p>
      </section>

      {s.categories.map((c) => {
        const items = data.items.filter((i) => i.category === c.category);
        return (
          <section key={c.category} aria-labelledby={`cat-${c.category}`} className={PANEL}>
            <div className="mb-3 flex items-baseline justify-between gap-3">
              <h2 id={`cat-${c.category}`} className="m-0 text-[15px] font-medium">
                {label(c.category)}
              </h2>
              <span className="text-sm text-muted-foreground">
                {formatINR(c.paid_paise)} of {formatINR(c.planned_paise)} paid
              </span>
            </div>
            <ul className="m-0 grid list-none gap-5 p-0">
              {items.map((it) => (
                <li key={it.item_id} className="grid gap-1 text-sm">
                  <div className="flex items-baseline justify-between gap-3">
                    <strong>{it.name}</strong>
                    <span>{formatINR(it.planned_paise)}</span>
                  </div>
                  <span className="text-muted-foreground">
                    {it.quote_paise !== null ? `Quote ${formatINR(it.quote_paise)} (estimate ${formatINR(it.estimate_paise)})` : `Estimate ${formatINR(it.estimate_paise)}`}
                    {it.vendor ? ` · ${it.vendor}` : ""} · Paid {formatINR(it.paid_paise)}
                  </span>
                  <ul className="m-0 grid list-none gap-1 p-0">
                    {it.milestones.map((m) => (
                      <li key={m.milestone_id} className="flex flex-wrap items-center justify-between gap-2">
                        <span className={m.overdue ? "text-destructive" : undefined}>
                          {formatINR(m.amount_paise)} {m.due_on ? `due ${formatDay(m.due_on)}` : "no date yet"}
                          {m.paid ? ` · paid ${m.paid_on ? formatDay(m.paid_on) : ""}` : m.overdue ? " · overdue" : ""}
                        </span>
                        <span className="flex gap-2">
                          {m.paid ? (
                            <button type="button" className="btn-why" disabled={busy} onClick={() => unpay.mutate(m.milestone_id)}>
                              Undo payment
                            </button>
                          ) : (
                            <>
                              <button
                                type="button"
                                className="btn-why"
                                aria-label={`Mark ${it.name} ${formatINR(m.amount_paise)} paid`}
                                disabled={busy}
                                onClick={() => pay.mutate({ id: m.milestone_id, what: `Wedding payment: ${it.name}` })}
                              >
                                Mark paid
                              </button>
                              <button type="button" className="btn-why" aria-label={`Remove ${it.name} payment of ${formatINR(m.amount_paise)}`} disabled={busy} onClick={() => unplan.mutate(m.milestone_id)}>
                                Remove
                              </button>
                            </>
                          )}
                        </span>
                      </li>
                    ))}
                  </ul>
                  <AddPayment item={it} busy={busy} onError={setError} onAdd={(m) => plan.mutate({ id: it.item_id, m })} />
                  <details>
                    <summary className="cursor-pointer">Edit {it.name}</summary>
                    <ItemForm categories={data.categories} initial={it} busy={busy} submitLabel="Save changes" onError={setError} onSubmit={(i) => save.mutate({ id: it.item_id, i })} />
                    <button type="button" className="btn-why mt-2" disabled={busy} onClick={() => drop.mutate(it.item_id)}>
                      Delete {it.name}
                    </button>
                  </details>
                </li>
              ))}
            </ul>
          </section>
        );
      })}

      {data.items.length === 0 && <p className="m-0 text-sm text-muted-foreground">Nothing here yet. Add the venue, the caterer, the clothes, with what you expect each to cost.</p>}

      <section className={PANEL}>
        <details>
          <summary className="cursor-pointer text-sm">Add an item</summary>
          <ItemForm categories={data.categories} busy={busy} submitLabel="Add item" onError={setError} onSubmit={(i) => add.mutate(i)} />
        </details>
      </section>

      {note && <p role="status" className="m-0 text-sm text-muted-foreground">{note}</p>}
      {error && (
        <p role="alert" className="m-0 text-sm text-destructive">
          {error}
        </p>
      )}
      <Link to="/" className="text-sm font-medium text-primary">
        Back to your goals
      </Link>
    </div>
  );
}
