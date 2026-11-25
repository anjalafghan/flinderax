import { Link } from "react-router-dom";
import {
  HeartHandshake,
  PiggyBank,
  Salad,
  Sprout,
  Wallet,
  type LucideIcon,
} from "lucide-react";

import {
  useRewardFund,
  type AreaScore,
  type FundArea,
} from "@/services/rewards";
import { formatINR } from "@/utils/money";
import { monthName, payoutLine, pct } from "@/utils/rewards";
import { cn } from "@/utils/cn";
import SloshGauge from "@/components/bits/SloshGauge";
import { MoneyCounter } from "@/components/feature/SafeToSpendHero";
import { useTheme } from "@/context/ThemeContext";

const ICON: Record<FundArea, LucideIcon> = {
  money: PiggyBank,
  health: Salad,
  relationships: HeartHandshake,
  future: Sprout,
};

/** One colour per area: money mint, health coral, people rose, the future grape. */
const AREA_C: Record<FundArea, string> = {
  money: "mint",
  health: "coral",
  relationships: "rose",
  future: "grape",
};

function AreaBar({ a, focus }: { a: AreaScore; focus: boolean }) {
  const Icon = ICON[a.area];
  const c = AREA_C[a.area];
  const full = a.score_bps >= 10_000;
  return (
    <li
      className={cn(
        "rounded-[14px] border px-3 py-2.5",
        focus && "border-2 border-dashed",
      )}
      style={{
        borderColor: focus
          ? "hsl(var(--warn) / 0.7)"
          : `hsl(var(--c-${c}) / 0.35)`,
        background: `linear-gradient(150deg, hsl(var(--c-${c}) / 0.12), transparent 70%)`,
      }}
    >
      <div className="flex items-center justify-between gap-2 text-[13px]">
        <span className="flex items-center gap-1.5 font-medium">
          <span
            aria-hidden
            className="inline-flex h-6 w-6 items-center justify-center rounded-full text-background"
            style={{ background: `hsl(var(--c-${c}))` }}
          >
            <Icon className="h-3.5 w-3.5" strokeWidth={1.8} />
          </span>
          {a.label}
          <span className="font-normal text-muted-foreground">
            · {pct(a.weight_bps)} of the pot
          </span>
        </span>
        <span
          className="font-semibold tabular-nums"
          style={{ color: `hsl(var(--c-${c}))` }}
        >
          {pct(a.score_bps)}
        </span>
      </div>
      <div
        className="mt-2 h-2 overflow-hidden rounded-full"
        style={{ background: `hsl(var(--c-${c}) / 0.15)` }}
        role="progressbar"
        aria-label={`${a.label} this month`}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.floor(a.score_bps / 100)}
      >
        <div
          className="h-full rounded-full transition-[width] duration-700 ease-out"
          style={{
            width: `${Math.min(100, a.score_bps / 100)}%`,
            background: full
              ? `linear-gradient(90deg, hsl(var(--c-${c})), hsl(var(--c-sun)))`
              : `hsl(var(--c-${c}))`,
          }}
        />
      </div>
      <p className="m-0 mt-1.5 text-xs text-muted-foreground tabular-nums">
        {a.raw} of {a.target} points{full ? ", full marks" : ""}
      </p>
      {focus && <p className="m-0 mt-1 text-xs">{a.tip}</p>}
    </li>
  );
}

/**
 * The reward fund on Today: what the wallet holds, how this month is scoring across money,
 * health, relationships and the future, and what past months paid. Targets learn from your own
 * months; the payout (up to the monthly most) is only ever money really saved under the allowance.
 */
export function RewardFundCard() {
  const { data } = useRewardFund();
  const { theme } = useTheme();
  if (!data) return null;
  const { score, days_left, month } = data.this_month;
  // How full this month's tank is: what has been earned against the most a month can pay.
  const level =
    data.max_payout_paise > 0
      ? Math.min(
          100,
          Math.round((score.earned_paise / data.max_payout_paise) * 100),
        )
      : 0;

  return (
    <section
      aria-label="Reward fund"
      className="glass my-6 w-full rounded-[22px] p-5"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <Wallet
              className="h-6 w-6 text-mint"
              strokeWidth={1.6}
              aria-hidden
            />
            <h2 className="brand-text m-0 font-display text-[32px] leading-none">
              Reward fund
            </h2>
          </div>
          <p className="m-0 mt-2 max-w-prose text-[13px] text-muted-foreground">
            Each month pays you up to {formatINR(data.max_payout_paise)} for
            looking after your money, health, people and future, but only from
            what you save under your allowance. Whatever isn't earned stays
            saved.
          </p>
        </div>
        <div className="flex items-end gap-4">
          <div className="text-right">
            <p className="m-0 text-[13px] text-muted-foreground">
              In your wallet
            </p>
            <p className="m-0 font-display text-[34px] leading-none tabular-nums text-[hsl(var(--foreground))] dark:text-mint">
              <MoneyCounter paise={data.wallet_paise} />
            </p>
            <Link
              to="/cards#rewards"
              className="inline-flex min-h-11 items-center text-[13px] font-semibold text-brand-ink"
            >
              Spend it on a treat
            </Link>
          </div>
          {/* The tank fills towards this month's most; it sloshes when the number moves. */}
          <div className="flex flex-col items-center gap-1">
            <SloshGauge
              value={level}
              liquidColor={theme === "dark" ? "#2c6b4d" : "#2c6b4d"}
              glassColor="hsl(var(--c-mint) / 0.12)"
              width={64}
              height={112}
              radius={18}
              ticks={4}
              ariaLabel={`Earned ${formatINR(score.earned_paise)} of ${formatINR(data.max_payout_paise)} this month`}
              className="border border-mint/40 text-[hsl(var(--foreground))] dark:text-mint"
            />
            <span className="text-[11px] text-muted-foreground">
              of {formatINR(data.max_payout_paise)}
            </span>
          </div>
        </div>
      </div>

      <div className="mt-5">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <h3 className="m-0 text-[15px] font-medium">
            {monthName(month)} so far:{" "}
            <span className="tabular-nums text-brand-ink">
              {pct(score.score_bps)}
            </span>
          </h3>
          <span className="pill pill-sun">
            {days_left === 1 ? "1 day left" : `${days_left} days left`}
          </span>
        </div>
        <p className="m-0 mt-1 text-[13px]">{payoutLine(score, formatINR)}</p>
        <ul className="m-0 mt-3 grid list-none grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-2 p-0">
          {score.areas.map((a) => (
            <AreaBar key={a.area} a={a} focus={a.area === score.focus} />
          ))}
        </ul>
        <p className="m-0 mt-2 text-xs text-muted-foreground">
          Targets learn from your own last six months, with a small stretch:
          matching your usual month earns most of an area, beating it earns all
          of it.
        </p>
      </div>

      {data.months.length > 0 && (
        <details className="mt-4 rounded-2xl border border-border px-4 py-3 text-sm">
          <summary className="cursor-pointer font-medium">
            Past months ({data.months.length})
          </summary>
          <ul className="ledger mt-2">
            {data.months.map((m) => (
              <li key={m.month} className="ledger-row">
                <span className="min-w-0 truncate font-medium">
                  {monthName(m.month)}
                </span>
                <span className="text-[13px] text-muted-foreground tabular-nums">
                  {pct(m.score_bps)} · paid {formatINR(m.payout_paise)}
                  {m.earned_paise > m.payout_paise &&
                    ` of ${formatINR(m.earned_paise)} earned`}
                </span>
              </li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
