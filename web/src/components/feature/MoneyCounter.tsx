import Counter from "@/components/bits/Counter";

/**
 * A rupee amount that rolls like an odometer when it changes. It reads exactly like
 * `formatINR(paise)` (screen readers get that text; the rolling digits are hidden from them).
 */
export function MoneyCounter({
  paise,
  className,
}: {
  paise: number;
  className?: string;
}) {
  return (
    <Counter
      value={paise / 100}
      locale="en-IN"
      format={{ style: "currency", currency: "INR" }}
      decimals={paise % 100 === 0 ? 0 : 2}
      fontSize="1em"
      duration={0.9}
      stagger={0.05}
      className={className}
    />
  );
}
