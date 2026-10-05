import { useEffect, useState } from "react";
import { Trash2 } from "lucide-react";

import { planApi } from "@/services/api";
import type { Account, Commitment, Income } from "@/services/types";
import { keys, useAccounts, useCommitments, useIncomes, usePlannerMutation, useSettings } from "@/hooks/usePlanner";
import { Header } from "@/components/layout/Header";
import { AccountSelect } from "@/components/feature/AccountSelect";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { InfoTip } from "@/components/ui/info-tip";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { formatINR, paiseToInput, toPaise } from "@/utils/money";
import { ordinal } from "@/utils/dates";
import { cn } from "@/utils/cn";

const CATEGORIES = ["sip", "house", "bills", "donation", "subscription", "rd", "other"];

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
  const create = usePlannerMutation(planApi.createIncome, [keys.incomes], { success: "Income added" });
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
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Income</CardTitle>
        <p className="text-sm text-muted-foreground">Your next payday sets the "until" date for Safe to Spend.</p>
      </CardHeader>
      <CardContent className="space-y-4">
        <ul className="space-y-2">
          {incomes?.map((i: Income) => (
            <li key={i.income_id} className="flex items-center justify-between gap-2 rounded-lg border p-3 text-sm">
              <div className="min-w-0">
                <p className="truncate font-medium">{i.label}</p>
                <p className="text-xs text-muted-foreground">
                  {formatINR(i.amount_paise)} on the {ordinal(i.day_of_month)}, into {accountName(accounts, i.account_id)}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                aria-label={`Remove ${i.label}`}
                onClick={() => window.confirm(`Remove "${i.label}"?`) && remove.mutate(i.income_id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </li>
          ))}
          {incomes?.length === 0 && <p className="text-sm text-muted-foreground">No income yet. Add your salary.</p>}
        </ul>
        <form onSubmit={submit} className="grid grid-cols-2 gap-2 border-t pt-4" noValidate>
          <div className="space-y-1">
            <Label htmlFor="inc-label">Name</Label>
            <Input id="inc-label" value={f.label} onChange={(e) => setF({ ...f, label: e.target.value })} placeholder="Salary" />
          </div>
          <div className="space-y-1">
            <Label htmlFor="inc-amount">Amount</Label>
            <MoneyInput id="inc-amount" value={f.amount} onChange={(e) => setF({ ...f, amount: e.target.value })} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="inc-day">Day of month</Label>
            <Input id="inc-day" inputMode="numeric" maxLength={2} value={f.day} onChange={(e) => setF({ ...f, day: e.target.value })} placeholder="30" />
          </div>
          <div className="space-y-1">
            <Label htmlFor="inc-account">Credited to</Label>
            <AccountSelect id="inc-account" accounts={accounts} value={f.account} onChange={(v) => setF({ ...f, account: v })} />
          </div>
          {error && <p role="alert" className="col-span-2 text-sm text-destructive">{error}</p>}
          <Button type="submit" variant="outline" className="col-span-2" disabled={create.isPending}>
            Add income
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

// ---------- commitments ----------

function CommitmentRow({ c, accounts }: { c: Commitment; accounts: Account[] }) {
  const refresh = [keys.commitments];
  const update = usePlannerMutation(planApi.updateCommitment, refresh);
  const remove = usePlannerMutation(planApi.removeCommitment, refresh, { success: "Commitment removed" });
  return (
    <li className={cn("flex items-center justify-between gap-2 rounded-lg border p-3 text-sm", !c.active && "opacity-60")}>
      <div className="min-w-0">
        <p className="truncate font-medium">{c.label}</p>
        <p className="text-xs text-muted-foreground">
          {formatINR(c.amount_paise)} on the {ordinal(c.day_of_month)} · {accountName(accounts, c.account_id)} · {c.category}
          {c.start_month && ` · from ${c.start_month}`}
          {c.end_month && ` · until ${c.end_month}`}
        </p>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <Button variant="outline" size="sm" onClick={() => update.mutate({ ...c, active: !c.active })} disabled={update.isPending}>
          {c.active ? "Pause" : "Resume"}
        </Button>
        <Button
          variant="ghost"
          size="icon"
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
  const blank = { label: "", amount: "", day: "", account: "", category: "sip", start: "", end: "" };
  const [f, setF] = useState(blank);
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(planApi.createCommitment, [keys.commitments], { success: "Commitment added" });

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
        category: f.category,
        active: true,
        start_month: f.start || null,
        end_month: f.end || null,
      },
      { onSuccess: () => setF(blank) },
    );
  };

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Fixed payments</CardTitle>
        <p className="text-sm text-muted-foreground">SIPs, rent, donations, subscriptions, an RD instalment, a bills fund...</p>
      </CardHeader>
      <CardContent className="space-y-4">
        <ul className="space-y-2">
          {commitments?.map((c) => <CommitmentRow key={c.commitment_id} c={c} accounts={accounts} />)}
          {commitments?.length === 0 && <p className="text-sm text-muted-foreground">Nothing yet.</p>}
        </ul>
        <form onSubmit={submit} className="grid grid-cols-2 gap-2 border-t pt-4" noValidate>
          <div className="space-y-1">
            <Label htmlFor="cm-label">Name</Label>
            <Input id="cm-label" value={f.label} onChange={(e) => setF({ ...f, label: e.target.value })} placeholder="e.g. SIP" />
          </div>
          <div className="space-y-1">
            <Label htmlFor="cm-amount">Amount</Label>
            <MoneyInput id="cm-amount" value={f.amount} onChange={(e) => setF({ ...f, amount: e.target.value })} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="cm-day">Day of month</Label>
            <Input id="cm-day" inputMode="numeric" maxLength={2} value={f.day} onChange={(e) => setF({ ...f, day: e.target.value })} placeholder="10" />
          </div>
          <div className="space-y-1">
            <Label htmlFor="cm-account">Paid from</Label>
            <AccountSelect id="cm-account" accounts={accounts} value={f.account} onChange={(v) => setF({ ...f, account: v })} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="cm-category">Category</Label>
            <Select id="cm-category" value={f.category} onChange={(e) => setF({ ...f, category: e.target.value })}>
              {CATEGORIES.map((c) => <option key={c} value={c}>{c}</option>)}
            </Select>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="cm-start" className="text-xs">First month</Label>
              <Input id="cm-start" type="month" value={f.start} onChange={(e) => setF({ ...f, start: e.target.value })} />
            </div>
            <div className="space-y-1">
              <Label htmlFor="cm-end" className="text-xs">Last month</Label>
              <Input id="cm-end" type="month" value={f.end} onChange={(e) => setF({ ...f, end: e.target.value })} />
            </div>
          </div>
          {error && <p role="alert" className="col-span-2 text-sm text-destructive">{error}</p>}
          <Button type="submit" variant="outline" className="col-span-2" disabled={create.isPending}>
            Add fixed payment
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

// ---------- settings ----------

function SettingsSection() {
  const { data } = useSettings();
  const [cushion, setCushion] = useState("");
  const [living, setLiving] = useState("");
  const [error, setError] = useState<string | null>(null);
  const save = usePlannerMutation(planApi.updateSettings, [keys.settings], { success: "Settings saved" });

  useEffect(() => {
    if (data) {
      setCushion(paiseToInput(data.cushion_paise));
      setLiving(paiseToInput(data.next_period_living_paise));
    }
  }, [data]);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const c = toPaise(cushion);
    const l = toPaise(living);
    if (c === null || l === null || c < 0 || l < 0 || !data) return setError("Both amounts must be zero or more.");
    setError(null);
    save.mutate({ ...data, cushion_paise: c, next_period_living_paise: l });
  };

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Settings</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={submit} className="grid gap-3 sm:grid-cols-2" noValidate>
          <div className="space-y-1">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="cushion">Safety cushion</Label>
              <InfoTip text="Money you never plan to spend. Safe to Spend is calculated after leaving this untouched." />
            </div>
            <MoneyInput id="cushion" value={cushion} onChange={(e) => setCushion(e.target.value)} />
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-1.5">
              <Label htmlFor="living">Living costs next period</Label>
              <InfoTip text="Food, travel and everyday spending you expect after your next payday. It's set aside so the month after is covered too." />
            </div>
            <MoneyInput id="living" value={living} onChange={(e) => setLiving(e.target.value)} />
          </div>
          {error && <p role="alert" className="text-sm text-destructive sm:col-span-2">{error}</p>}
          <Button type="submit" className="sm:col-span-2" disabled={save.isPending || !data}>
            Save settings
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

export default function PlanPage() {
  const { data: accounts = [] } = useAccounts();
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />
      <main className="container mx-auto max-w-3xl space-y-8 px-4 py-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Plan</h1>
          <p className="text-muted-foreground">What comes in, what goes out every month, and how much to keep aside.</p>
        </div>
        <IncomeSection accounts={accounts} />
        <CommitmentsSection accounts={accounts} />
        <SettingsSection />
      </main>
    </div>
  );
}
