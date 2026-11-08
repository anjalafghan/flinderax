import { useState } from "react";
import { toast } from "sonner";

import { useJoyConnect, useJoyDisconnect, useJoySetPartner, useJoySummary } from "@/hooks/usePlanner";
import type { JoySummary } from "@/services/types";
import { ConfirmationModal } from "@/components/ui/ConfirmationModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { paiseToInput, toPaise } from "@/utils/money";
import { syncedLabel } from "@/utils/joy";

/** Parses the partner amount field. Zero is allowed; blank or negative is not. */
function parsePartner(text: string): number | null {
  const paise = toPaise(text);
  return paise === null || paise < 0 ? null : paise;
}

function ConnectForm() {
  const [url, setUrl] = useState("");
  const [token, setToken] = useState("");
  const [partner, setPartner] = useState("");
  const [error, setError] = useState<string | null>(null);
  const connect = useJoyConnect();

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const partnerPaise = parsePartner(partner);
    if (!url.trim()) return setError("Enter the address of your Joy app.");
    if (!token.trim()) return setError("Enter the API token from Joy.");
    if (partnerPaise === null) return setError("Enter what your partner covers each month (0 or more).");
    setError(null);
    connect.mutate({ url: url.trim(), token: token.trim(), partner_paise: partnerPaise });
  };

  return (
    <div className="space-y-4">
      <p className="text-muted-foreground">Link Joy so your date budget and your share show here; Joy never sees your finances.</p>
      <form onSubmit={submit} className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1 sm:col-span-2">
          <Label htmlFor="joy-url">Joy address</Label>
          <Input
            id="joy-url"
            type="url"
            inputMode="url"
            autoComplete="off"
            placeholder="https://joy-jar.fly.dev"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>
        <div className="space-y-1 sm:col-span-2">
          <Label htmlFor="joy-token">API token</Label>
          <Input
            id="joy-token"
            type="password"
            autoComplete="off"
            value={token}
            onChange={(e) => setToken(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="joy-partner">Partner covers each month</Label>
          <MoneyInput id="joy-partner" value={partner} onChange={(e) => setPartner(e.target.value)} />
        </div>
        {error && (
          <p role="alert" className="text-sm text-destructive sm:col-span-2">
            {error}
          </p>
        )}
        <div className="sm:col-span-2">
          <Button type="submit" disabled={connect.isPending}>
            {connect.isPending ? "Connecting..." : "Connect"}
          </Button>
        </div>
      </form>
    </div>
  );
}

function ConnectedView({ summary }: { summary: JoySummary }) {
  const [partner, setPartner] = useState(() => paiseToInput(summary.partner_paise));
  const [confirmDisconnect, setConfirmDisconnect] = useState(false);
  const setPartnerMutation = useJoySetPartner();
  const disconnect = useJoyDisconnect();

  const parsed = parsePartner(partner);
  const unchanged = parsed === summary.partner_paise;

  const save = () => {
    if (parsed === null) return toast.error("Enter what your partner covers each month (0 or more).");
    setPartnerMutation.mutate(parsed);
  };

  return (
    <div className="space-y-5">
      <dl className="grid gap-3 text-sm sm:grid-cols-[auto_1fr] sm:gap-x-6">
        <dt className="text-muted-foreground">Joy address</dt>
        <dd className="min-w-0 break-all font-mono text-xs sm:text-sm">{summary.url}</dd>
        <dt className="text-muted-foreground">Last sync</dt>
        <dd>
          {summary.error ? (
            <span className="text-destructive">Last sync failed: {summary.error}</span>
          ) : summary.synced_at ? (
            <>Synced {syncedLabel(summary.synced_at)}</>
          ) : (
            "Not synced yet"
          )}
        </dd>
      </dl>

      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          if (!unchanged) save();
        }}
      >
        <div className="space-y-1 sm:max-w-xs">
          <Label htmlFor="joy-partner-edit">Partner covers each month</Label>
          <MoneyInput id="joy-partner-edit" value={partner} onChange={(e) => setPartner(e.target.value)} />
        </div>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <Button type="submit" size="sm" disabled={unchanged || setPartnerMutation.isPending}>
            {setPartnerMutation.isPending ? "Saving..." : "Save"}
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={() => setConfirmDisconnect(true)}>
            Disconnect
          </Button>
        </div>
      </form>

      <ConfirmationModal
        isOpen={confirmDisconnect}
        onClose={() => setConfirmDisconnect(false)}
        onConfirm={() =>
          disconnect.mutate(undefined, { onSuccess: () => setConfirmDisconnect(false) })
        }
        title="Disconnect Joy?"
        description="The date budget and your share will stop showing here. Your Joy data is not deleted, and you can connect again at any time."
        confirmText="Disconnect"
        variant="destructive"
        isPending={disconnect.isPending}
      />
    </div>
  );
}

/** Link to the shared date-jar app. Collapsed by default; hidden details appear once linked. */
export function JoyConnect() {
  const { data: summary, isError, isLoading } = useJoySummary();
  const connected = summary?.connected === true;

  return (
    <details className="group glass rounded-3xl p-4 md:p-6">
      <summary className="cursor-pointer list-none">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="font-display text-[32px] leading-none tracking-tight">Joy date budget</h2>
            <p className="text-sm text-muted-foreground">
              {isError
                ? "Couldn't check Joy."
                : isLoading
                  ? "Checking..."
                  : connected
                    ? `Connected${summary?.error ? " · last sync failed" : ""}`
                    : "Not linked yet. Link Joy to see your shared date budget here."}
            </p>
          </div>
          <span className="shrink-0 text-sm font-medium text-primary group-open:hidden">Show</span>
          <span className="hidden shrink-0 text-sm font-medium text-primary group-open:inline">Hide</span>
        </div>
      </summary>

      <div className="mt-6 text-sm">
        {summary && connected ? <ConnectedView summary={summary} /> : !isLoading && !isError && <ConnectForm />}
      </div>
    </details>
  );
}
