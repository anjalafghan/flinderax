import { rate, rupees, signedChange, type DietReport } from "@/services/checkin";
import { ordersPerWeek, signedKg } from "@/services/diet";
import { formatDay } from "@/utils/dates";

/** Eating-out change as basis points: null when there was no earlier spend to compare with. */
const changeBps = (now: number, before: number | null): number | null =>
  before === null || before === 0 ? null : Math.round(((now - before) / before) * 10_000);

/** The check-in's food and health section. Figures from the quarter, with the change from the quarter before. */
export function DietReportSection({ report }: { report: DietReport }) {
  const eatingChange = changeBps(report.eating_out_paise, report.prev_eating_out_paise);
  const groceriesChange = changeBps(report.groceries_paise, report.prev_groceries_paise);

  return (
    <div className="flex flex-col gap-6">
      <h3 className="m-0 text-[15px] font-medium">Food and health</h3>

      <dl className="m-0 grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-4">
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Eating out</dt>
          <dd className="m-0 font-display text-[26px] leading-none">{rupees(report.eating_out_paise)}</dd>
          <dd className="m-0 text-[13px] text-muted-foreground">{signedChange(eatingChange)}</dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Groceries</dt>
          <dd className="m-0 font-display text-[26px] leading-none">{rupees(report.groceries_paise)}</dd>
          <dd className="m-0 text-[13px] text-muted-foreground">{signedChange(groceriesChange)}</dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Eating out share</dt>
          <dd className="m-0 font-display text-[26px] leading-none">{rate(report.eating_out_share_bps)}</dd>
          <dd className="m-0 text-[13px] text-muted-foreground">of food</dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Orders</dt>
          <dd className="m-0 font-display text-[26px] leading-none">{ordersPerWeek(report.delivery_per_week_x100)}</dd>
          <dd className="m-0 text-[13px] text-muted-foreground">a week</dd>
        </div>
      </dl>

      <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
        <li>{`${report.weeks_logged} of ${report.weeks_in_period} weeks logged`}</li>
        {report.weight_change_x10 !== null && <li>{`Weight: ${signedKg(report.weight_change_x10)}`}</li>}
        {report.checkup_due_on && <li>{`Yearly check-up due ${formatDay(report.checkup_due_on)}.`}</li>}
      </ul>

      {report.habits.length > 0 && (
        <ul className="ledger">
          {report.habits.map((h) => (
            <li key={h.label} className="ledger-row">
              <span className="text-[15px] font-medium">{h.label}</span>
              <span className="text-right text-[13px] text-muted-foreground">
                {`On target in ${h.weeks_on_target} of ${h.weeks_logged} weeks, ${rate(h.hit_bps)} of days`}
              </span>
            </li>
          ))}
        </ul>
      )}

      {(report.wins.length > 0 || report.watch.length > 0) && (
        <div className="grid gap-8 md:grid-cols-2">
          <div className="flex flex-col gap-3">
            <h4 className="m-0 text-[15px] font-medium">Wins</h4>
            {report.wins.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
                {report.wins.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">Nothing standing out yet.</p>
            )}
          </div>
          <div className="flex flex-col gap-3">
            <h4 className="m-0 text-[15px] font-medium">Watch</h4>
            {report.watch.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
                {report.watch.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">Nothing to flag this time.</p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
