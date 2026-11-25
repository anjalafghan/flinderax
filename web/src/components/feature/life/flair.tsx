import Counter from "@/components/bits/Counter";
import { cn } from "@/utils/cn";

/**
 * Whole rupees that roll in like an odometer. Screen readers (and tests) read `text`, the amount
 * formatted as everywhere else; the rolling digits are hidden from them. Negative amounts stay plain.
 */
export function RupeeOdometer({
  paise,
  text,
  className,
}: {
  paise: number;
  text: string;
  className?: string;
}) {
  const r = Math.round(paise / 100);
  if (r < 0) return <span className={className}>{text}</span>;
  return (
    <span className={cn("inline-flex items-baseline", className)}>
      <span className="sr-only">{text}</span>
      <span aria-hidden className="inline-flex items-baseline">
        ₹
        <Counter
          value={r}
          locale="en-IN"
          format={{ maximumFractionDigits: 0 }}
          fontSize="1em"
        />
      </span>
    </span>
  );
}

/** A small whole number that rolls in, with its real text for screen readers. */
export function CountOdometer({
  value,
  className,
}: {
  value: number;
  className?: string;
}) {
  return (
    <span className={cn("inline-flex items-baseline", className)}>
      <span className="sr-only">{value}</span>
      <span aria-hidden className="inline-flex items-baseline">
        <Counter value={Math.max(0, value)} fontSize="1em" />
      </span>
    </span>
  );
}

/** A progress ring: `pct` from 0 to 100, in one party colour. Decorative; say the number in text beside it. */
export function Ring({
  pct,
  color,
  size = 44,
  stroke = 5,
  children,
}: {
  pct: number;
  color: string;
  size?: number;
  stroke?: number;
  children?: React.ReactNode;
}) {
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  const p = Math.max(0, Math.min(100, pct));
  return (
    <span
      aria-hidden
      className="relative inline-grid shrink-0 place-items-center"
      style={{ width: size, height: size }}
    >
      <svg width={size} height={size} className="-rotate-90">
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="hsl(var(--chip))"
          strokeWidth={stroke}
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke={color}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - p / 100)}
          className="transition-[stroke-dashoffset] duration-700 ease-out"
        />
      </svg>
      {children && (
        <span className="absolute inset-0 grid place-items-center text-[11px] font-semibold tabular-nums">
          {children}
        </span>
      )}
    </span>
  );
}
