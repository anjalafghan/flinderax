import type { PeopleReport } from "@/services/checkin";
import { rupees } from "@/services/checkin";
import { plural } from "@/services/people";

const TONE_NOTE = "text-[13px] text-muted-foreground";

/** The check-in's people section. No names: counts and money only. */
export function PeopleReportSection({ report }: { report: PeopleReport }) {
  const sentBefore =
    report.prev_sent_to_people_paise !== null ? `Before: ${rupees(report.prev_sent_to_people_paise)}` : "No earlier spend";

  return (
    <div className="flex flex-col gap-3">
      <h3 className="m-0 text-[15px] font-medium">
        People
      </h3>
      <ul className="ledger">
        <li className="ledger-row">
          <span className="text-[15px] font-medium">Catch-ups</span>
          <span className="flex flex-col items-end gap-1 text-right">
            <span className="font-display text-[20px] leading-none">{report.catch_ups}</span>
            <span className={TONE_NOTE}>
              {report.overdue_people > 0
                ? `${plural(report.overdue_people, "person", "people")} past catch-up date`
                : "Nobody overdue"}
            </span>
          </span>
        </li>
        <li className="ledger-row">
          <span className="text-[15px] font-medium">Gifts</span>
          <span className="flex flex-col items-end gap-1 text-right">
            <span className="font-display text-[20px] leading-none">{rupees(report.gifts_given_paise)}</span>
            <span className={TONE_NOTE}>{`of ${rupees(report.gifts_budgeted_paise)} budgeted`}</span>
          </span>
        </li>
        <li className="ledger-row">
          <span className="text-[15px] font-medium">Next quarter</span>
          <span className="flex flex-col items-end gap-1 text-right">
            <span className="font-display text-[20px] leading-none">{rupees(report.next_quarter_gifts_paise)}</span>
            <span className={TONE_NOTE}>
              {`across ${plural(report.next_quarter_occasions, "occasion", "occasions")}`}
            </span>
          </span>
        </li>
        <li className="ledger-row">
          <span className="text-[15px] font-medium">Visits</span>
          <span className="flex flex-col items-end gap-1 text-right">
            <span className="font-display text-[20px] leading-none">{rupees(report.visit_cost_paise)}</span>
            <span className={TONE_NOTE}>Cost of visits</span>
          </span>
        </li>
        <li className="ledger-row">
          <span className="text-[15px] font-medium">Sent to people</span>
          <span className="flex flex-col items-end gap-1 text-right">
            <span className="font-display text-[20px] leading-none">{rupees(report.sent_to_people_paise)}</span>
            <span className={TONE_NOTE}>{sentBefore}</span>
          </span>
        </li>
      </ul>

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
    </div>
  );
}
