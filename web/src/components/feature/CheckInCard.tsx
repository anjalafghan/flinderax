import { useState } from "react";
import { Link } from "react-router-dom";
import { CalendarHeart, ClipboardCheck } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  rupees,
  useCheckinStatus,
  useReminderLink,
  useResetReminderLink,
  type CheckinStatus,
} from "@/services/checkin";
import { ResetButton } from "@/components/ui/reset-button";
import { formatDay } from "@/utils/dates";
import { Skeleton } from "@/components/ui/skeleton";

const SHOWN_ESTIMATES = 2;

function statusLine(s: CheckinStatus): string {
  if (s.status === "overdue")
    return "Your check-in is due. It takes about 20 minutes.";
  if (s.status === "due_soon")
    return `Check-in due ${formatDay(s.next_due_on)}.`;
  return `Next check-in ${formatDay(s.next_due_on)}.`;
}

async function copyLink(url: string) {
  try {
    await navigator.clipboard.writeText(url);
    toast.success("Link copied");
  } catch {
    toast.error("Couldn't copy the link. Select it and copy it by hand.");
  }
}

/** The quarterly check-in's entry point on the dashboard. Hidden when its data can't be read. */
export function CheckInCard() {
  const { data, isLoading } = useCheckinStatus();
  const reminders = useReminderLink();
  const resetLink = useResetReminderLink();
  const [calendarUrl, setCalendarUrl] = useState<string | null>(null);

  if (!data) {
    return isLoading ? (
      <Skeleton className="mt-6 h-40 rounded-[28px]" label="Loading check-in" />
    ) : null;
  }

  const estimates = data.estimates.slice(0, SHOWN_ESTIMATES);

  const getLink = () => {
    reminders.mutate(undefined, {
      onSuccess: (out) => setCalendarUrl(out.url),
      onError: () => toast.error("Couldn't get the calendar link. Try again."),
    });
  };

  return (
    <section
      aria-labelledby="checkin-card-title"
      className="glass mt-6 flex flex-col gap-6 rounded-[28px] px-6 py-8 sm:px-12 sm:py-10"
    >
      <header className="flex items-start gap-4">
        <span
          aria-hidden
          className="inline-flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-[linear-gradient(145deg,hsl(var(--c-sky)),hsl(var(--c-grape)))] text-background shadow-[0_10px_24px_-12px_hsl(var(--c-grape))]"
        >
          <ClipboardCheck className="h-6 w-6" strokeWidth={1.7} />
        </span>
        <div className="flex min-w-0 flex-col gap-2">
          <h2
            id="checkin-card-title"
            className="brand-text m-0 font-display text-[32px] leading-none"
          >
            Quarterly check-in
          </h2>
          <p
            className={
              data.status === "overdue"
                ? "m-0 self-start rounded-full bg-over-tint px-3 py-1 text-sm font-medium text-over"
                : data.status === "due_soon"
                  ? "m-0 self-start rounded-full bg-warn-tint px-3 py-1 text-sm font-medium text-warn"
                  : "m-0 text-sm text-muted-foreground"
            }
          >
            {statusLine(data)}
          </p>
        </div>
      </header>

      {estimates.length > 0 && (
        <ul className="ledger">
          {estimates.map((e) => (
            <li key={e.account_id} className="ledger-row gap-3">
              <span
                aria-hidden
                className="h-2.5 w-2.5 shrink-0 rounded-full bg-sky shadow-[0_0_10px_hsl(var(--c-sky))]"
              />
              <span className="min-w-0 flex-1 text-[15px] font-medium">{`${e.name}: about ${rupees(e.estimate_paise)} (estimate)`}</span>
            </li>
          ))}
        </ul>
      )}

      <div>
        <Link
          to="/checkin"
          className="bg-brand-gradient inline-flex min-h-11 items-center rounded-full px-5 text-sm font-semibold no-underline shadow-[0_10px_24px_-12px_hsl(var(--brand))] transition-[transform,box-shadow] hover:-translate-y-px hover:shadow-[0_14px_30px_-12px_hsl(var(--brand))]"
        >
          {data.status === "ok" ? "Open check-in" : "Start check-in"}
        </Link>
      </div>

      <div className="flex flex-col gap-3">
        {calendarUrl ? (
          <>
            <label
              htmlFor="checkin-calendar-url"
              className="text-sm font-medium"
            >
              Your calendar link
            </label>
            <div className="flex flex-col gap-2 sm:flex-row">
              <Input
                id="checkin-calendar-url"
                readOnly
                value={calendarUrl}
                onFocus={(e) => e.currentTarget.select()}
                className="font-mono text-[13px]"
              />
              <Button
                variant="outline"
                onClick={() => void copyLink(calendarUrl)}
              >
                Copy link
              </Button>
            </div>
            <ResetButton
              label="Make a new link"
              confirm="Make a new calendar link? The old one will stop working, so you will need to add the new one to your calendar."
              disabled={resetLink.isPending}
              onReset={() =>
                resetLink.mutate(undefined, {
                  onSuccess: (out) => {
                    setCalendarUrl(out.url);
                    toast.success(
                      "New link ready. The old one no longer works.",
                    );
                  },
                  onError: () =>
                    toast.error("Couldn't make a new link. Try again."),
                })
              }
            />
            <p className="m-0 text-[13px] text-muted-foreground">
              Paste this into Google Calendar → Other calendars → From URL, or
              open it on your iPhone.
            </p>
          </>
        ) : (
          <Button
            variant="outline"
            className="self-start"
            onClick={getLink}
            disabled={reminders.isPending}
          >
            <CalendarHeart className="mr-1.5 h-4 w-4 text-rose" aria-hidden />
            Add reminders to your calendar
          </Button>
        )}
      </div>
    </section>
  );
}
