import { Trash2 } from "lucide-react";

import type { Account, Renewal, ScenarioKind } from "@/services/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { InfoTip } from "@/components/ui/info-tip";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import type { ScenarioDraft } from "@/utils/scenario";

const KIND_LABEL: Record<ScenarioKind, string> = {
  rd: "Recurring deposit (monthly)",
  fd: "Fixed deposit (lump sum)",
  one_off: "One-off spend",
};
const RENEWAL_LABEL: Record<Renewal, string> = {
  payout: "Pay out at maturity",
  renew_principal: "Renew principal, pay out interest",
  renew_all: "Renew principal + interest",
};

/** One what-if item. Every field is live: edit it and the projection below recomputes. */
export function ScenarioItemForm({
  draft,
  accounts,
  invalid,
  onChange,
  onRemove,
}: {
  draft: ScenarioDraft;
  accounts: Account[];
  invalid: boolean;
  onChange: (d: ScenarioDraft) => void;
  onRemove: () => void;
}) {
  const set = <K extends keyof ScenarioDraft>(k: K, v: ScenarioDraft[K]) => onChange({ ...draft, [k]: v });
  const id = (s: string) => `${s}-${draft.id}`;
  const isDeposit = draft.kind !== "one_off";
  const spendable = accounts.filter((a) => !a.locked);
  return (
    <Card className={invalid ? "border-warn/40" : undefined}>
      <CardContent className="form-grid p-5 md:p-6">
        <div className="field">
          <Label htmlFor={id("name")}>Name</Label>
          <Input id={id("name")} value={draft.name} onChange={(e) => set("name", e.target.value)} />
        </div>
        <div className="field">
          <Label htmlFor={id("kind")}>Type</Label>
          <Select id={id("kind")} value={draft.kind} onChange={(e) => set("kind", e.target.value as ScenarioKind)}>
            {(Object.keys(KIND_LABEL) as ScenarioKind[]).map((k) => <option key={k} value={k}>{KIND_LABEL[k]}</option>)}
          </Select>
        </div>
        <div className="field">
          <Label htmlFor={id("amount")}>{draft.kind === "rd" ? "Per month" : "Amount"}</Label>
          <MoneyInput id={id("amount")} value={draft.amount} onChange={(e) => set("amount", e.target.value)} />
        </div>
        <div className="field">
          <Label htmlFor={id("start")}>{draft.kind === "rd" ? "First instalment" : "Date"}</Label>
          <Input id={id("start")} type="date" value={draft.start} onChange={(e) => set("start", e.target.value)} />
        </div>
        {isDeposit && (
          <>
            <div className="field">
              <Label htmlFor={id("tenure")}>Tenure (months)</Label>
              <Input id={id("tenure")} inputMode="numeric" value={draft.tenure} onChange={(e) => set("tenure", e.target.value)} />
            </div>
            <div className="field">
              <Label htmlFor={id("rate")}>Interest rate %</Label>
              <Input id={id("rate")} inputMode="decimal" value={draft.rate} onChange={(e) => set("rate", e.target.value)} />
            </div>
          </>
        )}
        <div className="field">
          <Label htmlFor={id("from")}>Paid from</Label>
          <Select id={id("from")} value={draft.fundedFrom} onChange={(e) => set("fundedFrom", e.target.value)}>
            <option value="">Spending account</option>
            {spendable.map((a) => <option key={a.account_id} value={a.account_id}>{a.name}</option>)}
          </Select>
        </div>
        {isDeposit && (
          <div className="field">
            <Label htmlFor={id("payout")}>Payout goes to</Label>
            <Select id={id("payout")} value={draft.payoutTo} onChange={(e) => set("payoutTo", e.target.value)}>
              <option value="">Reserve account</option>
              {spendable.map((a) => <option key={a.account_id} value={a.account_id}>{a.name}</option>)}
            </Select>
          </div>
        )}
        {draft.kind === "fd" && (
          <div className="field col-span-full">
            <Label htmlFor={id("renewal")}>At maturity</Label>
            <Select id={id("renewal")} value={draft.renewal} onChange={(e) => set("renewal", e.target.value as Renewal)}>
              {(Object.keys(RENEWAL_LABEL) as Renewal[]).map((r) => <option key={r} value={r}>{RENEWAL_LABEL[r]}</option>)}
            </Select>
          </div>
        )}
        {isDeposit && (
          <>
            <div className="field">
              <div className="flex items-center gap-1.5">
                <Label htmlFor={id("target")}>Goal amount (optional)</Label>
                <InfoTip text="E.g. what the wedding needs. The page then shows whether this deposit gets you there, and what monthly amount would." />
              </div>
              <MoneyInput id={id("target")} value={draft.target} onChange={(e) => set("target", e.target.value)} />
            </div>
            <div className="field">
              <Label htmlFor={id("target-date")}>Needed by</Label>
              <Input id={id("target-date")} type="date" value={draft.targetDate} onChange={(e) => set("targetDate", e.target.value)} />
            </div>
          </>
        )}
        <div className="col-span-full flex flex-wrap items-center justify-between gap-3">
          <p className="text-xs text-warn">{invalid ? "Fill in a valid amount, date, tenure and rate." : ""}</p>
          <Button variant="ghost" size="sm" onClick={onRemove}>
            <Trash2 className="mr-1 h-4 w-4" /> Remove
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
