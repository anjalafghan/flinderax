import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { toast } from "sonner";

import { ingestApi } from "@/services/api";
import { keys, useIngestKey, usePlannerMutation } from "@/hooks/usePlanner";
import { ConfirmationModal } from "@/components/ui/ConfirmationModal";
import { Button } from "@/components/ui/button";
import { formatDay } from "@/utils/dates";
import { INGEST_WEBHOOK_PATH } from "@/utils/spending";

function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      toast.error("Couldn't copy. Select the text and copy it by hand.");
    }
  };
  return (
    <Button variant="outline" size="sm" onClick={copy} aria-label={label} className="shrink-0">
      {copied ? <Check className="mr-1.5 h-4 w-4" /> : <Copy className="mr-1.5 h-4 w-4" />}
      {copied ? "Copied" : "Copy"}
    </Button>
  );
}

/** Shows the key once, with a copy button. It is not shown again after this is dismissed. */
function NewKeyReveal({ value, onDone }: { value: string; onDone: () => void }) {
  return (
    <div className="space-y-3 rounded-lg border border-warn/40 bg-warn-tint p-3 text-sm">
      <p className="font-medium text-warn">
        Copy this key now. It is shown only once; you can't view it again later.
      </p>
      <div className="flex items-center gap-2">
        <code className="min-w-0 flex-1 break-all rounded-md bg-background px-3 py-2 font-mono text-xs">{value}</code>
        <CopyButton value={value} label="Copy import key" />
      </div>
      <div className="flex justify-end">
        <Button size="sm" variant="ghost" onClick={onDone}>
          I've copied it
        </Button>
      </div>
    </div>
  );
}

/** Setup for the phone: the key, the webhook, and the steps for iPhone, Android and email. */
export function IngestSetup() {
  const { data: status, isError, isLoading } = useIngestKey();
  const [newKey, setNewKey] = useState<string | null>(null);
  const [confirmReplace, setConfirmReplace] = useState(false);
  const rotate = usePlannerMutation(() => ingestApi.rotateKey(), [keys.ingestKey], {
    error: "Couldn't create a key. Try again.",
  });

  const generate = () => {
    if (status?.has_key) return setConfirmReplace(true);
    rotate.mutate(undefined, { onSuccess: (r) => setNewKey(r.key) });
  };

  const hasKey = status?.has_key === true;
  const webhook = `${window.location.origin}${INGEST_WEBHOOK_PATH}`;
  const lastUsed = status?.last_used_at ? formatDay(status.last_used_at.slice(0, 10)) : null;

  return (
    <details className="group glass rounded-3xl p-4 md:p-6">
      <summary className="cursor-pointer list-none">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="font-display text-[32px] leading-none tracking-tight">Automatic import setup</h2>
            <p className="max-w-prose text-sm text-muted-foreground">
              {isError
                ? "Couldn't check the import key."
                : hasKey
                  ? `Connected${lastUsed ? `, last alert ${lastUsed}` : ", no alerts received yet"}`
                  : "Not set up yet. Your phone can send bank alerts here so you don't have to type them."}
            </p>
          </div>
          <span className="shrink-0 text-sm font-medium text-primary group-open:hidden">Show</span>
          <span className="hidden shrink-0 text-sm font-medium text-primary group-open:inline">Hide</span>
        </div>
      </summary>

      <div className="mt-6 space-y-6 text-sm">
        <section className="space-y-3">
          <h3 className="font-semibold">1. Your import key</h3>
          {newKey ? (
            <NewKeyReveal value={newKey} onDone={() => setNewKey(null)} />
          ) : (
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="text-muted-foreground">
                {hasKey
                  ? `Created ${status?.created_at ? formatDay(status.created_at.slice(0, 10)) : "earlier"}. Your phone sends it with every alert.`
                  : "Create a key and paste it into your phone's shortcut or forwarding app."}
              </p>
              <Button size="sm" variant={hasKey ? "outline" : "default"} onClick={generate} disabled={rotate.isPending || isLoading}>
                {rotate.isPending ? "Creating..." : hasKey ? "Replace key" : "Generate key"}
              </Button>
            </div>
          )}
        </section>

        <section className="space-y-2">
          <h3 className="font-semibold">2. Webhook</h3>
          <div className="flex items-center gap-2">
            <code className="min-w-0 flex-1 break-all rounded-md bg-muted px-3 py-2 font-mono text-xs">POST {webhook}</code>
            <CopyButton value={webhook} label="Copy webhook URL" />
          </div>
          <p className="text-muted-foreground">
            Send the header <code className="font-mono text-xs">X-Ingest-Key: &lt;your key&gt;</code> and a JSON body:
          </p>
          <pre className="overflow-x-auto rounded-md bg-muted p-3 font-mono text-xs">{`{ "text": "<message text>", "source": "sms" }`}</pre>
        </section>

        <section className="space-y-2">
          <h3 className="font-semibold">3. iPhone</h3>
          <ol className="list-decimal space-y-1.5 pl-5 text-muted-foreground">
            <li>Open the Shortcuts app, go to Automation, and tap + then "Message".</li>
            <li>Choose "Sender contains" and enter your bank's sender, such as HDFCBK or KOTAKB.</li>
            <li>Pick "Run Immediately" and turn off "Notify When Run".</li>
            <li>Add the action "Get Contents of URL". Set the method to POST and the URL to the webhook above.</li>
            <li>Add the header X-Ingest-Key with your key.</li>
            <li>
              Set the request body to JSON. Add "text" = Shortcut Input's message content, and "source" = sms.
            </li>
          </ol>
        </section>

        <section className="space-y-2">
          <h3 className="font-semibold">4. Android</h3>
          <p className="text-muted-foreground">
            Use any SMS-to-webhook or forwarder app. Point it at the webhook URL, add the X-Ingest-Key header, and send
            the JSON body above with the message text and "sms" as the source.
          </p>
        </section>

        <section className="space-y-2">
          <h3 className="font-semibold">5. Email</h3>
          <p className="text-muted-foreground">
            Create a Gmail filter that labels your bank's alert emails (for example "Bank alerts"). Then see the readme
            for the Google Apps Script that posts newly labelled mails to the webhook with source "email".
          </p>
        </section>

        <section className="space-y-2 rounded-lg bg-muted/50 p-3">
          <h3 className="font-semibold">Balances</h3>
          <p className="text-muted-foreground">
            On the Accounts page, add the last 4 digits of each bank account. Alerts that include an available balance
            (Avl Bal) then update that account's balance automatically.
          </p>
        </section>
      </div>

      <ConfirmationModal
        isOpen={confirmReplace}
        onClose={() => setConfirmReplace(false)}
        onConfirm={() => {
          setConfirmReplace(false);
          rotate.mutate(undefined, { onSuccess: (r) => setNewKey(r.key) });
        }}
        title="Replace your import key?"
        description="The current key stops working straight away. Until you paste the new key into your phone, bank alerts won't import."
        confirmText="Replace key"
        variant="destructive"
        isPending={rotate.isPending}
      />
    </details>
  );
}
