import { useState } from "react";

import { ingestApi } from "@/services/api";
import type { ParseResult } from "@/services/types";
import { INGEST_INVALIDATE, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { categoryLabel, signedAmount } from "@/utils/spending";

function Outcome({ result }: { result: ParseResult }) {
  const t = result.transaction;
  let message: string;
  if (result.status === "created" && t) {
    message = `Added: ${t.merchant ?? "Unknown"}, ${signedAmount(t)} on ${formatDay(t.occurred_on)}, ${categoryLabel(t.category)}.`;
  } else if (result.status === "duplicate") {
    message = "Already recorded.";
  } else if (result.status === "created") {
    message = "Added.";
  } else {
    message =
      "Couldn't read that message. Paste the full text of a bank alert.";
  }
  const balance = result.balance_updated;
  return (
    <div
      role="status"
      className={
        result.status === "unrecognized"
          ? "space-y-1 rounded-lg border border-over/30 bg-over-tint p-3 text-sm"
          : "space-y-1 rounded-lg border border-good/30 bg-good-tint p-3 text-sm"
      }
    >
      <p
        className={
          result.status === "unrecognized" ? "text-destructive" : "font-medium"
        }
      >
        {message}
      </p>
      {balance && (
        <p className="text-muted-foreground">
          {balance.account_name} balance set to{" "}
          {formatINR(balance.balance_paise)}
        </p>
      )}
    </div>
  );
}

/** Paste a bank alert (SMS or email text) and let the server read it. */
export function PasteAlert() {
  const [text, setText] = useState("");
  const [result, setResult] = useState<ParseResult | null>(null);
  const parse = usePlannerMutation(ingestApi.parse, INGEST_INVALIDATE);

  const submit = () => {
    const body = text.trim();
    if (!body) return;
    parse.mutate(body, {
      onSuccess: (res) => {
        setResult(res);
        if (res.status === "created") setText("");
      },
    });
  };

  return (
    <div className="space-y-3">
      <textarea
        aria-label="Bank alert text"
        value={text}
        onChange={(e) => setText(e.target.value)}
        rows={4}
        placeholder="Paste a bank alert SMS or email here"
        className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm transition-[border-color,box-shadow] placeholder:text-muted-foreground hover:border-brand/40 focus-visible:border-brand focus-visible:shadow-[0_0_0_4px_hsl(var(--brand)/0.15)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      />
      <div className="flex flex-wrap items-center justify-end gap-2">
        <ResetButton
          label="Clear"
          disabled={!text && !result}
          onReset={() => {
            setText("");
            setResult(null);
          }}
        />
        <Button onClick={submit} disabled={parse.isPending || !text.trim()}>
          {parse.isPending ? "Reading..." : "Add alert"}
        </Button>
      </div>
      {result && <Outcome result={result} />}
    </div>
  );
}
