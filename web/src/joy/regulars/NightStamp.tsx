// One night at a regular, as a rubber stamp on a passport page: the place inked round the edge, the
// day in the middle. Every stamp is a little different (shape, ink, how hard it was pressed).
import type { CSSProperties } from "react";

const INKS = ["#a8321f", "#1f4b91", "#2f6d57", "#6b3a8a", "#b06a12"];
const SHAPES = ["circle", "square", "octagon", "oval"] as const;

const OCT = "M30 4 H70 L96 30 V70 L70 96 H30 L4 70 V30 Z";

export function NightStamp({
  id,
  place,
  day,
  year,
}: {
  id: number;
  place: string;
  day: string;
  year: string;
}) {
  const ink = INKS[id % INKS.length]!;
  const shape = SHAPES[(id >> 1) % SHAPES.length]!;
  const ring = place.length > 22 ? place.slice(0, 21) + "…" : place;
  const ringId = `ns-r${id}`;
  return (
    <svg
      className="ns"
      viewBox="0 0 100 100"
      style={
        { "--ink": ink, "--rot": `${((id * 53) % 17) - 8}deg` } as CSSProperties
      }
      role="img"
      aria-label={`${place}, ${day} ${year}`}
    >
      <defs>
        <path
          id={ringId}
          d="M50 50 m-35 0 a35 35 0 1 1 70 0 a35 35 0 1 1 -70 0"
        />
      </defs>
      <g filter="url(#ns-wear)" fill={ink} stroke={ink}>
        {shape === "circle" && (
          <>
            <circle cx="50" cy="50" r="46" fill="none" strokeWidth="3" />
            <circle cx="50" cy="50" r="27" fill="none" strokeWidth="1.4" />
          </>
        )}
        {shape === "square" && (
          <>
            <rect
              x="5"
              y="5"
              width="90"
              height="90"
              rx="12"
              fill="none"
              strokeWidth="3"
            />
            <circle cx="50" cy="50" r="27" fill="none" strokeWidth="1.4" />
          </>
        )}
        {shape === "octagon" && (
          <>
            <path d={OCT} fill="none" strokeWidth="3" />
            <circle cx="50" cy="50" r="27" fill="none" strokeWidth="1.4" />
          </>
        )}
        {shape === "oval" && (
          <>
            <ellipse
              cx="50"
              cy="50"
              rx="47"
              ry="40"
              fill="none"
              strokeWidth="3"
            />
            <circle cx="50" cy="50" r="27" fill="none" strokeWidth="1.4" />
          </>
        )}
        <text stroke="none" fontSize="9" letterSpacing="1.6" fontWeight="700">
          <textPath href={`#${ringId}`} startOffset="50%" textAnchor="middle">
            {ring.toUpperCase()} ★
          </textPath>
        </text>
        <text
          stroke="none"
          x="50"
          y="53"
          textAnchor="middle"
          fontSize="13"
          fontWeight="800"
          className="ns-day"
        >
          {day}
        </text>
        <text
          stroke="none"
          x="50"
          y="65"
          textAnchor="middle"
          fontSize="7.5"
          letterSpacing="1.5"
          fontWeight="600"
        >
          {year}
        </text>
      </g>
    </svg>
  );
}

/** The worn-ink filter every stamp on the page shares. Render once. */
export function NightStampDefs() {
  return (
    <svg
      width="0"
      height="0"
      style={{ position: "absolute" }}
      aria-hidden="true"
      focusable="false"
    >
      <filter id="ns-wear" x="-5%" y="-5%" width="110%" height="110%">
        <feTurbulence
          type="fractalNoise"
          baseFrequency="1.1"
          numOctaves="2"
          seed="11"
          result="n"
        />
        <feColorMatrix
          in="n"
          values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  -1.8 0 0 0 1.55"
          result="m"
        />
        <feComposite in="SourceGraphic" in2="m" operator="in" />
      </filter>
    </svg>
  );
}
