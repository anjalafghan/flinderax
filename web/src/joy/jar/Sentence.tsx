// "For Thursday evening we have ₹__, about __, around __, no further than __ from __, feeling __."
// Joy's decide.html: native selects for the blanks (the existing CSS styles and sizes them), and
// tappable chips for the places and where we start from. Chips are plain checkboxes (inside a
// fieldset whose legend names the group), drawn as Joy's pen-circle by the CSS.
import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import type { DecideView, Opt } from "./api";
import { toggleSetting, toggleStart, type Filters } from "./form";

const SETTING_WORDS: Record<string, string> = {
  day_trip: "a day trip",
  travel: "travelling",
  home: "at home together",
  apart: "long distance",
};
const settingWord = (o: Opt) => SETTING_WORDS[o.value] ?? o.label.toLowerCase();

interface Props {
  view: DecideView;
  filters: Filters;
  /** `slow`: a typed field, so the live match waits a moment longer. */
  onChange: (patch: Partial<Filters>, slow?: boolean) => void;
  busy: boolean;
  onDraw: () => void;
  onDrawMiddle: () => void;
  shake: { available: boolean; on: boolean; label: string; toggle: () => void };
}

const Ico = ({ d }: { d: string[] }) => (
  <svg className="ico" viewBox="0 0 24 24" aria-hidden="true">
    {d.map((p) => (
      <path key={p} d={p} />
    ))}
  </svg>
);

function Chip({ on, tint, onToggle, children }: { on: boolean; tint?: string; onToggle: () => void; children: string }) {
  return (
    <label className="j-chip" style={tint ? ({ "--tint": `var(--tint-${tint})` } as React.CSSProperties) : undefined}>
      <input type="checkbox" checked={on} onChange={onToggle} />
      <span>{children}</span>
    </label>
  );
}

export default function Sentence({ view: v, filters: f, onChange, busy, onDraw, onDrawMiddle, shake }: Props) {
  const formRef = useRef<HTMLFormElement>(null);
  const hasWhens = v.whens.length > 0;
  const bothIn = v.middle.both_in;

  // Browsers without field-sizing: size each blank to what it says.
  useEffect(() => {
    const form = formRef.current;
    if (!form || (typeof CSS !== "undefined" && CSS.supports?.("field-sizing", "content"))) return;
    const m = document.createElement("canvas").getContext("2d");
    if (!m) return;
    form.querySelectorAll<HTMLSelectElement | HTMLInputElement>(".blank select, .blank input").forEach((el) => {
      const cs = getComputedStyle(el);
      m.font = `${cs.fontStyle} ${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`;
      const text = el instanceof HTMLSelectElement ? el.selectedOptions[0]?.text ?? "" : el.value || el.placeholder;
      el.style.width = `${Math.ceil(m.measureText(text).width + parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight) + 4)}px`;
    });
  });

  const sel = (name: keyof Filters, label: string, anyText: string, opts: Opt[], word: (o: Opt) => string) => (
    <label className="blank">
      <span className="visually-hidden">{label}</span>
      <select name={name} value={f[name] as string} className={f[name] === "" ? "is-any" : undefined} onChange={(e) => onChange({ [name]: e.target.value })}>
        <option value="">{anyText}</option>
        {opts.map((o) => (
          <option key={o.value} value={o.value}>
            {word(o)}
          </option>
        ))}
      </select>
    </label>
  );

  return (
    <form
      id="tonight"
      className="tonight"
      ref={formRef}
      onSubmit={(e) => {
        e.preventDefault();
        onDraw();
      }}
    >
      {v.rain != null && (
        <p className="weather">
          <Ico d={["M7.2 15.4a4.3 4.3 0 0 1-.4-8.5 5.4 5.4 0 0 1 10.3 1.4 3.6 3.6 0 0 1 .3 7.1", "M9 18.2l-.9 2.2M13 17.6l-.9 2.6M16.6 18.2l-.8 2"]} />
          <span>Rain likely tonight ({v.rain}%), so cozy ones float up.</span>
        </p>
      )}
      <p className="sentence">
        {hasWhens ? (
          <>
            <span className="lead">For</span>{" "}
            <label className="blank blank-when">
              <span className="visually-hidden">Which date we're drawing for</span>
              <select name="when" value={f.when} onChange={(e) => onChange({ when: e.target.value })}>
                {v.whens.map((o) => (
                  <option key={o.value} value={o.value}>
                    {o.label}
                  </option>
                ))}
                <option value="any">any day at all</option>
              </select>
            </label>{" "}
            {v.office_name && (
              <>
                <fieldset className="start-from" title="Tap again to go back to the usual for that day">
                  <legend className="visually-hidden">Where the day starts</legend>
                  <span className="start-l" aria-hidden="true">
                    from
                  </span>{" "}
                  <Chip on={f.start === "office"} onToggle={() => onChange({ start: toggleStart(f.start, "office") })}>
                    {v.office_name}
                  </Chip>{" "}
                  <Chip on={f.start === "home"} onToggle={() => onChange({ start: toggleStart(f.start, "home") })}>
                    home
                  </Chip>
                </fieldset>{" "}
              </>
            )}
            <span className="lead-rest">we have</span>
          </>
        ) : (
          <span className="lead">Tonight we have</span>
        )}{" "}
        <label className="blank blank-money">
          <span className="visually-hidden">
            Budget for both of us, in {v.currency} (leave empty for any)
          </span>
          <span className="cur" aria-hidden="true">
            {v.currency}
          </span>
          <input type="number" name="budget" inputMode="numeric" min="0" step="any" placeholder="any amount" autoComplete="off" value={f.budget} onChange={(e) => onChange({ budget: e.target.value }, true)} />
        </label>{" "}
        and{" "}
        {sel("max_duration", "Time we have", "all the time in the world", v.durations, (o) => `up to ${o.label.toLowerCase()}`)},{" "}
        <fieldset className="where">
          <legend className="visually-hidden">Where</legend>
          <span className="where-l" aria-hidden="true">
            anywhere:
          </span>{" "}
          {v.settings.map((o, i) => (
            <span key={o.value}>
              <Chip on={f.settings.includes(o.value)} tint={o.value} onToggle={() => onChange({ settings: toggleSetting(f.settings, o.value) })}>
                {settingWord(o)}
              </Chip>
              {i < v.settings.length - 1 ? " " : ""}
            </span>
          ))}
        </fieldset>
        ,{" "}
        {sel("max_km", "How far we'll go", "any distance", v.kms, (o) => `within ${o.label}`)}
        <span className="from-clause">
          {" "}
          {sel("from", "Measured from", "from either of our homes", v.froms, (o) => `from ${o.label}`)}
        </span>
        , {sel("tag", "Mood", "any mood", v.tags, (o) => `feeling ${o.label}`)},{" "}
        {sel("energy", "Energy", "any energy", v.energies, (o) => `${o.label.toLowerCase()} energy`)}.
      </p>

      <div className="draw-row">
        {bothIn && (
          <button className="btn btn-stamp btn-draw btn-middle" type="button" disabled={busy} onClick={onDrawMiddle}>
            Draw for both of us
          </button>
        )}
        <button id="draw" className={`btn ${bothIn ? "btn-ink" : "btn-stamp btn-draw"}`} type="submit" disabled={busy}>
          {bothIn ? "Draw with my answers" : "Draw a slip"}
        </button>
        {shake.available && (
          <button id="shake" className="shake-opt" type="button" aria-pressed={shake.on} onClick={shake.toggle}>
            {shake.label}
          </button>
        )}
      </div>
      <p className="middle-link">
        <Link to="/joy/middle">Can't agree? Check in separately.</Link>
      </p>

      {v.turn_name && (
        <p className={`turn${v.my_turn ? " is-mine" : ""}`}>
          <svg className="turn-arrow" viewBox="0 0 40 24" aria-hidden="true" focusable="false">
            <path d="M36 20C26 21 14 17 8 6" />
            <path d="M4.6 10.4 7.6 5l5.6 2.2" />
          </svg>
          {v.my_turn ? <span>Your turn to draw, {v.me_name}. You get a bonus peek tonight.</span> : <span>It's {v.turn_name}'s turn to draw.</span>}
        </p>
      )}
      {v.nudge && (
        <p className={`nudge${v.nudge_soon ? " is-soon" : ""}`} role="note">
          {v.nudge_soon ? (
            <Ico d={["M3.6 5.2h16.8v15H3.6z", "M3.8 9.8h16.4M8 3.4v3.6M16 3.4v3.6", "M9 14.6l2 2 4-4.2"]} />
          ) : (
            <Ico d={["M12 20.3c-4.6-3.2-8.3-6.4-8.3-10.4 0-2.6 2-4.6 4.4-4.6 1.6 0 3 .9 3.9 2.2.9-1.3 2.3-2.2 3.9-2.2 2.4 0 4.4 2 4.4 4.6 0 4-3.7 7.2-8.3 10.4z"]} />
          )}
          <span>{v.nudge}</span>
        </p>
      )}
    </form>
  );
}
