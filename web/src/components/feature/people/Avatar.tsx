import { cn } from "@/utils/cn";
import { avatarHue, initials, STATUS_RING } from "@/components/feature/people/ui";
import type { CatchUpStatus } from "@/services/people";

/** A round initials badge in the person's own colour, ringed by how the catch-up is going. Decorative. */
export function Avatar({
  name,
  status,
  size = "md",
}: {
  name: string;
  status: CatchUpStatus | null;
  size?: "md" | "lg";
}) {
  const hue = avatarHue(name);
  return (
    <span
      aria-hidden
      className={cn(
        "grid shrink-0 place-items-center rounded-full font-display font-semibold ring-[3px] ring-offset-2 ring-offset-card",
        size === "lg" ? "h-16 w-16 text-2xl" : "h-10 w-10 text-[15px]",
        status ? STATUS_RING[status] : "ring-border",
      )}
      style={{
        background: `linear-gradient(140deg, hsl(var(--c-${hue}) / 0.28), hsl(var(--c-${hue}) / 0.12))`,
        color: `color-mix(in srgb, hsl(var(--c-${hue})) 62%, hsl(var(--foreground)))`,
      }}
    >
      {initials(name)}
    </span>
  );
}
