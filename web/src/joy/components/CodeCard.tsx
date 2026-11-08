import { useRef, useState } from "react";
import type { Invite } from "../api";

/** "Share this code with your partner: XXXX-XXXX (valid 7 days)" with a copy button. */
export function CodeCard({ invite }: { invite: Invite }) {
  const [msg, setMsg] = useState("");
  const input = useRef<HTMLInputElement>(null);
  const days = invite.expires_in_days;

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(invite.code);
      setMsg("Copied.");
    } catch {
      // No clipboard permission: select the code so a long-press can copy it.
      input.current?.select();
      setMsg("Select the code and copy it.");
    }
  };

  return (
    <div className="set-card" role="group" aria-label="Invite code">
      <p>
        Share this code with your partner: <strong className="code-big">{invite.code}</strong> (valid {days} {days === 1 ? "day" : "days"})
      </p>
      <input ref={input} className="visually-hidden" readOnly value={invite.code} aria-hidden="true" tabIndex={-1} />
      <div className="set-row">
        <button type="button" className="btn" onClick={copy}>
          Copy code
        </button>
      </div>
      <p className="set-msg" role="status">
        {msg}
      </p>
    </div>
  );
}
