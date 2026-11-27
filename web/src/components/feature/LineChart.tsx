import { useId } from "react";

import { formatCompactINR } from "@/utils/money";
import { cn } from "@/utils/cn";

export interface ChartSeries {
  label: string;
  values: number[];
  /** Tailwind stroke colour class, e.g. "stroke-good". */
  stroke: string;
  dashed?: boolean;
  /** Optional CSS colour for a soft area under the line, e.g. "hsl(var(--c-mint))". */
  area?: string;
}

export interface ChartMarker {
  index: number;
  label: string;
}

const W = 640;
const H = 260;
// Axis text is sized by class: the 640-wide viewBox shrinks to about 300px on a phone, so labels
// there are drawn at about twice the size (and every other month label is dropped) to stay legible.
const PAD = { l: 72, r: 12, t: 12, b: 34 };

/** A small dependency-free line chart. Money in paise; markers flag months to worry about. */
export function LineChart({
  labels,
  series,
  markers = [],
  ariaLabel = "Projection chart",
  markerLabel = "A month you'd fall short",
}: {
  labels: string[];
  series: ChartSeries[];
  markers?: ChartMarker[];
  ariaLabel?: string;
  /** What a marker dot means, for the legend. */
  markerLabel?: string;
}) {
  const gid = useId().replace(/:/g, "");
  const all = series.flatMap((s) => s.values);
  if (labels.length < 2 || all.length === 0) return null;
  let min = Math.min(0, ...all);
  let max = Math.max(0, ...all);
  if (max === min) max = min + 1;
  const pad = (max - min) * 0.05;
  min -= min < 0 ? pad : 0;
  max += pad;
  const x = (i: number) =>
    PAD.l + (i / (labels.length - 1)) * (W - PAD.l - PAD.r);
  const y = (v: number) =>
    PAD.t + (1 - (v - min) / (max - min)) * (H - PAD.t - PAD.b);
  const ticks = Array.from(
    { length: 5 },
    (_, i) => min + ((max - min) * i) / 4,
  );
  const every = Math.ceil(labels.length / 8);

  return (
    <figure>
      <svg
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={ariaLabel}
        className="w-full"
      >
        {ticks.map((t) => (
          <g key={t}>
            <line
              x1={PAD.l}
              x2={W - PAD.r}
              y1={y(t)}
              y2={y(t)}
              className="stroke-border"
              strokeWidth={1}
            />
            <text
              x={PAD.l - 8}
              y={y(t) + 4}
              textAnchor="end"
              className="fill-muted-foreground text-[20px] md:text-[11px]"
              fontSize={10}
            >
              {formatCompactINR(Math.round(t))}
            </text>
          </g>
        ))}
        {min < 0 && (
          <line
            x1={PAD.l}
            x2={W - PAD.r}
            y1={y(0)}
            y2={y(0)}
            className="stroke-muted-foreground"
            strokeWidth={1}
          />
        )}
        {labels.map((l, i) =>
          i % every === 0 ? (
            <text
              key={l}
              x={x(i)}
              y={H - 8}
              textAnchor={i === 0 ? "start" : i === labels.length - 1 ? "end" : "middle"}
              className={cn(
                "fill-muted-foreground text-[20px] md:text-[11px]",
                i % (every * 2) !== 0 && "max-md:hidden",
              )}
              fontSize={10}
            >
              {l}
            </text>
          ) : null,
        )}
        {series.map((s, si) =>
          s.area ? (
            <g key={`area-${s.label}`} aria-hidden>
              <defs>
                <linearGradient
                  id={`${gid}-a${si}`}
                  x1="0"
                  x2="0"
                  y1="0"
                  y2="1"
                >
                  <stop offset="0%" stopColor={s.area} stopOpacity={0.35} />
                  <stop offset="100%" stopColor={s.area} stopOpacity={0} />
                </linearGradient>
              </defs>
              <polygon
                fill={`url(#${gid}-a${si})`}
                points={`${x(0)},${y(Math.max(min, 0))} ${s.values
                  .map((v, i) => `${x(i)},${y(v)}`)
                  .join(" ")} ${x(s.values.length - 1)},${y(Math.max(min, 0))}`}
              />
              <circle
                cx={x(s.values.length - 1)}
                cy={y(s.values[s.values.length - 1])}
                r={5}
                fill={s.area}
                stroke="hsl(var(--card))"
                strokeWidth={2}
              />
            </g>
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
            <circle
              cx={x(m.index)}
              cy={H - PAD.b}
              r={4}
              className="fill-over"
            />
            <title>{m.label}</title>
          </g>
        ))}
      </svg>
      <figcaption className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        {series.map((s) => (
          <span key={s.label} className="inline-flex items-center gap-1.5">
            <svg width="18" height="6" aria-hidden>
              <line
                x1="0"
                x2="18"
                y1="3"
                y2="3"
                strokeWidth={s.dashed ? 1.5 : 2.5}
                strokeDasharray={s.dashed ? "4 3" : undefined}
                className={s.stroke}
              />
            </svg>
            {s.label}
          </span>
        ))}
        {markers.length > 0 && (
          <span className="inline-flex items-center gap-1.5">
            <span className="inline-block h-2 w-2 rounded-full bg-over" />{" "}
            {markerLabel}
          </span>
        )}
      </figcaption>
    </figure>
  );
}
