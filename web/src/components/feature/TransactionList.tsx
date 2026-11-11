import { useState } from "react";
import { toast } from "sonner";
import { ChevronLeft, ChevronRight, Layers, Trash2 } from "lucide-react";

import { ingestApi } from "@/services/api";
import type { Txn } from "@/services/types";
import { INGEST_INVALIDATE, useTransactions, usePlannerMutation } from "@/hooks/usePlanner";
import { ConfirmationModal } from "@/components/ui/ConfirmationModal";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import {
  CATEGORY_IDS,
  categoryLabel,
  monthKey,
  monthTitle,
  plural,
  shiftMonth,
  signedAmount,
  sourceDetail,
  sourceLabel,
} from "@/utils/spending";
import { cn } from "@/utils/cn";
import { SwipeSorter } from "@/components/feature/SwipeSorter";
import { useAnimatedList } from "@/hooks/useAnimatedList";

interface Draft {
  category: string;
  remember: boolean;
}

function TransactionRow({
  txn,
  saving,
  onSave,
  onDelete,
}: {
  txn: Txn;
  saving: boolean;
  onSave: (category: string, remember: boolean) => void;
  onDelete: () => void;
}) {
  const [draft, setDraft] = useState<Draft | null>(null);
  const name = txn.merchant ?? "Unknown";
  const where = sourceDetail(txn.source_name, txn.last4) ?? sourceLabel(txn.source);
  const credit = txn.direction === "credit";

  const pick = (category: string) => {
    if (category === txn.category) return setDraft(null);
    setDraft({ category, remember: txn.merchant !== null });
  };

  return (
    <li className="space-y-3 border-t border-border py-4 first:border-t-0 first:pt-1">
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="truncate text-sm font-medium">{name}</p>
          <p className="truncate text-xs text-muted-foreground">
            {formatDay(txn.occurred_on)} · {where}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <p
            className={cn(
              "text-sm font-semibold tabular-nums",
              credit ? "text-good" : "text-foreground",
            )}
          >
            {signedAmount(txn)}
          </p>
          <Button variant="ghost" size="icon" className="h-9 w-9" aria-label={`Delete ${name}`} onClick={onDelete}>
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <Select
        aria-label={`Category for ${name}`}
        className="h-9 text-sm"
        value={draft?.category ?? txn.category}
        onChange={(e) => pick(e.target.value)}
        disabled={saving}
      >
        {CATEGORY_IDS.map((id) => (
          <option key={id} value={id}>
            {categoryLabel(id)}
          </option>
        ))}
      </Select>

      {draft && (
        <div className="flex flex-wrap items-center gap-2 rounded-lg bg-muted/50 p-2 text-sm">
          {txn.merchant ? (
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                className="h-4 w-4"
                checked={draft.remember}
                onChange={(e) => setDraft({ ...draft, remember: e.target.checked })}
              />
              Remember for {txn.merchant}
            </label>
          ) : (
            <span className="text-muted-foreground">Only this transaction</span>
          )}
          <div className="ml-auto flex gap-2">
            <Button size="sm" variant="ghost" onClick={() => setDraft(null)} disabled={saving}>
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={() => onSave(draft.category, txn.merchant ? draft.remember : false)}
              disabled={saving}
            >
              {saving ? "Saving..." : "Save"}
            </Button>
          </div>
        </div>
      )}
    </li>
  );
}

/** The month's transactions, newest first, with category editing, a filter and delete. */
export function TransactionList({
  uncategorizedOnly,
  onUncategorizedOnlyChange,
}: {
  uncategorizedOnly: boolean;
  onUncategorizedOnlyChange: (on: boolean) => void;
}) {
  const listRef = useAnimatedList<HTMLUListElement>();
  const [thisMonth] = useState(() => monthKey(new Date()));
  const [month, setMonth] = useState(thisMonth);
  const { data: txns, isLoading, isError } = useTransactions(month);
  const [toDelete, setToDelete] = useState<Txn | null>(null);
  const [sorting, setSorting] = useState(false);

  const update = usePlannerMutation(ingestApi.updateCategory, INGEST_INVALIDATE);
  const remove = usePlannerMutation(ingestApi.removeTxn, INGEST_INVALIDATE, { success: "Transaction deleted" });

  const save = (txn: Txn, category: string, remember: boolean) => {
    update.mutate(
      { txn_id: txn.txn_id, category, merchant: txn.merchant, remember },
      {
        onSuccess: (res) => {
          if (res.recategorized > 0) {
            toast.success(`Also updated ${plural(res.recategorized, "other", "others")}`);
          } else {
            toast.success("Category updated");
          }
        },
      },
    );
  };

  // The swipe sorter saves quietly (a toast per swipe would pile up); a failure brings the card back.
  const sortOne = (txn: Txn, category: string, remember: boolean, undo: () => void) => {
    update.mutate(
      { txn_id: txn.txn_id, category, merchant: txn.merchant, remember },
      { onError: () => undo() },
    );
  };
  const unsorted = (txns ?? []).filter((t) => t.category === "uncategorized");

  const shown = (txns ?? []).filter((t) => !uncategorizedOnly || t.category === "uncategorized");
  const isCurrent = month === thisMonth;

  return (
    <section id="transactions" aria-label="Transactions" className="scroll-mt-20 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-display text-[32px] leading-none">Transactions</h2>
        <div className="flex items-center gap-1">
          <Button variant="outline" size="icon" className="h-9 w-9" aria-label="Previous month" onClick={() => setMonth(shiftMonth(month, -1))}>
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="min-w-36 text-center text-sm font-medium tabular-nums">{monthTitle(month)}</span>
          <Button
            variant="outline"
            size="icon"
            className="h-9 w-9"
            aria-label="Next month"
            disabled={isCurrent}
            onClick={() => setMonth(shiftMonth(month, 1))}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div>
        <Button
          size="sm"
          variant={uncategorizedOnly ? "default" : "outline"}
          aria-pressed={uncategorizedOnly}
          onClick={() => onUncategorizedOnlyChange(!uncategorizedOnly)}
        >
          Uncategorized only
        </Button>
        {unsorted.length > 0 && !sorting && (
          <Button size="sm" variant="outline" className="ml-2 gap-2" onClick={() => setSorting(true)}>
            <Layers className="h-4 w-4" aria-hidden /> Sort {unsorted.length} one by one
          </Button>
        )}
      </div>

      {sorting && txns && <SwipeSorter queue={unsorted} all={txns} onCategorise={sortOne} onClose={() => setSorting(false)} />}

      {isLoading && <div className="h-24 animate-pulse rounded-xl bg-muted/40" />}
      {isError && <p className="text-sm text-destructive">Couldn't load transactions.</p>}

      {txns && (
        shown.length === 0 ? (
          <p className="empty-note">
            {uncategorizedOnly
              ? "Nothing to categorize here."
              : `No transactions in ${monthTitle(month)}. Alerts from your bank show up here automatically.`}
          </p>
        ) : (
          <ul className="ledger" ref={listRef}>
            {shown.map((t) => (
              <TransactionRow
                key={t.txn_id}
                txn={t}
                saving={update.isPending && update.variables?.txn_id === t.txn_id}
                onSave={(category, remember) => save(t, category, remember)}
                onDelete={() => setToDelete(t)}
              />
            ))}
          </ul>
        )
      )}

      <ConfirmationModal
        isOpen={toDelete !== null}
        onClose={() => setToDelete(null)}
        onConfirm={() => {
          if (!toDelete) return;
          remove.mutate(toDelete.txn_id, { onSettled: () => setToDelete(null) });
        }}
        title="Delete transaction?"
        description={
          toDelete
            ? `Remove ${formatINR(toDelete.amount_paise)} ${toDelete.merchant ? `at ${toDelete.merchant}` : "transaction"} from ${formatDay(toDelete.occurred_on)} from your spending history.`
            : ""
        }
        confirmText="Delete"
        variant="destructive"
        isPending={remove.isPending}
      />
    </section>
  );
}
