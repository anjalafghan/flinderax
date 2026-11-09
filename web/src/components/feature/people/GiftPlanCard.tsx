import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { SeasonsEditor } from "@/components/feature/people/SeasonsEditor";
import { toast } from "sonner";

import { Retry } from "@/components/feature/people/Retry";
import { MONTH_NAMES, NOTE, PANEL, SECTION_TITLE, TEXT_LINK, errorMessage } from "@/components/feature/people/ui";
import { rupees } from "@/services/checkin";
import { useGiftPlan, useMakeGiftsGoal } from "@/services/people";

/** "2026-12" -> "Dec". */
const monthShort = (ym: string) => (MONTH_NAMES[Number(ym.slice(5, 7)) - 1] ?? ym).slice(0, 3);

/** Twelve months of gift budget as bars, the monthly set-aside, and the festival seasons. */
export function GiftPlanCard() {
  const plan = useGiftPlan();
  const makeGoal = useMakeGiftsGoal();

  const body = (() => {
    if (plan.isLoading) return <div className="h-48 animate-pulse rounded-[18px] bg-muted/40" aria-label="Loading your gift plan" />;
    if (plan.isError && !plan.data) return <Retry message="Couldn't load your gift plan." onRetry={() => void plan.refetch()} />;
    const data = plan.data;
    if (!data) return null;

    const peak = Math.max(1, ...data.months.map((m) => m.budget_paise));
    const occasions = data.months.reduce((n, m) => n + m.occasions, 0);
    const summary = `Gift budget by month. About ${rupees(data.monthly_paise)} a month across the next twelve months, ${occasions} occasions.`;

    return (
      <div className="flex flex-col gap-8">
        <div className="flex flex-col gap-2">
          <p className="m-0 font-display text-[40px] leading-none">{`About ${rupees(data.monthly_paise)} a month`}</p>
          <p className={NOTE}>{`Given this year ${rupees(data.given_this_year_paise)}.`}</p>
        </div>

        <div role="img" aria-label={summary} className="flex h-40 items-end gap-2">
          {data.months.map((m) => (
            <div key={m.month} className="flex h-full min-w-0 flex-1 flex-col justify-end gap-2" aria-hidden>
              <div
                className="w-full rounded-t-[6px] bg-primary/70"
                style={{ height: `${Math.max(2, Math.round((m.budget_paise / peak) * 100))}%` }}
              />
              <span className="text-center text-[11px] text-muted-foreground">{monthShort(m.month)}</span>
            </div>
          ))}
        </div>

        {(data.goal_id !== null || data.total_paise > 0) && (
          <div className="flex flex-wrap items-center gap-3">
            {data.goal_id ? (
              <Link to="/plan" className={TEXT_LINK}>
                Gifts goal set
              </Link>
            ) : (
              <Button
                variant="outline"
                className="min-h-11"
                disabled={makeGoal.isPending}
                onClick={() =>
                  makeGoal.mutate(data.total_paise, {
                    onSuccess: () => toast.success("Gifts goal made."),
                    onError: (err) => toast.error(errorMessage(err)),
                  })
                }
              >
                Make a Gifts goal
              </Button>
            )}
          </div>
        )}

        <SeasonsEditor />
      </div>
    );
  })();

  return (
    <section aria-labelledby="people-gift-plan-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="people-gift-plan-title" className={SECTION_TITLE}>
          Gift plan
        </h2>
        <p className={NOTE}>What gifts cost across the year, from people's dates and the festival seasons you set.</p>
      </header>
      {body}
    </section>
  );
}
