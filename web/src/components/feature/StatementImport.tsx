import { useRef, useState, type DragEvent, type FormEvent, type KeyboardEvent } from "react";
import { useQuery } from "@tanstack/react-query";

import api, { ingestApi } from "@/services/api";
import { decodeCardList } from "@/proto/decoder";
import type { SignRule, StatementMapping, StatementNeedsMapping, StatementPreview, StatementResult, StatementTarget } from "@/services/types";
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
const CARDS_KEY = ["statement-cards"];

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

/** The account or card the select names ("acct:<id>" or "card:<id>"); none for "Not sure". */
function targetOf(value: string): Pick<StatementTarget, "account_id" | "card_id"> {
  if (value.startsWith("acct:")) return { account_id: value.slice(5) };
  if (value.startsWith("card:")) return { card_id: value.slice(5) };
  return {};
}

/** The 422 an upload answers with when its layout can't be read, or null for any other error. */
function needsMapping(e: unknown): StatementNeedsMapping | null {
  if (!(e instanceof Error)) return null;
  try {
    const body = JSON.parse(e.message) as Partial<StatementNeedsMapping>;
    return body.error === "needs_mapping" && body.preview ? (body as StatementNeedsMapping) : null;
  } catch {
    return null;
  }
}

/** What a column holds, as the user picks it. */
type Field = "ignore" | "date" | "narration" | "debit" | "credit" | "amount" | "flag" | "balance" | "reference";

const FIELDS: { value: Field; label: string }[] = [
  { value: "ignore", label: "Ignore" },
  { value: "date", label: "Date" },
  { value: "narration", label: "Description" },
  { value: "debit", label: "Debit (money out)" },
  { value: "credit", label: "Credit (money in)" },
  { value: "amount", label: "Amount (signed)" },
  { value: "flag", label: "Dr/Cr flag" },
  { value: "balance", label: "Balance" },
  { value: "reference", label: "Reference" },
];

/** Fields that take one column each. */
const SINGLE: Field[] = ["date", "narration", "debit", "credit", "amount", "flag", "balance", "reference"];

/** A field a column's header most likely means, from its name alone. */
function guessField(name: string): Field {
  const n = name.toLowerCase();
  if (/date/.test(n) && !/value/.test(n)) return "date";
  if (/narration|description|particular|details|remark/.test(n)) return "narration";
  if (/withdraw|debit|paid out|^dr$/.test(n)) return "debit";
  if (/deposit|credit|paid in|^cr$/.test(n)) return "credit";
  if (/balance/.test(n)) return "balance";
  if (/dr\/cr|cr\/dr|^type$/.test(n)) return "flag";
  if (/amount|amt/.test(n)) return "amount";
  if (/ref|chq|cheque|utr/.test(n)) return "reference";
  return "ignore";
}

function columnCount(preview: StatementPreview): number {
  return Math.max(0, ...preview.rows.map((r) => r.length));
}

/** The column a field is assigned to, or null. */
function columnOf(assign: Record<number, Field>, field: Field): number | null {
  const entry = Object.entries(assign).find(([, f]) => f === field);
  return entry ? Number(entry[0]) : null;
}

/** The starting assignment: the detected mapping when there is one, else each header's name. */
function initialAssignments(preview: StatementPreview): Record<number, Field> {
  const header = preview.rows[0] ?? [];
  const assign: Record<number, Field> = {};
  const g = preview.guessed;
  if (g) {
    const put = (col: number | null, field: Field) => {
      if (col !== null) assign[col] = field;
    };
    put(g.date, "date");
    put(g.narration, "narration");
    put(g.debit, "debit");
    put(g.credit, "credit");
    put(g.amount, "amount");
    put(g.balance, "balance");
    put(g.reference, "reference");
    if (g.sign.kind === "dr_cr_column") put(g.sign.column, "flag");
  } else {
    header.forEach((name, j) => {
      assign[j] = guessField(name);
    });
  }
  // A field takes one column: the first column that names it keeps it.
  const seen = new Set<Field>();
  for (const j of Object.keys(assign).map(Number).sort((a, b) => a - b)) {
    const field = assign[j];
    if (!SINGLE.includes(field)) continue;
    if (seen.has(field)) assign[j] = "ignore";
    else seen.add(field);
  }
  return assign;
}

/** The mapping the picked columns describe, or the reason it can't be used yet. */
function buildMapping(
  headerRow: number,
  assign: Record<number, Field>,
  amountSign: "negative_is_debit" | "cr_suffix",
  dateOrder: StatementMapping["date_order"],
): { mapping: StatementMapping } | { problem: string } {
  const date = columnOf(assign, "date");
  const narration = columnOf(assign, "narration");
  const debit = columnOf(assign, "debit");
  const credit = columnOf(assign, "credit");
  const amount = columnOf(assign, "amount");
  const flag = columnOf(assign, "flag");
  const balance = columnOf(assign, "balance");
  const reference = columnOf(assign, "reference");
  if (date === null) return { problem: "Choose the column with the dates." };
  if (narration === null) return { problem: "Choose the column with the descriptions." };
  if (debit === null && credit === null && amount === null) return { problem: "Choose a Debit or Credit column, or an Amount column." };

  let sign: SignRule;
  if (debit !== null || credit !== null) sign = { kind: "separate_columns" };
  else if (flag !== null) sign = { kind: "dr_cr_column", column: flag };
  else sign = { kind: amountSign };

  return {
    mapping: {
      header_row: headerRow,
      date,
      narration,
      debit,
      credit,
      amount,
      sign,
      balance,
      reference,
      date_order: dateOrder,
    },
  };
}

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

/** Lets the user say what each column of a statement holds, against the file's own first rows. */
function ColumnMapper({
  preview,
  busy,
  error,
  onImport,
  onCancel,
}: {
  preview: StatementPreview;
  busy: boolean;
  error: string | null;
  onImport: (mapping: StatementMapping, options: { save_profile: boolean; label: string }) => void;
  onCancel: () => void;
}) {
  const guessed = preview.guessed;
  const [assign, setAssign] = useState(() => initialAssignments(preview));
  const [amountSign, setAmountSign] = useState<"negative_is_debit" | "cr_suffix">(
    guessed?.sign.kind === "cr_suffix" ? "cr_suffix" : "negative_is_debit",
  );
  const [dateOrder, setDateOrder] = useState<StatementMapping["date_order"]>(guessed?.date_order ?? "dmy");
  const [label, setLabel] = useState("");
  const [remember, setRemember] = useState(true);
  const [problem, setProblem] = useState<string | null>(null);

  const cols = columnCount(preview);
  // A single amount column with no Dr/Cr flag and no debit or credit column: its sign needs saying.
  const amountOnly =
    columnOf(assign, "amount") !== null &&
    columnOf(assign, "flag") === null &&
    columnOf(assign, "debit") === null &&
    columnOf(assign, "credit") === null;

  const setField = (col: number, field: Field) =>
    setAssign((prev) => {
      const next = { ...prev };
      if (SINGLE.includes(field)) {
        for (const k of Object.keys(next)) {
          if (Number(k) !== col && next[Number(k)] === field) next[Number(k)] = "ignore";
        }
      }
      next[col] = field;
      return next;
    });

  const submit = () => {
    const built = buildMapping(preview.header_row, assign, amountSign, dateOrder);
    if ("problem" in built) return setProblem(built.problem);
    setProblem(null);
    onImport(built.mapping, { save_profile: remember, label: label.trim() });
  };

  return (
    <section aria-labelledby="map-heading" className="space-y-4 rounded-[18px] border border-border p-4">
      <div className="field">
        <h3 id="map-heading" className="text-base font-medium">
          Map the columns
        </h3>
        <p className="max-w-prose text-sm leading-snug text-muted-foreground">
          Choose what each column holds. Nothing is imported until you press Import. The first row is the header.
        </p>
      </div>

      <div className="overflow-x-auto rounded-lg border border-border">
        <table className="min-w-full text-sm">
          <thead>
            <tr>
              {Array.from({ length: cols }, (_, j) => (
                <th key={j} scope="col" className="p-2 text-left align-top font-normal">
                  <Select
                    aria-label={`Column ${j + 1}`}
                    value={assign[j] ?? "ignore"}
                    onChange={(e) => setField(j, e.target.value as Field)}
                    className="h-9 min-w-[9rem]"
                  >
                    {FIELDS.map((f) => (
                      <option key={f.value} value={f.value}>
                        {f.label}
                      </option>
                    ))}
                  </Select>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {preview.rows.map((row, i) => (
              <tr key={i} className={i === 0 ? "font-medium" : "text-muted-foreground"}>
                {Array.from({ length: cols }, (_, j) => (
                  <td key={j} className="whitespace-nowrap border-t border-border px-2 py-1">
                    {row[j] ?? ""}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        <div className="field">
          <Label htmlFor="map-date-order">Dates are written</Label>
          <Select id="map-date-order" value={dateOrder} onChange={(e) => setDateOrder(e.target.value as StatementMapping["date_order"])}>
            <option value="dmy">Day first (05/11/2026)</option>
            <option value="mdy">Month first (11/05/2026)</option>
            <option value="ymd">Year first (2026-11-05)</option>
          </Select>
        </div>
        {amountOnly && (
          <div className="field">
            <Label htmlFor="map-sign">The amounts</Label>
            <Select id="map-sign" value={amountSign} onChange={(e) => setAmountSign(e.target.value as "negative_is_debit" | "cr_suffix")}>
              <option value="negative_is_debit">A minus sign is money out</option>
              <option value="cr_suffix">Plain figures are spending, Cr marks payments</option>
            </Select>
          </div>
        )}
        <div className="field">
          <Label htmlFor="map-label">Name this layout</Label>
          <Input id="map-label" value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Kotak savings" maxLength={60} />
        </div>
      </div>

      <label className="flex items-center gap-2 text-sm">
        <input type="checkbox" checked={remember} onChange={(e) => setRemember(e.target.checked)} className="h-4 w-4 accent-primary" />
        Remember this layout
      </label>

      {(problem ?? error) && (
        <p role="alert" className="text-sm text-destructive">
          {problem ?? error}
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button onClick={submit} disabled={busy}>
          {busy ? "Importing..." : "Import"}
        </Button>
        <Button variant="outline" onClick={onCancel} disabled={busy}>
          Cancel
        </Button>
      </div>
    </section>
  );
}

/** Upload a bank statement (.txt or .csv); the server reads and stores every row, then forgets the file. */
export function StatementImport() {
  const { data: accounts } = useAccounts();
  const names = useQuery({ queryKey: HOLDER_NAMES_KEY, queryFn: ingestApi.holderNames, retry: 1 });
  const cards = useQuery({
    queryKey: CARDS_KEY,
    queryFn: async () => {
      const decoded = await decodeCardList(await api.getProtobuf("/api/card/get_all_cards"));
      return decoded.cards.map((c) => ({ card_id: c.card_id, card_name: c.card_name, last_4_digits: c.last_4_digits }));
    },
    retry: 1,
  });
  const saved = names.data?.names ?? [];

  const [target, setTarget] = useState("");
  const [result, setResult] = useState<StatementResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const [draft, setDraft] = useState("");
  const [nameError, setNameError] = useState<string | null>(null);
  const [mapper, setMapper] = useState<{ file: File; preview: StatementPreview } | null>(null);
  // Bumped each time a mapper opens, so its choices start over for the file it now shows.
  const [mapperKey, setMapperKey] = useState(0);
  const [mapError, setMapError] = useState<string | null>(null);
  const [previewing, setPreviewing] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const mapInput = useRef<HTMLInputElement>(null);

  const showMapper = (next: { file: File; preview: StatementPreview }) => {
    setMapError(null);
    setMapperKey((k) => k + 1);
    setMapper(next);
  };

  // A file the layout of which can't be read comes back as a value, not an error, so no toast shows for it.
  const upload = usePlannerMutation(
    async ({ file, target: destination }: { file: File; target: StatementTarget }): Promise<{ result: StatementResult } | { needs: StatementNeedsMapping }> => {
      try {
        return { result: await ingestApi.uploadStatement(file, destination) };
      } catch (e) {
        const needs = needsMapping(e);
        if (needs) return { needs };
        throw e;
      }
    },
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
    setMapper(null);
    setMapError(null);
    const problem = fileProblem(file);
    if (problem) return setError(problem);
    upload.mutate(
      { file, target: targetOf(target) },
      {
        onSuccess: (out) => {
          if ("needs" in out) return showMapper({ file, preview: out.needs.preview });
          setResult(out.result);
        },
        onError: (e) => setError(e instanceof Error ? e.message : "Couldn't import that file."),
      },
    );
    if (input.current) input.current.value = "";
  };

  // Mapping first: the file is previewed, not imported, until the user presses Import in the mapper.
  const openMapper = async (file: File | undefined) => {
    if (!file) return;
    setResult(null);
    setError(null);
    setMapper(null);
    setMapError(null);
    const problem = fileProblem(file);
    if (problem) return setError(problem);
    setPreviewing(true);
    try {
      const { preview } = await ingestApi.previewStatement(file, { card_id: targetOf(target).card_id });
      showMapper({ file, preview });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Couldn't read that file.");
    } finally {
      setPreviewing(false);
      if (mapInput.current) mapInput.current.value = "";
    }
  };

  const importMapped = (mapping: StatementMapping, options: { save_profile: boolean; label: string }) => {
    if (!mapper) return;
    setMapError(null);
    upload.mutate(
      { file: mapper.file, target: { ...targetOf(target), mapping, save_profile: options.save_profile, label: options.label } },
      {
        onSuccess: (out) => {
          if ("needs" in out) return setMapError("These columns still don't read the file. Check the choices above.");
          setResult(out.result);
          setMapper(null);
        },
        onError: (e) => setMapError(e instanceof Error ? e.message : "Couldn't import that file."),
      },
    );
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
  const cardList = cards.data ?? [];
  const suggestion = result?.holder_name_suggestion && saved.length === 0 ? result.holder_name_suggestion : null;

  return (
    <div className="space-y-4">
      <div className="field">
        <Label htmlFor="statement-account">Which account or card is this statement for?</Label>
        <Select id="statement-account" value={target} onChange={(e) => setTarget(e.target.value)}>
          <option value="">Not sure</option>
          {banks.length > 0 && (
            <optgroup label="Bank accounts">
              {banks.map((a) => (
                <option key={a.account_id} value={`acct:${a.account_id}`}>
                  {a.name}
                  {a.last_4_digits ? ` (${a.last_4_digits})` : ""}
                </option>
              ))}
            </optgroup>
          )}
          {cardList.length > 0 && (
            <optgroup label="Credit cards">
              {cardList.map((c) => (
                <option key={c.card_id} value={`card:${c.card_id}`}>
                  {c.card_name}
                  {c.last_4_digits ? ` (${c.last_4_digits})` : ""}
                </option>
              ))}
            </optgroup>
          )}
        </Select>
        <p className="max-w-prose text-sm leading-snug text-muted-foreground">
          Picking an account also sets its balance from the statement's newest line, unless you set a newer one. Picking a card
          counts purchases as spending and treats payments and refunds as money coming back, never as income.
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
      <input
        ref={mapInput}
        type="file"
        accept=".txt,.csv,text/plain,text/csv"
        aria-label="Statement file to map"
        className="hidden"
        disabled={previewing}
        onChange={(e) => openMapper(e.target.files?.[0])}
      />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-prose text-sm leading-snug text-muted-foreground">
          Your file is read in memory and discarded straight after import. Only the transactions are kept.
        </p>
        <Button variant="outline" size="sm" onClick={() => mapInput.current?.click()} disabled={upload.isPending || previewing}>
          {previewing ? "Reading..." : "Map columns"}
        </Button>
      </div>

      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      {result && (
        <Outcome r={result} suggestion={suggestion} onConfirmSuggestion={confirmSuggestion} confirming={confirmSelf.isPending} />
      )}

      {mapper && (
        <ColumnMapper
          key={mapperKey}
          preview={mapper.preview}
          busy={upload.isPending}
          error={mapError}
          onImport={importMapped}
          onCancel={() => {
            setMapper(null);
            setMapError(null);
          }}
        />
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
