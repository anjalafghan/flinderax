import { useEffect, useState } from "react";
import api from "@/services/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";

const SCOPES = [
  ["money:read", "Read money"],
  ["money:write", "Change money"],
  ["dates:read", "Read Dates"],
  ["dates:write", "Change Dates"],
  ["inbox:write", "Clear bank text inbox"],
  ["analysis:write", "Save reports"],
] as const;

type Token = {
  id: string;
  name: string;
  scopes: string[];
  created_at: number;
  last_used_at: number | null;
  expires_at: number | null;
  revoked_at: number | null;
};
type Audit = {
  id: number;
  token_id: string;
  method: string;
  route: string;
  status: number;
  created_at: number;
};

const date = (value: number | null) =>
  value ? new Date(value * 1000).toLocaleString() : "Never";
const CARD = "rounded-[28px] border border-border bg-card p-6 stack";

export function ApiTokensCard() {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [audit, setAudit] = useState<Audit[]>([]);
  const [name, setName] = useState("");
  const [scopes, setScopes] = useState<string[]>(["money:read", "dates:read"]);
  const [expiry, setExpiry] = useState("90");
  const [oneTimeToken, setOneTimeToken] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = async () => {
    const [t, a] = await Promise.all([
      api.get<Token[]>("/api/tokens"),
      api.get<Audit[]>("/api/tokens/audit"),
    ]);
    setTokens(t.data);
    setAudit(a.data);
  };
  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      const [t, a] = await Promise.all([
        api.get<Token[]>("/api/tokens"),
        api.get<Audit[]>("/api/tokens/audit"),
      ]);
      if (!cancelled) {
        setTokens(t.data);
        setAudit(a.data);
      }
    };
    void load().catch((e: unknown) => {
      if (!cancelled) setError(e instanceof Error ? e.message : "Could not load tokens.");
    });
    return () => { cancelled = true; };
  }, []);

  const create = async (event: React.FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      const result = await api.post<Token & { token: string }>("/api/tokens", {
        name,
        scopes,
        expires_in_days: expiry ? Number(expiry) : null,
      });
      setOneTimeToken(result.data.token);
      setName("");
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not create token.");
    } finally {
      setBusy(false);
    }
  };

  const revoke = async (id: string) => {
    setBusy(true);
    setError("");
    try {
      await api.post<void>(`/api/tokens/${encodeURIComponent(id)}/revoke`, {});
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not revoke token.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className={CARD} aria-labelledby="api-tokens-heading">
      <div>
        <h2 id="api-tokens-heading" className="text-lg font-medium">Assistant access</h2>
        <p className="mt-1 text-sm text-muted-foreground">Create a limited token for a remote assistant. You can revoke it here at any time.</p>
      </div>
      {oneTimeToken && (
        <div className="rounded-2xl border border-brand/30 bg-brand-tint p-4" role="status">
          <p className="font-medium">Copy this token now. It will only be shown once.</p>
          <code className="mt-2 block break-all text-sm">{oneTimeToken}</code>
          <div className="mt-3 flex flex-wrap gap-2">
            <Button type="button" variant="outline" onClick={() => void navigator.clipboard?.writeText(oneTimeToken)}>Copy token</Button>
          <Button type="button" variant="outline" onClick={() => setOneTimeToken("")}>I saved it</Button>
          </div>
        </div>
      )}
      <form onSubmit={(e) => void create(e)} className="stack">
        <label className="field">
          <span>Token name</span>
          <Input value={name} onChange={(e) => setName(e.target.value)} maxLength={80} required placeholder="Claude daily brief" />
        </label>
        <fieldset className="stack">
          <legend className="font-medium">Access</legend>
          <div className="grid gap-2 sm:grid-cols-2">
            {SCOPES.map(([scope, label]) => (
              <label key={scope} className="flex min-h-10 items-center gap-3 text-sm md:min-h-0">
                <input type="checkbox" className="h-5 w-5 shrink-0 accent-brand md:h-4 md:w-4" checked={scopes.includes(scope)} onChange={(e) => setScopes((old) => e.target.checked ? [...old, scope] : old.filter((s) => s !== scope))} />
                {label}
              </label>
            ))}
          </div>
        </fieldset>
        <label className="field">
          <span>Expires</span>
          <Select value={expiry} onChange={(e) => setExpiry(e.target.value)}>
            <option value="30">In 30 days</option><option value="90">In 90 days</option><option value="180">In 180 days</option><option value="365">In 1 year</option><option value="">Never</option>
          </Select>
        </label>
        <Button className="w-fit" type="submit" disabled={busy || !name.trim() || scopes.length === 0}>Create token</Button>
      </form>
      {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
      <div className="stack">
        <h3 className="font-medium">Tokens</h3>
        {tokens.length === 0 ? <p className="text-sm text-muted-foreground">No assistant tokens yet.</p> : tokens.map((t) => (
          <article key={t.id} className="rounded-2xl border border-border p-4">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0"><p className="break-words font-medium">{t.name}{t.revoked_at ? " · Revoked" : ""}</p><p className="break-words text-xs text-muted-foreground">{t.scopes.join(", ")}</p></div>
              {!t.revoked_at && <Button type="button" variant="outline" size="sm" disabled={busy} onClick={() => void revoke(t.id)}>Revoke</Button>}
            </div>
            <p className="mt-2 text-xs text-muted-foreground">Created {date(t.created_at)} · Last used {date(t.last_used_at)} · Expires {date(t.expires_at)}</p>
          </article>
        ))}
      </div>
      <details>
        <summary className="min-h-10 cursor-pointer content-center font-medium">Recent assistant activity</summary>
        <ul className="mt-3 max-h-64 space-y-2 overflow-auto text-xs">
          {audit.length === 0 ? <li className="text-muted-foreground">No token activity yet.</li> : audit.map((a) => <li key={a.id} className="break-words">{date(a.created_at)} · {a.method} {a.route} · {a.status}</li>)}
        </ul>
      </details>
    </section>
  );
}
