import { useState } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { Trash2 } from "lucide-react";

import { planApi } from "@/services/api";
import type { Account, Bucket, Commitment, Income } from "@/services/types";
import {
  keys,
  useAccounts,
  useCommitments,
  useIncomes,
  usePlannerMutation,
  useResetSettings,
  useSettings,
  useSummary,
} from "@/hooks/usePlanner";
import { ResetButton } from "@/components/ui/reset-button";
import { Header } from "@/components/layout/Header";
import { AffordPanel } from "@/components/feature/AffordPanel";
import { BillsPanel } from "@/components/feature/BillsPanel";
import { GoalsPanel } from "@/components/feature/GoalsPanel";
import { AccountSelect } from "@/components/feature/AccountSelect";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { InfoTip } from "@/components/ui/info-tip";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { ChipGroup } from "@/components/ui/chips";
import { Stepper } from "@/components/ui/stepper";
import {
  formatINR,
  formatPercent,
  paiseToInput,
  parseAmountOrPercent,
  toPaise,
} from "@/utils/money";
import {
  addMonths,
  firstBuyMonth,
  formatYearMonth,
  ordinal,
  thisYearMonth,
} from "@/utils/dates";
import { BUCKET_HELP, BUCKET_LABEL, defaultBucket } from "@/utils/buckets";
import { cn } from "@/utils/cn";
import { freqOf } from "@/utils/freq";
import { GuidanceCard } from "@/components/feature/GuidanceCard";
import { ScrollFade } from "@/components/ui/scroll-fade";

// Fixed-payment categories, most used first: drawn from what this household actually pays on a
// fixed day (EMIs, house, phone and internet, office travel, the car, insurance, fees like the CA).
// Each lands in a bucket by default (`defaultBucket`), which the user can change.
const CATEGORIES = [
  "bills",
  "emi",
  "house",
  "rent",
  "family",
  "phone",
  "internet",
  "transport",
  "car",
  "insurance",
  "health",
  "gym",
  "fees",
  "sip",
  "rd",
  "investment",
  "subscription",
  "entertainment",
  "shopping",
  "dates",
  "gifts",
  "travel",
  "donation",
  "other",
];
/** Abbreviations the chips would otherwise show as "Emi", "Sip", "Rd". */
const CATEGORY_LABEL: Record<string, string> = {
  emi: "EMI",
  sip: "SIP",
  rd: "RD",
};
const BUCKETS: Bucket[] = ["needs", "savings", "wants"];
const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

const parseDay = (v: string): number | null => {
  const n = Number(v);
  return v.trim() && Number.isInteger(n) && n >= 1 && n <= 31 ? n : null;
};
const accountName = (accounts: Account[], id: string | null) =>
  id
    ? (accounts.find((a) => a.account_id === id)?.name ?? "Unknown account")
    : "Any account";

// ---------- income ----------

function IncomeSection({ accounts }: { accounts: Account[] }) {
  const { data: incomes } = useIncomes();
  const [f, setF] = useState({ label: "", amount: "", day: "", account: "" });
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(planApi.createIncome, [keys.incomes], {
    success: "Income added. Nice!",
  });
  const remove = usePlannerMutation(planApi.removeIncome, [keys.incomes], {
    success: "Income removed",
  });

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(f.amount);
    const day = parseDay(f.day);
    if (!f.label.trim() || amount === null || amount <= 0 || day === null) {
      return setError(
        "Add a name, a positive amount and the day of the month (1-31).",
      );
    }
    setError(null);
    create.mutate(
      {
        label: f.label.trim(),
        amount_paise: amount,
        day_of_month: day,
        account_id: f.account || null,
      },
      {
        onSuccess: () => setF({ label: "", amount: "", day: "", account: "" }),
      },
    );
  };

  return (
    <Card id="income" className="scroll-mt-24">
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">
          <span className="brand-text">Income</span>
        </CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Your next payday sets the "until" date for Safe to Spend.
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        <ul className="ledger">
          {incomes?.map((i: Income) => (
            <li key={i.income_id} className="ledger-row">
              <div className="flex min-w-0 items-center gap-3">
                <span
                  aria-hidden
                  className="h-9 w-1.5 shrink-0 rounded-full bg-gradient-to-b from-mint to-teal"
                />
                <div className="min-w-0">
                <p className="truncate text-[15px] font-medium" title={i.label}>
                  {i.label}
                </p>
                <p className="text-[13px] leading-snug text-muted-foreground">
                  {formatINR(i.amount_paise)} on the {ordinal(i.day_of_month)},
                  into {accountName(accounts, i.account_id)}
                </p>
                </div>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="h-9 w-9 shrink-0"
                aria-label={`Remove ${i.label}`}
                onClick={() =>
                  window.confirm(`Remove "${i.label}"?`) &&
                  remove.mutate(i.income_id)
                }
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </li>
          ))}
        </ul>
        {incomes?.length === 0 && (
          <p className="empty-note">No income yet. Add your salary below.</p>
        )}
        <form
          onSubmit={submit}
          className="form-grid border-t border-border pt-6"
          noValidate
        >
          <div className="field">
            <Label htmlFor="inc-label">Name</Label>
            <Input
              id="inc-label"
              value={f.label}
              onChange={(e) => setF({ ...f, label: e.target.value })}
              placeholder="Salary"
            />
          </div>
          <div className="field">
            <Label htmlFor="inc-amount">Amount</Label>
            <MoneyInput
              id="inc-amount"
              value={f.amount}
              onChange={(e) => setF({ ...f, amount: e.target.value })}
            />
          </div>
          <div className="field">
            <Label htmlFor="inc-day">Day of month</Label>
            <Input
              id="inc-day"
              inputMode="numeric"
              maxLength={2}
              value={f.day}
              onChange={(e) => setF({ ...f, day: e.target.value })}
              placeholder="30"
            />
          </div>
          <div className="field">
            <Label htmlFor="inc-account">Credited to</Label>
            <AccountSelect
              id="inc-account"
              accounts={accounts}
              value={f.account}
              onChange={(v) => setF({ ...f, account: v })}
            />
          </div>
          {error && (
            <p role="alert" className="col-span-full text-sm text-destructive">
              {error}
            </p>
          )}
          <div className="form-actions">
            <ResetButton
              label="Clear"
              onReset={() => {
                setF({ label: "", amount: "", day: "", account: "" });
                setError(null);
              }}
            />
            <Button type="submit" disabled={create.isPending}>
              Add income
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

// ---------- commitments ----------

function CommitmentRow({
  c,
  accounts,
}: {
  c: Commitment;
  accounts: Account[];
}) {
  const refresh = [keys.commitments, keys.bills];
  const update = usePlannerMutation(planApi.updateCommitment, refresh);
  const remove = usePlannerMutation(planApi.removeCommitment, refresh, {
    success: "Removed",
  });
  const freq = freqOf(c);
  const everyFew = freq.kind === "every";
  const when =
    freq.kind === "yearly"
      ? `yearly${c.due_month ? ` in ${MONTHS[c.due_month - 1]}` : ", month not set"} on the ${ordinal(c.day_of_month)}`
      : everyFew
        ? `every ${c.every_months} months${c.first_month ? ` from ${formatYearMonth(c.first_month)}` : ""} on the ${ordinal(c.day_of_month)}`
        : `on the ${ordinal(c.day_of_month)}`;
  // Its first one comes sooner than a full round of saving: offer to start saving now instead.
  const now = thisYearMonth();
  const tooSoon =
    everyFew &&
    c.active &&
    !!c.first_month &&
    c.first_month >= now &&
    c.first_month < addMonths(now, c.every_months);
  return (
    <li
      className={cn("ledger-row flex-wrap text-sm", !c.active && "opacity-60")}
    >
      <div className="flex min-w-0 items-center gap-3">
        {/* Needs sky, spending coral, savings mint, as in the monthly split. */}
        <span
          aria-hidden
          className={cn(
            "h-9 w-1.5 shrink-0 rounded-full bg-gradient-to-b",
            c.bucket === "savings"
              ? "from-mint to-teal"
              : c.bucket === "wants"
                ? "from-sun to-coral"
                : "from-sky to-teal",
          )}
        />
        <div className="min-w-0">
        <p className="truncate text-[15px] font-medium" title={c.label}>
          {c.label}
        </p>
        <p className="text-[13px] leading-snug text-muted-foreground">
          {formatINR(c.amount_paise)} {when} ·{" "}
          {accountName(accounts, c.account_id)}
          {c.to_account_id &&
            ` → ${accountName(accounts, c.to_account_id)}`} ·{" "}
          {BUCKET_LABEL[c.bucket]}
          {freq.kind === "yearly" &&
            ` · ${formatINR(Math.floor(c.amount_paise / 12))}/month set aside`}
          {everyFew &&
            ` · ${formatINR(Math.floor(c.amount_paise / c.every_months))}/month set aside`}
          {c.start_month && ` · saving from ${formatYearMonth(c.start_month)}`}
          {c.end_month && ` · until ${formatYearMonth(c.end_month)}`}
        </p>
        </div>
      </div>
      <div className="flex shrink-0 flex-wrap items-center gap-1">
        <Select
          aria-label={`Bucket for ${c.label}`}
          className="h-9 w-auto px-2 text-xs"
          value={c.bucket}
          onChange={(e) =>
            update.mutate({ ...c, bucket: e.target.value as Bucket })
          }
        >
          {BUCKETS.map((b) => (
            <option key={b} value={b}>
              {BUCKET_LABEL[b]}
            </option>
          ))}
        </Select>
        <Button
          variant="outline"
          size="sm"
          onClick={() => update.mutate({ ...c, active: !c.active })}
          disabled={update.isPending}
        >
          {c.active ? "Pause" : "Resume"}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-9 w-9"
          aria-label={`Remove ${c.label}`}
          onClick={() =>
            window.confirm(`Remove "${c.label}"?`) &&
            remove.mutate(c.commitment_id)
          }
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
      {c.active && (
        <StartSaving
          c={c}
          openAtFirst={tooSoon}
          pending={update.isPending}
          onSave={(month) =>
            update.mutate(
              everyFew
                ? { ...c, first_month: firstBuyMonth(month, c.every_months) }
                : { ...c, start_month: month || null },
            )
          }
        />
      )}
    </li>
  );
}

interface EveryFewFields {
  saving: "start" | "saved";
  saveFrom: string;
  nextDue: string;
}

/** The first month a payment every few months falls due: after a full round of saving, or the month given. */
function everyFewFirstMonth(f: EveryFewFields, every: number): string | null {
  return f.saving === "start"
    ? firstBuyMonth(f.saveFrom || thisYearMonth(), every)
    : f.nextDue || null;
}

/** "₹1,000 a month from Dec 2026. First buy in Apr 2027, then every 4 months." */
function everyFewPreview(
  f: EveryFewFields,
  amount: number | null,
  every: number,
): string {
  const share = amount
    ? `${formatINR(Math.floor(amount / every))} a month`
    : "A share each month";
  const first = everyFewFirstMonth(f, every) ?? thisYearMonth();
  if (f.saving === "start") {
    return `${share} from ${formatYearMonth(f.saveFrom || thisYearMonth())}. First buy in ${formatYearMonth(first)}, then every ${every} months.`;
  }
  return `${share} is set aside. Next due in ${formatYearMonth(first)}, then every ${every} months.`;
}

/**
 * "Start saving from" for any fixed payment, in Needs, Spending or Savings. For a payment every few
 * months it moves the first buy to a full round of saving after that month; for a monthly or yearly
 * one it is the payment's first month, so nothing is set aside or due before it. Offered up front
 * when a payment every few months comes before a full round of saving could.
 */
function StartSaving({
  c,
  openAtFirst,
  onSave,
  pending,
}: {
  c: Commitment;
  openAtFirst: boolean;
  onSave: (month: string) => void;
  pending: boolean;
}) {
  const freq = freqOf(c);
  const everyFew = freq.kind === "every";
  const [open, setOpen] = useState(openAtFirst);
  const [from, setFrom] = useState(
    everyFew ? thisYearMonth() : (c.start_month ?? thisYearMonth()),
  );
  if (!open) {
    // Its own line under the row, so long rows don't push it to the side.
    return (
      <div className="w-full">
        <Button variant="outline" size="sm" onClick={() => setOpen(true)}>
          {c.start_month && !everyFew
            ? `Saving from ${formatYearMonth(c.start_month)}...`
            : "Start saving from..."}
        </Button>
      </div>
    );
  }
  const month = from || thisYearMonth();
  const preview = everyFew
    ? `First buy in ${formatYearMonth(firstBuyMonth(month, c.every_months))}`
    : freq.kind === "yearly"
      ? `${formatINR(Math.floor(c.amount_paise / 12))} a month set aside from ${formatYearMonth(month)}`
      : `First payment in ${formatYearMonth(month)}`;
  return (
    <div className="flex w-full flex-wrap items-center gap-2 text-[13px]">
      <Label htmlFor={`save-from-${c.commitment_id}`}>Start saving from</Label>
      <Input
        id={`save-from-${c.commitment_id}`}
        type="month"
        className="h-9 w-auto"
        value={from}
        onChange={(e) => setFrom(e.target.value)}
      />
      <span className="text-muted-foreground">{preview}</span>
      <Button
        size="sm"
        disabled={pending || !from}
        onClick={() => {
          onSave(from);
          setOpen(false);
        }}
      >
        Save
      </Button>
      {!everyFew && c.start_month && (
        <Button
          variant="ghost"
          size="sm"
          disabled={pending}
          onClick={() => {
            onSave("");
            setOpen(false);
          }}
        >
          Start now
        </Button>
      )}
      <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
        Cancel
      </Button>
    </div>
  );
}

function CommitmentsSection({ accounts }: { accounts: Account[] }) {
  const { data: commitments } = useCommitments();
  const blank = {
    label: "",
    amount: "",
    day: "",
    account: "",
    to: "",
    category: "bills",
    bucket: "needs" as Bucket,
    yearly: false,
    dueMonth: "",
    every: "1",
    saving: "start" as "start" | "saved",
    saveFrom: "",
    nextDue: "",
    start: "",
    end: "",
  };
  const [f, setF] = useState(blank);
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(
    planApi.createCommitment,
    [keys.commitments, keys.bills],
    { success: "Added" },
  );

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(f.amount);
    const day = parseDay(f.day);
    if (!f.label.trim() || amount === null || amount <= 0 || day === null) {
      return setError(
        "Add a name, a positive amount and the day of the month (1-31).",
      );
    }
    const every = f.yearly ? 1 : Number(f.every);
    const firstMonth = every > 1 ? everyFewFirstMonth(f, every) : null;
    if (firstMonth && f.end && firstMonth > f.end) {
      return setError(
        "The first time it falls due is after the last month, so it would never fall due.",
      );
    }
    setError(null);
    create.mutate(
      {
        label: f.label.trim(),
        amount_paise: amount,
        day_of_month: day,
        account_id: f.account || null,
        to_account_id: f.to || null,
        category: f.category,
        bucket: f.bucket,
        frequency: f.yearly ? "yearly" : "monthly",
        due_month: f.yearly && f.dueMonth ? Number(f.dueMonth) : null,
        every_months: every,
        // Blank (already saved, no month given) means this month: the server fills it in.
        first_month: firstMonth,
        active: true,
        start_month: f.start || null,
        end_month: f.end || null,
      },
      { onSuccess: () => setF(blank) },
    );
  };

  return (
    <Card id="fixed" className="scroll-mt-24">
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">
          <span className="brand-text">Fixed payments</span>
        </CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Anything paid on a fixed day: SIPs, house, phone and internet, gym,
          subscriptions, an RD instalment. Yearly bills are set aside a twelfth
          a month. Each counts toward Needs, Savings or Spending.
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        <ul className="ledger">
          {commitments?.map((c) => (
            <CommitmentRow key={c.commitment_id} c={c} accounts={accounts} />
          ))}
        </ul>
        {commitments?.length === 0 && (
          <p className="empty-note">Nothing yet.</p>
        )}
        <form
          onSubmit={submit}
          className="form-grid border-t border-border pt-6"
          noValidate
        >
          <div className="field">
            <Label htmlFor="cm-label">Name</Label>
            <Input
              id="cm-label"
              value={f.label}
              onChange={(e) => setF({ ...f, label: e.target.value })}
              placeholder="e.g. Phone bill"
            />
          </div>
          <div className="field">
            <Label htmlFor="cm-amount">
              {f.yearly
                ? "Amount per year"
                : Number(f.every) > 1
                  ? "Amount each time"
                  : "Amount"}
            </Label>
            <MoneyInput
              id="cm-amount"
              value={f.amount}
              onChange={(e) => setF({ ...f, amount: e.target.value })}
            />
          </div>
          <div className="field">
            <Label htmlFor="cm-day">Day of month</Label>
            <Input
              id="cm-day"
              inputMode="numeric"
              maxLength={2}
              value={f.day}
              onChange={(e) => setF({ ...f, day: e.target.value })}
              placeholder="10"
            />
          </div>
          <div className="field">
            <Label htmlFor="cm-account">Paid from</Label>
            <AccountSelect
              id="cm-account"
              accounts={accounts}
              value={f.account}
              onChange={(v) => setF({ ...f, account: v })}
            />
          </div>
          <label className="col-span-full flex items-start gap-3 text-sm">
            <input
              type="checkbox"
              className="mt-0.5 h-4 w-4 accent-brand"
              checked={f.yearly}
              onChange={(e) => setF({ ...f, yearly: e.target.checked })}
            />
            <span>Yearly bill (set aside monthly, due once a year)</span>
          </label>
          {!f.yearly && (
            <div className="field col-span-full">
              <Label id="cm-every-label">Repeats</Label>
              <Stepper
                label="Repeats every how many months"
                value={Number(f.every)}
                min={1}
                max={11}
                onChange={(n) => setF({ ...f, every: String(n) })}
                format={(n) => (n === 1 ? "Every month" : `Every ${n} months`)}
              />
              {Number(f.every) > 1 && (
                <>
                  <Label id="cm-saving-label" className="mt-3">
                    Saving for it
                  </Label>
                  <ChipGroup
                    label="Saving for it"
                    value={f.saving}
                    onChange={(v) => setF({ ...f, saving: v })}
                    options={[
                      { value: "start", label: "Start saving now" },
                      { value: "saved", label: "Already saved up" },
                    ]}
                  />
                  {f.saving === "start" ? (
                    <>
                      <Label htmlFor="cm-save-from" className="mt-3">
                        Start saving from
                      </Label>
                      <Input
                        id="cm-save-from"
                        type="month"
                        value={f.saveFrom || thisYearMonth()}
                        onChange={(e) =>
                          setF({ ...f, saveFrom: e.target.value })
                        }
                      />
                    </>
                  ) : (
                    <>
                      <Label htmlFor="cm-next-due" className="mt-3">
                        Next due month (empty = this month)
                      </Label>
                      <Input
                        id="cm-next-due"
                        type="month"
                        value={f.nextDue}
                        onChange={(e) =>
                          setF({ ...f, nextDue: e.target.value })
                        }
                      />
                    </>
                  )}
                  <p className="mt-1 text-xs text-muted-foreground">
                    {everyFewPreview(f, toPaise(f.amount), Number(f.every))}
                  </p>
                </>
              )}
            </div>
          )}
          {f.yearly && (
            <div className="field col-span-full">
              <Label id="cm-due-label">
                Due month (leave empty if you don't remember)
              </Label>
              <ChipGroup
                label="Due month"
                value={f.dueMonth}
                onChange={(v) => setF({ ...f, dueMonth: v })}
                options={[
                  { value: "", label: "Not sure" },
                  ...MONTHS.map((m, i) => ({ value: String(i + 1), label: m })),
                ]}
              />
            </div>
          )}
          <div className="field">
            <Label id="cm-category-label">Category</Label>
            <ChipGroup
              label="Category"
              value={f.category}
              onChange={(c) =>
                setF({ ...f, category: c, bucket: defaultBucket(c) })
              }
              options={CATEGORIES.map((c) => ({
                value: c,
                label: CATEGORY_LABEL[c] ?? c,
              }))}
            />
          </div>
          <div className="field">
            <div className="flex items-center gap-1.5">
              <Label id="cm-bucket-label">Counts toward</Label>
              <InfoTip text={BUCKET_HELP[f.bucket]} />
            </div>
            <ChipGroup
              label="Counts toward"
              value={f.bucket}
              onChange={(b) => setF({ ...f, bucket: b })}
              options={BUCKETS.map((b) => ({
                value: b,
                label: BUCKET_LABEL[b],
              }))}
            />
          </div>
          <div className="field col-span-full">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="cm-to">Goes into (optional)</Label>
              <InfoTip text="For an RD or FD top-up: the account the money ends up in, so the projection grows it. Leave empty if the money just leaves (a SIP, a bill)." />
            </div>
            <AccountSelect
              id="cm-to"
              accounts={accounts}
              value={f.to}
              onChange={(v) => setF({ ...f, to: v })}
            />
          </div>
          <div className="form-grid col-span-full">
            {/* A payment every few months saved up from now has its own "Start saving from" above. */}
            {(f.yearly || Number(f.every) === 1 || f.saving !== "start") && (
            <div className="field">
              <Label htmlFor="cm-start">Start saving from</Label>
              <Input
                id="cm-start"
                type="month"
                value={f.start}
                onChange={(e) => setF({ ...f, start: e.target.value })}
              />
            </div>
            )}
            <div className="field">
              <Label htmlFor="cm-end">Last month</Label>
              <Input
                id="cm-end"
                type="month"
                value={f.end}
                onChange={(e) => setF({ ...f, end: e.target.value })}
              />
            </div>
          </div>
          {error && (
            <p role="alert" className="col-span-full text-sm text-destructive">
              {error}
            </p>
          )}
          <div className="form-actions">
            <ResetButton
              label="Clear"
              onReset={() => {
                setF(blank);
                setError(null);
              }}
            />
            <Button type="submit" disabled={create.isPending}>
              Add fixed payment
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

// ---------- settings ----------

export function SettingsSection() {
  const { data } = useSettings();
  const { data: summary } = useSummary();
  const [spending, setSpending] = useState("");
  const [allocDay, setAllocDay] = useState("10");
  const [floor, setFloor] = useState("");
  const [months, setMonths] = useState("3");
  const [autoPost, setAutoPost] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const save = usePlannerMutation(planApi.updateSettings, [keys.settings], {
    success: "Saved",
  });

  const resetDefaults = useResetSettings();

  // Fill the form from the settings each time they arrive. Done during render rather than in an effect.
  const [seenData, setSeenData] = useState<typeof data>(undefined);
  const fillFrom = (d: NonNullable<typeof data>) => {
    setError(null);
    setSpending(
      d.spending_bps != null
        ? formatPercent(d.spending_bps)
        : d.spending_paise == null
          ? ""
          : paiseToInput(d.spending_paise),
    );
    setAllocDay(String(d.allocation_day));
    setFloor(
      d.savings_floor_paise == null ? "" : paiseToInput(d.savings_floor_paise),
    );
    setMonths(String(d.recovery_months));
    setAutoPost(d.auto_post);
  };
  if (data !== seenData) {
    setSeenData(data);
    if (data) {
      setSpending(
        data.spending_bps != null
          ? formatPercent(data.spending_bps)
          : data.spending_paise == null
            ? ""
            : paiseToInput(data.spending_paise),
      );
      setAllocDay(String(data.allocation_day));
      setFloor(
        data.savings_floor_paise == null
          ? ""
          : paiseToInput(data.savings_floor_paise),
      );
      setMonths(String(data.recovery_months));
      setAutoPost(data.auto_post);
    }
  }

  const income = summary?.budget.income_paise ?? 0;
  const spendParsed = parseAmountOrPercent(spending);
  // what the spending box works out to in rupees a month (a percentage follows your income)
  const spendPaise =
    spendParsed.kind === "amount"
      ? spendParsed.paise
      : spendParsed.kind === "percent"
        ? Math.floor((income * spendParsed.bps) / 10_000)
        : spendParsed.kind === "empty"
          ? Math.floor((income * 20) / 100)
          : null;
  const rest =
    income > 0 && summary && spendPaise !== null
      ? income - summary.budget.needs_paise - spendPaise
      : null;

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const fl = floor.trim() ? toPaise(floor) : null;
    const day = parseDay(allocDay);
    const m = Number(months);
    if (spendParsed.kind === "invalid") {
      return setError(
        "Spending must be an amount like 18875 or a percentage like 25%.",
      );
    }
    if (floor.trim() && (fl === null || fl < 0)) {
      return setError("The floor must be zero or more.");
    }
    if (day === null || !Number.isInteger(m) || m < 1 || m > 24) {
      return setError(
        "Allocation day must be 1-31 and months between 1 and 24.",
      );
    }
    if (!data) return;
    setError(null);
    save.mutate({
      ...data,
      spending_paise: spendParsed.kind === "amount" ? spendParsed.paise : null,
      spending_bps: spendParsed.kind === "percent" ? spendParsed.bps : null,
      allocation_day: day,
      savings_floor_paise: fl,
      recovery_months: m,
      auto_post: autoPost,
    });
  };

  return (
    <Card>
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">
          <span className="brand-text">Budget</span>
        </CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Needs are your listed bills. You set how much to spend each month;
          everything left over is savings.
        </p>
      </CardHeader>
      <CardContent>
        <form onSubmit={submit} className="form-grid" noValidate>
          <div className="field">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="spending">Spending per month</Label>
              <InfoTip text="Food, other travel and fun: the pot you can swipe against. Type an amount (18875) or a percentage of your income (25%). Empty = 20% of your income." />
            </div>
            <Input
              id="spending"
              inputMode="decimal"
              value={spending}
              onChange={(e) => setSpending(e.target.value)}
              placeholder="18875 or 25%"
              aria-invalid={spendParsed.kind === "invalid"}
            />
            {spendParsed.kind === "invalid" && (
              <p className="text-xs leading-snug text-destructive">
                Use an amount like 18875 or a percentage like 25%.
              </p>
            )}
            {spendParsed.kind === "percent" &&
              spendPaise !== null &&
              income > 0 && (
                <p className="text-xs leading-snug text-muted-foreground">
                  {formatPercent(spendParsed.bps)} of {formatINR(income)} ={" "}
                  {formatINR(spendPaise)} a month, and it follows your income.
                </p>
              )}
            {rest !== null && (
              <p className="text-xs leading-snug text-muted-foreground">
                That leaves {formatINR(rest)} a month for savings.
              </p>
            )}
          </div>
          <div className="field">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="alloc-day">Savings move by day</Label>
              <InfoTip text="The day of the month your needs money and the rest of your savings should have moved to the reserve account." />
            </div>
            <Input
              id="alloc-day"
              inputMode="numeric"
              maxLength={2}
              value={allocDay}
              onChange={(e) => setAllocDay(e.target.value)}
            />
          </div>
          <div className="field col-span-full">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="floor">Savings floor in the reserve</Label>
              <InfoTip text="Savings in the reserve account that card bills must never eat. You'll be warned if it drops below this." />
            </div>
            <div className="flex flex-wrap gap-2">
              <div className="min-w-0 flex-1">
                <MoneyInput
                  id="floor"
                  value={floor}
                  onChange={(e) => setFloor(e.target.value)}
                  placeholder="No floor"
                />
              </div>
              {summary && summary.pots.reserve.account_name && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-10 shrink-0"
                  onClick={() =>
                    setFloor(
                      paiseToInput(
                        Math.max(0, summary.pots.reserve.free_savings_paise),
                      ),
                    )
                  }
                >
                  Use today's free savings
                </Button>
              )}
            </div>
          </div>
          <div className="field col-span-full">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="months">
                Pay an overspend back over (months)
              </Label>
              <InfoTip text="If you spend more than planned, the working page shows how much to rebuild each month for this many months so no FD is touched." />
            </div>
            <Input
              id="months"
              inputMode="numeric"
              maxLength={2}
              value={months}
              onChange={(e) => setMonths(e.target.value)}
            />
          </div>
          <label className="col-span-full flex items-start gap-3 text-sm">
            <input
              id="auto-post"
              type="checkbox"
              className="mt-0.5 h-4 w-4 accent-brand"
              checked={autoPost}
              onChange={(e) => setAutoPost(e.target.checked)}
            />
            <span className="field">
              <span className="font-medium">
                Update balances automatically on payday and bill dates
              </span>
              <span className="text-xs leading-snug text-muted-foreground">
                Salary is added and fixed payments are taken off your account
                balances on their dates. You only re-type a balance when it
                drifts.
              </span>
            </span>
          </label>
          {error && (
            <p role="alert" className="col-span-full text-sm text-destructive">
              {error}
            </p>
          )}
          <div className="form-actions">
            <ResetButton
              label="Undo my edits"
              disabled={!data}
              onReset={() => data && fillFrom(data)}
            />
            <ResetButton
              label="Reset to defaults"
              disabled={resetDefaults.isPending}
              confirm="Put the budget settings back to their defaults? Your saved spending, savings floor and other choices here will be cleared."
              onReset={() => resetDefaults.mutate()}
            />
            <Button type="submit" disabled={save.isPending || !data}>
              Save budget
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

const SECTIONS = [
  { id: "month", label: "This month" },
  { id: "afford", label: "Can I afford it?" },
  { id: "bills", label: "Bills" },
  { id: "goals", label: "Goals" },
  { id: "income", label: "Income" },
  { id: "fixed", label: "Fixed payments" },
] as const;
type SectionId = (typeof SECTIONS)[number]["id"];
const isSection = (v: string): v is SectionId =>
  SECTIONS.some((s) => s.id === v);

/**
 * The Budget page used to stack six forms into one 4,500px scroll. Now one section shows at a time,
 * picked from a bar that sticks to the top. The section lives in the URL hash, so /plan#afford, back
 * and refresh all still land where you were.
 */
export default function PlanPage() {
  const { data: accounts = [] } = useAccounts();
  const { data: incomes } = useIncomes();
  const { data: commitments } = useCommitments();
  const { hash } = useLocation();
  const navigate = useNavigate();
  const wanted = decodeURIComponent(hash.slice(1));
  const active: SectionId = isSection(wanted) ? wanted : "month";

  const counts: Partial<Record<SectionId, number>> = {
    income: incomes?.length,
    fixed: commitments?.length,
  };
  const pick = (id: SectionId) => {
    window.scrollTo?.({ top: 0 });
    navigate(
      { hash: id === "month" ? "" : `#${id}` },
      { replace: true, viewTransition: true },
    );
  };

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page page-narrow">
        <div className="mb-4 md:mb-6">
          <h1 className="page-title">Budget</h1>
          <p className="page-sub">
            This month's plan: what to set aside, what's due, and what you can
            afford.
          </p>
        </div>

        <nav
          aria-label="Budget sections"
          className="sticky top-0 z-30 -mx-4 mb-6 border-b border-border bg-background/90 px-4 backdrop-blur md:-mx-0 md:px-0"
        >
          <ScrollFade watch={active}>
            {SECTIONS.map((s) => {
              const on = s.id === active;
              const n = counts[s.id];
              return (
                <button
                  key={s.id}
                  type="button"
                  aria-current={on ? "true" : undefined}
                  onClick={() => pick(s.id)}
                  className={cn(
                    "inline-flex min-h-11 shrink-0 items-center gap-1.5 whitespace-nowrap rounded-full px-3.5 text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    on
                      ? "bg-brand-gradient font-medium shadow-[0_8px_18px_-10px_hsl(var(--brand)/0.8)]"
                      : "text-muted-foreground hover:bg-brand-tint hover:text-brand-ink",
                  )}
                >
                  {s.label}
                  {n ? (
                    <span
                      className={cn(
                        "text-xs tabular-nums",
                        on ? "opacity-70" : "text-muted-foreground",
                      )}
                    >
                      {n}
                    </span>
                  ) : null}
                </button>
              );
            })}
          </ScrollFade>
        </nav>

        <div className="stack">
          {active === "month" && <GuidanceCard />}
          {active === "afford" && <AffordPanel />}
          {active === "bills" && <BillsPanel />}
          {active === "goals" && <GoalsPanel />}
          {active === "income" && <IncomeSection accounts={accounts} />}
          {active === "fixed" && <CommitmentsSection accounts={accounts} />}
          <p className="text-sm text-muted-foreground">
            Pay day, buffers and how the pots split are in{" "}
            <Link to="/account" className="underline underline-offset-4">
              Settings › Budget settings
            </Link>
            .
          </p>
        </div>
      </main>
    </div>
  );
}
