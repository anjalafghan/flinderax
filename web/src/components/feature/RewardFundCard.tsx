import { Link } from "react-router-dom";
import { HeartHandshake, PiggyBank, Salad, Sprout, Wallet, type LucideIcon } from "lucide-react";

import { useRewardFund, type AreaScore, type FundArea } from "@/services/rewards";
import { formatINR } from "@/utils/money";
import { monthName, payoutLine, pct } from "@/utils/rewards";
import { cn } from "@/utils/cn";

const ICON: Record<FundArea, LucideIcon> = {
  money: PiggyBank,
  health: Salad,
  relationships: HeartHandshake,
  future: Sprout,
};

function AreaBar({ a, focus }: { a: AreaScore; focus: boolean }) {
  const Icon = ICON[a.area];
  const full = a.score_bps >= 10_000;
  return (
    <li className={cn("rounded-[14px] border border-border px-3 py-2.5", focus && "border-warn")}>
      <div className="flex items-center justify-between gap-2 text-[13px]">
        <span className="flex items-center gap-1.5 font-medium">
          <Icon className="h-4 w-4 text-muted-foreground" strokeWidth={1.6} aria-hidden />
          {a.label}
          <span className="font-normal text-muted-foreground">· {pct(a.weight_bps)} of the pot</span>
        </span>
        <span className="tabular-nums">{pct(a.score_bps)}</span>
      </div>
      <div
        className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted"
        role="progressbar"
        aria-label={`${a.label} this month`}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.floor(a.score_bps / 100)}
      >
        <div className={cn("h-full rounded-full", full ? "bg-good" : "bg-primary")} style={{ width: `${Math.min(100, a.score_bps / 100)}%` }} />
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
  if (!data) return null;
  const { score, days_left, month } = data.this_month;

  return (
    <section aria-label="Reward fund" className="my-6 w-full rounded-[22px] border border-border p-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <Wallet className="h-5 w-5 text-muted-foreground" strokeWidth={1.6} aria-hidden />
            <h2 className="m-0 font-display text-[32px] leading-none">Reward fund</h2>
          </div>
          <p className="m-0 mt-2 max-w-prose text-[13px] text-muted-foreground">
            Each month pays you up to {formatINR(data.max_payout_paise)} for looking after your money, health, people and future,
            but only from what you save under your allowance. Whatever isn't earned stays saved.
          </p>
        </div>
        <div className="text-right">
          <p className="m-0 text-[13px] text-muted-foreground">In your wallet</p>
          <p className="m-0 font-display text-[34px] leading-none tabular-nums">{formatINR(data.wallet_paise)}</p>
          <Link to="/cards#rewards" className="text-[13px]">Spend it on a treat</Link>
        </div>
      </div>

      <div className="mt-5">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <h3 className="m-0 text-[15px] font-medium">
            {monthName(month)} so far: <span className="tabular-nums">{pct(score.score_bps)}</span>
          </h3>
          <span className="text-[13px] text-muted-foreground">
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
          Targets learn from your own last six months, with a small stretch: matching your usual month earns most of an area, beating it earns all of it.
        </p>
      </div>

      {data.months.length > 0 && (
        <details className="mt-4 rounded-2xl border border-border px-4 py-3 text-sm">
          <summary className="cursor-pointer font-medium">Past months ({data.months.length})</summary>
          <ul className="ledger mt-2">
            {data.months.map((m) => (
              <li key={m.month} className="ledger-row">
                <span className="min-w-0 truncate font-medium">{monthName(m.month)}</span>
                <span className="text-[13px] text-muted-foreground tabular-nums">
                  {pct(m.score_bps)} · paid {formatINR(m.payout_paise)}
                  {m.earned_paise > m.payout_paise && ` of ${formatINR(m.earned_paise)} earned`}
                </span>
              </li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
