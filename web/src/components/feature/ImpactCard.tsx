import type { Absorption, ItemImpact } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/utils/cn";

function Option({ title, a }: { title: string; a: Absorption }) {
  const rows: [string, number, number][] = [
    ["Savings top-up to the reserve", a.from_rest_savings_paise, a.rest_savings_after_paise],
    ["SIP", a.from_sip_paise, a.sip_after_paise],
    ["Spending allowance", a.from_spending_paise, a.spending_after_paise],
  ];
  return (
    <div className="space-y-2 rounded-lg border p-3 text-sm">
      <p className="font-semibold">{title}</p>
      <ul className="space-y-1">
        {rows.map(([label, cut, after]) => (
          <li key={label} className={cn("flex justify-between gap-2", cut === 0 && "text-muted-foreground")}>
            <span>{label}</span>
            <span className="tabular-nums">
              {cut > 0 ? `−${formatINR(cut)} → ${formatINR(after)}` : "untouched"}
            </span>
          </li>
        ))}
      </ul>
      {a.from_fd_paise > 0 && (
        <p className="font-medium text-rose-600 dark:text-rose-400">
          {formatINR(a.from_fd_paise)} a month would have to come from an FD.
        </p>
      )}
    </div>
  );
}

/** What one scenario item does: who absorbs its monthly cost, the thresholds, and the goal check. */
export function ImpactCard({ impact }: { impact: ItemImpact }) {
  const g = impact.goal;
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">{impact.name}</CardTitle>
        {impact.monthly_paise > 0 && (
          <p className="text-sm text-muted-foreground">Costs {formatINR(impact.monthly_paise)} a month. Who gives way?</p>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        {impact.savings_first && impact.spending_first && (
          <div className="grid gap-3 md:grid-cols-2">
            <Option title="Savings take the cut" a={impact.savings_first} />
            <Option title="Spending takes the cut" a={impact.spending_first} />
          </div>
        )}
        {impact.savings_first_thresholds.length > 0 && (
          <div className="text-sm">
            <p className="mb-1 font-semibold">As the monthly amount grows</p>
            <ul className="space-y-1 text-muted-foreground">
              {impact.savings_first_thresholds.slice(0, 3).map((t, i, all) => (
                <li key={t.label}>
                  <span className="font-medium text-foreground tabular-nums">
                    {formatINR(i === 0 ? 0 : all[i - 1].up_to_paise)} to {formatINR(t.up_to_paise)}
                  </span>
                  : {t.label.charAt(0).toLowerCase() + t.label.slice(1)}
                </li>
              ))}
              <li>
                <span className="font-medium text-foreground tabular-nums">
                  Above {formatINR(impact.savings_first_thresholds[3].up_to_paise)}
                </span>
                : an FD would have to be touched
              </li>
            </ul>
          </div>
        )}
        {impact.maturity_value_paise > 0 && (
          <p className="text-sm">
            Worth <span className="font-semibold tabular-nums">{formatINR(impact.maturity_value_paise)}</span> at maturity.
          </p>
        )}
        {g && (
          <div
            className={cn(
              "rounded-lg border p-3 text-sm",
              g.gap_paise >= 0 ? "border-emerald-500/40 bg-emerald-500/10" : "border-amber-500/40 bg-amber-500/10",
            )}
          >
            <p className="font-semibold">
              Goal {formatINR(g.target_paise)} by {formatDay(g.target_date)} {g.target_date.slice(0, 4)}
            </p>
            <p>
              {g.gap_paise >= 0
                ? `On track: ${formatINR(g.gap_paise)} ahead.`
                : `Short by ${formatINR(-g.gap_paise)}.`}
              {g.required_monthly_paise != null && g.gap_paise < 0 && (
                <> You'd need about {formatINR(g.required_monthly_paise)} a month to get there.</>
              )}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
