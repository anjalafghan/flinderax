import type { Pots } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

function Row({ label, value, strong, tone }: { label: string; value: number; strong?: boolean; tone?: "bad" | "good" }) {
  return (
    <div className={cn("flex justify-between gap-3 text-sm", strong && "font-semibold")}>
      <dt className="text-muted-foreground">{label}</dt>
      <dd
        className={cn(
          "tabular-nums",
          tone === "bad" && "text-rose-600 dark:text-rose-400",
          tone === "good" && "text-emerald-600 dark:text-emerald-400",
        )}
      >
        {formatINR(value)}
      </dd>
    </div>
  );
}

/** Where your money sits: the spending account, the reserve, and what's locked away. */
export function PotsPanel({ pots }: { pots: Pots }) {
  const { spending, reserve, locked } = pots;
  const neg = (n: number) => (n === 0 ? 0 : -n);
  return (
    <section aria-label="Where your money sits" className="space-y-3">
      <h2 className="font-display text-3xl leading-none tracking-tight">Where your money sits</h2>
      <div className="grid gap-3 md:grid-cols-3">
        <div className="space-y-2 glass rounded-3xl p-4">
          <p className="font-semibold">{spending.account_name ?? "Spending"}</p>
          <dl className="space-y-1">
            <Row label="Balance" value={spending.balance_paise} />
            <Row label="Leaves before payday" value={neg(spending.committed_paise)} />
            <Row label="Still to move to reserve" value={neg(spending.to_move_paise)} />
            <Row label="Free to use" value={spending.free_paise} strong tone={spending.free_paise < 0 ? "bad" : undefined} />
          </dl>
        </div>
        <div className="space-y-2 glass rounded-3xl p-4">
          <p className="font-semibold">{reserve.account_name ?? "Reserve"}</p>
          {reserve.account_name ? (
            <dl className="space-y-1">
              <Row label="Balance" value={reserve.balance_paise} />
              <Row label="Card bills + needs due" value={neg(reserve.required_paise)} />
              <Row
                label="Free savings"
                value={reserve.free_savings_paise}
                strong
                tone={reserve.below_floor ? "bad" : "good"}
              />
              {reserve.floor_paise > 0 && <Row label="Your floor" value={reserve.floor_paise} />}
            </dl>
          ) : (
            <p className="text-sm text-muted-foreground">Mark an account as "reserve" on the Accounts page.</p>
          )}
        </div>
        <div className="space-y-2 glass rounded-3xl p-4">
          <p className="font-semibold">FDs &amp; deposits</p>
          <dl className="space-y-1">
            <Row label={`${locked.count} locked`} value={locked.total_paise} strong />
          </dl>
          <p className="text-xs text-muted-foreground">Never counted as spendable.</p>
        </div>
      </div>
    </section>
  );
}
