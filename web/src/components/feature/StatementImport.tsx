import { useRef, useState, type DragEvent, type FormEvent, type KeyboardEvent } from "react";
import { useQuery } from "@tanstack/react-query";

import { ingestApi } from "@/services/api";
import type { StatementResult } from "@/services/types";
import { INGEST_INVALIDATE, useAccounts, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";

const MAX_BYTES = 5 * 1024 * 1024;
const MAX_NAMES = 5;
const HOLDER_NAMES_KEY = ["ingest-holder-names"];

/** A reason the file can't be sent, or null when it can. */
function fileProblem(file: File): string | null {
  if (/\.(xlsx?|pdf)$/i.test(file.name)) {
    return "Excel files can't be read. Download the Delimited (.txt) format from your bank, or save the sheet as CSV.";
  }
  if (!/\.(txt|csv)$/i.test(file.name)) return "Upload the statement as a .txt or .csv file.";
  if (file.size > MAX_BYTES) return "That file is over 5 MB. Export one period at a time.";
  return null;
}

/** A reason the name can't be saved, or null when it can. */
function nameProblem(raw: string, saved: string[]): string | null {
  const name = raw.trim();
  const length = [...name].length;
  if (length < 2 || length > 60) return "Use 2 to 60 characters.";
  if (saved.length >= MAX_NAMES) return `Save up to ${MAX_NAMES} names.`;
  if (saved.some((s) => s.toLowerCase() === name.toLowerCase())) return "That name is already saved.";
  return null;
}

const plural = (n: number, one: string, many: string) => (n === 1 ? `${n} ${one}` : `${n} ${many}`);

function Outcome({
  r,
  suggestion,
  onConfirmSuggestion,
  confirming,
}: {
  r: StatementResult;
  /** The payee to offer as the user, when the file suggests one and no names are saved. */
  suggestion: string | null;
  onConfirmSuggestion: (name: string) => void;
  confirming: boolean;
}) {
  const parts = [`${r.added} added`];
  if (r.upgraded) parts.push(`${r.upgraded} bank ${r.upgraded === 1 ? "alert" : "alerts"} confirmed`);
  if (r.duplicates) parts.push(`${r.duplicates} already imported`);
  if (r.skipped) parts.push(`${r.skipped} unreadable`);
  const hasBalances = r.opening_paise !== null && r.closing_paise !== null;
  return (
    <div role="status" className="space-y-2 rounded-lg bg-muted/50 p-3 text-sm">
      <p className="font-medium">
        {parts.join(", ")}
        {r.first_date && r.last_date ? ` (${formatDay(r.first_date)} to ${formatDay(r.last_date)})` : ""}.
      </p>
      {r.categorized > 0 && <p className="text-muted-foreground">{r.categorized} categorised automatically.</p>}
      {hasBalances && (
        <div className="flex flex-wrap items-center gap-2">
          {r.chain_ok ? (
            <span className="tone-good rounded-full px-2 py-0.5 text-xs font-medium">Balance check passed</span>
          ) : (
            <span className="tone-over rounded-full px-2 py-0.5 text-xs font-medium">Balances don't add up</span>
          )}
          <span className="text-muted-foreground">
            Balance {formatINR(r.opening_paise ?? 0)} to {formatINR(r.closing_paise ?? 0)}
          </span>
        </div>
      )}
      {hasBalances && !r.chain_ok && (
        <p className="text-muted-foreground">
          Some rows may be missing or edited in this file. Check it against the bank before you rely on the totals.
        </p>
      )}
      {r.not_on_statement > 0 && (
        <p className="text-muted-foreground">
          {plural(r.not_on_statement, "bank alert in this period isn't on the statement", "bank alerts in this period aren't on the statement")}.
          They may be pending, or from another account.
        </p>
      )}
      {r.uncategorized > 0 && (
        <p className="text-muted-foreground">{r.uncategorized} need a category. Fix them in the list above and tick Remember.</p>
      )}
      {r.balance_updated && (
        <p className="text-muted-foreground">
          {r.balance_updated.account_name} balance set to {formatINR(r.balance_updated.balance_paise)}
        </p>
      )}
      {suggestion && (
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border pt-3">
          <p>
            Is <span className="font-medium">{suggestion}</span> you? Transfers to and from your own accounts won't count as
            spending.
          </p>
          <Button size="sm" onClick={() => onConfirmSuggestion(suggestion)} disabled={confirming}>
            Yes, that's me
          </Button>
        </div>
      )}
    </div>
  );
}

/** Upload a bank statement (.txt or .csv); the server reads and stores every row, then forgets the file. */
export function StatementImport() {
  const { data: accounts } = useAccounts();
  const names = useQuery({ queryKey: HOLDER_NAMES_KEY, queryFn: ingestApi.holderNames, retry: 1 });
  const saved = names.data?.names ?? [];

  const [accountId, setAccountId] = useState("");
  const [result, setResult] = useState<StatementResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const [draft, setDraft] = useState("");
  const [nameError, setNameError] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);

  const upload = usePlannerMutation(
    ({ file, account }: { file: File; account?: string }) => ingestApi.uploadStatement(file, { account_id: account }),
    [...INGEST_INVALIDATE, ["accounts"]],
  );
  const saveNames = usePlannerMutation((next: string[]) => ingestApi.saveHolderNames(next), [HOLDER_NAMES_KEY], {
    success: "Names saved",
  });
  const confirmSelf = usePlannerMutation(
    async (next: string[]) => {
      await ingestApi.saveHolderNames(next);
      return ingestApi.recategorize();
    },
    [...INGEST_INVALIDATE, HOLDER_NAMES_KEY],
    { success: "Saved. Transfers to your own accounts won't count as spending." },
  );

  const pick = (file: File | undefined) => {
    if (!file) return;
    setResult(null);
    setError(null);
    const problem = fileProblem(file);
    if (problem) return setError(problem);
    upload.mutate(
      { file, account: accountId || undefined },
      {
        onSuccess: setResult,
        onError: (e) => setError(e instanceof Error ? e.message : "Couldn't import that file."),
      },
    );
    if (input.current) input.current.value = "";
  };

  const onDrop = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setDragging(false);
    pick(e.dataTransfer.files[0]);
  };

  const onZoneKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      input.current?.click();
    }
  };

  const addName = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const problem = nameProblem(draft, saved);
    setNameError(problem);
    if (problem) return;
    saveNames.mutate([...saved, draft.trim()], { onSuccess: () => setDraft("") });
  };

  const confirmSuggestion = (name: string) => {
    confirmSelf.mutate([...saved, name], {
      onSuccess: () => setResult((r) => (r ? { ...r, holder_name_suggestion: null } : r)),
    });
  };

  const banks = (accounts ?? []).filter((a) => a.kind === "bank");
  const suggestion = result?.holder_name_suggestion && saved.length === 0 ? result.holder_name_suggestion : null;

  return (
    <div className="space-y-4">
      <div className="field">
        <Label htmlFor="statement-account">Which account is this statement for?</Label>
        <Select id="statement-account" value={accountId} onChange={(e) => setAccountId(e.target.value)}>
          <option value="">Not sure / a credit card</option>
          {banks.map((a) => (
            <option key={a.account_id} value={a.account_id}>
              {a.name}
              {a.last_4_digits ? ` (${a.last_4_digits})` : ""}
            </option>
          ))}
        </Select>
        <p className="max-w-prose text-sm leading-snug text-muted-foreground">
          Picking an account also sets its balance from the statement's newest line, unless you set a newer one.
        </p>
      </div>

      <div
        role="button"
        tabIndex={0}
        aria-label="Choose a statement file, or drop it here"
        aria-disabled={upload.isPending}
        onClick={() => !upload.isPending && input.current?.click()}
        onKeyDown={onZoneKey}
        onDragOver={(e) => {
          e.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={onDrop}
        className={`flex flex-col items-center gap-1 rounded-[18px] border border-dashed px-5 py-8 text-center transition-colors ${
          dragging ? "border-primary bg-accent" : "border-border"
        } ${upload.isPending ? "pointer-events-none opacity-60" : "cursor-pointer hover:bg-accent/40"}`}
      >
        <p className="font-display text-xl">{upload.isPending ? "Importing..." : "Drop your statement here"}</p>
        <p className="text-sm text-muted-foreground">or click to choose a .txt or .csv file, up to 5 MB</p>
      </div>
      <input
        ref={input}
        type="file"
        accept=".txt,.csv,text/plain,text/csv"
        aria-label="Statement file"
        className="hidden"
        disabled={upload.isPending}
        onChange={(e) => pick(e.target.files?.[0])}
      />
      <p className="max-w-prose text-sm leading-snug text-muted-foreground">
        Your file is read in memory and discarded straight after import. Only the transactions are kept.
      </p>

      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      {result && (
        <Outcome r={result} suggestion={suggestion} onConfirmSuggestion={confirmSuggestion} confirming={confirmSelf.isPending} />
      )}

      <section className="space-y-3 border-t border-border pt-4" aria-labelledby="holder-names-heading">
        <div className="field">
          <p id="holder-names-heading" className="text-sm font-medium">
            Names on your accounts
          </p>
          <p className="max-w-prose text-sm leading-snug text-muted-foreground">
            Transfers to and from these names count as moving your own money, not spending.
          </p>
        </div>
        {saved.length > 0 ? (
          <ul className="flex flex-wrap gap-2" aria-label="Saved names">
            {saved.map((n) => (
              <li key={n} className="tone-quiet inline-flex items-center gap-1 rounded-full py-1 pl-3 pr-1 text-sm">
                {n}
                <button
                  type="button"
                  aria-label={`Remove ${n}`}
                  onClick={() => saveNames.mutate(saved.filter((s) => s !== n))}
                  disabled={saveNames.isPending}
                  className="rounded-full px-1.5 leading-none hover:bg-accent"
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="empty-note">No names saved yet.</p>
        )}
        <form onSubmit={addName} className="flex flex-wrap gap-2">
          <Input
            value={draft}
            onChange={(e) => {
              setDraft(e.target.value);
              setNameError(null);
            }}
            placeholder="Full name as the bank prints it"
            aria-label="Add a name"
            maxLength={60}
            className="min-w-0 flex-1 sm:max-w-sm"
          />
          <Button type="submit" variant="outline" disabled={!draft.trim() || saveNames.isPending}>
            Add
          </Button>
        </form>
        {nameError && (
          <p role="alert" className="text-sm text-destructive">
            {nameError}
          </p>
        )}
      </section>
    </div>
  );
}
