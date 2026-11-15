import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

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
  const { data } = useQuery({ queryKey: KEY, queryFn: async () => (await api.get<Out>("/api/couple-goals")).data, retry: 1 });
  const refresh = () => qc.invalidateQueries({ queryKey: KEY });
  const add = useMutation({
    mutationFn: ({ id, paise }: { id: string; paise: number }) => api.post(`/api/couple-goals/${id}/contribute`, { amount_paise: paise }),
    onSuccess: refresh,
  });
  const start = useMutation({
    mutationFn: (g: { name: string; target_paise: number; target_date?: string }) => api.post("/api/couple-goals", g),
    onSuccess: refresh,
  });
  const [amounts, setAmounts] = useState<Record<string, string>>({});
  const [form, setForm] = useState({ name: "", amount: "", date: "" });
  const [error, setError] = useState("");
  if (!data?.in_couple) return null;

  const put = (g: Goal) => {
    const paise = toPaise(amounts[g.goal_id] ?? "");
    if (!paise || paise <= 0) return setError("Enter an amount in rupees, like 2000.");
    setError("");
    add.mutate({ id: g.goal_id, paise }, { onSuccess: () => setAmounts((a) => ({ ...a, [g.goal_id]: "" })) });
  };
  const begin = () => {
    const paise = toPaise(form.amount);
    if (!form.name.trim() || !paise || paise <= 0) return setError("Give it a name and an amount in rupees.");
    setError("");
    start.mutate({ name: form.name.trim(), target_paise: paise, ...(form.date ? { target_date: form.date } : {}) }, { onSuccess: () => setForm({ name: "", amount: "", date: "" }) });
  };

  return (
    <section aria-labelledby="our-goals-title" className="rounded-[28px] border border-border bg-card px-5 py-5 sm:px-8">
      <h2 id="our-goals-title" className="m-0 mb-3 text-[15px] font-medium text-muted-foreground">
        Our goals
      </h2>
      <ul className="m-0 grid list-none gap-4 p-0">
        {data.goals.map((g) => (
          <li key={g.goal_id} className="grid gap-1 text-sm">
            <div className="flex items-baseline justify-between gap-3">
              <strong>{g.name}</strong>
              <span>{g.percent}%</span>
            </div>
            <progress value={g.percent} max={100} aria-label={`${g.name} progress`} className="w-full" />
            <span className="text-muted-foreground">
              {formatINR(g.saved_paise)} of {formatINR(g.target_paise)}
              {g.monthly_needed_paise ? ` · ${formatINR(g.monthly_needed_paise)} a month to get there` : ""}
            </span>
            <span className="text-muted-foreground">{g.shares.map((s) => `${s.name} ${formatINR(s.paise)}`).join(" · ")}</span>
            <span className="flex items-center gap-2">
              <input
                aria-label={`Add to ${g.name}`}
                inputMode="decimal"
                placeholder="₹"
                className="w-28 rounded border border-border bg-background px-2 py-1"
                value={amounts[g.goal_id] ?? ""}
                onChange={(e) => setAmounts((a) => ({ ...a, [g.goal_id]: e.target.value }))}
              />
              <button type="button" className="btn-why" disabled={add.isPending} onClick={() => put(g)}>
                Put in
              </button>
            </span>
          </li>
        ))}
      </ul>
      <details className="mt-3">
        <summary className="cursor-pointer text-sm">Start a goal together</summary>
        <div className="mt-2 grid gap-2 text-sm">
          <label>
            What are you saving for?
            <input className="ml-2 rounded border border-border bg-background px-2 py-1" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          </label>
          <label>
            How much (₹)?
            <input className="ml-2 w-32 rounded border border-border bg-background px-2 py-1" inputMode="decimal" value={form.amount} onChange={(e) => setForm({ ...form, amount: e.target.value })} />
          </label>
          <label>
            By when (optional)
            <input type="date" className="ml-2 rounded border border-border bg-background px-2 py-1" value={form.date} onChange={(e) => setForm({ ...form, date: e.target.value })} />
          </label>
          <div>
            <button type="button" className="btn-why" disabled={start.isPending} onClick={begin}>
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
