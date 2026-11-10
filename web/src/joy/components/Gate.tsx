import { useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createCouple, getCouple, inviteCode, joinCouple, keys, type Invite } from "../api";
import { CodeCard } from "./CodeCard";
import { ErrorState } from "./ErrorState";
import { errText } from "./useBusy";

function Jar() {
  return (
    <svg className="empty-jar-art" viewBox="0 0 200 220" aria-hidden="true">
      <path className="lid" d="M62 30.5c24-2.2 52-2 76 .4 2.2.3 3.4 1.6 3.4 3.8v10.2c0 2-1.3 3.2-3.4 3.3-25 1.4-51 1.5-76-.2-2.1-.1-3.4-1.4-3.4-3.4V34.2c0-2.1 1.3-3.4 3.4-3.7z" />
      <path className="glass" d="M66 50c-3 8-11 12-19 19-9 8-12 19-12 33v72c0 17 10 27 27 28 26 1.6 50 1.7 76 .2 17-1 27-11 27-28v-72c0-14-3-25-12-33-8-7-16-11-19-19" />
      <path className="shine" d="M52 104c0-10 3-17 9-22M50 122v30" />
      <path className="floor" d="M48 186c30 3.2 74 3.4 104 .3" />
      <path className="lone-slip" d="M118 170.5l22-6.2 3.4 11.6-22 6.4z" />
    </svg>
  );
}

/**
 * Dates needs a couple. Until there is one, offer to start it or join with a code. The code is
 * held here, above the couple query, so refreshing the status after "Start Dates" doesn't throw
 * the card away before it's been read.
 */
export function Gate({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  const couple = useQuery({ queryKey: keys.couple, queryFn: getCouple });
  const [issued, setIssued] = useState<Invite | null>(null);
  const [joining, setJoining] = useState(false);
  const [code, setCode] = useState("");
  const [error, setError] = useState("");

  const refresh = () => qc.invalidateQueries({ queryKey: keys.couple });

  const start = useMutation({
    mutationFn: async () => {
      await createCouple();
      return inviteCode();
    },
    onSuccess: (invite) => {
      setError("");
      setIssued(invite);
      void refresh();
    },
    onError: (e) => {
      setError(errText(e));
      void refresh();
    },
  });

  const join = useMutation({
    mutationFn: () => joinCouple(code.trim()),
    onSuccess: () => {
      setError("");
      void refresh();
    },
    onError: (e) => setError(errText(e)),
  });

  if (couple.isPending) return <main id="main" tabIndex={-1}><p className="loading" role="status">Opening the jar…</p></main>;
  if (couple.isError) {
    return (
      <main id="main" tabIndex={-1}>
        <section className="onboard">
          <ErrorState title="We couldn’t reach Dates." message={errText(couple.error, "Something went wrong.")} onRetry={() => void couple.refetch()} />
        </section>
      </main>
    );
  }

  if (issued) {
    return (
      <main id="main" tabIndex={-1}>
        <section className="onboard" aria-labelledby="made-h">
          <Jar />
          <h2 id="made-h">Your jar is ready.</h2>
          <CodeCard invite={issued} />
          <p className="soft">
            Send this code to your partner. They sign up, open People → Us and tap Join with a code. You can start writing slips now, and get a new code any time from the bar at the top.
          </p>
          <button type="button" className="btn btn-stamp" onClick={() => setIssued(null)}>
            Continue
          </button>
        </section>
      </main>
    );
  }

  if (!couple.data.in_couple) {
    const busy = start.isPending || join.isPending;
    return (
      <main id="main" tabIndex={-1}>
        <section className="empty-jar onboard" aria-labelledby="empty-h">
          <Jar />
          <h2 id="empty-h">Dates is for two.</h2>
          <p className="soft">Every date starts as a little idea. Start a jar and send your partner a code, or join the one they already started.</p>
          {error && (
            <p className="set-msg set-err" role="alert">
              {error}
            </p>
          )}
          <div className="onboard-acts">
            <button type="button" className="btn btn-stamp" disabled={busy} onClick={() => start.mutate()}>
              Start our jar (I’ll invite them)
            </button>
            <button
              type="button"
              className="btn"
              aria-expanded={joining}
              aria-controls="join-form"
              disabled={busy}
              onClick={() => setJoining((j) => !j)}
            >
              Join with a code
            </button>
          </div>
          {joining && (
            <form
              id="join-form"
              className="join-form"
              onSubmit={(e) => {
                e.preventDefault();
                if (!busy && code.trim()) join.mutate();
              }}
            >
              <label htmlFor="join-code" className="soft">
                Your partner’s code
              </label>
              <input
                id="join-code"
                className="set-input"
                value={code}
                onChange={(e) => setCode(e.target.value.toUpperCase())}
                placeholder="XXXX-XXXX"
                autoComplete="off"
                autoCapitalize="characters"
                spellCheck={false}
                autoFocus
              />
              <button type="submit" className="btn btn-stamp" disabled={busy || !code.trim()}>
                Join
              </button>
            </form>
          )}
        </section>
      </main>
    );
  }

  return (
    <>
      {couple.data.waiting && <WaitingBar />}
      {children}
    </>
  );
}

/** "Waiting for your partner": the code is one tap away, right here (a new one replaces the last). */
function WaitingBar() {
  const [invite, setInvite] = useState<Invite | null>(null);
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);
  const get = async () => {
    setBusy(true);
    setErr("");
    try {
      setInvite(await inviteCode());
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="waiting-bar" style={{ maxWidth: 680, margin: "0 auto", padding: "0 var(--gutter)" }}>
      <p role="status">
        Waiting for your partner to join.{" "}
        {!invite && (
          <button type="button" className="btn btn-quiet" disabled={busy} onClick={() => void get()}>
            Get a code to send
          </button>
        )}
      </p>
      {invite && <CodeCard invite={invite} />}
      {err && (
        <p className="set-msg set-err" role="alert">
          {err}
        </p>
      )}
    </div>
  );
}
