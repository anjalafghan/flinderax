import Counter from "@/components/bits/Counter";

// Counter keeps its own screen-reader copy of the formatted value, so these read exactly
// like formatINR and the whole-percent text they replace.

/** Rupees that roll into place. Same text as formatINR: paise only when the amount has them. */
export function RupeeOdometer({
  paise,
  className,
}: {
  paise: number;
  className?: string;
}) {
  const digits = paise % 100 === 0 ? 0 : 2;
  return (
    <Counter
      value={paise / 100}
      locale="en-IN"
      format={{
        style: "currency",
        currency: "INR",
        minimumFractionDigits: digits,
        maximumFractionDigits: digits,
      }}
      fontSize="1em"
      duration={0.9}
      intro
      className={className}
    />
  );
}

/** A whole percent that rolls into place ("28%"). */
export function PercentOdometer({
  percent,
  className,
}: {
  percent: number;
  className?: string;
}) {
  return (
    <Counter
      value={Math.round(percent)}
      locale="en-IN"
      suffix="%"
      fontSize="1em"
      duration={0.9}
      intro
      className={className}
    />
  );
}
