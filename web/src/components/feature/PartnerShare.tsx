import { useState } from "react";
import { Link } from "react-router-dom";
import { toast } from "sonner";

import { useJoySetPartner, useJoySummary } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { paiseToInput, toPaise } from "@/utils/money";

/** Parses the partner amount field. Zero is allowed; blank or negative is not. */
function parsePartner(text: string): number | null {
  const paise = toPaise(text);
  return paise === null || paise < 0 ? null : paise;
}

function ShareForm({ saved }: { saved: number }) {
  const [partner, setPartner] = useState(() => paiseToInput(saved));
  const mutation = useJoySetPartner();
  const parsed = parsePartner(partner);
  const unchanged = parsed === saved;

  return (
    <form
      className="form-grid"
      onSubmit={(e) => {
        e.preventDefault();
        if (parsed === null) return toast.error("Enter what your partner covers each month (0 or more).");
        if (!unchanged) mutation.mutate(parsed);
      }}
    >
      <div className="field">
        <Label htmlFor="partner-covers">Partner covers each month</Label>
        <MoneyInput id="partner-covers" value={partner} onChange={(e) => setPartner(e.target.value)} />
      </div>
      <div className="form-actions">
        <Button type="submit" disabled={unchanged || mutation.isPending}>
          {mutation.isPending ? "Saving..." : "Save"}
        </Button>
      </div>
    </form>
  );
}

/** What your partner covers of the shared date budget. The budget itself lives in Dates. */
export function PartnerShare() {
  const { data: summary, isError, isLoading } = useJoySummary();

  return (
    <div className="glass space-y-4 rounded-3xl p-4 text-sm md:p-6">
      <div>
        <h2 className="font-display text-[32px] leading-none tracking-tight">Date budget share</h2>
        <p className="text-muted-foreground">
          {isError
            ? "Couldn't load your date budget."
            : isLoading
              ? "Checking..."
              : summary?.connected
                ? "Your share of the shared date budget shows on Today."
                : "Start Dates, or join your partner's, to share a date budget."}
        </p>
      </div>
      {summary?.connected ? (
        <ShareForm key={summary.partner_paise} saved={summary.partner_paise} />
      ) : (
        !isLoading &&
        !isError && (
          <Link to="/joy" className="font-medium text-primary">
            Open Dates
          </Link>
        )
      )}
    </div>
  );
}
