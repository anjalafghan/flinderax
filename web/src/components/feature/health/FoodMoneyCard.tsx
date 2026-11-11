import { useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { rate, rupees } from "@/services/checkin";
import { ordersPerWeek, useSaveDietCap, type FoodMoney } from "@/services/diet";
import { toPaise, paiseToInput } from "@/utils/money";

const SECTION = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
const MONTH_NAMES = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

const monthName = (ym: string) => MONTH_NAMES[Number(ym.slice(5)) - 1] ?? ym;

/** Eating out and groceries for each of the last six months, as paired bars on one shared scale. */
function MonthBars({ money }: { money: FoodMoney }) {
  const peak = Math.max(1, ...money.months.flatMap((m) => [m.eating_out_paise, m.groceries_paise]));
  const last = money.months[money.months.length - 1];
  const summary = last
    ? `Eating out and groceries for the last six months. This month eating out is ${rupees(last.eating_out_paise)} and groceries ${rupees(last.groceries_paise)}.`
    : "No food spending yet.";

  return (
    <div className="flex flex-col gap-4">
      <div role="img" aria-label={summary} className="flex flex-col gap-4">
        {money.months.map((m) => (
          <div key={m.month} className="grid grid-cols-[3rem_1fr] items-center gap-3">
            <span className="text-[13px] text-muted-foreground">{monthName(m.month)}</span>
            <div className="flex flex-col gap-1.5">
              <div className="h-3 overflow-hidden rounded-full bg-chip">
                <div
                  className="h-full rounded-full bg-primary"
                  style={{ width: `${(m.eating_out_paise / peak) * 100}%` }}
                />
              </div>
              <div className="h-3 overflow-hidden rounded-full bg-chip">
                <div
                  className="h-full rounded-full bg-muted-foreground/50"
                  style={{ width: `${(m.groceries_paise / peak) * 100}%` }}
                />
              </div>
            </div>
          </div>
        ))}
      </div>
      <ul className="ledger text-[13px]">
        {money.months.map((m) => (
          <li key={m.month} className="ledger-row">
            <span className="font-medium">{monthName(m.month)}</span>
            <span className="text-right text-muted-foreground">
              {`Eating out ${rupees(m.eating_out_paise)}, groceries ${rupees(m.groceries_paise)}, ${m.delivery_orders} ${m.delivery_orders === 1 ? "order" : "orders"}`}
            </span>
          </li>
        ))}
      </ul>
      <p className="m-0 flex flex-wrap gap-x-5 gap-y-2 text-[13px] text-muted-foreground">
        <span className="inline-flex items-center gap-2">
          <span aria-hidden className="h-2.5 w-2.5 rounded-full bg-primary" />
          Eating out
        </span>
        <span className="inline-flex items-center gap-2">
          <span aria-hidden className="h-2.5 w-2.5 rounded-full bg-muted-foreground/50" />
          Groceries
        </span>
      </p>
    </div>
  );
}

/** The food money card: six months of spend, the eating-out share, delivery frequency and an optional cap. */
export function FoodMoneyCard({ money }: { money: FoodMoney }) {
  const save = useSaveDietCap();
  const [cap, setCap] = useState(money.cap_paise === null ? "" : paiseToInput(money.cap_paise));
  const [error, setError] = useState<string | null>(null);

  const share =
    money.eating_out_share_bps === null
      ? "Not enough food spending in the last 90 days to measure yet."
      : `Eating out is ${rate(money.eating_out_share_bps)} of food over the last 90 days.`;

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    const text = cap.trim();
    let next: number | null = null;
    if (text !== "") {
      const paise = toPaise(text);
      if (paise === null || paise <= 0) {
        setError("Enter an amount in rupees, or leave it blank to clear the cap.");
        return;
      }
      next = paise;
    }
    setError(null);
    save.mutate(next, {
      onSuccess: () => toast.success(next === null ? "Eating-out cap cleared." : "Eating-out cap saved."),
      onError: (err) => toast.error(err.message || "Couldn't save the cap. Try again."),
    });
  };

  return (
    <section aria-labelledby="health-money-title" className={SECTION}>
      <div className="flex flex-col gap-2">
        <h2 id="health-money-title" className="m-0 font-display text-[32px] leading-none">
          Food money
        </h2>
        <p className="m-0 text-sm text-muted-foreground">
          Eating out and delivery count as food. Groceries are kept apart.
        </p>
      </div>

      <div className="flex flex-col gap-2">
        <p className="m-0 text-[17px]">{share}</p>
        <p className="m-0 text-[15px] text-muted-foreground">
          {`About ${ordersPerWeek(money.delivery_per_week_x100)} orders a week over the last 90 days.`}
        </p>
      </div>

      <MonthBars money={money} />

      <form className="flex flex-col gap-4 sm:max-w-md" onSubmit={onSubmit} noValidate>
        <div className="field">
          <Label htmlFor="diet-cap">Monthly cap on eating out, in rupees (blank for none)</Label>
          <MoneyInput
            id="diet-cap"
            value={cap}
            placeholder="No cap"
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? "diet-cap-error" : undefined}
            onChange={(e) => setCap(e.target.value)}
          />
        </div>
        {error && (
          <p id="diet-cap-error" role="alert" className="m-0 text-sm text-destructive">
            {error}
          </p>
        )}
        <div className="flex flex-wrap items-center gap-2">
          <Button type="submit" disabled={save.isPending}>
            Save cap
          </Button>
          <ResetButton
            label="Undo my edits"
            onReset={() => {
              setCap(money.cap_paise === null ? "" : paiseToInput(money.cap_paise));
              setError(null);
            }}
          />
        </div>
      </form>
    </section>
  );
}
