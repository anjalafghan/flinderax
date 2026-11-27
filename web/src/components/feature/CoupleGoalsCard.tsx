import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { HeartHandshake } from "lucide-react";

import api from "@/services/api";
import { formatINR, toPaise } from "@/utils/money";

interface Goal {
  goal_id: string;
  name: string;
  target_paise: number;
  target_date: string | null;
  saved_paise: number;
  percent: number;
  monthly_needed_paise: number | null;
  shares: { name: string; paise: number }[];
}

interface Out {
  in_couple: boolean;
  goals: Goal[];
}

const KEY = ["couple-goals"] as const;

/** Goals the two of you agreed on, that you both pay into. Hidden for someone who is not in a couple. */
export function CoupleGoalsCard() {
  const qc = useQueryClient();
  const { data } = useQuery({
    queryKey: KEY,
    queryFn: async () => (await api.get<Out>("/api/couple-goals")).data,
    retry: 1,
  });
  const refresh = () => qc.invalidateQueries({ queryKey: KEY });
  const add = useMutation({
    mutationFn: ({ id, paise }: { id: string; paise: number }) =>
      api.post(`/api/couple-goals/${id}/contribute`, { amount_paise: paise }),
    onSuccess: refresh,
  });
  const start = useMutation({
    mutationFn: (g: {
      name: string;
      target_paise: number;
      target_date?: string;
    }) => api.post("/api/couple-goals", g),
    onSuccess: refresh,
  });
  const [amounts, setAmounts] = useState<Record<string, string>>({});
  const [form, setForm] = useState({ name: "", amount: "", date: "" });
  const [error, setError] = useState("");
  if (!data?.in_couple) return null;

  const put = (g: Goal) => {
    const paise = toPaise(amounts[g.goal_id] ?? "");
    if (!paise || paise <= 0)
      return setError("Enter an amount in rupees, like 2000.");
    setError("");
    add.mutate(
      { id: g.goal_id, paise },
      { onSuccess: () => setAmounts((a) => ({ ...a, [g.goal_id]: "" })) },
    );
  };
  const begin = () => {
    const paise = toPaise(form.amount);
    if (!form.name.trim() || !paise || paise <= 0)
      return setError("Give it a name and an amount in rupees.");
    setError("");
    start.mutate(
      {
        name: form.name.trim(),
        target_paise: paise,
        ...(form.date ? { target_date: form.date } : {}),
      },
      { onSuccess: () => setForm({ name: "", amount: "", date: "" }) },
    );
  };

  return (
    <section
      aria-labelledby="our-goals-title"
      className="rounded-[28px] border border-rose/30 px-5 py-5 shadow-[0_18px_40px_-28px_hsl(var(--c-rose))] sm:px-8"
      style={{
        background:
          "radial-gradient(110% 80% at 100% 0%, hsl(var(--c-berry) / 0.1), transparent 60%), radial-gradient(110% 90% at 0% 0%, hsl(var(--c-rose) / 0.12), transparent 60%), hsl(var(--card))",
      }}
    >
      <h2
        id="our-goals-title"
        className="m-0 mb-4 flex items-center gap-2.5 font-display text-[32px] leading-none"
      >
        <span
          aria-hidden
          className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[linear-gradient(145deg,hsl(var(--c-rose)),hsl(var(--c-berry)))] text-background shadow-[0_8px_18px_-8px_hsl(var(--c-rose))]"
        >
          <HeartHandshake className="h-5 w-5" strokeWidth={1.8} />
        </span>
        Our goals
      </h2>
      <ul className="m-0 grid list-none gap-4 p-0">
        {data.goals.map((g) => (
          <li key={g.goal_id} className="grid gap-1 text-sm">
            <div className="flex items-baseline justify-between gap-3">
              <strong>{g.name}</strong>
              <span className="pill pill-rose">{g.percent}%</span>
            </div>
            <progress
              value={g.percent}
              max={100}
              aria-label={`${g.name} progress`}
              // A rose-to-berry bar for money the two of you put in together.
              className="h-2.5 w-full appearance-none overflow-hidden rounded-full bg-rose/15 [&::-moz-progress-bar]:rounded-full [&::-moz-progress-bar]:bg-[linear-gradient(90deg,hsl(var(--c-rose)),hsl(var(--c-berry)))] [&::-webkit-progress-bar]:bg-transparent [&::-webkit-progress-value]:rounded-full [&::-webkit-progress-value]:bg-[linear-gradient(90deg,hsl(var(--c-rose)),hsl(var(--c-berry)))]"
            />
            <span className="text-muted-foreground">
              {formatINR(g.saved_paise)} of {formatINR(g.target_paise)}
              {g.monthly_needed_paise
                ? ` · ${formatINR(g.monthly_needed_paise)} a month to get there`
                : ""}
            </span>
            <span className="text-muted-foreground">
              {g.shares
                .map((s) => `${s.name} ${formatINR(s.paise)}`)
                .join(" · ")}
            </span>
            <Link
              to={`/wedding/${g.goal_id}`}
              className="inline-flex min-h-11 items-center self-start text-sm font-semibold text-[hsl(var(--foreground))] dark:text-rose"
            >
              Plan the budget
            </Link>
            <span className="flex items-center gap-2">
              <input
                aria-label={`Add to ${g.name}`}
                inputMode="decimal"
                placeholder="₹"
                className="min-h-11 w-28 rounded-xl border border-rose/30 bg-background px-3 py-1 text-base focus-visible:border-rose md:text-sm"
                value={amounts[g.goal_id] ?? ""}
                onChange={(e) =>
                  setAmounts((a) => ({ ...a, [g.goal_id]: e.target.value }))
                }
              />
              <button
                type="button"
                className="inline-flex min-h-11 items-center rounded-full bg-[linear-gradient(110deg,hsl(var(--c-rose)),hsl(var(--c-berry)))] px-4 text-sm font-semibold text-background shadow-[0_8px_18px_-10px_hsl(var(--c-rose))] transition-transform hover:-translate-y-px disabled:opacity-50"
                disabled={add.isPending}
                onClick={() => put(g)}
              >
                Put in
              </button>
            </span>
          </li>
        ))}
      </ul>
      <details className="mt-3">
        <summary className="inline-flex min-h-11 cursor-pointer items-center text-sm font-medium text-[hsl(var(--foreground))] dark:text-rose">
          Start a goal together
        </summary>
        <div className="mt-2 grid gap-2 text-sm">
          <label className="flex flex-col gap-1 sm:block">
            What are you saving for?
            <input
              className="min-h-11 rounded-xl border border-border bg-background px-3 py-1 text-base md:text-sm sm:ml-2 sm:min-h-0 sm:rounded sm:px-2"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </label>
          <label className="flex flex-col gap-1 sm:block">
            How much (₹)?
            <input
              className="min-h-11 rounded-xl border border-border bg-background px-3 py-1 text-base md:text-sm sm:ml-2 sm:min-h-0 sm:w-32 sm:rounded sm:px-2"
              inputMode="decimal"
              value={form.amount}
              onChange={(e) => setForm({ ...form, amount: e.target.value })}
            />
          </label>
          <label className="flex flex-col gap-1 sm:block">
            By when (optional)
            <input
              type="date"
              className="min-h-11 rounded-xl border border-border bg-background px-3 py-1 text-base md:text-sm sm:ml-2 sm:min-h-0 sm:rounded sm:px-2"
              value={form.date}
              onChange={(e) => setForm({ ...form, date: e.target.value })}
            />
          </label>
          <div>
            <button
              type="button"
              className="inline-flex min-h-11 items-center rounded-full bg-[linear-gradient(110deg,hsl(var(--c-rose)),hsl(var(--c-berry)))] px-4 text-sm font-semibold text-background shadow-[0_8px_18px_-10px_hsl(var(--c-rose))] transition-transform hover:-translate-y-px disabled:opacity-50"
              disabled={start.isPending}
              onClick={begin}
            >
              Start this goal
            </button>
          </div>
        </div>
      </details>
      {error && (
        <p role="alert" className="mt-2 text-sm text-destructive">
          {error}
        </p>
      )}
    </section>
  );
}
