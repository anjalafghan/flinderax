import { cn } from "@/utils/cn";
import { PILL } from "@/components/feature/people/ui";
import type { CatchUpStatus } from "@/services/people";

const LABEL: Record<CatchUpStatus, { text: string; tone: string }> = {
  overdue: { text: "Overdue", tone: "tone-over" },
  due_soon: { text: "Due soon", tone: "tone-warn" },
  ok: { text: "On track", tone: "tone-good" },
};

/** The catch-up state of one person. Renders nothing when no rhythm is set. */
export function StatusPill({ status }: { status: CatchUpStatus | null }) {
  if (status === null) return null;
  const s = LABEL[status];
  return <span className={cn(PILL, s.tone)}>{s.text}</span>;
}
