import type { Pots } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";
import { Lock, Shield, Wallet } from "lucide-react";
import type { ReactNode } from "react";
import { partyColor, type PartyColor } from "@/utils/categoryColors";

/** One pot: its own colour, an icon chip and a tinted card. */
function Pot({
  color,
  icon,
  title,
  children,
}: {
  color: PartyColor;
  icon: ReactNode;
  title: string;
  children: ReactNode;
}) {
  return (
    <div
      className="relative space-y-3 overflow-hidden rounded-[28px] border p-5 md:p-6 transition-transform hover:-translate-y-0.5"
      style={{
        borderColor: partyColor(color, 0.35),
        background: `radial-gradient(120% 90% at 0% 0%, ${partyColor(color, 0.16)}, transparent 65%), hsl(var(--card))`,
        boxShadow: `0 14px 32px -20px ${partyColor(color, 0.7)}`,
      }}
    >
      <div className="flex items-center gap-2.5">
        <span
          aria-hidden
          className="grid h-9 w-9 shrink-0 place-items-center rounded-xl text-background"
          style={{ background: partyColor(color) }}
        >
          {icon}
        </span>
        <p className="min-w-0 truncate font-semibold">{title}</p>
      </div>
      {children}
    </div>
  );
}

function Row({
  label,
  value,
  strong,
  tone,
}: {
  label: string;
  value: number;
  strong?: boolean;
  tone?: "bad" | "good";
}) {
  return (
    <div
      className={cn(
        "flex justify-between gap-4 text-sm",
        strong && "font-semibold",
      )}
    >
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
      <h2 className="font-display brand-text text-[32px] leading-none tracking-tight">
        Where your money sits
      </h2>
      <div className="grid gap-4 md:grid-cols-3">
        <Pot
          color="coral"
          icon={<Wallet className="h-4 w-4" />}
          title={spending.account_name ?? "Spending"}
        >
          <dl className="space-y-2">
            <Row label="Balance" value={spending.balance_paise} />
            <Row
              label="Leaves before payday"
              value={neg(spending.committed_paise)}
            />
            <Row
              label="Still to move to reserve"
              value={neg(spending.to_move_paise)}
            />
            <Row
              label="Free to use"
              value={spending.free_paise}
              strong
              tone={spending.free_paise < 0 ? "bad" : undefined}
            />
          </dl>
        </Pot>
        <Pot
          color="sky"
          icon={<Shield className="h-4 w-4" />}
          title={reserve.account_name ?? "Reserve"}
        >
          {reserve.account_name ? (
            <dl className="space-y-2">
              <Row label="Balance" value={reserve.balance_paise} />
              <Row
                label="Card bills + needs due"
                value={neg(reserve.required_paise)}
              />
              <Row
                label="Free savings"
                value={reserve.free_savings_paise}
                strong
                tone={reserve.below_floor ? "bad" : "good"}
              />
              {reserve.floor_paise > 0 && (
                <Row label="Your floor" value={reserve.floor_paise} />
              )}
            </dl>
          ) : (
            <p className="text-sm leading-snug text-muted-foreground">
              Mark an account as "reserve" on the Accounts page.
            </p>
          )}
        </Pot>
        <Pot
          color="grape"
          icon={<Lock className="h-4 w-4" />}
          title="FDs & deposits"
        >
          <dl className="space-y-2">
            <Row
              label={`${locked.count} locked`}
              value={locked.total_paise}
              strong
            />
          </dl>
          <p className="text-xs text-muted-foreground">
            Never counted as spendable.
          </p>
        </Pot>
      </div>
    </section>
  );
}
