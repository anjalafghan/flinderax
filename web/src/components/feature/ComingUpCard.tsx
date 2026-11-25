import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import {
  CalendarClock,
  Cake,
  Flag,
  Heart,
  MessageCircle,
  Receipt,
  type LucideIcon,
} from "lucide-react";

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

/** Each kind wears its own colour: bills coral, dates rose, birthdays sun, follow-ups sky, milestones grape. */
const KIND: Record<Entry["kind"], { icon: LucideIcon; bubble: string }> = {
  bill: { icon: Receipt, bubble: "bg-coral/15 text-coral" },
  date: { icon: Heart, bubble: "bg-rose/15 text-rose" },
  birthday: { icon: Cake, bubble: "bg-sun/20 text-[hsl(var(--foreground))] dark:text-sun" },
  follow_up: { icon: MessageCircle, bubble: "bg-sky/15 text-sky" },
  milestone: { icon: Flag, bubble: "bg-grape/15 text-grape" },
};

/** Bills, planned dates, birthdays and follow-ups in one list, soonest first. Hidden when empty. */
export function ComingUpCard() {
  const { data } = useQuery({
    queryKey: ["coming-up"],
    queryFn: async () =>
      (await api.get<{ items: Entry[] }>("/api/coming-up?days=14")).data.items,
    retry: 1,
  });
  const items = (data ?? []).slice(0, SHOWN);
  if (items.length === 0) return null;
  return (
    <section
      aria-labelledby="coming-up-title"
      className="glass rounded-[28px] px-5 py-5 sm:px-8"
    >
      <h2
        id="coming-up-title"
        className="m-0 mb-4 flex items-center gap-2.5 font-display text-[32px] leading-none"
      >
        <CalendarClock
          className="h-6 w-6 shrink-0 text-brand"
          strokeWidth={1.6}
          aria-hidden
        />
        Coming up
      </h2>
      <ul className="m-0 grid list-none gap-1.5 p-0">
        {items.map((i) => {
          const k = KIND[i.kind] ?? KIND.milestone;
          return (
            <li
              key={`${i.kind}-${i.on}-${i.title}`}
              className="flex min-h-11 items-center justify-between gap-3 rounded-2xl px-1.5 text-sm transition-colors hover:bg-brand-tint"
            >
              <span className="flex min-w-0 items-center gap-3">
                <span
                  aria-hidden
                  className={`inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${k.bubble}`}
                >
                  <k.icon className="h-4 w-4" strokeWidth={1.8} />
                </span>
                <Link
                  to={i.link || "/"}
                  className="min-w-0 truncate font-medium hover:underline"
                >
                  {i.title}
                </Link>
              </span>
              <span className="flex shrink-0 items-center gap-2 text-muted-foreground">
                {i.detail && <span>{i.detail}</span>}
                <span
                  className={
                    i.days_away < 0
                      ? "pill pill-coral"
                      : i.days_away <= 1
                        ? "pill pill-sun"
                        : "pill"
                  }
                >
                  {i.when}
                </span>
              </span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
