import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { LineChart, Trash2 } from "lucide-react";

import { planApi } from "@/services/api";
import type { Account, Bucket, Commitment, Income } from "@/services/types";
import { keys, useAccounts, useCommitments, useIncomes, usePlannerMutation, useResetSettings, useSettings, useSummary } from "@/hooks/usePlanner";
import { ResetButton } from "@/components/ui/reset-button";
import { Header } from "@/components/layout/Header";
import { AffordPanel } from "@/components/feature/AffordPanel";
import { BillsPanel } from "@/components/feature/BillsPanel";
import { GoalsPanel } from "@/components/feature/GoalsPanel";
import { RewardsPanel } from "@/components/feature/RewardsPanel";
import { AccountSelect } from "@/components/feature/AccountSelect";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { InfoTip } from "@/components/ui/info-tip";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { formatINR, formatPercent, paiseToInput, parseAmountOrPercent, toPaise } from "@/utils/money";
import { ordinal } from "@/utils/dates";
import { BUCKET_HELP, BUCKET_LABEL, defaultBucket } from "@/utils/buckets";
import { cn } from "@/utils/cn";

const CATEGORIES = ["bills", "gym", "house", "sip", "rd", "subscription", "donation", "entertainment", "other"];
const BUCKETS: Bucket[] = ["needs", "savings", "wants"];
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

const parseDay = (v: string): number | null => {
  const n = Number(v);
  return v.trim() && Number.isInteger(n) && n >= 1 && n <= 31 ? n : null;
};
const accountName = (accounts: Account[], id: string | null) =>
  id ? (accounts.find((a) => a.account_id === id)?.name ?? "Unknown account") : "Any account";

// ---------- income ----------

function IncomeSection({ accounts }: { accounts: Account[] }) {
  const { data: incomes } = useIncomes();
  const [f, setF] = useState({ label: "", amount: "", day: "", account: "" });
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(planApi.createIncome, [keys.incomes], { success: "Income added. Nice!" });
  const remove = usePlannerMutation(planApi.removeIncome, [keys.incomes], { success: "Income removed" });

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(f.amount);
    const day = parseDay(f.day);
    if (!f.label.trim() || amount === null || amount <= 0 || day === null) {
      return setError("Add a name, a positive amount and the day of the month (1-31).");
    }
    setError(null);
    create.mutate(
      { label: f.label.trim(), amount_paise: amount, day_of_month: day, account_id: f.account || null },
      { onSuccess: () => setF({ label: "", amount: "", day: "", account: "" }) },
    );
  };

  return (
    <Card>
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">Income</CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">Your next payday sets the "until" date for Safe to Spend.</p>
      </CardHeader>
      <CardContent className="space-y-6">
        <ul className="ledger">
          {incomes?.map((i: Income) => (
            <li key={i.income_id} className="ledger-row">
              <div className="min-w-0">
                <p className="truncate text-[15px] font-medium" title={i.label}>{i.label}</p>
                <p className="text-[13px] leading-snug text-muted-foreground">
                  {formatINR(i.amount_paise)} on the {ordinal(i.day_of_month)}, into {accountName(accounts, i.account_id)}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="h-9 w-9 shrink-0"
                aria-label={`Remove ${i.label}`}
                onClick={() => window.confirm(`Remove "${i.label}"?`) && remove.mutate(i.income_id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </li>
          ))}
        </ul>
        {incomes?.length === 0 && <p className="empty-note">No income yet. Add your salary below.</p>}
        <form onSubmit={submit} className="form-grid border-t border-border pt-6" noValidate>
          <div className="field">
            <Label htmlFor="inc-label">Name</Label>
            <Input id="inc-label" value={f.label} onChange={(e) => setF({ ...f, label: e.target.value })} placeholder="Salary" />
          </div>
          <div className="field">
            <Label htmlFor="inc-amount">Amount</Label>
            <MoneyInput id="inc-amount" value={f.amount} onChange={(e) => setF({ ...f, amount: e.target.value })} />
          </div>
          <div className="field">
            <Label htmlFor="inc-day">Day of month</Label>
            <Input id="inc-day" inputMode="numeric" maxLength={2} value={f.day} onChange={(e) => setF({ ...f, day: e.target.value })} placeholder="30" />
          </div>
          <div className="field">
            <Label htmlFor="inc-account">Credited to</Label>
            <AccountSelect id="inc-account" accounts={accounts} value={f.account} onChange={(v) => setF({ ...f, account: v })} />
          </div>
          {error && <p role="alert" className="col-span-full text-sm text-destructive">{error}</p>}
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

function CommitmentRow({ c, accounts }: { c: Commitment; accounts: Account[] }) {
  const refresh = [keys.commitments];
  const update = usePlannerMutation(planApi.updateCommitment, refresh);
  const remove = usePlannerMutation(planApi.removeCommitment, refresh, { success: "Removed" });
  const when =
    c.frequency === "yearly"
      ? `yearly${c.due_month ? ` in ${MONTHS[c.due_month - 1]}` : ", month not set"} on the ${ordinal(c.day_of_month)}`
      : `on the ${ordinal(c.day_of_month)}`;
  return (
    <li className={cn("ledger-row flex-wrap text-sm", !c.active && "opacity-60")}>
      <div className="min-w-0">
        <p className="truncate text-[15px] font-medium" title={c.label}>{c.label}</p>
        <p className="text-[13px] leading-snug text-muted-foreground">
          {formatINR(c.amount_paise)} {when} · {accountName(accounts, c.account_id)}
          {c.to_account_id && ` → ${accountName(accounts, c.to_account_id)}`} · {BUCKET_LABEL[c.bucket]}
          {c.frequency === "yearly" && ` · ${formatINR(Math.floor(c.amount_paise / 12))}/month set aside`}
          {c.start_month && ` · from ${c.start_month}`}
          {c.end_month && ` · until ${c.end_month}`}
        </p>
      </div>
      <div className="flex shrink-0 flex-wrap items-center gap-1">
        <Select
          aria-label={`Bucket for ${c.label}`}
          className="h-9 w-auto px-2 text-xs"
          value={c.bucket}
          onChange={(e) => update.mutate({ ...c, bucket: e.target.value as Bucket })}
        >
          {BUCKETS.map((b) => <option key={b} value={b}>{BUCKET_LABEL[b]}</option>)}
        </Select>
        <Button variant="outline" size="sm" onClick={() => update.mutate({ ...c, active: !c.active })} disabled={update.isPending}>
          {c.active ? "Pause" : "Resume"}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-9 w-9"
          aria-label={`Remove ${c.label}`}
          onClick={() => window.confirm(`Remove "${c.label}"?`) && remove.mutate(c.commitment_id)}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </li>
  );
}

function CommitmentsSection({ accounts }: { accounts: Account[] }) {
  const { data: commitments } = useCommitments();
  const blank = {
    label: "", amount: "", day: "", account: "", to: "", category: "bills", bucket: "needs" as Bucket,
    yearly: false, dueMonth: "", start: "", end: "",
  };
  const [f, setF] = useState(blank);
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(planApi.createCommitment, [keys.commitments], { success: "Added" });

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(f.amount);
    const day = parseDay(f.day);
    if (!f.label.trim() || amount === null || amount <= 0 || day === null) {
      return setError("Add a name, a positive amount and the day of the month (1-31).");
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
        active: true,
        start_month: f.start || null,
        end_month: f.end || null,
      },
      { onSuccess: () => setF(blank) },
    );
  };

  return (
    <Card>
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">Fixed payments</CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Anything paid on a fixed day: SIPs, house, phone and internet, gym, subscriptions, an RD instalment. Yearly bills
          are set aside a twelfth a month. Each counts toward Needs, Savings or Spending.
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        <ul className="ledger">
          {commitments?.map((c) => <CommitmentRow key={c.commitment_id} c={c} accounts={accounts} />)}
        </ul>
        {commitments?.length === 0 && <p className="empty-note">Nothing yet.</p>}
        <form onSubmit={submit} className="form-grid border-t border-border pt-6" noValidate>
          <div className="field">
            <Label htmlFor="cm-label">Name</Label>
            <Input id="cm-label" value={f.label} onChange={(e) => setF({ ...f, label: e.target.value })} placeholder="e.g. Phone bill" />
          </div>
          <div className="field">
            <Label htmlFor="cm-amount">{f.yearly ? "Amount per year" : "Amount"}</Label>
            <MoneyInput id="cm-amount" value={f.amount} onChange={(e) => setF({ ...f, amount: e.target.value })} />
          </div>
          <div className="field">
            <Label htmlFor="cm-day">Day of month</Label>
            <Input id="cm-day" inputMode="numeric" maxLength={2} value={f.day} onChange={(e) => setF({ ...f, day: e.target.value })} placeholder="10" />
          </div>
          <div className="field">
            <Label htmlFor="cm-account">Paid from</Label>
            <AccountSelect id="cm-account" accounts={accounts} value={f.account} onChange={(v) => setF({ ...f, account: v })} />
          </div>
          <label className="col-span-full flex items-start gap-3 text-sm">
            <input
              type="checkbox"
              className="mt-0.5 h-4 w-4 accent-foreground"
              checked={f.yearly}
              onChange={(e) => setF({ ...f, yearly: e.target.checked })}
            />
            <span>Yearly bill (set aside monthly, due once a year)</span>
          </label>
          {f.yearly && (
            <div className="field col-span-full">
              <Label htmlFor="cm-due-month">Due month (leave empty if you don't remember)</Label>
              <Select id="cm-due-month" value={f.dueMonth} onChange={(e) => setF({ ...f, dueMonth: e.target.value })}>
                <option value="">Not sure yet</option>
                {MONTHS.map((m, i) => <option key={m} value={i + 1}>{m}</option>)}
              </Select>
            </div>
          )}
          <div className="field">
            <Label htmlFor="cm-category">Category</Label>
            <Select
              id="cm-category"
              value={f.category}
              onChange={(e) => setF({ ...f, category: e.target.value, bucket: defaultBucket(e.target.value) })}
            >
              {CATEGORIES.map((c) => <option key={c} value={c}>{c}</option>)}
            </Select>
          </div>
          <div className="field">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="cm-bucket">Counts toward</Label>
              <InfoTip text={BUCKET_HELP[f.bucket]} />
            </div>
            <Select id="cm-bucket" value={f.bucket} onChange={(e) => setF({ ...f, bucket: e.target.value as Bucket })}>
              {BUCKETS.map((b) => <option key={b} value={b}>{BUCKET_LABEL[b]}</option>)}
            </Select>
          </div>
          <div className="field col-span-full">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="cm-to">Goes into (optional)</Label>
              <InfoTip text="For an RD or FD top-up: the account the money ends up in, so the projection grows it. Leave empty if the money just leaves (a SIP, a bill)." />
            </div>
            <AccountSelect id="cm-to" accounts={accounts} value={f.to} onChange={(v) => setF({ ...f, to: v })} />
          </div>
          <div className="form-grid col-span-full">
            <div className="field">
              <Label htmlFor="cm-start">First month</Label>
              <Input id="cm-start" type="month" value={f.start} onChange={(e) => setF({ ...f, start: e.target.value })} />
            </div>
            <div className="field">
              <Label htmlFor="cm-end">Last month</Label>
              <Input id="cm-end" type="month" value={f.end} onChange={(e) => setF({ ...f, end: e.target.value })} />
            </div>
          </div>
          {error && <p role="alert" className="col-span-full text-sm text-destructive">{error}</p>}
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

function SettingsSection() {
  const { data } = useSettings();
  const { data: summary } = useSummary();
  const [spending, setSpending] = useState("");
  const [allocDay, setAllocDay] = useState("10");
  const [floor, setFloor] = useState("");
  const [months, setMonths] = useState("3");
  const [autoPost, setAutoPost] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const save = usePlannerMutation(planApi.updateSettings, [keys.settings], { success: "Saved" });

  const resetDefaults = useResetSettings();

  // Fill the form from the settings each time they arrive. Done during render rather than in an effect.
  const [seenData, setSeenData] = useState<typeof data>(undefined);
  const fillFrom = (d: NonNullable<typeof data>) => {
    setError(null);
    setSpending(
      d.spending_bps != null ? formatPercent(d.spending_bps) : d.spending_paise == null ? "" : paiseToInput(d.spending_paise),
    );
    setAllocDay(String(d.allocation_day));
    setFloor(d.savings_floor_paise == null ? "" : paiseToInput(d.savings_floor_paise));
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
      setFloor(data.savings_floor_paise == null ? "" : paiseToInput(data.savings_floor_paise));
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
  const rest = income > 0 && summary && spendPaise !== null ? income - summary.budget.needs_paise - spendPaise : null;

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const fl = floor.trim() ? toPaise(floor) : null;
    const day = parseDay(allocDay);
    const m = Number(months);
    if (spendParsed.kind === "invalid") {
      return setError("Spending must be an amount like 18875 or a percentage like 25%.");
    }
    if (floor.trim() && (fl === null || fl < 0)) {
      return setError("The floor must be zero or more.");
    }
    if (day === null || !Number.isInteger(m) || m < 1 || m > 24) {
      return setError("Allocation day must be 1-31 and months between 1 and 24.");
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
        <CardTitle className="text-[32px]">Budget</CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Needs are your listed bills. You set how much to spend each month; everything left over is savings.
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
              <p className="text-xs leading-snug text-destructive">Use an amount like 18875 or a percentage like 25%.</p>
            )}
            {spendParsed.kind === "percent" && spendPaise !== null && income > 0 && (
              <p className="text-xs leading-snug text-muted-foreground">
                {formatPercent(spendParsed.bps)} of {formatINR(income)} = {formatINR(spendPaise)} a month, and it follows your income.
              </p>
            )}
            {rest !== null && <p className="text-xs leading-snug text-muted-foreground">That leaves {formatINR(rest)} a month for savings.</p>}
          </div>
          <div className="field">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="alloc-day">Savings move by day</Label>
              <InfoTip text="The day of the month your needs money and the rest of your savings should have moved to the reserve account." />
            </div>
            <Input id="alloc-day" inputMode="numeric" maxLength={2} value={allocDay} onChange={(e) => setAllocDay(e.target.value)} />
          </div>
          <div className="field col-span-full">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="floor">Savings floor in the reserve</Label>
              <InfoTip text="Savings in the reserve account that card bills must never eat. You'll be warned if it drops below this." />
            </div>
            <div className="flex flex-wrap gap-2">
              <div className="min-w-0 flex-1">
                <MoneyInput id="floor" value={floor} onChange={(e) => setFloor(e.target.value)} placeholder="No floor" />
              </div>
              {summary && summary.pots.reserve.account_name && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-10 shrink-0"
                  onClick={() => setFloor(paiseToInput(Math.max(0, summary.pots.reserve.free_savings_paise)))}
                >
                  Use today's free savings
                </Button>
              )}
            </div>
          </div>
          <div className="field col-span-full">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="months">Pay an overspend back over (months)</Label>
              <InfoTip text="If you spend more than planned, the working page shows how much to rebuild each month for this many months so no FD is touched." />
            </div>
            <Input id="months" inputMode="numeric" maxLength={2} value={months} onChange={(e) => setMonths(e.target.value)} />
          </div>
          <label className="col-span-full flex items-start gap-3 text-sm">
            <input
              id="auto-post"
              type="checkbox"
              className="mt-0.5 h-4 w-4 accent-foreground"
              checked={autoPost}
              onChange={(e) => setAutoPost(e.target.checked)}
            />
            <span className="field">
              <span className="font-medium">Update balances automatically on payday and bill dates</span>
              <span className="text-xs leading-snug text-muted-foreground">
                Salary is added and fixed payments are taken off your account balances on their dates. You only re-type a balance when it drifts.
              </span>
            </span>
          </label>
          {error && <p role="alert" className="col-span-full text-sm text-destructive">{error}</p>}
          <div className="form-actions">
            <ResetButton label="Undo my edits" disabled={!data} onReset={() => data && fillFrom(data)} />
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

export default function PlanPage() {
  const { data: accounts = [] } = useAccounts();
  const { hash } = useLocation();

  // Links such as /plan#afford or /plan#bills scroll to that section once the page is up.
  useEffect(() => {
    if (!hash) return;
    document.getElementById(decodeURIComponent(hash.slice(1)))?.scrollIntoView({ behavior: "smooth", block: "start" });
  }, [hash]);

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl">
        <div className="mb-6 flex flex-wrap items-end justify-between gap-3 md:mb-8">
          <div>
            <h1 className="page-title">Plan</h1>
            <p className="page-sub">What's coming, what you're saving for, and what you can afford.</p>
          </div>
          <Link
            to="/what-if"
            className="inline-flex min-h-11 items-center gap-2 rounded-full border border-border px-4 text-sm font-medium"
          >
            <LineChart className="h-4 w-4" aria-hidden />
            What if...
          </Link>
        </div>
        <div className="stack">
          <Link
            to="/life"
            className="glass flex flex-wrap items-center justify-between gap-4 rounded-[1.75rem] px-6 py-5 text-foreground no-underline sm:px-8"
          >
            <span className="flex min-w-0 flex-col gap-1">
              <span className="font-display text-[26px] leading-none">Life plan</span>
              <span className="text-[13px] text-muted-foreground">Your wedding, your child, and retirement, and what each needs a month.</span>
            </span>
            <span className="text-sm font-medium">Open →</span>
          </Link>
          <AffordPanel />
          <BillsPanel />
          <GoalsPanel />
          <RewardsPanel />
          <IncomeSection accounts={accounts} />
          <CommitmentsSection accounts={accounts} />
          <SettingsSection />
        </div>
      </main>
    </div>
  );
}
