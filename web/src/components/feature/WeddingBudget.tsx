import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";

import SloshGauge from "@/components/bits/SloshGauge";
import { hue, type Hue } from "@/components/feature/life/hue";
import { useSection } from "@/theme/useSection";

import { QUEUED_NOTE } from "@/offline";
import {
  budgetKey,
  weddingApi,
  type Budget,
  type BudgetItem,
  type ItemInput,
} from "@/services/wedding";
import { formatDay } from "@/utils/dates";
import { formatINR, toPaise } from "@/utils/money";

const PANEL = "glass rounded-[28px] px-5 py-5 sm:px-8";
const FIELD =
  "min-h-11 rounded-xl border border-border bg-background px-3 py-1 focus-visible:border-rose";
// A soft rose pill for every small action on this page.
const BTN =
  "inline-flex min-h-11 items-center rounded-full border border-rose/40 bg-rose/10 px-4 text-sm font-medium text-[hsl(var(--foreground))] transition-colors hover:bg-rose/20 disabled:opacity-50 dark:text-[hsl(var(--c-rose))]";

// On track mint, tight sun, short coral.
const BANNER_HUE: Record<Budget["verdict"]["status"], Hue> = {
  on_track: "mint",
  tight: "sun",
  short: "coral",
};

/** Each category gets its own colour, in the order the server lists them. */
const CAT_HUES: Hue[] = [
  "rose",
  "berry",
  "sun",
  "coral",
  "grape",
  "sky",
  "mint",
  "teal",
];
const catHue = (i: number) => CAT_HUES[i % CAT_HUES.length];

const label = (c: string) => c.charAt(0).toUpperCase() + c.slice(1);

/** How much of the plan is paid, as a rose liquid gauge. */
function PaidGauge({ paid, planned }: { paid: number; planned: number }) {
  const { dark } = useSection();
  const pct = planned > 0 ? Math.round(Math.min(100, (paid / planned) * 100)) : 0;
  return (
    <div className="flex shrink-0 items-center gap-3 sm:flex-col sm:gap-1.5">
      <SloshGauge
        value={pct}
        liquidColor={dark ? "#8a8073" : "#8a8073"}
        glassColor={dark ? "#3b1d2a" : "#ffe4e6"}
        width={72}
        height={120}
        radius={18}
        ariaLabel="Share of the wedding plan paid"
      />
      <span className="text-[13px] text-muted-foreground">Paid so far</span>
    </div>
  );
}

/** What the plan is made of: one bar split by category, with a legend. */
function SplitBar({
  parts,
}: {
  parts: { key: string; name: string; paise: number; color: string }[];
}) {
  const total = parts.reduce((t, p) => t + Math.max(0, p.paise), 0);
  if (total <= 0) return null;
  return (
    <div className="mt-2 flex flex-col gap-2">
      <div
        aria-hidden
        className="flex h-3 w-full overflow-hidden rounded-full bg-chip"
      >
        {parts.map((p) => (
          <span
            key={p.key}
            className="h-full first:rounded-l-full last:rounded-r-full"
            style={{
              width: `${(Math.max(0, p.paise) / total) * 100}%`,
              background: p.color,
            }}
          />
        ))}
      </div>
      <p className="m-0 flex flex-wrap gap-x-4 gap-y-1 text-[12px] text-muted-foreground">
        {parts.map((p) => (
          <span key={p.key} className="inline-flex items-center gap-1.5">
            <i
              aria-hidden
              className="inline-block h-2 w-2 rounded-full"
              style={{ background: p.color }}
            />
            {`${p.name} ${Math.round((Math.max(0, p.paise) / total) * 100)}%`}
          </span>
        ))}
      </p>
    </div>
  );
}

/** One line at the top: whether what is set aside covers every payment by its date. */
function Verdict({ b }: { b: Budget }) {
  const v = b.verdict;
  return (
    <section
      role="status"
      aria-label="Budget verdict"
      className="relative overflow-hidden rounded-[28px] border px-5 py-4 sm:px-8"
      style={{
        borderColor: hue(BANNER_HUE[v.status], 0.5),
        background: `linear-gradient(120deg, ${hue(BANNER_HUE[v.status], 0.2)}, ${hue("rose", 0.08)})`,
      }}
    >
      <span
        aria-hidden
        className="absolute inset-y-0 left-0 w-1.5"
        style={{ background: hue(BANNER_HUE[v.status]) }}
      />
      <p className="m-0 text-lg font-medium">{v.line}</p>
      <p className="m-0 mt-1 text-sm text-muted-foreground">
        {formatINR(b.on_hand_paise)} set aside and not yet paid out
        {b.expected_monthly_paise > 0
          ? `, and about ${formatINR(b.expected_monthly_paise)} a month going in`
          : ""}
        .
        {b.next_due
          ? ` Next payment: ${b.next_due.item_name}, ${formatINR(b.next_due.amount_paise)} on ${formatDay(b.next_due.due_on)}.`
          : ""}
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

function ItemForm({
  categories,
  initial,
  busy,
  submitLabel,
  onSubmit,
  onError,
}: ItemFormProps) {
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
    if (estimate === null || estimate < 0 || (quote !== null && quote < 0))
      return onError("Enter amounts in rupees, like 50000.");
    onError("");
    onSubmit({
      name: f.name.trim(),
      category: f.category,
      estimate_paise: estimate,
      quote_paise: quote,
      vendor: f.vendor.trim(),
      notes: initial?.notes ?? "",
    });
    if (!initial) setF({ ...f, name: "", estimate: "", quote: "", vendor: "" });
  };
  const id = initial?.item_id ?? "new";
  return (
    <div className="mt-2 grid gap-2 text-sm">
      <label>
        Item
        <input
          aria-label={`Item name ${id}`}
          className={`ml-2 ${FIELD}`}
          value={f.name}
          onChange={(e) => setF({ ...f, name: e.target.value })}
        />
      </label>
      <label>
        Category
        <select
          aria-label={`Category ${id}`}
          className={`ml-2 ${FIELD}`}
          value={f.category}
          onChange={(e) => setF({ ...f, category: e.target.value })}
        >
          {categories.map((c) => (
            <option key={c} value={c}>
              {label(c)}
            </option>
          ))}
        </select>
      </label>
      <label>
        Estimate (₹)
        <input
          aria-label={`Estimate ${id}`}
          inputMode="decimal"
          className={`ml-2 w-32 ${FIELD}`}
          value={f.estimate}
          onChange={(e) => setF({ ...f, estimate: e.target.value })}
        />
      </label>
      <label>
        Quote (₹, if you have one)
        <input
          aria-label={`Quote ${id}`}
          inputMode="decimal"
          className={`ml-2 w-32 ${FIELD}`}
          value={f.quote}
          onChange={(e) => setF({ ...f, quote: e.target.value })}
        />
      </label>
      <label>
        Vendor
        <input
          aria-label={`Vendor ${id}`}
          className={`ml-2 ${FIELD}`}
          value={f.vendor}
          onChange={(e) => setF({ ...f, vendor: e.target.value })}
        />
      </label>
      <div>
        <button type="button" className={BTN} disabled={busy} onClick={go}>
          {submitLabel}
        </button>
      </div>
    </div>
  );
}

function AddPayment({
  item,
  busy,
  onAdd,
  onError,
}: {
  item: BudgetItem;
  busy: boolean;
  onAdd: (m: { due_on: string | null; amount_paise: number }) => void;
  onError: (m: string) => void;
}) {
  const [due, setDue] = useState("");
  const [amount, setAmount] = useState("");
  const go = () => {
    const paise = toPaise(amount);
    if (!paise || paise <= 0)
      return onError("Enter the payment amount in rupees, like 25000.");
    onError("");
    onAdd({ due_on: due || null, amount_paise: paise });
    setAmount("");
    setDue("");
  };
  return (
    <span className="flex flex-wrap items-center gap-2">
      <input
        type="date"
        aria-label={`Due date for ${item.name}`}
        className={FIELD}
        value={due}
        onChange={(e) => setDue(e.target.value)}
      />
      <input
        aria-label={`Payment amount for ${item.name}`}
        inputMode="decimal"
        placeholder="₹"
        className={`w-28 ${FIELD}`}
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
      />
      <button type="button" className={BTN} disabled={busy} onClick={go}>
        Add payment
      </button>
    </span>
  );
}

/** The wedding budget under one couple goal: items by category, their payments, and the verdict. */
export function WeddingBudget({ goalId }: { goalId: string }) {
  const qc = useQueryClient();
  const { data, isError } = useQuery({
    queryKey: budgetKey(goalId),
    queryFn: () => weddingApi.get(goalId),
    retry: 1,
  });
  const [error, setError] = useState("");
  const [note, setNote] = useState("");
  const refresh = () => {
    void qc.invalidateQueries({ queryKey: budgetKey(goalId) });
    void qc.invalidateQueries({ queryKey: ["couple-goals"] });
    void qc.invalidateQueries({ queryKey: ["coming-up"] });
  };
  const fail = (e: unknown) =>
    setError(e instanceof Error ? e.message : "That didn't work.");
  const opts = { onSuccess: refresh, onError: fail };
  const add = useMutation({
    mutationFn: (i: ItemInput) => weddingApi.addItem(goalId, i),
    ...opts,
  });
  const save = useMutation({
    mutationFn: ({ id, i }: { id: string; i: ItemInput }) =>
      weddingApi.updateItem(goalId, id, i),
    ...opts,
  });
  const drop = useMutation({
    mutationFn: (id: string) => weddingApi.removeItem(goalId, id),
    ...opts,
  });
  const plan = useMutation({
    mutationFn: ({
      id,
      m,
    }: {
      id: string;
      m: { due_on: string | null; amount_paise: number };
    }) => weddingApi.addMilestone(goalId, id, m),
    ...opts,
  });
  const unplan = useMutation({
    mutationFn: (id: string) => weddingApi.removeMilestone(goalId, id),
    ...opts,
  });
  const unpay = useMutation({
    mutationFn: (id: string) => weddingApi.unpay(goalId, id),
    ...opts,
  });
  const pay = useMutation({
    mutationFn: ({ id, what }: { id: string; what: string }) =>
      weddingApi.pay(goalId, id, what),
    onSuccess: (r) => {
      setNote(r.queued ? QUEUED_NOTE : "");
      refresh();
    },
    onError: fail,
  });
  const busy =
    add.isPending ||
    save.isPending ||
    drop.isPending ||
    plan.isPending ||
    unplan.isPending ||
    unpay.isPending ||
    pay.isPending;

  if (isError && !data) return <p role="alert">Couldn't load the budget.</p>;
  if (!data)
    return (
      <div
        className="h-48 animate-pulse rounded-[28px] bg-muted/40"
        aria-label="Loading the budget"
      />
    );

  const s = data.summary;
  return (
    <div className="flex flex-col gap-6">
      <Verdict b={data} />

      <section
        aria-labelledby="budget-totals"
        className={`${PANEL} flex flex-col gap-4 sm:flex-row sm:items-center sm:gap-8`}
      >
        <PaidGauge paid={s.paid_paise} planned={s.planned_paise} />
        <div className="flex min-w-0 flex-1 flex-col gap-1">
        <h2
          id="budget-totals"
          className="m-0 mb-2 text-[15px] font-semibold text-[hsl(var(--foreground))] dark:text-[hsl(var(--c-rose))]"
        >
          {data.name}: the numbers
        </h2>
        <p className="m-0 text-sm">
          Planned {formatINR(s.planned_paise)} ({formatINR(s.committed_paise)}{" "}
          quoted, {formatINR(s.estimated_paise)} still an estimate) · Paid{" "}
          {formatINR(s.paid_paise)} · Left to pay {formatINR(s.remaining_paise)}
        </p>
        <SplitBar
          parts={s.categories.map((c, i) => ({
            key: c.category,
            name: label(c.category),
            paise: c.planned_paise,
            color: hue(catHue(i)),
          }))}
        />
        <p className="m-0 mt-1 text-sm text-muted-foreground">
          Goal {formatINR(data.target_paise)}, {formatINR(data.saved_paise)}{" "}
          saved
          {s.overdue_paise > 0
            ? ` · ${formatINR(s.overdue_paise)} overdue`
            : ""}
          {s.unscheduled_paise > 0
            ? ` · ${formatINR(s.unscheduled_paise)} has no payment date yet`
            : ""}
        </p>
        </div>
      </section>

      {s.categories.map((c, ci) => {
        const items = data.items.filter((i) => i.category === c.category);
        const h = catHue(ci);
        const paidPct =
          c.planned_paise > 0
            ? Math.min(100, (c.paid_paise / c.planned_paise) * 100)
            : 0;
        return (
          <section
            key={c.category}
            aria-labelledby={`cat-${c.category}`}
            className="relative overflow-hidden rounded-[28px] border px-5 py-5 sm:px-8"
            style={{
              borderColor: hue(h, 0.3),
              background: `linear-gradient(160deg, ${hue(h, 0.12)}, transparent 55%), hsl(var(--card))`,
            }}
          >
            <span
              aria-hidden
              className="absolute inset-y-0 left-0 w-1.5"
              style={{ background: hue(h) }}
            />
            <div className="mb-2 flex items-baseline justify-between gap-3">
              <h2
                id={`cat-${c.category}`}
                className="m-0 flex items-center gap-2 text-[17px] font-semibold"
              >
                <span
                  aria-hidden
                  className="h-3 w-3 rounded-full"
                  style={{
                    background: hue(h),
                    boxShadow: `0 0 0 3px ${hue(h, 0.25)}`,
                  }}
                />
                {label(c.category)}
              </h2>
              <span className="text-sm text-muted-foreground">
                {formatINR(c.paid_paise)} of {formatINR(c.planned_paise)} paid
              </span>
            </div>
            <span
              aria-hidden
              className="mb-4 block h-1.5 w-full overflow-hidden rounded-full bg-chip"
            >
              <span
                className="block h-full rounded-full"
                style={{ width: `${paidPct}%`, background: hue(h) }}
              />
            </span>
            <ul className="m-0 grid list-none gap-5 p-0">
              {items.map((it) => (
                <li key={it.item_id} className="grid gap-1 text-sm">
                  <div className="flex items-baseline justify-between gap-3">
                    <strong>{it.name}</strong>
                    <span>{formatINR(it.planned_paise)}</span>
                  </div>
                  <span className="text-muted-foreground">
                    {it.quote_paise !== null
                      ? `Quote ${formatINR(it.quote_paise)} (estimate ${formatINR(it.estimate_paise)})`
                      : `Estimate ${formatINR(it.estimate_paise)}`}
                    {it.vendor ? ` · ${it.vendor}` : ""} · Paid{" "}
                    {formatINR(it.paid_paise)}
                  </span>
                  <ul className="m-0 grid list-none gap-1 p-0">
                    {it.milestones.map((m) => (
                      <li
                        key={m.milestone_id}
                        className="flex flex-wrap items-center justify-between gap-2"
                      >
                        <span
                          className={
                            m.overdue
                              ? "text-destructive"
                              : m.paid
                                ? "text-[hsl(var(--foreground))] dark:text-[hsl(var(--c-mint))]"
                                : undefined
                          }
                        >
                          {formatINR(m.amount_paise)}{" "}
                          {m.due_on
                            ? `due ${formatDay(m.due_on)}`
                            : "no date yet"}
                          {m.paid
                            ? ` · paid ${m.paid_on ? formatDay(m.paid_on) : ""}`
                            : m.overdue
                              ? " · overdue"
                              : ""}
                        </span>
                        <span className="flex gap-2">
                          {m.paid ? (
                            <button
                              type="button"
                              className={BTN}
                              disabled={busy}
                              onClick={() => unpay.mutate(m.milestone_id)}
                            >
                              Undo payment
                            </button>
                          ) : (
                            <>
                              <button
                                type="button"
                                className={BTN}
                                aria-label={`Mark ${it.name} ${formatINR(m.amount_paise)} paid`}
                                disabled={busy}
                                onClick={() =>
                                  pay.mutate({
                                    id: m.milestone_id,
                                    what: `Wedding payment: ${it.name}`,
                                  })
                                }
                              >
                                Mark paid
                              </button>
                              <button
                                type="button"
                                className={BTN}
                                aria-label={`Remove ${it.name} payment of ${formatINR(m.amount_paise)}`}
                                disabled={busy}
                                onClick={() => unplan.mutate(m.milestone_id)}
                              >
                                Remove
                              </button>
                            </>
                          )}
                        </span>
                      </li>
                    ))}
                  </ul>
                  <AddPayment
                    item={it}
                    busy={busy}
                    onError={setError}
                    onAdd={(m) => plan.mutate({ id: it.item_id, m })}
                  />
                  <details>
                    <summary className="cursor-pointer">Edit {it.name}</summary>
                    <ItemForm
                      categories={data.categories}
                      initial={it}
                      busy={busy}
                      submitLabel="Save changes"
                      onError={setError}
                      onSubmit={(i) => save.mutate({ id: it.item_id, i })}
                    />
                    <button
                      type="button"
                      className={`${BTN} mt-2`}
                      disabled={busy}
                      onClick={() => drop.mutate(it.item_id)}
                    >
                      Delete {it.name}
                    </button>
                  </details>
                </li>
              ))}
            </ul>
          </section>
        );
      })}

      {data.items.length === 0 && (
        <p className="empty-note m-0 text-sm">
          Nothing here yet. Add the venue, the caterer, the clothes, with what
          you expect each to cost.
        </p>
      )}

      <section className={PANEL}>
        <details>
          <summary className="flex min-h-11 cursor-pointer items-center text-sm font-semibold text-[hsl(var(--foreground))] dark:text-[hsl(var(--c-rose))]">
            Add an item
          </summary>
          <ItemForm
            categories={data.categories}
            busy={busy}
            submitLabel="Add item"
            onError={setError}
            onSubmit={(i) => add.mutate(i)}
          />
        </details>
      </section>

      {note && (
        <p role="status" className="m-0 text-sm text-muted-foreground">
          {note}
        </p>
      )}
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
