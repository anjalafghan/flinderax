import { useState } from "react";
import { Lock, LockOpen, Trash2 } from "lucide-react";

import { accountApi } from "@/services/api";
import type { Account, AccountKind, AccountRole, Renewal } from "@/services/types";
import { keys, useAccounts, usePlannerMutation } from "@/hooks/usePlanner";
import { Header } from "@/components/layout/Header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { InfoTip } from "@/components/ui/info-tip";
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
const ROLE_LABEL: Record<AccountRole, string> = {
  spending: "Spending account (salary, SIP, spending pot)",
  reserve: "Reserve (savings, card bills, needs)",
  other: "Other",
};
const ROLE_SHORT: Record<AccountRole, string> = { spending: "Spending", reserve: "Reserve", other: "" };
const RENEWAL_LABEL: Record<Renewal, string> = {
  payout: "Pay out principal + interest at maturity",
  renew_principal: "Renew the principal, pay out the interest",
  renew_all: "Renew principal + interest together",
};
const STALE_DAYS = 7;

/** "" -> null (no digits set); "1234" -> "1234"; anything else -> undefined (invalid). */
const parseLast4 = (v: string): string | null | undefined => {
  const t = v.trim();
  if (!t) return null;
  return /^\d{4}$/.test(t) ? t : undefined;
};
const LAST4_ERROR = "Last 4 digits must be exactly 4 numbers.";

/** Role, where an FD is held, rate, maturity and what happens when it matures. */
function AccountDetails({ account, accounts }: { account: Account; accounts: Account[] }) {
  const isDeposit = account.kind === "fd" || account.kind === "rd";
  const [role, setRole] = useState<AccountRole>(account.role);
  const [institution, setInstitution] = useState(account.institution ?? "");
  const [rate, setRate] = useState(account.interest_bps == null ? "" : String(account.interest_bps / 100));
  const [maturity, setMaturity] = useState(account.maturity_date ?? "");
  const [renewal, setRenewal] = useState<Renewal>(account.renewal);
  const [payout, setPayout] = useState(account.payout_account_id ?? "");
  const [last4, setLast4] = useState(account.last_4_digits ?? "");
  const [error, setError] = useState<string | null>(null);
  const save = usePlannerMutation(accountApi.update, [keys.accounts], { success: "Saved" });

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const pct = rate.trim() ? Number(rate) : null;
    const digits = parseLast4(last4);
    if (pct !== null && !(pct >= 0)) return setError("Interest rate must be a number.");
    if (digits === undefined) return setError(LAST4_ERROR);
    setError(null);
    save.mutate({
      account_id: account.account_id,
      name: account.name,
      kind: account.kind,
      locked: account.locked,
      role: isDeposit ? "other" : role,
      institution: institution.trim() || null,
      last_4_digits: digits,
      interest_bps: isDeposit && pct !== null ? Math.round(pct * 100) : null,
      maturity_date: isDeposit && maturity ? maturity : null,
      renewal,
      payout_account_id: isDeposit && payout ? payout : null,
    });
  };

  const others = accounts.filter((a) => a.account_id !== account.account_id && !a.locked);
  return (
    <details className="glass rounded-2xl p-3 text-sm">
      <summary className="cursor-pointer font-medium">Details</summary>
      <form onSubmit={submit} className="mt-3 grid gap-3 sm:grid-cols-2" noValidate>
        {!isDeposit && (
          <div className="space-y-1 sm:col-span-2">
            <div className="flex items-center gap-1.5">
              <Label htmlFor={`role-${account.account_id}`}>What is it for?</Label>
              <InfoTip text="Spending account: salary lands here and the SIP debits from here, so what's left is your spending pot. Reserve: your savings, which pay card bills and needs." />
            </div>
            <Select id={`role-${account.account_id}`} value={role} onChange={(e) => setRole(e.target.value as AccountRole)}>
              {(Object.keys(ROLE_LABEL) as AccountRole[]).map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}
            </Select>
          </div>
        )}
        <div className="space-y-1">
          <Label htmlFor={`inst-${account.account_id}`}>{isDeposit ? "Where is it held?" : "Bank / branch (optional)"}</Label>
          <Input id={`inst-${account.account_id}`} value={institution} onChange={(e) => setInstitution(e.target.value)} placeholder="e.g. SBI, Post Office" />
        </div>
        <div className="space-y-1">
          <Label htmlFor={`l4-${account.account_id}`}>Last 4 digits of account (optional)</Label>
          <Input
            id={`l4-${account.account_id}`}
            inputMode="numeric"
            maxLength={4}
            value={last4}
            onChange={(e) => setLast4(e.target.value)}
            placeholder="e.g. 1234"
          />
        </div>
        {isDeposit && (
          <>
            <div className="space-y-1">
              <Label htmlFor={`rate-${account.account_id}`}>Interest rate %</Label>
              <Input id={`rate-${account.account_id}`} inputMode="decimal" value={rate} onChange={(e) => setRate(e.target.value)} placeholder="6.35" />
            </div>
            <div className="space-y-1">
              <Label htmlFor={`mat-${account.account_id}`}>Matures on</Label>
              <Input id={`mat-${account.account_id}`} type="date" value={maturity} onChange={(e) => setMaturity(e.target.value)} />
            </div>
            <div className="space-y-1">
              <div className="flex items-center gap-1.5">
                <Label htmlFor={`ren-${account.account_id}`}>At maturity</Label>
                <InfoTip text="Used in the projections. Interest compounds quarterly, the usual convention for Indian FDs and RDs." />
              </div>
              <Select id={`ren-${account.account_id}`} value={renewal} onChange={(e) => setRenewal(e.target.value as Renewal)}>
                {(Object.keys(RENEWAL_LABEL) as Renewal[]).map((r) => <option key={r} value={r}>{RENEWAL_LABEL[r]}</option>)}
              </Select>
            </div>
            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor={`pay-${account.account_id}`}>Payout goes to</Label>
              <Select id={`pay-${account.account_id}`} value={payout} onChange={(e) => setPayout(e.target.value)}>
                <option value="">Reserve account</option>
                {others.map((a) => <option key={a.account_id} value={a.account_id}>{a.name}</option>)}
              </Select>
            </div>
          </>
        )}
        {error && <p role="alert" className="text-sm text-destructive sm:col-span-2">{error}</p>}
        <Button type="submit" size="sm" className="sm:col-span-2" disabled={save.isPending}>Save details</Button>
      </form>
    </details>
  );
}

function AccountRow({ account, accounts }: { account: Account; accounts: Account[] }) {
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
        role: account.role,
        institution: account.institution,
        last_4_digits: account.last_4_digits,
        interest_bps: account.interest_bps,
        maturity_date: account.maturity_date,
        renewal: account.renewal,
        payout_account_id: account.payout_account_id,
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
    <li className="space-y-3 glass rounded-3xl p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate font-semibold">
            {account.name}
            {ROLE_SHORT[account.role] && (
              <span className="ml-2 rounded-full bg-muted px-2 py-0.5 text-xs font-medium">{ROLE_SHORT[account.role]}</span>
            )}
          </p>
          <p className="text-xs text-muted-foreground">
            {KIND_LABEL[account.kind]}
            {account.institution && ` · ${account.institution}`}
            {account.last_4_digits && ` · ••${account.last_4_digits}`}
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
        <Button size="sm" disabled={!dirty || updateBalance.isPending} onClick={() => parsed !== null && updateBalance.mutate(parsed)}>
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
      <AccountDetails account={account} accounts={accounts} />
    </li>
  );
}

function AddAccountForm({ accounts }: { accounts: Account[] }) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<AccountKind>("bank");
  const [balance, setBalance] = useState("");
  const [role, setRole] = useState<AccountRole>("other");
  const [locked, setLocked] = useState<boolean | null>(null); // null = default for the kind
  const [institution, setInstitution] = useState("");
  const [last4, setLast4] = useState("");
  const [rate, setRate] = useState("");
  const [maturity, setMaturity] = useState("");
  const [renewal, setRenewal] = useState<Renewal>("payout");
  const [payout, setPayout] = useState("");
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(accountApi.create, [keys.accounts], { success: "Account added" });

  const isSavings = kind === "fd" || kind === "rd";

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const paise = balance.trim() ? toPaise(balance) : 0;
    const pct = rate.trim() ? Number(rate) : null;
    if (!name.trim()) return setError("Give the account a name.");
    if (paise === null) return setError("Balance must be an amount.");
    const digits = parseLast4(last4);
    if (pct !== null && !(pct >= 0)) return setError("Interest rate must be a number.");
    if (digits === undefined) return setError(LAST4_ERROR);
    setError(null);
    create.mutate(
      {
        name: name.trim(),
        kind,
        balance_paise: paise,
        locked: locked ?? undefined,
        role: isSavings ? "other" : role,
        institution: institution.trim() || null,
        last_4_digits: digits,
        interest_bps: isSavings && pct !== null ? Math.round(pct * 100) : null,
        maturity_date: isSavings && maturity ? maturity : null,
        renewal: isSavings ? renewal : "payout",
        payout_account_id: isSavings && payout ? payout : null,
      },
      {
        onSuccess: () => {
          setName("");
          setBalance("");
          setRate("");
          setMaturity("");
          setInstitution("");
          setLast4("");
          setLocked(null);
          setPayout("");
        },
      },
    );
  };

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-3xl">Add an account</CardTitle>
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
          <div className="space-y-1">
            <Label htmlFor="acc-last4">Last 4 digits of account (optional)</Label>
            <Input
              id="acc-last4"
              inputMode="numeric"
              maxLength={4}
              value={last4}
              onChange={(e) => setLast4(e.target.value)}
              placeholder="e.g. 1234"
            />
          </div>
          {!isSavings ? (
            <div className="space-y-1">
              <Label htmlFor="acc-role">What is it for?</Label>
              <Select id="acc-role" value={role} onChange={(e) => setRole(e.target.value as AccountRole)}>
                {(Object.keys(ROLE_LABEL) as AccountRole[]).map((r) => <option key={r} value={r}>{ROLE_LABEL[r]}</option>)}
              </Select>
            </div>
          ) : (
            <label className="flex items-center gap-2 self-end pb-2 text-sm">
              <input type="checkbox" checked={locked ?? true} onChange={(e) => setLocked(e.target.checked)} className="h-4 w-4" />
              Don't touch (locked, left out of Safe to Spend)
            </label>
          )}
          {isSavings && (
            <>
              <div className="space-y-1">
                <Label htmlFor="acc-inst">Where is it held?</Label>
                <Input id="acc-inst" value={institution} onChange={(e) => setInstitution(e.target.value)} placeholder="e.g. SBI, Post Office" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="acc-rate">Interest rate % (optional)</Label>
                <Input id="acc-rate" inputMode="decimal" value={rate} onChange={(e) => setRate(e.target.value)} placeholder="7.1" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="acc-maturity">Maturity date (optional)</Label>
                <Input id="acc-maturity" type="date" value={maturity} onChange={(e) => setMaturity(e.target.value)} />
              </div>
              <div className="space-y-1">
                <Label htmlFor="acc-renewal">At maturity</Label>
                <Select id="acc-renewal" value={renewal} onChange={(e) => setRenewal(e.target.value as Renewal)}>
                  {(Object.keys(RENEWAL_LABEL) as Renewal[]).map((r) => <option key={r} value={r}>{RENEWAL_LABEL[r]}</option>)}
                </Select>
              </div>
              <div className="space-y-1">
                <Label htmlFor="acc-payout">Payout goes to</Label>
                <Select id="acc-payout" value={payout} onChange={(e) => setPayout(e.target.value)}>
                  <option value="">Reserve account</option>
                  {accounts.filter((a) => !a.locked).map((a) => <option key={a.account_id} value={a.account_id}>{a.name}</option>)}
                </Select>
              </div>
            </>
          )}
          {kind === "rd" && (
            <p className="text-xs text-muted-foreground sm:col-span-2">
              The monthly instalment goes on the Plan page as a fixed payment that goes into this RD, so the projection
              grows it and it counts against your savings.
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
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl space-y-8">
        <div>
          <h1 className="page-title">Accounts</h1>
          <p className="text-muted-foreground">
            Mark one account as your spending account and one as your reserve. Locked accounts (like an FD) are never
            counted as spendable.
          </p>
        </div>

        {isLoading && <div className="h-24 animate-pulse rounded-xl bg-muted/40" />}
        {isError && <p className="text-destructive">Couldn't load accounts.</p>}

        {accounts && (
          <>
            <section className="space-y-3">
              <h2 className="font-display text-3xl leading-none">Bank &amp; cash</h2>
              {spendable.length === 0 ? (
                <p className="rounded-3xl border border-dashed border-primary/25 bg-card/30 p-4 text-sm text-muted-foreground">No accounts yet.</p>
              ) : (
                <ul className="space-y-3">{spendable.map((a) => <AccountRow key={a.account_id} account={a} accounts={accounts} />)}</ul>
              )}
            </section>
            <section className="space-y-3">
              <h2 className="font-display text-3xl leading-none">Deposits (FD / RD)</h2>
              {savings.length === 0 ? (
                <p className="rounded-3xl border border-dashed border-primary/25 bg-card/30 p-4 text-sm text-muted-foreground">No deposits yet.</p>
              ) : (
                <ul className="space-y-3">{savings.map((a) => <AccountRow key={a.account_id} account={a} accounts={accounts} />)}</ul>
              )}
            </section>
          </>
        )}

        <AddAccountForm accounts={accounts ?? []} />
      </main>
    </div>
  );
}
