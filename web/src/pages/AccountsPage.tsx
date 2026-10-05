import { useState } from "react";
import { Lock, LockOpen, Trash2 } from "lucide-react";

import { accountApi } from "@/services/api";
import type { Account, AccountKind } from "@/services/types";
import { keys, useAccounts, usePlannerMutation } from "@/hooks/usePlanner";
import { Header } from "@/components/layout/Header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { daysAgo, formatDay } from "@/utils/dates";
import { formatINR, paiseToInput, toPaise } from "@/utils/money";
import { cn } from "@/utils/cn";

const KIND_LABEL: Record<AccountKind, string> = {
  bank: "Bank account",
  cash: "Cash",
  fd: "Fixed deposit",
  rd: "Recurring deposit",
};
const STALE_DAYS = 7;

function AccountRow({ account }: { account: Account }) {
  const [balance, setBalance] = useState(paiseToInput(account.balance_paise));
  const refresh = [keys.accounts];
  const updateBalance = usePlannerMutation(
    (paise: number) => accountApi.updateBalance(account.account_id, paise),
    refresh,
    { success: "Balance updated" },
  );
  const toggleLock = usePlannerMutation(
    () =>
      accountApi.update({
        account_id: account.account_id,
        name: account.name,
        kind: account.kind,
        locked: !account.locked,
        interest_bps: account.interest_bps,
        maturity_date: account.maturity_date,
      }),
    refresh,
  );
  const remove = usePlannerMutation(() => accountApi.remove(account.account_id), refresh, {
    success: "Account removed",
  });

  const parsed = toPaise(balance);
  const dirty = parsed !== null && parsed !== account.balance_paise;
  const age = daysAgo(account.updated_at);
  const stale = !account.locked && age > STALE_DAYS;

  return (
    <li className="space-y-3 rounded-xl border p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate font-semibold">{account.name}</p>
          <p className="text-xs text-muted-foreground">
            {KIND_LABEL[account.kind]}
            {account.interest_bps != null && ` · ${(account.interest_bps / 100).toFixed(2)}%`}
            {account.maturity_date && ` · matures ${formatDay(account.maturity_date)} ${account.maturity_date.slice(0, 4)}`}
          </p>
        </div>
        <p className="text-lg font-bold tabular-nums">{formatINR(account.balance_paise)}</p>
      </div>

      <div className="flex flex-wrap items-end gap-2">
        <div className="min-w-32 flex-1 space-y-1">
          <Label htmlFor={`bal-${account.account_id}`} className="text-xs">
            Balance now
          </Label>
          <MoneyInput
            id={`bal-${account.account_id}`}
            value={balance}
            onChange={(e) => setBalance(e.target.value)}
            aria-invalid={parsed === null}
          />
        </div>
        <Button
          size="sm"
          disabled={!dirty || updateBalance.isPending}
          onClick={() => parsed !== null && updateBalance.mutate(parsed)}
        >
          Update
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={() => toggleLock.mutate(undefined)}
          disabled={toggleLock.isPending}
          title={account.locked ? "Locked: not counted in Safe to Spend. Click to unlock." : "Counted in Safe to Spend. Click to lock."}
        >
          {account.locked ? <Lock className="mr-1.5 h-4 w-4" /> : <LockOpen className="mr-1.5 h-4 w-4" />}
          {account.locked ? "Locked" : "Spendable"}
        </Button>
        <Button
          size="icon"
          variant="ghost"
          aria-label={`Remove ${account.name}`}
          onClick={() => window.confirm(`Remove "${account.name}"? Payments tied to it become "any account".`) && remove.mutate(undefined)}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
      <p className={cn("text-xs", stale ? "font-medium text-amber-600 dark:text-amber-400" : "text-muted-foreground")}>
        {age === 0 ? "Updated today" : `Updated ${age} day${age === 1 ? "" : "s"} ago`}
        {stale && ", update it for a reliable number"}
      </p>
    </li>
  );
}

function AddAccountForm() {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<AccountKind>("bank");
  const [balance, setBalance] = useState("");
  const [locked, setLocked] = useState<boolean | null>(null); // null = default for the kind
  const [rate, setRate] = useState("");
  const [maturity, setMaturity] = useState("");
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(accountApi.create, [keys.accounts], { success: "Account added" });

  const isSavings = kind === "fd" || kind === "rd";

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const paise = balance.trim() ? toPaise(balance) : 0;
    const pct = rate.trim() ? Number(rate) : null;
    if (!name.trim()) return setError("Give the account a name.");
    if (paise === null) return setError("Balance must be an amount.");
    if (pct !== null && !(pct >= 0)) return setError("Interest rate must be a number.");
    setError(null);
    create.mutate(
      {
        name: name.trim(),
        kind,
        balance_paise: paise,
        locked: locked ?? undefined,
        interest_bps: isSavings && pct !== null ? Math.round(pct * 100) : null,
        maturity_date: isSavings && maturity ? maturity : null,
      },
      {
        onSuccess: () => {
          setName("");
          setBalance("");
          setRate("");
          setMaturity("");
          setLocked(null);
        },
      },
    );
  };

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Add an account</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={submit} className="grid gap-3 sm:grid-cols-2" noValidate>
          <div className="space-y-1">
            <Label htmlFor="acc-name">Name</Label>
            <Input id="acc-name" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. HDFC Savings" />
          </div>
          <div className="space-y-1">
            <Label htmlFor="acc-kind">Type</Label>
            <Select id="acc-kind" value={kind} onChange={(e) => setKind(e.target.value as AccountKind)}>
              {(Object.keys(KIND_LABEL) as AccountKind[]).map((k) => (
                <option key={k} value={k}>
                  {KIND_LABEL[k]}
                </option>
              ))}
            </Select>
          </div>
          <div className="space-y-1">
            <Label htmlFor="acc-balance">Balance</Label>
            <MoneyInput id="acc-balance" value={balance} onChange={(e) => setBalance(e.target.value)} />
          </div>
          <label className="flex items-center gap-2 self-end pb-2 text-sm">
            <input
              type="checkbox"
              checked={locked ?? isSavings}
              onChange={(e) => setLocked(e.target.checked)}
              className="h-4 w-4"
            />
            Don't touch (locked, left out of Safe to Spend)
          </label>
          {isSavings && (
            <>
              <div className="space-y-1">
                <Label htmlFor="acc-rate">Interest rate % (optional)</Label>
                <Input id="acc-rate" inputMode="decimal" value={rate} onChange={(e) => setRate(e.target.value)} placeholder="7.1" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="acc-maturity">Maturity date (optional)</Label>
                <Input id="acc-maturity" type="date" value={maturity} onChange={(e) => setMaturity(e.target.value)} />
              </div>
            </>
          )}
          {kind === "rd" && (
            <p className="text-xs text-muted-foreground sm:col-span-2">
              The monthly instalment goes on the Plan page as a commitment, so it counts against Safe to Spend.
            </p>
          )}
          {error && <p role="alert" className="text-sm text-destructive sm:col-span-2">{error}</p>}
          <Button type="submit" className="sm:col-span-2" disabled={create.isPending}>
            {create.isPending ? "Adding..." : "Add account"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

export default function AccountsPage() {
  const { data: accounts, isLoading, isError } = useAccounts();
  const spendable = accounts?.filter((a) => a.kind === "bank" || a.kind === "cash") ?? [];
  const savings = accounts?.filter((a) => a.kind === "fd" || a.kind === "rd") ?? [];

  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />
      <main className="container mx-auto max-w-3xl space-y-8 px-4 py-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Accounts</h1>
          <p className="text-muted-foreground">Keep balances current. Locked accounts (like an FD) are left out of Safe to Spend.</p>
        </div>

        {isLoading && <div className="h-24 animate-pulse rounded-xl bg-muted/40" />}
        {isError && <p className="text-destructive">Couldn't load accounts.</p>}

        {accounts && (
          <>
            <section className="space-y-3">
              <h2 className="text-lg font-bold">Bank &amp; cash</h2>
              {spendable.length === 0 ? (
                <p className="rounded-xl border border-dashed p-4 text-sm text-muted-foreground">No accounts yet.</p>
              ) : (
                <ul className="space-y-3">{spendable.map((a) => <AccountRow key={a.account_id} account={a} />)}</ul>
              )}
            </section>
            <section className="space-y-3">
              <h2 className="text-lg font-bold">Deposits (FD / RD)</h2>
              {savings.length === 0 ? (
                <p className="rounded-xl border border-dashed p-4 text-sm text-muted-foreground">No deposits yet.</p>
              ) : (
                <ul className="space-y-3">{savings.map((a) => <AccountRow key={a.account_id} account={a} />)}</ul>
              )}
            </section>
          </>
        )}

        <AddAccountForm />
      </main>
    </div>
  );
}
