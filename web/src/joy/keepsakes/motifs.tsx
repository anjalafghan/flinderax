// Draws a catalogue motif: a Lucide icon by name (icons.ts) or one of our own (x:name).
import { ICONS } from "./icons";
import { Drawn } from "./drawn";

export function Motif({ name }: { name: string }) {
  if (name.startsWith("x:")) return <Drawn name={name.slice(2)} />;
  const Icon = ICONS[name];
  return Icon ? <Icon aria-hidden="true" strokeWidth={1.6} /> : null;
}
