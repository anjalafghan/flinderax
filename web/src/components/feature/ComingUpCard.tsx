import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";

import api from "@/services/api";

interface Entry {
  on: string;
  kind: "bill" | "date" | "birthday" | "follow_up" | "milestone";
  title: string;
  detail: string;
  link: string;
  days_away: number;
  when: string;
}

const SHOWN = 6;

/** Bills, planned dates, birthdays and follow-ups in one list, soonest first. Hidden when empty. */
export function ComingUpCard() {
  const { data } = useQuery({
    queryKey: ["coming-up"],
    queryFn: async () => (await api.get<{ items: Entry[] }>("/api/coming-up?days=14")).data.items,
    retry: 1,
  });
  const items = (data ?? []).slice(0, SHOWN);
  if (items.length === 0) return null;
  return (
    <section aria-labelledby="coming-up-title" className="rounded-[28px] border border-border bg-card px-5 py-5 sm:px-8">
      <h2 id="coming-up-title" className="m-0 mb-3 text-[15px] font-medium text-muted-foreground">
        Coming up
      </h2>
      <ul className="m-0 grid list-none gap-2 p-0">
        {items.map((i) => (
          <li key={`${i.kind}-${i.on}-${i.title}`} className="flex items-baseline justify-between gap-3 text-sm">
            <Link to={i.link || "/"} className="min-w-0 truncate font-medium hover:underline">
              {i.title}
            </Link>
            <span className="shrink-0 text-muted-foreground">
              {i.detail && <span className="mr-2">{i.detail}</span>}
              <span className={i.days_away < 0 ? "text-destructive" : undefined}>{i.when}</span>
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
