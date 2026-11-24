import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";

import type { Road, RoadArea, RoadSource } from "@/services/road";
import { formatINR } from "@/utils/money";
import { byYear, monthShort } from "./glance";

const AREA_COLOR: Record<RoadArea, string> = {
  money: "hsl(var(--primary))",
  family: "hsl(var(--over))",
  health: "hsl(var(--good))",
};

const AREA_NAME: Record<RoadArea, string> = { money: "Money", family: "Family", health: "Cover" };

const SOURCE_NAME: Record<RoadSource, string> = {
  life: "Life plan",
  goal: "Savings goal",
  payment: "Year ahead",
  cover: "Missing cover",
};

/** Whole rupees for planning figures. */
const rupees = (p: number) => formatINR(Math.round(p / 100) * 100);

/** The road ahead, year by year: what is coming, what it costs then, and what it needs a month from now. */
export function RoadAhead({ road }: { road: Road }) {
  const years = byYear(road.milestones);
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
        {(Object.keys(AREA_NAME) as RoadArea[]).map((a) => (
          <span key={a} className="inline-flex items-center gap-1.5">
            <i className="inline-block h-2.5 w-2.5 rounded-full" style={{ background: AREA_COLOR[a] }} />
            {AREA_NAME[a]}
          </span>
        ))}
      </div>

      {years.length === 0 && (
        <p className="text-sm text-muted-foreground">Nothing dated yet. Give a goal a date, or fill in the life plan, and it shows up here.</p>
      )}

      <ol className="flex flex-col gap-6" aria-label="The road ahead">
        {years.map(({ year, items }) => (
          <li key={year} className="grid grid-cols-[3.5rem_1fr] gap-4">
            <span className="font-display text-2xl leading-tight text-muted-foreground">{year}</span>
            <ul className="flex flex-col gap-3 border-l border-border pl-5">
              {items.map((m) => (
                <li key={`${m.on}-${m.title}`} className="relative">
                  <i
                    aria-hidden
                    className="absolute -left-[1.6rem] top-1.5 h-3 w-3 rounded-full border-2 border-background"
                    style={{ background: AREA_COLOR[m.area] }}
                  />
                  <Link to={m.link} viewTransition className="group block no-underline">
                    <span className="flex items-center gap-1.5 text-[15px] font-medium">
                      {m.title}
                      <ArrowRight className="h-3.5 w-3.5 opacity-0 transition-opacity group-hover:opacity-60" aria-hidden />
                    </span>
                    <span className="block text-[13px] text-muted-foreground">
                      {monthShort(m.on.slice(0, 7), true)} · {SOURCE_NAME[m.source]}
                      {m.cost_paise > 0 && <> · {rupees(m.cost_paise)}</>}
                      {m.per_month_paise > 0 && <> · {rupees(m.per_month_paise)} a month from now</>}
                      {m.saved_paise > 0 && <> · {rupees(m.saved_paise)} saved</>}
                    </span>
                  </Link>
                </li>
              ))}
            </ul>
          </li>
        ))}
      </ol>

      {road.prompts.length > 0 && (
        <div className="grid gap-3 sm:grid-cols-2">
          {road.prompts.map((p) => (
            <Link
              key={p.title}
              to={p.link}
              viewTransition
              className="rounded-[18px] border border-dashed border-border px-5 py-4 no-underline transition-colors hover:bg-[hsl(var(--chip))]"
            >
              <span className="block text-[15px] font-medium">{p.title}</span>
              <span className="block text-[13px] text-muted-foreground">{p.body}</span>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
