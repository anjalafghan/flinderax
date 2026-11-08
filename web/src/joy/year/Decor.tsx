// Fixed decoration for the year page: symbols, the fairy lights and the jar.
import type { JarSlip } from "./api";

export const Defs = () => (
  <svg className="y-defs" width="0" height="0" aria-hidden="true" focusable="false">
    <defs>
      <symbol id="y-heart" viewBox="0 0 24 22">
        <path d="M12 21s-8.5-5.4-10.6-10.3C-.2 6.4 2.7 1.5 7.2 1.5c2.2 0 3.8 1.1 4.8 2.7 1-1.6 2.6-2.7 4.8-2.7 4.5 0 7.4 4.9 5.8 9.2C20.5 15.6 12 21 12 21z" />
      </symbol>
      <symbol id="y-star" viewBox="0 0 24 24">
        <path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3 6.1 20.6l1.3-6.6L2.5 9.4l6.6-.8z" />
      </symbol>
      <symbol id="y-sparkle" viewBox="0 0 24 24">
        <path d="M12 2C13 9 15 11 22 12 15 13 13 15 12 22 11 15 9 13 2 12 9 11 11 9 12 2z" />
      </symbol>
    </defs>
  </svg>
);

const BULBS: Array<[number, number, string]> = [
  [80, 58, ".1s"], [160, 73, "1.2s"], [240, 83, ".6s"], [320, 88, "2s"], [400, 88, ".9s"], [480, 83, "1.6s"], [560, 73, ".3s"], [640, 58, "2.3s"],
  [720, 43, "1s"], [800, 31, ".5s"], [880, 24, "1.8s"], [960, 21, ".2s"], [1040, 22, "1.4s"], [1120, 26, "2.6s"], [1200, 35, ".8s"],
];

export const Lights = () => (
  <div className="y-sky" aria-hidden="true">
    <svg viewBox="0 0 1280 110" width="1280" height="110">
      <defs>
        <radialGradient id="y-bulb-glow">
          <stop offset="0" stopColor="#ffd58a" />
          <stop offset=".45" stopColor="#ffd58a" stopOpacity=".5" />
          <stop offset="1" stopColor="#ffd58a" stopOpacity="0" />
        </radialGradient>
      </defs>
      <path className="y-wire" d="M0,30 Q320,120 640,50 T1280,40" />
      <g className="y-bulbs">
        {BULBS.map(([x, y, d]) => (
          <g key={x}>
            <circle className="y-bulb" cx={x} cy={y} r="20" style={{ "--d": d } as React.CSSProperties} />
            <circle cx={x} cy={y} r="3.5" />
          </g>
        ))}
      </g>
    </svg>
  </div>
);

const FLIES: Array<[number, number, string]> = [[88, 150, ".2s"], [146, 128, "1.6s"], [118, 184, ".9s"], [168, 172, "2.4s"], [70, 196, "3.1s"], [128, 110, "1.2s"]];
const JAR_OUTLINE = "M84,62 L156,62 L156,82 C156,92 200,96 200,130 L200,250 C200,268 188,276 170,276 L70,276 C52,276 40,268 40,250 L40,130 C40,96 84,92 84,82 Z";

/** The jar itself: one folded slip per date, a few fireflies, a tag with the year. */
export function Jar({ year, dates, slips }: { year: number; dates: number; slips: JarSlip[] }) {
  return (
    <svg className="y-jar" viewBox="0 0 260 300" role="img" aria-label={`A glass jar holding ${dates} folded date ${dates === 1 ? "slip" : "slips"}`}>
      <defs>
        <radialGradient id="y-jar-glow" cx="50%" cy="58%" r="55%">
          <stop offset="0" stopColor="#ffcf8a" stopOpacity=".5" />
          <stop offset="1" stopColor="#ffcf8a" stopOpacity="0" />
        </radialGradient>
        <radialGradient id="y-fly-glow">
          <stop offset="0" stopColor="#ffe7a3" />
          <stop offset=".4" stopColor="#ffe7a3" stopOpacity=".5" />
          <stop offset="1" stopColor="#ffe7a3" stopOpacity="0" />
        </radialGradient>
        <clipPath id="y-jar-clip">
          <path d={JAR_OUTLINE} />
        </clipPath>
      </defs>
      <ellipse className="y-jar-glow" cx="120" cy="180" rx="135" ry="130" fill="url(#y-jar-glow)" />
      <ellipse className="y-jar-shadow" cx="120" cy="284" rx="96" ry="9" />
      <g clipPath="url(#y-jar-clip)">
        <rect className="y-jar-glass" width="260" height="300" />
        {slips.map((sl, i) => (
          <g key={i} className="y-jar-slip" data-kind={sl.kind} transform={`translate(${sl.x},${sl.y}) rotate(${sl.rot})`}>
            <rect width="42" height="18" rx="2" />
            <line x1="21" y1="0" x2="21" y2="18" />
          </g>
        ))}
        <g className="y-flies">
          {FLIES.map(([x, y, d]) => (
            <circle key={x} className="y-fly" cx={x} cy={y} r="10" style={{ "--d": d } as React.CSSProperties} />
          ))}
        </g>
      </g>
      <path className="y-jar-line" d={JAR_OUTLINE} />
      <path className="y-jar-shine" d="M54,136 C51,176 51,216 56,250" />
      <path className="y-jar-shine y-jar-shine-s" d="M66,120 C64,128 63,134 63,140" />
      <rect x="74" y="36" width="92" height="28" rx="6" fill="#c9915f" />
      <path d="M78,44 H162 M78,50 H162 M78,56 H162" stroke="#8f5d36" strokeOpacity=".55" strokeWidth="1.2" />
      <path d="M82,74 C110,82 130,82 158,74" fill="none" stroke="#d9c39a" strokeWidth="3" />
      <path d="M82,79 C110,87 130,87 158,79" fill="none" stroke="#c4ab7f" strokeWidth="2.5" />
      <path d="M158,76 C172,84 180,98 190,108" fill="none" stroke="#d9c39a" strokeWidth="2" />
      <g transform="translate(186,100) rotate(16)">
        <path d="M0,9 L9,0 L62,0 L62,38 L9,38 L0,29 Z" fill="#f4ead8" />
        <circle className="y-jar-hole" cx="9" cy="19" r="3.2" />
        <text x="37" y="27" className="y-jar-tag" textAnchor="middle">{year}</text>
      </g>
    </svg>
  );
}
