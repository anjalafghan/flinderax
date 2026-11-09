import { useState } from "react";
import { Link } from "react-router-dom";
import { toast } from "sonner";

import { Header } from "@/components/layout/Header";
import { type PayeeRow, useCategorizePayee, usePayees } from "@/services/payees";
import { formatINR } from "@/utils/money";
import { CATEGORY_IDS, CATEGORY_LABEL, categoryLabel, monthShort, plural } from "@/utils/spending";

// The categories one tap can reach. "Friends & family" has no category id on the server yet, so it
// is left to the "More…" list rather than sending a made-up id.
const PRESETS: { id: string; label: string }[] = [
  { id: "food", label: "Food" },
  { id: "groceries", label: "Groceries" },
  { id: "people", label: "Friends & family" },
  { id: "shopping", label: "Shopping" },
  { id: "transport", label: "Transport" },
  { id: "bills", label: "Bills" },
  { id: "health", label: "Health" },
  { id: "rent", label: "Rent" },
  { id: "transfer", label: "Transfer (own money)" },
  { id: "other", label: "Other" },
];

const presetLabel = (id: string) => PRESETS.find((p) => p.id === id)?.label ?? categoryLabel(id);

/** "SWIGGY INDIA" -> "Swiggy India". */
const titleCase = (s: string) =>
  s
    .toLowerCase()
    .split(" ")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");

/** "2026-11-08" -> "8 Nov 2026". Parsed from the string so the viewer's timezone can't shift it. */
const shortDate = (iso: string) => {
  const [y, m, d] = iso.split("-").map(Number);
  return `${d} ${monthShort(`${y}-${String(m).padStart(2, "0")}`)} ${y}`;
};

const chipClass =
  "min-h-[36px] rounded-full border border-border bg-chip px-3.5 text-[13px] font-medium transition-colors hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

function PayeeItem({
  row,
  busy,
  onTag,
}: {
  row: PayeeRow;
  busy: boolean;
  onTag: (row: PayeeRow, category: string) => void;
}) {
  const name = titleCase(row.payee);
  // The suggestion comes first and is highlighted; the presets follow, minus any duplicate of it.
  const suggestion = row.suggestion && row.suggestion !== "uncategorized" ? row.suggestion : null;
  const rest = PRESETS.filter((p) => p.id !== suggestion);

  return (
    <li className="ledger-row flex-col items-stretch gap-3">
      <div className="min-w-0">
        <p className="truncate font-medium">{name}</p>
        <p className="text-xs text-muted-foreground">
          {`${plural(row.count, "payment", "payments")} · ${formatINR(row.paise)} · last ${shortDate(row.last_on)}`}
        </p>
        <p className="mt-1 truncate font-mono text-[12px] text-muted-foreground" title={row.sample}>
          {row.sample}
        </p>
      </div>

      <div role="group" aria-label={`Category for ${name}`} className="flex flex-wrap items-center gap-2">
        {suggestion && (
          <button
            type="button"
            disabled={busy}
            aria-label={`Tag ${name} as ${presetLabel(suggestion)}`}
            onClick={() => onTag(row, suggestion)}
            className={`${chipClass} tone-good border-[hsl(var(--good))]`}
          >
            {presetLabel(suggestion)}
          </button>
        )}
        {rest.map((p) => (
          <button
            key={p.id}
            type="button"
            disabled={busy}
            aria-label={`Tag ${name} as ${p.label}`}
            onClick={() => onTag(row, p.id)}
            className={`${chipClass} text-foreground`}
          >
            {p.label}
          </button>
        ))}
        <select
          aria-label={`More categories for ${name}`}
          disabled={busy}
          value=""
          onChange={(e) => {
            if (e.target.value) onTag(row, e.target.value);
          }}
          className={`${chipClass} text-foreground`}
        >
          <option value="" disabled>
            More…
          </option>
          {CATEGORY_IDS.filter((id) => id !== "uncategorized").map((id) => (
            <option key={id} value={id}>
              {CATEGORY_LABEL[id]}
            </option>
          ))}
        </select>
      </div>
    </li>
  );
}

export default function SortPayeesPage() {
  const { data, isLoading, isError } = usePayees();
  const categorize = useCategorizePayee();
  // Tagged this session: their rows leave the list, and their money counts toward the progress.
  const [tagged, setTagged] = useState<ReadonlySet<string>>(() => new Set());
  const [taggedPaise, setTaggedPaise] = useState(0);

  const tag = (row: PayeeRow, category: string) => {
    categorize.mutate(
      { payee: row.payee, category, remember: true },
      {
        onSuccess: () => {
          setTagged((prev) => new Set(prev).add(row.payee));
          setTaggedPaise((p) => p + row.paise);
          toast.success(`Tagged ${plural(row.count, "payment", "payments")} as ${presetLabel(category)}`);
        },
      },
    );
  };

  const total = data?.total_uncategorized_paise ?? 0;
  const pct = total > 0 ? Math.min(100, Math.round((taggedPaise * 100) / total)) : 100;
  const untagged = Math.max(0, total - taggedPaise);
  const rows = (data?.payees ?? []).filter((r) => !tagged.has(r.payee));
  const pendingPayee = categorize.isPending ? categorize.variables?.payee : undefined;

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl">
        <div className="mb-6 md:mb-8">
          <h1 className="page-title">Sort your payees</h1>
          <p className="page-sub">Tag each payee once. joy remembers it for past and future payments.</p>
        </div>

        {isLoading ? (
          <div aria-label="Loading payees" className="h-48 animate-pulse rounded-[28px] bg-muted/40" />
        ) : isError || !data ? (
          <p className="text-sm text-destructive">Couldn't load your payees.</p>
        ) : (
          <div className="stack">
            <section aria-label="Progress" className="glass space-y-4 rounded-[28px] p-5 md:p-6">
              <div>
                <p className="text-sm text-muted-foreground">Tagged this session</p>
                <p className="font-display text-5xl tabular-nums md:text-6xl">{pct}%</p>
              </div>
              <div
                role="progressbar"
                aria-label="Uncategorised money tagged this session"
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={pct}
                className="h-1.5 w-full overflow-hidden rounded-full bg-chip"
              >
                <div className="h-full rounded-full bg-primary transition-[width]" style={{ width: `${pct}%` }} />
              </div>
              <p className="text-sm text-muted-foreground">{formatINR(untagged)} still untagged</p>
            </section>

            {rows.length === 0 ? (
              <div className="empty-note space-y-2">
                <p>Everything is tagged. Nice.</p>
                <Link to="/money" className="font-medium text-foreground underline underline-offset-4">
                  Back to Money
                </Link>
              </div>
            ) : (
              <ol className="ledger" aria-label="Payees to tag">
                {rows.map((row) => (
                  <PayeeItem
                    key={row.payee}
                    row={row}
                    busy={pendingPayee === row.payee}
                    onTag={tag}
                  />
                ))}
              </ol>
            )}
          </div>
        )}
      </main>
    </div>
  );
}
