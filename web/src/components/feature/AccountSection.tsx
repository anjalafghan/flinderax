import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/context/AuthContext";
import { sessionApi } from "@/services/session";

const msg = (e: unknown) => (e instanceof Error && e.message ? e.message : "Something went wrong. Please try again.");

/** Password, sessions and deleting the account. */
export function AccountSection() {
  return (
    <div className="stack">
      <ChangePassword />
      <SignOutEverywhere />
      <DeleteAccount />
    </div>
  );
}

const CARD = "rounded-[28px] border border-border bg-card p-6 stack";

function ChangePassword() {
  const { login } = useAuth();
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [again, setAgain] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [done, setDone] = useState(false);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setDone(false);
    if (next.length < 8) return setError("Use at least 8 characters for the new password.");
    if (next !== again) return setError("The two new passwords don't match.");
    setBusy(true);
    setError("");
    try {
      const t = await sessionApi.changePassword(current, next);
      // Every other sign-in has just ended; this one carries on with the fresh token.
      login(t.access_token, t.expires_at);
      setCurrent("");
      setNext("");
      setAgain("");
      setDone(true);
    } catch (err) {
      setError(msg(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className={CARD} aria-labelledby="pw-h">
      <h2 id="pw-h" className="text-lg font-medium">
        Change password
      </h2>
      <form onSubmit={(e) => void submit(e)} className="stack">
        <div className="space-y-1.5">
          <Label htmlFor="pw-current">Current password</Label>
          <Input id="pw-current" type="password" autoComplete="current-password" value={current} onChange={(e) => setCurrent(e.target.value)} required />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="pw-new">New password</Label>
          <Input id="pw-new" type="password" autoComplete="new-password" value={next} onChange={(e) => setNext(e.target.value)} required />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="pw-again">New password again</Label>
          <Input id="pw-again" type="password" autoComplete="new-password" value={again} onChange={(e) => setAgain(e.target.value)} required />
        </div>
        <p className="text-sm text-muted-foreground">Changing it signs you out everywhere else.</p>
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
        {done && (
          <p role="status" className="text-sm text-muted-foreground">
            Password changed. Other devices are signed out.
          </p>
        )}
        <div className="flex flex-wrap items-center gap-2">
          <Button type="submit" disabled={busy || !current || !next || !again}>
            Change password
          </Button>
          <ResetButton
            label="Clear"
            disabled={busy || (!current && !next && !again && !error)}
            onReset={() => {
              setCurrent("");
              setNext("");
              setAgain("");
              setError("");
            }}
          />
        </div>
      </form>
    </section>
  );
}

function SignOutEverywhere() {
  const { logout } = useAuth();
  const navigate = useNavigate();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const go = async () => {
    setBusy(true);
    setError("");
    try {
      await sessionApi.logoutEverywhere();
      logout();
      navigate("/auth");
    } catch (err) {
      setError(msg(err));
      setBusy(false);
    }
  };

  return (
    <section className={CARD} aria-labelledby="out-h">
      <h2 id="out-h" className="text-lg font-medium">
        Sign out everywhere
      </h2>
      <p className="text-sm text-muted-foreground">Ends every sign-in on every device, including this one. Use it if a phone is lost or shared.</p>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
      <div>
        <Button variant="outline" disabled={busy} onClick={() => void go()}>
          Sign out everywhere
        </Button>
      </div>
    </section>
  );
}

function DeleteAccount() {
  const { logout } = useAuth();
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [word, setWord] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const remove = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      await sessionApi.deleteAccount(password);
      logout();
      navigate("/auth");
    } catch (err) {
      setError(msg(err));
      setBusy(false);
    }
  };

  return (
    <section className={CARD} aria-labelledby="del-h">
      <h2 id="del-h" className="text-lg font-medium">
        Delete my account
      </h2>
      {!open ? (
        <>
          <p className="text-sm text-muted-foreground">
            Removes your account and everything in it: money, plans, health, people and your check-ins. This can't be undone.
          </p>
          <div>
            <Button variant="outline" aria-expanded={false} onClick={() => setOpen(true)}>
              Delete my account…
            </Button>
          </div>
        </>
      ) : (
        <form onSubmit={(e) => void remove(e)} className="stack">
          <p className="text-sm">
            <strong>This can't be undone.</strong> If you share Dates with someone, they keep it. If you are the last one in it, Dates and its photos are deleted too. Backups are rotated out within 30 days.
          </p>
          <div className="space-y-1.5">
            <Label htmlFor="del-word">Type delete to confirm</Label>
            <Input id="del-word" autoComplete="off" value={word} onChange={(e) => setWord(e.target.value)} />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="del-pw">Your password</Label>
            <Input id="del-pw" type="password" autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} />
          </div>
          {error && (
            <p role="alert" className="text-sm text-destructive">
              {error}
            </p>
          )}
          <div className="flex flex-wrap gap-2">
            <Button type="submit" variant="destructive" disabled={busy || word.trim() !== "delete" || !password}>
              Delete everything
            </Button>
            <Button type="button" variant="ghost" disabled={busy} onClick={() => setOpen(false)}>
              Keep my account
            </Button>
          </div>
        </form>
      )}
    </section>
  );
}
