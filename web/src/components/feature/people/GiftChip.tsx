import type { Upcoming } from "@/services/people";
import { cn } from "@/utils/cn";

const CHIP: Record<Upcoming["gift_status"], { label: string; tone: string }> = {
  none: { label: "No gift yet", tone: "tone-warn" },
  idea: { label: "Gift idea", tone: "tone-quiet" },
  bought: { label: "Bought", tone: "tone-good" },
  given: { label: "Given", tone: "tone-good" },
};

/** The gift state for an upcoming date. A visible word, so the colour is never the only signal. */
export function GiftChip({ status }: { status: Upcoming["gift_status"] }) {
  const chip = CHIP[status];
  return (
    <span className={cn("inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold", chip.tone)}>
      {chip.label}
    </span>
  );
}
