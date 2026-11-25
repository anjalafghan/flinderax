// Our museum: every keepsake in the jar in one place. What we've kept is on display, in its case,
// with its photo tucked behind and our note on the placard; everything else waits as a grey shape,
// still in the jar, so you can see what each slip will bring home.
import { useMemo } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { getSlips, keys } from "../api";
import { getKept, keptKey } from "../kept/api";
import type { KeptView } from "../kept/types";
import { errText } from "../components/useBusy";
import {
  SceneArt,
  SceneButton,
  TuckedPrint,
  type SceneDef,
} from "../keepsakes/Scene";
import { sceneFor, visitsOf } from "../keepsakes/scene-find";
import { isSeen, markSeen } from "../keepsakes/seen";
import "../kept/kept.css";

/** Today in India, as "YYYY-MM-DD" (kept days are IST dates). */
const isoToday = () =>
  new Date(Date.now() + 5.5 * 3600_000).toISOString().slice(0, 10);

interface Piece {
  h: KeptView;
  scene: SceneDef;
}

export default function MuseumPage() {
  const kept = useQuery({ queryKey: keptKey, queryFn: getKept });
  const slips = useQuery({ queryKey: keys.slips, queryFn: getSlips });

  const { display, plain } = useMemo(() => {
    const done = (kept.data?.months ?? [])
      .flatMap((m) => m.items)
      .filter((h) => !h.sealed && h.day && h.day <= isoToday());
    const visits = visitsOf(done);
    const display: Piece[] = [];
    let plain = 0;
    for (const h of [...done].sort(
      (a, b) => b.day.localeCompare(a.day) || b.id - a.id,
    )) {
      const scene = sceneFor(h.title, visits.get(h.id));
      if (scene) display.push({ h, scene });
      else plain += 1;
    }
    return { display, plain };
  }, [kept.data]);

  const jar = useMemo(
    () =>
      (slips.data?.groups ?? [])
        .map((g) => ({
          ...g,
          items: g.items
            .filter((a) => a.times_done === 0)
            .map((a) => ({ a, scene: sceneFor(a.title, 1) }))
            .filter((x) => x.scene),
        }))
        .filter((g) => g.items.length > 0),
    [slips.data],
  );
  const waiting = jar.reduce((n, g) => n + g.items.length, 0);

  if (kept.isPending || slips.isPending)
    return (
      <p className="loading" role="status">
        Unlocking the museum…
      </p>
    );
  if (kept.isError || slips.isError) {
    return (
      <div>
        <p className="kind-error" role="alert">
          {errText(kept.error ?? slips.error)}
        </p>
        <button
          type="button"
          className="btn"
          onClick={() => void Promise.all([kept.refetch(), slips.refetch()])}
        >
          Try again
        </button>
      </div>
    );
  }

  const total = display.length + waiting;
  return (
    <div className="mu">
      <header className="mu-head">
        <h1 className="mu-title">Our museum</h1>
        <p className="mu-count">
          <strong>{display.length}</strong>{" "}
          {display.length === 1 ? "keepsake" : "keepsakes"} on display,{" "}
          <strong>{waiting}</strong> still in the jar
        </p>
        {total > 0 && (
          <p
            className="mu-shelf"
            role="img"
            aria-label={`${display.length} of ${total} keepsakes collected`}
          >
            {Array.from({ length: total }, (_, i) => (
              <span
                key={i}
                className={i < display.length ? "is-in" : undefined}
              />
            ))}
          </p>
        )}
      </header>

      <section className="mu-room" aria-labelledby="mu-display-h">
        <h2 id="mu-display-h" className="mu-h">
          On display
        </h2>
        {display.length === 0 ? (
          <p className="mu-empty">
            Nothing on display yet. Keep a date from the jar and its keepsake
            comes to live here. <Link to="/joy">Go to the jar</Link>
          </p>
        ) : (
          <ol className="mu-cases">
            {display.map(({ h, scene }, i) => (
              <li
                key={h.id}
                className="mu-case"
                style={
                  {
                    "--t": ((h.id * 37) % 7) - 3,
                    "--i": i,
                  } as React.CSSProperties
                }
              >
                <div className="mu-ex">
                  {h.photo_url && (
                    <TuckedPrint
                      className="mu-print"
                      src={h.photo_url}
                      alt={`Our photo from ${h.title}, ${h.day_label}`}
                    />
                  )}
                  <SceneButton
                    className="mu-obj"
                    scene={scene}
                    what={`${h.title}, ${h.day_label}`}
                    arrive={!isSeen(h.id)}
                    onArrived={() => markSeen(h.id)}
                  />
                </div>
                <div className="mu-placard">
                  <p className="mu-pl-name">{scene.name}</p>
                  <p className="mu-pl-what">
                    <Link to={`/joy/kept#k${h.id}`}>{h.title}</Link>,{" "}
                    {h.day_label}
                    {h.rating ? (
                      <span
                        className="mu-hearts"
                        aria-label={`${h.rating} out of 5`}
                      >
                        {"♥".repeat(h.rating)}
                      </span>
                    ) : null}
                  </p>
                  {h.note && <p className="mu-pl-note">“{h.note}”</p>}
                </div>
              </li>
            ))}
          </ol>
        )}
        {plain > 0 && (
          <p className="mu-plain">
            …and {plain} more {plain === 1 ? "date" : "dates"} kept with a
            stamp, waiting for a keepsake of {plain === 1 ? "its" : "their"}{" "}
            own.
          </p>
        )}
      </section>

      {jar.length > 0 && (
        <section className="mu-room mu-jar" aria-labelledby="mu-jar-h">
          <h2 id="mu-jar-h" className="mu-h">
            Still in the jar
          </h2>
          <p className="mu-sub">What each slip brings home, once we keep it.</p>
          {jar.map((g) => (
            <div key={g.key} className="mu-group">
              <h3 className="mu-group-h">
                {g.label} <span>{g.items.length}</span>
              </h3>
              <ul className="mu-ghosts">
                {g.items.map(({ a, scene }) => (
                  <li key={a.id}>
                    <Link
                      to={`/joy/slips/${a.id}`}
                      aria-label={`${a.title}: ${scene!.name}, not kept yet`}
                    >
                      <SceneArt scene={scene!} ghost />
                      <span className="mu-ghost-t">{a.title}</span>
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </section>
      )}
    </div>
  );
}
