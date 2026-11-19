// Strokes one of our own drawings (drawn-paths.ts) in the current colour, like a Lucide icon.
import type { SVGProps } from "react";
import { DRAWN } from "./drawn-paths";

export function Drawn({ name, ...rest }: { name: string } & SVGProps<SVGSVGElement>) {
  const paths = DRAWN[name] ?? [];
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" {...rest}>
      {paths.map((d, i) => <path key={i} d={d} />)}
    </svg>
  );
}
