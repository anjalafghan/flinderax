import type { Pots } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

function Row({ label, value, strong, tone }: { label: string; value: number; strong?: boolean; tone?: "bad" | "good" }) {
  return (
    <div className={cn("flex justify-between gap-4 text-sm", strong && "font-semibold")}>
      <dt className="min-w-0 text-muted-foreground">{label}</dt>
      <dd
        className={cn(
          "shrink-0 tabular-nums",
          tone === "bad" && "text-over",
          tone === "good" && "text-good",
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
    <section aria-label="Where your money sits" className="space-y-4">
      <h2 className="font-display text-[32px] leading-none tracking-tight">Where your money sits</h2>
      <div className="grid gap-4 md:grid-cols-3">
        <div className="space-y-3 glass rounded-3xl p-5 md:p-6">
          <p className="font-semibold">{spending.account_name ?? "Spending"}</p>
          <dl className="space-y-2">
            <Row label="Balance" value={spending.balance_paise} />
            <Row label="Leaves before payday" value={neg(spending.committed_paise)} />
            <Row label="Still to move to reserve" value={neg(spending.to_move_paise)} />
            <Row label="Free to use" value={spending.free_paise} strong tone={spending.free_paise < 0 ? "bad" : undefined} />
          </dl>
        </div>
        <div className="space-y-3 glass rounded-3xl p-5 md:p-6">
          <p className="font-semibold">{reserve.account_name ?? "Reserve"}</p>
          {reserve.account_name ? (
            <dl className="space-y-2">
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
            <p className="text-sm leading-snug text-muted-foreground">Mark an account as "reserve" on the Accounts page.</p>
          )}
        </div>
        <div className="space-y-3 glass rounded-3xl p-5 md:p-6">
          <p className="font-semibold">FDs &amp; deposits</p>
          <dl className="space-y-2">
            <Row label={`${locked.count} locked`} value={locked.total_paise} strong />
          </dl>
          <p className="text-xs text-muted-foreground">Never counted as spendable.</p>
        </div>
      </div>
    </section>
  );
}
