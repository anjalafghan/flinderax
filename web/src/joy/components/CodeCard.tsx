import { useRef, useState } from "react";
import type { Invite } from "../api";

/** What to send: a line a partner can follow from a message alone. */
function shareText(code: string): string {
  return `Join our jar on joy: sign up, open People, then Us, tap Join with a code and enter ${code}`;
}

/** "Send this code to your partner: XXXX-XXXX (valid 7 days)" with Share and Copy buttons. */
export function CodeCard({ invite, partner }: { invite: Invite; partner?: string | null }) {
  const [msg, setMsg] = useState("");
  const input = useRef<HTMLInputElement>(null);
  const days = invite.expires_in_days;
  const canShare = typeof navigator !== "undefined" && typeof navigator.share === "function";

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

  const share = async () => {
    try {
      await navigator.share({ title: "Join our jar", text: shareText(invite.code) });
    } catch (e) {
      // Closing the share sheet is not a failure; anything else falls back to copying.
      if (e instanceof DOMException && e.name === "AbortError") return;
      await copy();
    }
  };

  return (
    <div className="set-card" role="group" aria-label="Invite code">
      <p>
        Send this code to {partner || "your partner"}: <strong className="code-big">{invite.code}</strong> (valid {days} {days === 1 ? "day" : "days"})
      </p>
      <input ref={input} className="visually-hidden" readOnly value={invite.code} aria-hidden="true" tabIndex={-1} />
      <div className="set-row">
        {canShare && (
          <button type="button" className="btn btn-stamp" onClick={() => void share()}>
            Share
          </button>
        )}
        <button type="button" className="btn" onClick={() => void copy()}>
          Copy code
        </button>
      </div>
      <p className="set-msg" role="status">
        {msg}
      </p>
    </div>
  );
}
