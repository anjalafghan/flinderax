import { useRef, useState } from "react";

import { ingestApi } from "@/services/api";
import type { StatementResult } from "@/services/types";
import { INGEST_INVALIDATE, useAccounts, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";

const MAX_BYTES = 1_500_000;

function Outcome({ r }: { r: StatementResult }) {
  const parts = [`${r.added} added`];
  if (r.duplicates) parts.push(`${r.duplicates} already imported`);
  if (r.matched_alerts) parts.push(`${r.matched_alerts} matched bank alerts`);
  if (r.skipped) parts.push(`${r.skipped} unreadable`);
  return (
    <div role="status" className="space-y-1 rounded-lg bg-muted/50 p-3 text-sm">
      <p className="font-medium">
        {parts.join(", ")}
        {r.first_date && r.last_date ? ` (${formatDay(r.first_date)} to ${formatDay(r.last_date)})` : ""}.
      </p>
      {r.uncategorized > 0 && (
        <p className="text-muted-foreground">{r.uncategorized} need a category. Fix them in the list above and tick Remember.</p>
      )}
      {r.balance_updated && (
        <p className="text-muted-foreground">
          {r.balance_updated.account_name} balance set to {formatINR(r.balance_updated.balance_paise)}
        </p>
      )}
    </div>
  );
}

/** Upload a bank statement CSV; the server reads and stores every row. */
export function StatementImport() {
  const { data: accounts } = useAccounts();
  const [accountId, setAccountId] = useState("");
  const [result, setResult] = useState<StatementResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);
  const upload = usePlannerMutation(ingestApi.importStatement, [...INGEST_INVALIDATE, ["accounts"]]);

  const onFile = async (file: File | undefined) => {
    if (!file) return;
    setResult(null);
    setError(null);
    if (file.size > MAX_BYTES) return setError("That file is too large. Export one period at a time.");
    if (/\.(xlsx?|pdf)$/i.test(file.name)) {
      return setError("Only CSV files for now. In Excel use Save As > CSV, or pick CSV on your bank's download page.");
    }
    const csv = await file.text();
    upload.mutate(
      { csv, account_id: accountId || undefined },
      {
        onSuccess: setResult,
        onError: (e) => setError(e instanceof Error ? e.message : "Couldn't import that file."),
      },
    );
    if (input.current) input.current.value = "";
  };

  const banks = (accounts ?? []).filter((a) => a.kind === "bank");

  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
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
        <p className="text-xs text-muted-foreground">Picking an account also sets its balance from the statement's newest line, unless you set a newer one.</p>
      </div>
      <input
        ref={input}
        type="file"
        accept=".csv,.txt,text/csv"
        aria-label="Statement CSV file"
        className="hidden"
        onChange={(e) => void onFile(e.target.files?.[0])}
      />
      <Button onClick={() => input.current?.click()} disabled={upload.isPending}>
        {upload.isPending ? "Importing..." : "Choose CSV file"}
      </Button>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      {result && <Outcome r={result} />}
    </div>
  );
}
