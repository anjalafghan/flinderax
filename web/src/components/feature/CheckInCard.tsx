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

  if (data.status === "ok") return null;

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
      className="flex flex-col gap-3 rounded-[22px] border border-border bg-card px-5 py-4 sm:flex-row sm:flex-wrap sm:items-center sm:gap-x-5 sm:gap-y-2"
    >
      <header className="flex min-w-0 flex-1 items-center gap-3">
        <span
          aria-hidden
          className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-border bg-secondary text-muted-foreground"
        >
          <ClipboardCheck className="h-5 w-5" strokeWidth={1.7} />
        </span>
        <div className="flex min-w-0 flex-col gap-2">
          <h2
            id="checkin-card-title"
            className="m-0 text-[16px] font-semibold leading-tight"
          >
            Quarterly check-in
          </h2>
          <p className="m-0 text-sm text-muted-foreground">
            {statusLine(data)}
          </p>
        </div>
        <Link
          to="/checkin"
          className="bg-brand-gradient inline-flex min-h-10 shrink-0 items-center justify-center rounded-full px-5 text-sm font-semibold no-underline sm:ml-auto"
        >
          Open check-in
        </Link>
      </header>

      <details className="text-sm sm:basis-full sm:pl-[60px]">
        <summary className="min-h-10 cursor-pointer content-center text-muted-foreground">
          More check-in options
        </summary>
        <div className="mt-3 flex flex-col gap-4">
          {estimates.length > 0 && (
            <ul className="ledger">
              {estimates.map((e) => (
                <li key={e.account_id} className="ledger-row gap-3">
                  <span className="min-w-0 flex-1 text-[14px] font-medium">
                    {`${e.name}: about ${rupees(e.estimate_paise)} (estimate)`}
                  </span>
                </li>
              ))}
            </ul>
          )}
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
      </details>
    </section>
  );
}
