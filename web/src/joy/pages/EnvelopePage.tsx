import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import "../styles/index.css";
import "../styles/app.css";
import { getEnvelope, keys } from "../api";

/** The date, arriving in a sealed envelope. Public (no login) and bare (no app chrome). */
export default function EnvelopePage() {
  const { token = "" } = useParams();
  const q = useQuery({ queryKey: keys.envelope(token), queryFn: () => getEnvelope(token), retry: false, staleTime: Infinity });
  const [open, setOpen] = useState(false);
  const [out, setOut] = useState(false);
  const letter = useRef<HTMLElement>(null);
  const title = useRef<HTMLHeadingElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const env = q.data;

  useEffect(() => () => clearTimeout(timer.current), []);

  // Only the people with the link: keep it out of search.
  useEffect(() => {
    const m = document.createElement("meta");
    m.name = "robots";
    m.content = "noindex";
    document.head.appendChild(m);
    return () => m.remove();
  }, []);

  useEffect(() => {
    const prev = document.title;
    document.title = env ? (env.from ? `${env.from} sent you a date · joy` : "A note from our jar · joy") : "joy";
    return () => {
      document.title = prev;
    };
  }, [env]);

  const still = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

  const openIt = () => {
    if (open) return;
    setOpen(true);
    try {
      navigator.vibrate?.(10);
    } catch {
      /* not supported */
    }
    timer.current = setTimeout(() => {
      setOut(true);
      requestAnimationFrame(() => {
        letter.current?.scrollIntoView?.({ behavior: still ? "auto" : "smooth", block: "start" });
        title.current?.focus({ preventScroll: true });
      });
    }, still ? 0 : 1150);
  };

  return (
    <div className="joy grain joy-bare" data-page="envelope">
      <main id="main">
        <div className="js">
          {q.isPending ? (
            <p className="loading" role="status">Opening…</p>
          ) : q.isError || !env ? (
            <div className="env-page">
              <h1 className="env-to" role="status">This envelope has gone.</h1>
            </div>
          ) : (
            <div className="env-page">
              <p className="env-to">{env.to ? `For ${env.to}${env.from ? `, from ${env.from}` : ""}` : "From our jar"}</p>

              <div className={`env${open ? " is-open" : ""}`} id="env">
                <div className="env-back" aria-hidden="true"></div>
                <div className="env-peek" aria-hidden="true">
                  <p className="env-peek-k">{env.kicker}</p>
                  <p className="env-peek-t">{env.title}</p>
                </div>
                <div className="env-pocket" aria-hidden="true"></div>
                <div className="env-flap" aria-hidden="true"></div>
                <button className="env-seal" id="env-open" type="button" aria-controls="letter" aria-expanded={open} disabled={open} onClick={openIt}>
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M12 20.3c-4.6-3.2-8.3-6.4-8.3-10.4 0-2.6 2-4.6 4.4-4.6 1.6 0 3 .9 3.9 2.2.9-1.3 2.3-2.2 3.9-2.2 2.4 0 4.4 2 4.4 4.6 0 4-3.7 7.2-8.3 10.4z" />
                  </svg>
                  <span className="visually-hidden">Open the envelope</span>
                </button>
              </div>
              <p className="env-hint" id="env-hint">{open ? "" : "Tap the seal to open it"}</p>

              <article ref={letter} className={`env-letter grain${env.sealed ? " is-sealed" : ""}${out ? " is-out" : ""}`} id="letter" aria-labelledby="letter-t">
                <p className="env-k">{env.kicker}</p>
                <h1 className="env-t" id="letter-t" ref={title} tabIndex={-1}>{env.title}</h1>
                {env.when && <p className="env-when">{env.when}</p>}
                {env.place && (
                  <p className="env-place">
                    <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M12 21.2c-3.9-4.6-6.4-8.2-6.4-11.2a6.4 6.4 0 0 1 12.8-.2c.1 3.1-2.5 6.7-6.4 11.4z" />
                      <circle cx="12" cy="9.8" r="2.3" />
                    </svg>
                    {env.maps_url ? (
                      <a href={env.maps_url} target="_blank" rel="noopener noreferrer">
                        {env.place}<span className="visually-hidden"> (opens the map)</span>
                      </a>
                    ) : (
                      <span>{env.place}</span>
                    )}
                  </p>
                )}
                {env.meta && <p className="env-meta">{env.meta}</p>}
                {env.steps.length > 0 && (
                  <ol className="env-route">
                    {env.steps.map((st, i) => (
                      <li key={i} className={st.mine ? "is-mine" : ""}>
                        <span className="env-at">{st.at}</span>
                        <span>{st.what}</span>
                      </li>
                    ))}
                  </ol>
                )}
                {env.hours_note && <p className="env-note">{env.hours_note}</p>}
                {env.notes && <p className="env-notes">{env.notes}</p>}
                {env.sealed && (
                  <p className="env-locked">
                    <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
                      <rect x="5" y="10.5" width="14" height="9.5" rx="2" />
                      <path d="M8.2 10.5V8a3.8 3.8 0 0 1 7.6 0v2.5" />
                    </svg>
                    <span>The rest stays locked until {env.from ? `${env.from} reveals it` : "it's"}. Just be ready.</span>
                  </p>
                )}
                <p className="env-sign">— joy</p>
              </article>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
