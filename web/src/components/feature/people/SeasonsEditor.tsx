import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { MONTH_NAMES, errorMessage } from "@/components/feature/people/ui";
import { rupees } from "@/services/checkin";
import { addSeason, deleteSeason, updateSeason, useSeasons, usePeopleMutation, type Season } from "@/services/people";
import { paiseToInput, toPaise } from "@/utils/money";

/** Add or edit a festival season: a name, the month it starts, and the budget for that season. */
export function SeasonForm({ initial, onDone }: { initial?: Season; onDone: () => void }) {
  const id = useId();
  const save = usePeopleMutation((s: { id?: string; label: string; month: number; budget_paise: number }) =>
    s.id ? updateSeason(s.id, { label: s.label, month: s.month, budget_paise: s.budget_paise }) : addSeason(s),
  );
  const [label, setLabel] = useState(initial?.label ?? "");
  const [month, setMonth] = useState(initial?.month ?? 10);
  const [budget, setBudget] = useState(initial ? paiseToInput(initial.budget_paise) : "");

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const paise = toPaise(budget);
    if (label.trim() === "") {
      toast.error("Name the season first.");
      return;
    }
    if (paise === null || paise <= 0) {
      toast.error("Enter a budget in rupees, like 5000.");
      return;
    }
    save.mutate(
      { id: initial?.id, label: label.trim(), month, budget_paise: paise },
      {
        onSuccess: () => {
          toast.success(initial ? "Season updated." : "Season added.");
          onDone();
        },
        onError: (err) => toast.error(errorMessage(err)),
      },
    );
  };

  return (
    <form onSubmit={submit} className="grid gap-4 sm:grid-cols-3" aria-label={initial ? `Edit ${initial.label}` : "Add a season"}>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-label`}>Season</Label>
        <Input id={`${id}-label`} maxLength={40} value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Diwali" />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-month`}>Month</Label>
        <Select id={`${id}-month`} value={month} onChange={(e) => setMonth(Number(e.target.value))}>
          {MONTH_NAMES.map((name, i) => (
            <option key={name} value={i + 1}>
              {name}
            </option>
          ))}
        </Select>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-budget`}>Budget (₹)</Label>
        <MoneyInput id={`${id}-budget`} value={budget} onChange={(e) => setBudget(e.target.value)} />
      </div>
      <div className="flex flex-wrap gap-2 sm:col-span-3">
        <Button type="submit" disabled={save.isPending}>
          {initial ? "Save season" : "Add season"}
        </Button>
        {initial && (
          <Button type="button" variant="ghost" onClick={onDone}>
            Cancel
          </Button>
        )}
      </div>
    </form>
  );
}

/** One season in the list, with edit and delete. */
function SeasonRow({ season }: { season: Season }) {
  const [editing, setEditing] = useState(false);
  const remove = usePeopleMutation(deleteSeason);

  if (editing) {
    return (
      <li className="border-t border-border py-5 first:border-t-0 first:pt-1">
        <SeasonForm initial={season} onDone={() => setEditing(false)} />
      </li>
    );
  }
  return (
    <li className="ledger-row">
      <span className="flex min-w-0 flex-col gap-1">
        <span className="text-[15px] font-medium">{season.label}</span>
        <span className="text-[13px] text-muted-foreground">{`${MONTH_NAMES[season.month - 1]} · budget ${rupees(season.budget_paise)}`}</span>
      </span>
      <span className="flex shrink-0 gap-2">
        <Button variant="outline" className="min-h-11" onClick={() => setEditing(true)}>
          {`Edit ${season.label}`}
        </Button>
        <Button
          variant="ghost"
          className="min-h-11"
          disabled={remove.isPending}
          onClick={() => {
            if (!window.confirm(`Remove ${season.label}?`)) return;
            remove.mutate(season.id, {
              onSuccess: () => toast.success(`${season.label} removed.`),
              onError: (err) => toast.error(errorMessage(err)),
            });
          }}
        >
          {`Delete ${season.label}`}
        </Button>
      </span>
    </li>
  );
}

/** Festival seasons: budgets set by hand, once a year. */
export function SeasonsEditor() {
  const seasons = useSeasons();
  const [adding, setAdding] = useState(false);
  const list = seasons.data ?? [];

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h3 className="m-0 text-[15px] font-medium">Festival seasons</h3>
        {!adding && (
          <Button variant="outline" className="min-h-11" onClick={() => setAdding(true)}>
            Add a season
          </Button>
        )}
      </div>
      {seasons.isError && !seasons.data && (
        <p className="m-0 text-sm text-muted-foreground">Couldn't load your seasons.</p>
      )}
      {list.length > 0 ? (
        <ul className="ledger">
          {list.map((s) => (
            <SeasonRow key={s.id} season={s} />
          ))}
        </ul>
      ) : (
        seasons.data && <p className="m-0 text-sm text-muted-foreground">No festival seasons yet.</p>
      )}
      {adding && <SeasonForm onDone={() => setAdding(false)} />}
    </div>
  );
}

