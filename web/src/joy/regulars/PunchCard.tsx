// A regular, drawn as the loyalty card we'd keep in a wallet: its name printed big, the days it's
// on, and a hole punched for every night we went (ten to a card; full cards stack up behind).
import type { CSSProperties } from "react";
import Counter from "@/components/bits/Counter";

/** Fixed card stocks: the same in light and dark, like real card. [paper, ink, accent] */
const STOCKS: [string, string, string][] = [
  ["#efe2c6", "#3b2618", "#b5452c"], // kraft and rust
  ["#cfe0d6", "#1d3a32", "#2f6d57"], // sage
  ["#24407a", "#f4ecd8", "#f1c84b"], // cobalt and gold
  ["#e9b9a2", "#3d1d14", "#8a2f1c"], // terracotta
  ["#f1d77a", "#2e2410", "#a8321f"], // mustard
  ["#2c2a33", "#f1e9df", "#ff7a63"], // night
];
const stockOf = (seed: number) => STOCKS[Math.abs(seed) % STOCKS.length]!;

const HOLES = 10;

export function PunchCard({
  id,
  title,
  line,
  count,
  big = false,
}: {
  id: number;
  title: string;
  line: string;
  count: number;
  big?: boolean;
}) {
  const [paper, ink, accent] = stockOf(id);
  const full = Math.floor(count / HOLES);
  const punched = count === 0 ? 0 : count % HOLES || HOLES;
  const stacked = Math.min(
    3,
    count % HOLES === 0 && count > 0 ? full - 1 : full,
  );
  const tilt = ((id * 37) % 7) - 3;
  return (
    <span
      className={`pc${big ? " pc-big" : ""}`}
      style={
        {
          "--paper": paper,
          "--ink": ink,
          "--accent": accent,
          "--tilt": `${tilt * 0.6}deg`,
        } as CSSProperties
      }
    >
      {Array.from({ length: stacked }, (_, i) => (
        <span
          key={i}
          className="pc-under"
          style={{ "--u": i + 1 } as CSSProperties}
          aria-hidden="true"
        />
      ))}
      <span className="pc-card">
        <span className="pc-stub" aria-hidden="true">
          <span>admit</span>
          <b>2</b>
        </span>
        <span className="pc-body">
          {count > 0 && (
            // the running tally, rolling up like a ticket counter (the holes carry the label)
            <span className="pc-n" aria-hidden="true">
              <Counter value={count} fontSize="1em" duration={1.1} intro />
              <small>×</small>
            </span>
          )}
          <span className="pc-title">{title}</span>
          <span className="pc-line">{line}</span>
          <span
            className="pc-holes"
            role="img"
            aria-label={`${count} ${count === 1 ? "night" : "nights"} punched${full > 0 ? `, ${full} full ${full === 1 ? "card" : "cards"}` : ""}`}
          >
            {Array.from({ length: HOLES }, (_, i) => (
              <span
                key={i}
                className={i < punched ? "is-punched" : undefined}
                style={{ "--h": i } as CSSProperties}
              />
            ))}
          </span>
          {full > 0 && (
            <span className="pc-full" aria-hidden="true">
              {full} full {full === 1 ? "card" : "cards"} behind this one
            </span>
          )}
        </span>
      </span>
    </span>
  );
}
