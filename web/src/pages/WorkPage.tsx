import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router-dom";
import { Trash2 } from "lucide-react";

import { Header } from "@/components/layout/Header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import type { Choice } from "@/services/lifeline";
import { STAGES, stageLabel, useAddLead, useRemoveLead, useSaveLead, useWork, type Lead, type LeadInput, type Stage } from "@/services/work";
import { formatDay } from "@/utils/dates";
import { formatINR, paiseToInput, toPaise } from "@/utils/money";

const PANEL = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";

interface Draft {
  company: string;
  role: string;
  stage: Stage;
  take_home: string;
  next_step: string;
  next_on: string;
  note: string;
}

const blank: Draft = { company: "", role: "", stage: "idea", take_home: "", next_step: "", next_on: "", note: "" };

const toDraft = (l: Lead): Draft => ({
  company: l.company,
  role: l.role,
  stage: l.stage,
  take_home: l.take_home_paise === null ? "" : paiseToInput(l.take_home_paise),
  next_step: l.next_step,
  next_on: l.next_on ?? "",
  note: l.note,
});

/** The typed fields as a lead, or an Error that names what to fix. */
function toInput(d: Draft): LeadInput {
  if (d.company.trim() === "") throw new Error("Company: fill this in.");
  let pay: number | null = null;
  if (d.take_home.trim() !== "") {
    pay = toPaise(d.take_home);
    if (pay === null || pay <= 0) throw new Error("Take-home: enter a plain rupee amount, like 220000.");
  }
  return {
    company: d.company.trim(),
    role: d.role.trim(),
    stage: d.stage,
    take_home_paise: pay,
    next_step: d.next_step.trim(),
    next_on: d.next_on || null,
    note: d.note.trim(),
  };
}

/** The month after this one, as "YYYY-MM": the earliest a new job could start. */
function nextMonth(): string {
  const d = new Date();
  d.setMonth(d.getMonth() + 1, 1);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

function LeadForm({ id, start, busy, submitLabel, onSubmit, onCancel }: {
  id: string;
  start: Draft;
  busy: boolean;
  submitLabel: string;
  onSubmit: (lead: LeadInput, reset: () => void) => void;
  onCancel?: () => void;
}) {
  const [d, setD] = useState<Draft>(start);
  const [error, setError] = useState<string | null>(null);
  const set = (k: keyof Draft) => (e: { target: { value: string } }) => setD((x) => ({ ...x, [k]: e.target.value }));

  const submit = (e: FormEvent) => {
    e.preventDefault();
    try {
      const lead = toInput(d);
      setError(null);
      onSubmit(lead, () => setD(blank));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Check the details and try again.");
    }
  };

  return (
    <form onSubmit={submit} noValidate className="form-grid">
      <div className="field">
        <Label htmlFor={`${id}-company`}>Company</Label>
        <Input id={`${id}-company`} value={d.company} onChange={set("company")} maxLength={60} />
      </div>
      <div className="field">
        <Label htmlFor={`${id}-role`}>Role</Label>
        <Input id={`${id}-role`} value={d.role} onChange={set("role")} maxLength={60} />
      </div>
      <div className="field">
        <Label htmlFor={`${id}-stage`}>Stage</Label>
        <Select id={`${id}-stage`} value={d.stage} onChange={(e) => setD((x) => ({ ...x, stage: e.target.value as Stage }))}>
          {STAGES.map((s) => (
            <option key={s.value} value={s.value}>
              {s.label}
            </option>
          ))}
        </Select>
      </div>
      <div className="field">
        <Label htmlFor={`${id}-pay`}>Take-home a month (₹)</Label>
        <MoneyInput id={`${id}-pay`} value={d.take_home} onChange={set("take_home")} />
        <p className="m-0 text-[13px] leading-snug text-muted-foreground">Expected or offered. With it, you can see the job on your life line.</p>
      </div>
      <div className="field">
        <Label htmlFor={`${id}-step`}>Next step</Label>
        <Input id={`${id}-step`} value={d.next_step} onChange={set("next_step")} maxLength={80} placeholder="Send CV, prepare for round 2" />
      </div>
      <div className="field">
        <Label htmlFor={`${id}-on`}>By when</Label>
        <Input id={`${id}-on`} type="date" value={d.next_on} onChange={set("next_on")} />
      </div>
      <div className="field col-span-full">
        <Label htmlFor={`${id}-note`}>Note</Label>
        <Input id={`${id}-note`} value={d.note} onChange={set("note")} maxLength={280} />
      </div>
      {error && (
        <p role="alert" className="col-span-full m-0 text-sm text-destructive">
          {error}
        </p>
      )}
      <div className="form-actions">
        {onCancel && (
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
        )}
        <Button type="submit" disabled={busy}>
          {busy ? "Saving..." : submitLabel}
        </Button>
      </div>
    </form>
  );
}

function LeadRow({ lead }: { lead: Lead }) {
  const navigate = useNavigate();
  const save = useSaveLead();
  const remove = useRemoveLead();
  const [editing, setEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fail = (e: unknown) => setError(e instanceof Error && e.message ? e.message : "That didn't work. Try again.");

  const onLifeLine = () => {
    if (lead.take_home_paise === null) return;
    const choice: Choice = { kind: "job_switch", month: nextMonth(), take_home_paise: lead.take_home_paise, gap_months: 0, raise_bps: null };
    navigate("/life", { state: { choice } });
  };

  if (editing) {
    return (
      <li className="border-t border-border py-5 first:border-t-0 first:pt-1">
        <LeadForm
          id={`lead-${lead.id}`}
          start={toDraft(lead)}
          busy={save.isPending}
          submitLabel="Save"
          onCancel={() => setEditing(false)}
          onSubmit={(next) => save.mutate({ id: lead.id, lead: next }, { onSuccess: () => setEditing(false), onError: fail })}
        />
        {error && (
          <p role="alert" className="m-0 pt-3 text-sm text-destructive">
            {error}
          </p>
        )}
      </li>
    );
  }

  const details = [
    lead.role,
    lead.take_home_paise !== null ? `${formatINR(lead.take_home_paise)} a month` : "",
    lead.next_step ? `Next: ${lead.next_step}${lead.next_on ? ` by ${formatDay(lead.next_on)}` : ""}` : lead.next_on ? `Next step by ${formatDay(lead.next_on)}` : "",
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <li className="flex flex-col gap-3 border-t border-border py-5 first:border-t-0 first:pt-1">
      <div className="flex min-w-0 flex-col gap-1">
        <span className="text-[15px] font-medium">{lead.company}</span>
        {details && <span className="text-[13px] text-muted-foreground">{details}</span>}
        {lead.note && <span className="text-[13px] text-muted-foreground">{lead.note}</span>}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {lead.take_home_paise !== null && (
          <Button variant="outline" className="h-11 rounded-full" onClick={onLifeLine}>
            See it on my life line
          </Button>
        )}
        <Button variant="outline" className="h-11 rounded-full" onClick={() => setEditing(true)}>
          Edit
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-11 w-11"
          disabled={remove.isPending}
          onClick={() => remove.mutate(lead.id, { onError: fail })}
          aria-label={`Remove ${lead.company}`}
        >
          <Trash2 className="h-4 w-4" aria-hidden />
        </Button>
      </div>
      {error && (
        <p role="alert" className="m-0 text-sm text-destructive">
          {error}
        </p>
      )}
    </li>
  );
}

/** Work: the job search as a short list of leads, each with one next step. */
export default function WorkPage() {
  const work = useWork();
  const add = useAddLead();
  const [error, setError] = useState<string | null>(null);
  const leads = work.data?.leads ?? [];
  const groups = STAGES.map((s) => ({ stage: s.value, leads: leads.filter((l) => l.stage === s.value) })).filter((g) => g.leads.length > 0);
  // Furthest along first, as the server sorts them.
  groups.sort((a, b) => leads.findIndex((l) => l.stage === a.stage) - leads.findIndex((l) => l.stage === b.stage));

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page max-w-3xl stack">
        <header>
          <h1 className="page-title">Work</h1>
          <p className="page-sub">Your job search: a few leads, one next step each, and what an offer would do to your life line.</p>
        </header>

        {work.isLoading && <div className="h-48 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your leads" />}
        {work.isError && !work.data && (
          <div className="rounded-[28px] border border-dashed border-border px-6 py-8 text-center text-sm text-muted-foreground">
            Couldn't load your leads.{" "}
            <Button variant="link" className="h-auto p-0" onClick={() => void work.refetch()}>
              Try again
            </Button>
          </div>
        )}

        {work.data && leads.length === 0 && <p className="empty-note m-0">No leads yet. Add the first company you would like to work for.</p>}

        {groups.map((g) => (
          <section key={g.stage} aria-labelledby={`work-${g.stage}`} className={PANEL}>
            <h2 id={`work-${g.stage}`} className="m-0 font-display text-[32px] leading-none">
              {stageLabel(g.stage)}
            </h2>
            <ul className="m-0 list-none p-0">
              {g.leads.map((l) => (
                <LeadRow key={l.id} lead={l} />
              ))}
            </ul>
          </section>
        ))}

        <section aria-labelledby="work-add" className={PANEL}>
          <h2 id="work-add" className="m-0 font-display text-[32px] leading-none">
            Add a lead
          </h2>
          <LeadForm
            id="new-lead"
            start={blank}
            busy={add.isPending}
            submitLabel="Add lead"
            onSubmit={(lead, reset) =>
              add.mutate(lead, {
                onSuccess: () => {
                  setError(null);
                  reset();
                },
                onError: (e) => setError(e instanceof Error && e.message ? e.message : "Couldn't add the lead. Try again."),
              })
            }
          />
          {error && (
            <p role="alert" className="m-0 text-sm text-destructive">
              {error}
            </p>
          )}
        </section>
      </main>
    </div>
  );
}
