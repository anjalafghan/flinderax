import { useEffect, useRef, useState } from "react";

import { formatINR } from "@/utils/money";
import { niceTicks, shortINR } from "./glance";

export interface BarSeries {
  name: string;
  /** A CSS colour, e.g. "hsl(var(--primary))". */
  color: string;
}

export interface BarRow {
  key: string;
  label: string;
  /** One value per series, in paise. */
  values: number[];
  /** Extra tooltip lines. */
  notes?: string[];
  /** Draws this row's bars in the "over" colour. */
  flag?: boolean;
}

function useWidth<T extends HTMLElement>(): [React.RefObject<T | null>, number] {
  const ref = useRef<T>(null);
  const [w, setW] = useState(0);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    setW(el.clientWidth);
    if (typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => setW(el.clientWidth));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);
  return [ref, w];
}

const H = 220;
const TOP = 18;
const BOTTOM = 24;

/**
 * Vertical bars, grouped when there is more than one series. Sized from its box, so it reads at
 * phone width; a value label prints over each bar when there is room, and pointing at (or
 * focusing) a column shows a tooltip with every value.
 */
export function Bars({ rows, series, label }: { rows: BarRow[]; series: BarSeries[]; label: string }) {
  const [ref, width] = useWidth<HTMLDivElement>();
  const [hover, setHover] = useState<{ i: number; x: number } | null>(null);
  const W = Math.max(260, width || 600);
  const max = Math.max(1, ...rows.flatMap((r) => r.values));
  const ticks = niceTicks(max);
  const top = ticks[ticks.length - 1];
  const left = Math.max(...ticks.map((t) => shortINR(t).length)) * 7 + 10;
  const plotW = W - left - 4;
  const step = rows.length ? plotW / rows.length : plotW;
  const groupW = step * 0.72;
  const barW = groupW / Math.max(1, series.length);
  const y = (v: number) => TOP + (H - TOP - BOTTOM) * (1 - v / top);
  const showValues = series.length === 1 && barW >= 26;
  const everyOther = step < 30;

  return (
    <div ref={ref} className="relative w-full" style={{ minHeight: H }}>
      {width > 0 && (
        <svg width="100%" viewBox={`0 0 ${W} ${H}`} role="img" aria-label={label} className="block overflow-visible">
          {ticks.map((t) => (
            <g key={t}>
              <line x1={left} x2={W - 4} y1={y(t)} y2={y(t)} stroke="hsl(var(--border))" />
              <text x={left - 8} y={y(t) + 4} textAnchor="end" className="fill-muted-foreground text-[11px] tabular-nums">
                {shortINR(t)}
              </text>
            </g>
          ))}
          {rows.map((r, i) => {
            const x0 = left + i * step + (step - groupW) / 2;
            return (
              <g key={r.key}>
                {r.values.map((v, s) => (
                  <rect
                    key={s}
                    x={x0 + s * barW + (series.length > 1 ? 1 : 0)}
                    width={Math.max(1, barW - (series.length > 1 ? 2 : 0))}
                    y={y(Math.max(0, v))}
                    height={Math.max(0, y(0) - y(Math.max(0, v)))}
                    rx={2}
                    fill={r.flag ? "hsl(var(--over))" : series[s]?.color}
                    opacity={hover && hover.i !== i ? 0.55 : 1}
                  />
                ))}
                {showValues && r.values[0] > 0 && (
                  <text x={x0 + groupW / 2} y={y(r.values[0]) - 5} textAnchor="middle" className="fill-muted-foreground text-[11px] tabular-nums">
                    {shortINR(r.values[0])}
                  </text>
                )}
                {(!everyOther || i % 2 === 0) && (
                  <text x={left + i * step + step / 2} y={H - 6} textAnchor="middle" className="fill-muted-foreground text-[11px]">
                    {r.label}
                  </text>
                )}
                <rect
                  x={left + i * step}
                  width={step}
                  y={TOP}
                  height={H - TOP - BOTTOM}
                  fill="transparent"
                  tabIndex={0}
                  aria-label={`${r.label}: ${r.values.map((v, s) => `${series[s]?.name} ${formatINR(v)}`).join(", ")}`}
                  onPointerMove={() => setHover({ i, x: ((left + i * step + step / 2) / W) * 100 })}
                  onPointerLeave={() => setHover(null)}
                  onFocus={() => setHover({ i, x: ((left + i * step + step / 2) / W) * 100 })}
                  onBlur={() => setHover(null)}
                  className="outline-none"
                />
              </g>
            );
          })}
        </svg>
      )}
      {hover && rows[hover.i] && (
        <div
          role="status"
          className="pointer-events-none absolute top-0 z-10 rounded-xl border border-border bg-popover px-3 py-2 text-xs shadow-lg"
          style={{ left: `${Math.min(70, Math.max(0, hover.x - 10))}%` }}
        >
          <div className="font-medium">{rows[hover.i].label}</div>
          {rows[hover.i].values.map((v, s) => (
            <div key={s} className="text-muted-foreground tabular-nums">
              {series[s]?.name} {formatINR(v)}
            </div>
          ))}
          {rows[hover.i].notes?.map((n) => (
            <div key={n} className="text-muted-foreground">
              {n}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function Legend({ series }: { series: BarSeries[] }) {
  return (
    <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
      {series.map((s) => (
        <span key={s.name} className="inline-flex items-center gap-1.5">
          <i className="inline-block h-2.5 w-2.5 rounded-sm" style={{ background: s.color }} />
          {s.name}
        </span>
      ))}
    </div>
  );
}
