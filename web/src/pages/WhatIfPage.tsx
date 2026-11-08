import { useEffect, useMemo, useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Plus } from "lucide-react";

import { planApi } from "@/services/api";
import type { ScenarioKind } from "@/services/types";
import { useAccounts } from "@/hooks/usePlanner";
import { Header } from "@/components/layout/Header";
import { LineChart, type ChartMarker, type ChartSeries } from "@/components/feature/LineChart";
import { ScenarioItemForm } from "@/components/feature/ScenarioItemForm";
import { ImpactCard } from "@/components/feature/ImpactCard";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { formatINR } from "@/utils/money";
import { newDraft, toScenarioItem, type ScenarioDraft } from "@/utils/scenario";
import { cn } from "@/utils/cn";

const STORE = "flinderax-whatif-v1";
type Metric = "total" | "spending" | "reserve" | "deposits";
const METRICS: { id: Metric; label: string }[] = [
  { id: "total", label: "Everything" },
  { id: "spending", label: "Spending account" },
  { id: "reserve", label: "Reserve" },
  { id: "deposits", label: "FDs & RDs" },
];
const KIND_NAME: Record<string, string> = {
  spending_short: "spending account runs short",
  reserve_below_floor: "reserve dips below your floor",
  fd_needed: "an FD would have to be touched",
};
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

const monthLabel = (m: string) => {
  const [y, mo] = m.split("-");
  return `${MONTHS[Number(mo) - 1]} '${y.slice(2)}`;
};
const monthLong = (m: string) => {
  const [y, mo] = m.split("-");
  return `${MONTHS[Number(mo) - 1]} ${y}`;
};

function useDebounced<T>(value: T, ms: number): T {
  const [v, setV] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setV(value), ms);
    return () => clearTimeout(t);
  }, [value, ms]);
  return v;
}

function loadDrafts(): ScenarioDraft[] {
  try {
    const raw = localStorage.getItem(STORE);
    return raw ? (JSON.parse(raw) as ScenarioDraft[]) : [];
  } catch {
    return [];
  }
}

export default function WhatIfPage() {
  const { data: accounts = [] } = useAccounts();
  const [drafts, setDrafts] = useState<ScenarioDraft[]>(loadDrafts);
  const [months, setMonths] = useState(24);
  const [spendPct, setSpendPct] = useState(100);
  const [metric, setMetric] = useState<Metric>("spending");

  useEffect(() => {
    try {
      localStorage.setItem(STORE, JSON.stringify(drafts));
    } catch {
      /* private mode: the page works without it */
    }
  }, [drafts]);

  const items = useMemo(() => drafts.map(toScenarioItem), [drafts]);
  const valid = useMemo(() => items.filter((i) => i !== null), [items]);
  const request = useDebounced({ months, spend_pct: spendPct, items: valid }, 250);

  const { data, isError, isFetching } = useQuery({
    queryKey: ["projection", request],
    queryFn: () => planApi.project(request),
    placeholderData: keepPreviousData,
  });

  const pick = (p: NonNullable<typeof data>["baseline"]["points"][number]) =>
    metric === "total" ? p.total_paise : metric === "spending" ? p.spending_paise : metric === "reserve" ? p.reserve_paise : p.fds_paise + p.rds_paise;

  const chart = useMemo(() => {
    if (!data) return null;
    const base = data.baseline.points;
    const labels = base.map((p) => (p.month === "start" ? "Now" : monthLabel(p.month)));
    const series: ChartSeries[] = [];
    if (data.scenario) {
      // the scenario may run longer than the baseline requested: pad the baseline with its last value
      const sc = data.scenario.points;
      const baseVals = sc.map((_, i) => pick(base[Math.min(i, base.length - 1)]));
      const scLabels = sc.map((p) => (p.month === "start" ? "Now" : monthLabel(p.month)));
      series.push({ label: "Without this", values: baseVals, stroke: "stroke-muted-foreground", dashed: true });
      series.push({ label: "With this", values: sc.map(pick), stroke: "stroke-emerald-500" });
      return { labels: scLabels, series, stress: data.scenario.stress, points: sc };
    }
    series.push({ label: "Baseline", values: base.map(pick), stroke: "stroke-sky-500" });
    return { labels, series, stress: data.baseline.stress, points: base };
  }, [data, metric]); // eslint-disable-line react-hooks/exhaustive-deps

  const markers: ChartMarker[] = useMemo(() => {
    if (!chart) return [];
    const seen = new Set<string>();
    const out: ChartMarker[] = [];
    for (const s of chart.stress) {
      const idx = chart.points.findIndex((p) => p.month === s.month);
      const key = `${idx}`;
      if (idx >= 0 && !seen.has(key)) {
        seen.add(key);
        out.push({ index: idx, label: `${monthLabel(s.month)}: ${KIND_NAME[s.kind]} by ${formatINR(s.amount_paise)}` });
      }
    }
    return out;
  }, [chart]);

  const add = (kind: ScenarioKind) => setDrafts((d) => [...d, newDraft(kind)]);
  const stressSummary = useMemo(() => {
    if (!chart) return [];
    const total = chart.points.length - 1; // the first point is "now"
    const by = new Map<string, { months: number; max: number; first: string }>();
    for (const s of chart.stress) {
      const e = by.get(s.kind) ?? { months: 0, max: 0, first: s.month };
      by.set(s.kind, { months: e.months + 1, max: Math.max(e.max, s.amount_paise), first: e.first });
    }
    return Array.from(by, ([kind, e]) => ({ kind, total, ...e }));
  }, [chart]);

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-4xl space-y-8">
        <div>
          <h1 className="page-title">What if...</h1>
          <p className="text-muted-foreground">
            Try an RD, an FD or a big purchase and see what it does to your savings, your spending and your balances
            over the coming months. Nothing here changes your real data.
          </p>
        </div>

        <section className="space-y-3">
          <div className="flex flex-wrap gap-2">
            <Button size="sm" onClick={() => add("rd")}><Plus className="mr-1 h-4 w-4" /> Recurring deposit</Button>
            <Button size="sm" variant="outline" onClick={() => add("fd")}><Plus className="mr-1 h-4 w-4" /> Fixed deposit</Button>
            <Button size="sm" variant="outline" onClick={() => add("one_off")}><Plus className="mr-1 h-4 w-4" /> One-off spend</Button>
          </div>
          {drafts.map((d, i) => (
            <ScenarioItemForm
              key={d.id}
              draft={d}
              accounts={accounts}
              invalid={items[i] === null}
              onChange={(nd) => setDrafts((all) => all.map((x) => (x.id === nd.id ? nd : x)))}
              onRemove={() => setDrafts((all) => all.filter((x) => x.id !== d.id))}
            />
          ))}
          {drafts.length === 0 && (
            <p className="rounded-3xl border border-dashed border-primary/25 bg-card/30 p-4 text-sm text-muted-foreground">
              Nothing to try yet. Add the wedding RD to start, or just look at where you're heading below.
            </p>
          )}
        </section>

        <section className="space-y-3">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-1">
              <Label htmlFor="months">Look ahead: {months} months</Label>
              <input id="months" type="range" min={6} max={60} step={6} value={months} onChange={(e) => setMonths(Number(e.target.value))} className="w-full" />
            </div>
            <div className="space-y-1">
              <Label htmlFor="spend-pct">You spend {spendPct}% of your spending allowance</Label>
              <input id="spend-pct" type="range" min={0} max={150} step={5} value={spendPct} onChange={(e) => setSpendPct(Number(e.target.value))} className="w-full" />
            </div>
          </div>

          <div className="flex flex-wrap gap-2" role="tablist" aria-label="What to chart">
            {METRICS.map((m) => (
              <Button key={m.id} size="sm" variant={metric === m.id ? "default" : "outline"} role="tab" aria-selected={metric === m.id} onClick={() => setMetric(m.id)}>
                {m.label}
              </Button>
            ))}
          </div>

          <div className={cn("glass rounded-3xl p-3 transition-opacity", isFetching && "opacity-70")}>
            {isError && <p className="text-sm text-destructive">Couldn't compute the projection.</p>}
            {chart && <LineChart labels={chart.labels} series={chart.series} markers={markers} />}
            {!chart && !isError && <div className="h-64 animate-pulse rounded-lg bg-muted/40" />}
          </div>

          {chart && (
            <div className="space-y-1 text-sm text-muted-foreground">
              {stressSummary.length === 0 ? (
                <p>No month falls short in this window.</p>
              ) : (
                <ul className="space-y-1">
                  {stressSummary.map((e) => (
                    <li key={e.kind}>
                      <span className="font-medium text-foreground">Your {KIND_NAME[e.kind]}</span> in {e.months} of{" "}
                      {e.total} months, by up to {formatINR(e.max)}. First: {monthLong(e.first)}.
                    </li>
                  ))}
                </ul>
              )}
              <p>
                Assumes salary and fixed payments continue, you spend {spendPct}% of your allowance, the SIP is not valued
                (contributions only), and deposits compound quarterly.
              </p>
            </div>
          )}
        </section>

        {data && data.impacts.length > 0 && (
          <section className="space-y-4">
            <h2 className="font-display text-3xl leading-none tracking-tight">What each change does</h2>
            {data.impacts.map((imp, i) => <ImpactCard key={`${imp.name}-${i}`} impact={imp} />)}
          </section>
        )}
      </main>
    </div>
  );
}
