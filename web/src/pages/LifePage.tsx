import { useState, type ChangeEvent, type FormEvent, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Baby, ChevronDown, GraduationCap, Heart, Shield, Sunset, type LucideIcon } from "lucide-react";

import { ForecastPanel } from "@/components/feature/ForecastPanel";
import { LifeLine } from "@/components/feature/life/LifeLine";
import { Header } from "@/components/layout/Header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import {
  getLife,
  lifeKey,
  saveLifeProfile,
  type LifeGoal,
  type LifePlan,
  type LifeProfile,
} from "@/services/life";
import { forecastKey } from "@/services/forecast";
import { lifelineKey } from "@/services/lifeline";
import { formatDay } from "@/utils/dates";
import { formatINR as formatExact, formatPercent, paiseToInput, toPaise } from "@/utils/money";

/** Planning figures in whole rupees: paise are noise on a 20-year horizon. */
const formatINR = (paise: number) => formatExact(Math.round(paise / 100) * 100);
import { cn } from "@/utils/cn";

// ---------- data ----------

function useLife() {
  return useQuery({ queryKey: lifeKey, queryFn: getLife });
}

function useSaveLife() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: saveLifeProfile,
    // The server answers with the recomputed plan, so the page refreshes straight from that.
    onSuccess: (out) => {
      qc.setQueryData(lifeKey, out);
      // The forecast reads the same profile, so it is stale as soon as the details change.
      void qc.invalidateQueries({ queryKey: forecastKey });
      void qc.invalidateQueries({ queryKey: lifelineKey });
    },
  });
}

// ---------- shared bits ----------

const GOAL_ICON: Record<LifeGoal["key"], LucideIcon> = {
  emergency: Shield,
  wedding: Heart,
  child_birth: Baby,
  child_education: GraduationCap,
  retirement: Sunset,
};

const STATUS: Record<LifeGoal["status"], { label: string; tone: "good" | "warn" }> = {
  done: { label: "Done", tone: "good" },
  on_track: { label: "On track", tone: "good" },
  needs_more: { label: "Needs more", tone: "warn" },
};

const MIX_LABEL: Record<LifeGoal["mix"], string> = { debt: "Debt", hybrid: "Hybrid", equity: "Equity" };

const sum = (xs: number[]) => xs.reduce((a, b) => a + b, 0);

/** "30 Oct 2031", or "Now" for goals with no target date. */
function whenLabel(when: string | null): string {
  return when ? `${formatDay(when)} ${when.slice(0, 4)}` : "Now";
}

function yearsLabel(n: number): string {
  if (n <= 0) return "This year";
  return n === 1 ? "1 year away" : `${n} years away`;
}

function Pill({ tone, children, className }: { tone: "good" | "warn" | "over" | "quiet"; children: ReactNode; className?: string }) {
  return (
    <span
      className={cn(
        "inline-flex min-h-7 items-center gap-2 rounded-full px-3 text-[13px] font-semibold",
        `tone-${tone}`,
        className,
      )}
    >
      {children}
    </span>
  );
}

// ---------- 1. summary ----------

function PlanSummary({ plan, profile }: { plan: LifePlan; profile: LifeProfile }) {
  const m = plan.monthly;
  const myShare = sum(plan.goals.map((g) => g.my_share_paise));
  const partnerShare = sum(plan.goals.map((g) => g.partner_share_paise));
  const spare = m.surplus_paise - m.total_step_up_sip_paise;
  const pill = m.fits ? (
    <Pill tone="good">Fits, {formatINR(spare)} to spare</Pill>
  ) : (
    <Pill tone="over">Short by {formatINR(m.shortfall_paise)} a month</Pill>
  );

  return (
    <section
      aria-labelledby="life-plan-title"
      data-fits={m.fits ? "yes" : "no"}
      className="flex flex-col gap-7 rounded-[28px] border border-border bg-card px-6 pb-8 pt-8 sm:px-12 sm:pb-10 sm:pt-12"
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 id="life-plan-title" className="m-0 text-[15px] font-medium text-muted-foreground">
          Your plan
        </h2>
        {pill}
      </div>

      <div className="flex flex-col gap-3.5">
        <p className="m-0 font-display text-[48px] font-light leading-[0.95] tracking-[-0.03em] sm:text-[80px]">
          {formatINR(m.total_step_up_sip_paise)}
          <span className="ml-2 whitespace-nowrap align-middle text-base font-medium tracking-normal text-muted-foreground">a month</span>
        </p>
        <p className="m-0 text-base text-muted-foreground">
          To reach every goal below, with SIPs stepping up each year. Your surplus is{" "}
          <span className="font-medium text-foreground">{formatINR(m.surplus_paise)} a month</span> after expenses.
        </p>
        <p className="m-0 text-sm text-muted-foreground">
          You {formatINR(myShare)} · Partner {formatINR(partnerShare)} a month
          {profile.partner_take_home_paise === 0 && " (all on you until a partner income is added)"}
        </p>
      </div>
    </section>
  );
}

// ---------- 2. goals ledger ----------

function GoalRow({ goal }: { goal: LifeGoal }) {
  const Icon = GOAL_ICON[goal.key];
  const st = STATUS[goal.status];
  return (
    <li className="flex flex-col gap-5 border-t border-border py-7">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3.5">
          <span className="grid h-10 w-10 shrink-0 place-items-center rounded-full bg-chip text-foreground">
            <Icon className="h-[18px] w-[18px]" strokeWidth={1.6} aria-hidden />
          </span>
          <div className="min-w-0">
            <h3 className="m-0 text-[17px] font-medium">{goal.label}</h3>
            <p className="m-0 text-[13px] text-muted-foreground">
              {whenLabel(goal.when)} · {yearsLabel(goal.years_away)}
            </p>
          </div>
        </div>
        <Pill tone={st.tone}>{st.label}</Pill>
      </div>

      {/* Phones: cost and saved side by side, the monthly figure across both; three columns from sm. */}
      <div className="grid grid-cols-2 gap-x-5 gap-y-5 sm:grid-cols-3 sm:gap-x-6">
        <div className="flex min-w-0 flex-col gap-1">
          <span className="text-[13px] text-muted-foreground">Cost then</span>
          <span className="font-display text-xl sm:text-2xl">{formatINR(goal.future_cost_paise)}</span>
          <span className="text-[13px] text-muted-foreground">{formatINR(goal.today_cost_paise)} today</span>
        </div>
        <div className="flex min-w-0 flex-col gap-1">
          <span className="text-[13px] text-muted-foreground">Saved</span>
          <span className="font-display text-xl sm:text-2xl">{formatINR(goal.saved_paise)}</span>
          <span className="text-[13px] text-muted-foreground">
            {goal.gap_paise > 0 ? `${formatINR(goal.gap_paise)} still to go` : "Fully funded"}
          </span>
        </div>
        <div className="col-span-2 flex min-w-0 flex-col gap-1 border-t border-border pt-4 sm:col-span-1 sm:border-t-0 sm:pt-0">
          <span className="text-[13px] text-muted-foreground">A month</span>
          <span className="font-display text-2xl">{formatINR(goal.step_up_sip_paise)}</span>
          <span className="text-[13px] text-muted-foreground">
            First year, stepping up · flat {formatINR(goal.monthly_sip_paise)}
          </span>
          <span className="text-[13px] text-muted-foreground">
            You {formatINR(goal.my_share_paise)} · Partner {formatINR(goal.partner_share_paise)}
          </span>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5 text-[13px] text-muted-foreground">
        <Pill tone="quiet" className="min-h-6 px-2.5 text-xs">
          {MIX_LABEL[goal.mix]}
        </Pill>
        <span>{goal.mix_note}</span>
      </div>

      <p className="m-0 text-sm leading-relaxed">{goal.note}</p>
    </li>
  );
}

function GoalsLedger({ goals }: { goals: LifeGoal[] }) {
  return (
    <section aria-labelledby="life-goals-title" className="flex w-full flex-col">
      <div className="flex items-baseline justify-between gap-4 pb-1">
        <h2 id="life-goals-title" className="m-0 font-display text-[32px] leading-none">
          Goals
        </h2>
        <span className="text-[13px] text-muted-foreground">In date order</span>
      </div>
      {goals.length === 0 ? (
        <p className="border-t border-border py-7 text-sm text-muted-foreground">No goals yet. Add your details below.</p>
      ) : (
        <ul className="m-0 list-none p-0">
          {goals.map((g) => (
            <GoalRow key={g.key} goal={g} />
          ))}
        </ul>
      )}
    </section>
  );
}

// ---------- 4. insurance ----------

function CoverRow({ label, needed, have, gap }: { label: string; needed: number; have: number; gap: number }) {
  return (
    <li className="flex flex-col gap-4 border-t border-border py-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h3 className="m-0 text-[17px] font-medium">{label}</h3>
        {gap > 0 ? <Pill tone="warn">{formatINR(gap)} short</Pill> : <Pill tone="good">Covered</Pill>}
      </div>
      <div className="grid gap-x-6 gap-y-5 sm:grid-cols-3">
        <div className="flex min-w-0 flex-col gap-1">
          <span className="text-[13px] text-muted-foreground">Needed</span>
          <span className="font-display text-2xl">{formatINR(needed)}</span>
        </div>
        <div className="flex min-w-0 flex-col gap-1">
          <span className="text-[13px] text-muted-foreground">Have</span>
          <span className="font-display text-2xl">{formatINR(have)}</span>
        </div>
        <div className="flex min-w-0 flex-col gap-1">
          <span className="text-[13px] text-muted-foreground">Gap</span>
          <span className="font-display text-2xl">{formatINR(gap)}</span>
        </div>
      </div>
    </li>
  );
}

function Insurance({ insurance }: { insurance: LifePlan["insurance"] }) {
  return (
    <section aria-labelledby="life-insurance-title" className="flex w-full flex-col">
      <div className="flex items-baseline justify-between gap-4 pb-1">
        <h2 id="life-insurance-title" className="m-0 font-display text-[32px] leading-none">
          Insurance
        </h2>
        <span className="text-[13px] text-muted-foreground">Term and health</span>
      </div>
      <ul className="m-0 list-none p-0">
        <CoverRow
          label="Term life"
          needed={insurance.term_needed_paise}
          have={insurance.term_have_paise}
          gap={insurance.term_gap_paise}
        />
        <CoverRow
          label="Health"
          needed={insurance.health_needed_paise}
          have={insurance.health_have_paise}
          gap={insurance.health_gap_paise}
        />
      </ul>
      {insurance.notes.length > 0 && (
        <ul className="m-0 mt-2 list-none space-y-1.5 p-0 text-[13px] text-muted-foreground">
          {insurance.notes.map((n) => (
            <li key={n}>{n}</li>
          ))}
        </ul>
      )}
    </section>
  );
}

// ---------- 5. assumptions ----------

function Assumptions({ a }: { a: LifePlan["assumptions"] }) {
  return (
    <section aria-labelledby="life-assumptions-title" className="flex w-full flex-col gap-3">
      <h2 id="life-assumptions-title" className="m-0 font-display text-[32px] leading-none">
        Assumptions
      </h2>
      <ul className="m-0 list-none space-y-1.5 p-0 text-[13px] text-muted-foreground">
        <li>
          Inflation a year: general {formatPercent(a.general_inflation_bps)}, education {formatPercent(a.education_inflation_bps)},
          medical {formatPercent(a.medical_inflation_bps)}, wedding {formatPercent(a.wedding_inflation_bps)}.
        </li>
        <li>
          Returns a year before tax: equity {formatPercent(a.equity_return_bps)}, hybrid {formatPercent(a.hybrid_return_bps)}, debt{" "}
          {formatPercent(a.debt_return_bps)}. SIPs step up {formatPercent(a.step_up_bps)} a year.
        </li>
        <li>Retirement corpus is sized to withdraw {formatPercent(a.swr_bps)} a year.</li>
      </ul>
      <p className="m-0 text-[13px] text-muted-foreground">
        These are planning defaults, not advice. Check them against your own numbers before acting on them.
      </p>
    </section>
  );
}

// ---------- 6. profile form ----------

type Draft = {
  my_age: string;
  partner_age: string;
  my_take_home: string;
  partner_take_home: string;
  monthly_expenses: string;
  wedding_date: string;
  wedding_budget: string;
  wedding_saved: string;
  child_birth_year: string;
  child_education_today: string;
  child_saved: string;
  retirement_age: string;
  retirement_saved: string;
  emergency_saved: string;
  term_cover: string;
  health_cover: string;
  loans: string;
  raise: string;
  partner_raise: string;
  live_to_age: string;
};

const money = (p: number | null) => (p === null ? "" : paiseToInput(p));

function toDraft(p: LifeProfile): Draft {
  return {
    my_age: String(p.my_age),
    partner_age: p.partner_age === null ? "" : String(p.partner_age),
    my_take_home: money(p.my_take_home_paise),
    partner_take_home: money(p.partner_take_home_paise),
    monthly_expenses: money(p.monthly_expenses_paise),
    wedding_date: p.wedding_date ?? "",
    wedding_budget: money(p.wedding_budget_paise),
    wedding_saved: money(p.wedding_saved_paise),
    child_birth_year: p.child_birth_year === null ? "" : String(p.child_birth_year),
    child_education_today: money(p.child_education_today_paise),
    child_saved: money(p.child_saved_paise),
    retirement_age: String(p.retirement_age),
    retirement_saved: money(p.retirement_saved_paise),
    emergency_saved: money(p.emergency_saved_paise),
    term_cover: money(p.term_cover_paise),
    health_cover: money(p.health_cover_paise),
    loans: money(p.loans_paise),
    raise: String((p.raise_bps ?? 800) / 100),
    partner_raise: String((p.partner_raise_bps ?? 800) / 100),
    live_to_age: String(p.live_to_age ?? 90),
  };
}

/** Blank money is zero, or null where the field is optional. Anything else must be a plain rupee amount. */
function rupees(label: string, text: string, optional = false): number | null {
  const t = text.trim();
  if (t === "") {
    if (optional) return null;
    return 0;
  }
  const p = toPaise(t);
  if (p === null || p < 0) throw new Error(`${label}: enter a plain rupee amount, like 25000 or 25,000.50.`);
  return p;
}

function whole(label: string, text: string, min: number, max: number): number {
  const n = Number(text.trim());
  if (!Number.isInteger(n) || n < min || n > max) throw new Error(`${label} must be a whole number from ${min} to ${max}.`);
  return n;
}

/** A percentage from 0 to 30, as basis points. */
function percent(label: string, text: string): number {
  const n = Number(text.trim());
  if (text.trim() === "" || !Number.isFinite(n) || n < 0 || n > 30) throw new Error(`${label} must be between 0 and 30.`);
  return Math.round(n * 100);
}

function buildProfile(d: Draft): LifeProfile {
  const myAge = whole("Your age", d.my_age, 18, 80);
  const retirementAge = whole("Retirement age", d.retirement_age, 40, 75);
  if (retirementAge <= myAge) throw new Error("Retirement age must be after your age.");
  const partnerAge = d.partner_age.trim() === "" ? null : whole("Partner's age", d.partner_age, 18, 80);
  const childYear = d.child_birth_year.trim() === "" ? null : whole("Child's birth year", d.child_birth_year, 1950, 2200);
  return {
    my_age: myAge,
    partner_age: partnerAge,
    my_take_home_paise: rupees("Your take-home", d.my_take_home) ?? 0,
    partner_take_home_paise: rupees("Partner's take-home", d.partner_take_home) ?? 0,
    monthly_expenses_paise: rupees("Monthly expenses", d.monthly_expenses, true),
    wedding_date: d.wedding_date || null,
    wedding_budget_paise: d.wedding_date ? rupees("Wedding budget", d.wedding_budget, true) : null,
    wedding_saved_paise: rupees("Saved for the wedding", d.wedding_saved) ?? 0,
    child_birth_year: childYear,
    child_education_today_paise: rupees("Education cost today", d.child_education_today) ?? 0,
    child_saved_paise: rupees("Saved for education", d.child_saved) ?? 0,
    retirement_age: retirementAge,
    retirement_saved_paise: rupees("Saved for retirement", d.retirement_saved) ?? 0,
    emergency_saved_paise: rupees("Emergency savings", d.emergency_saved) ?? 0,
    term_cover_paise: rupees("Term life cover", d.term_cover) ?? 0,
    health_cover_paise: rupees("Health cover", d.health_cover) ?? 0,
    loans_paise: rupees("Outstanding loans", d.loans) ?? 0,
    raise_bps: percent("Your yearly raise", d.raise),
    partner_raise_bps: percent("Partner's yearly raise", d.partner_raise),
    live_to_age: whole("Plan to age", d.live_to_age, retirementAge, 110),
  };
}

function Field({
  id,
  label,
  hint,
  className,
  children,
}: {
  id: string;
  label: string;
  hint?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <div className={cn("field", className)}>
      <Label htmlFor={id}>{label}</Label>
      {children}
      {hint && <p className="m-0 text-[13px] leading-snug text-muted-foreground">{hint}</p>}
    </div>
  );
}

/** A small subheading over a two-column field grid. */
function Group({ title, children }: { title: string; children: ReactNode }) {
  return (
    <fieldset className="m-0 min-w-0 border-0 p-0">
      <legend className="mb-4 p-0 text-sm font-medium text-foreground">{title}</legend>
      <div className="form-grid">{children}</div>
    </fieldset>
  );
}

function ProfileForm({ profile, saved }: { profile: LifeProfile; saved: boolean }) {
  const [draft, setDraft] = useState<Draft>(() => toDraft(profile));
  const [error, setError] = useState<string | null>(null);
  const save = useSaveLife();

  const set = (k: keyof Draft) => (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setDraft((d) => ({ ...d, [k]: value }));
  };

  const submit = (e: FormEvent) => {
    e.preventDefault();
    let next: LifeProfile;
    try {
      next = buildProfile(draft);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Check the details and try again.");
      return;
    }
    setError(null);
    save.mutate(next, {
      onError: (err) => setError(err instanceof Error ? err.message : "Couldn't save. Try again."),
    });
  };

  const moneyField = (k: keyof Draft, id: string, label: string, hint?: string, className?: string) => (
    <Field id={id} label={label} hint={hint} className={className}>
      <MoneyInput id={id} value={draft[k]} onChange={set(k)} />
    </Field>
  );

  return (
    <details open={!saved} className="group rounded-[28px] border border-border bg-card">
      <summary className="flex cursor-pointer list-none items-center justify-between gap-4 px-6 py-6 md:px-8">
        <span className="flex flex-col gap-1.5">
          <span className="font-display text-[32px] leading-none">Your details</span>
          <span className="text-[13px] text-muted-foreground">
            {saved ? "Saved. Change anything here and the plan updates." : "Fill these in to see your plan."}
          </span>
        </span>
        <ChevronDown className="h-5 w-5 shrink-0 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden />
      </summary>

      <form onSubmit={submit} noValidate className="grid gap-8 px-6 pb-8 md:px-8">
        <Group title="You two">
          <Field id="my_age" label="Your age">
            <Input id="my_age" type="number" inputMode="numeric" min={18} max={80} value={draft.my_age} onChange={set("my_age")} />
          </Field>
          <Field id="partner_age" label="Partner's age" hint="Optional.">
            <Input id="partner_age" type="number" inputMode="numeric" min={18} max={80} value={draft.partner_age} onChange={set("partner_age")} />
          </Field>
          {moneyField("my_take_home", "my_take_home", "Your take-home a month (₹)")}
          {moneyField("partner_take_home", "partner_take_home", "Partner's take-home a month (₹)", "Leave blank if there is no partner income.")}
          {moneyField(
            "monthly_expenses",
            "monthly_expenses",
            "Monthly expenses (₹)",
            "Blank uses what your statements show you spend, else your spending budget, else half of combined take-home.",
            "col-span-full",
          )}
        </Group>

        <Group title="Wedding">
          <Field id="wedding_date" label="Wedding date" hint="Leave blank if there is no wedding goal.">
            <Input id="wedding_date" type="date" value={draft.wedding_date} onChange={set("wedding_date")} />
          </Field>
          {moneyField("wedding_budget", "wedding_budget", "Wedding budget, in today's rupees (₹)", "Blank uses ₹25,00,000.")}
          {moneyField("wedding_saved", "wedding_saved", "Already saved for the wedding (₹)")}
        </Group>

        <Group title="Child">
          <Field id="child_birth_year" label="Child's birth year" hint="Leave blank if there is no child goal.">
            <Input id="child_birth_year" type="number" inputMode="numeric" min={1950} max={2200} value={draft.child_birth_year} onChange={set("child_birth_year")} />
          </Field>
          {moneyField("child_education_today", "child_education_today", "Education cost, UG + PG, today (₹)")}
          {moneyField("child_saved", "child_saved", "Already saved for education (₹)")}
        </Group>

        <Group title="Retirement">
          <Field id="retirement_age" label="Retirement age">
            <Input id="retirement_age" type="number" inputMode="numeric" min={40} max={75} value={draft.retirement_age} onChange={set("retirement_age")} />
          </Field>
          {moneyField("retirement_saved", "retirement_saved", "Already saved for retirement (₹)", "EPF, PPF, NPS and earmarked mutual funds.")}
        </Group>

        <Group title="The long run">
          <Field id="raise" label="Your yearly raise (%)" hint="Inflation is taken as 6%, so 8 is a real raise of 2.">
            <Input id="raise" inputMode="decimal" value={draft.raise} onChange={set("raise")} />
          </Field>
          <Field id="partner_raise" label="Partner's yearly raise (%)">
            <Input id="partner_raise" inputMode="decimal" value={draft.partner_raise} onChange={set("partner_raise")} />
          </Field>
          <Field id="live_to_age" label="Plan to age" hint="The life line runs to this age.">
            <Input id="live_to_age" type="number" inputMode="numeric" min={60} max={110} value={draft.live_to_age} onChange={set("live_to_age")} />
          </Field>
        </Group>

        <Group title="Safety net">
          {moneyField("emergency_saved", "emergency_saved", "Emergency savings (₹)")}
          {moneyField("loans", "loans", "Outstanding loans (₹)")}
          {moneyField("term_cover", "term_cover", "Existing term life cover, both (₹)")}
          {moneyField("health_cover", "health_cover", "Existing family health cover (₹)")}
        </Group>

        {error && (
          <p role="alert" className="m-0 text-sm text-destructive">
            {error}
          </p>
        )}
        <div className="form-actions">
          <Button type="submit" disabled={save.isPending}>
            {save.isPending ? "Saving..." : "Save details"}
          </Button>
        </div>
      </form>
    </details>
  );
}

// ---------- page ----------

export default function LifePage() {
  const { data, isLoading, isError, refetch } = useLife();

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl stack">
        <div>
          <h1 className="page-title">Life line</h1>
          <p className="page-sub">Your money from now to 90, and what each choice does to it.</p>
        </div>

        {isLoading && <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your life plan" />}

        {isError && !data && (
          <div className="rounded-[28px] border border-dashed border-border px-6 py-8 text-center text-sm text-muted-foreground">
            Couldn't load your life plan.{" "}
            <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
              Try again
            </Button>
          </div>
        )}

        <LifeLine />

        {data && (
          <>
            <ProfileForm profile={data.profile} saved={data.saved} />
            <div className="pt-4">
              <h2 className="m-0 font-display text-[32px] leading-none">The goals inside it</h2>
              <p className="page-sub">What each goal needs a month, and whether your surplus covers it.</p>
            </div>
            <ForecastPanel />
            <PlanSummary plan={data.plan} profile={data.profile} />
            <GoalsLedger goals={data.plan.goals} />
            <Insurance insurance={data.plan.insurance} />
            <Assumptions a={data.plan.assumptions} />
          </>
        )}
      </main>
    </div>
  );
}
