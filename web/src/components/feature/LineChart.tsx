import { formatCompactINR } from "@/utils/money";
import { cn } from "@/utils/cn";

export interface ChartSeries {
  label: string;
  values: number[];
  /** Tailwind stroke colour class, e.g. "stroke-emerald-500". */
  stroke: string;
  dashed?: boolean;
}

export interface ChartMarker {
  index: number;
  label: string;
}

const W = 640;
const H = 260;
const PAD = { l: 52, r: 12, t: 12, b: 28 };

/** A small dependency-free line chart. Money in paise; markers flag months to worry about. */
export function LineChart({ labels, series, markers = [] }: { labels: string[]; series: ChartSeries[]; markers?: ChartMarker[] }) {
  const all = series.flatMap((s) => s.values);
  if (labels.length < 2 || all.length === 0) return null;
  let min = Math.min(0, ...all);
  let max = Math.max(0, ...all);
  if (max === min) max = min + 1;
  const pad = (max - min) * 0.05;
  min -= min < 0 ? pad : 0;
  max += pad;
  const x = (i: number) => PAD.l + (i / (labels.length - 1)) * (W - PAD.l - PAD.r);
  const y = (v: number) => PAD.t + (1 - (v - min) / (max - min)) * (H - PAD.t - PAD.b);
  const ticks = Array.from({ length: 5 }, (_, i) => min + ((max - min) * i) / 4);
  const every = Math.ceil(labels.length / 8);

  return (
    <figure>
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Projection chart" className="w-full">
        {ticks.map((t) => (
          <g key={t}>
            <line x1={PAD.l} x2={W - PAD.r} y1={y(t)} y2={y(t)} className="stroke-border" strokeWidth={1} />
            <text x={PAD.l - 6} y={y(t) + 4} textAnchor="end" className="fill-muted-foreground" fontSize={10}>
              {formatCompactINR(Math.round(t))}
            </text>
          </g>
        ))}
        {min < 0 && <line x1={PAD.l} x2={W - PAD.r} y1={y(0)} y2={y(0)} className="stroke-muted-foreground" strokeWidth={1} />}
        {labels.map((l, i) =>
          i % every === 0 ? (
            <text key={l} x={x(i)} y={H - 8} textAnchor="middle" className="fill-muted-foreground" fontSize={10}>
              {l}
            </text>
          ) : null,
        )}
        {series.map((s) => (
          <polyline
            key={s.label}
            fill="none"
            strokeWidth={s.dashed ? 1.5 : 2.5}
            strokeDasharray={s.dashed ? "5 4" : undefined}
            strokeLinejoin="round"
            className={cn(s.stroke, s.dashed && "opacity-70")}
            points={s.values.map((v, i) => `${x(i)},${y(v)}`).join(" ")}
          />
        ))}
        {markers.map((m) => (
          <g key={`${m.index}-${m.label}`}>
            <circle cx={x(m.index)} cy={H - PAD.b} r={4} className="fill-rose-500" />
            <title>{m.label}</title>
          </g>
        ))}
      </svg>
      <figcaption className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        {series.map((s) => (
          <span key={s.label} className="inline-flex items-center gap-1.5">
            <svg width="18" height="6" aria-hidden>
              <line x1="0" x2="18" y1="3" y2="3" strokeWidth={s.dashed ? 1.5 : 2.5} strokeDasharray={s.dashed ? "4 3" : undefined} className={s.stroke} />
            </svg>
            {s.label}
          </span>
        ))}
        {markers.length > 0 && (
          <span className="inline-flex items-center gap-1.5">
            <span className="inline-block h-2 w-2 rounded-full bg-rose-500" /> A month you'd fall short
          </span>
        )}
      </figcaption>
    </figure>
  );
}
