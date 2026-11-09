import { useState } from "react";
import { Link } from "react-router-dom";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { rupees, useCheckinStatus, useReminderLink, type CheckinStatus } from "@/services/checkin";
import { formatDay } from "@/utils/dates";

const SHOWN_ESTIMATES = 2;

function statusLine(s: CheckinStatus): string {
  if (s.status === "overdue") return "Your check-in is due. It takes about 20 minutes.";
  if (s.status === "due_soon") return `Check-in due ${formatDay(s.next_due_on)}.`;
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
  const [calendarUrl, setCalendarUrl] = useState<string | null>(null);

  if (!data) {
    return isLoading ? (
      <div className="mt-6 h-40 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading check-in" />
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
      className="mt-6 flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10"
    >
      <header className="flex flex-col gap-2">
        <h2 id="checkin-card-title" className="m-0 font-display text-[32px] leading-none">
          Quarterly check-in
        </h2>
        <p className="m-0 text-sm text-muted-foreground">{statusLine(data)}</p>
      </header>

      {estimates.length > 0 && (
        <ul className="ledger">
          {estimates.map((e) => (
            <li key={e.account_id} className="ledger-row">
              <span className="text-[15px] font-medium">{`${e.name}: about ${rupees(e.estimate_paise)} (estimate)`}</span>
            </li>
          ))}
        </ul>
      )}

      <div>
        <Link
          to="/checkin"
          className="inline-flex min-h-11 items-center rounded-full bg-primary px-5 text-sm font-medium text-primary-foreground no-underline transition-colors hover:bg-primary/90"
        >
          {data.status === "ok" ? "Open check-in" : "Start check-in"}
        </Link>
      </div>

      <div className="flex flex-col gap-3">
        {calendarUrl ? (
          <>
            <label htmlFor="checkin-calendar-url" className="text-sm font-medium">
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
              <Button variant="outline" onClick={() => void copyLink(calendarUrl)}>
                Copy link
              </Button>
            </div>
            <p className="m-0 text-[13px] text-muted-foreground">
              Paste this into Google Calendar → Other calendars → From URL, or open it on your iPhone.
            </p>
          </>
        ) : (
          <Button
            variant="outline"
            className="self-start"
            onClick={getLink}
            disabled={reminders.isPending}
          >
            Add reminders to your calendar
          </Button>
        )}
      </div>
    </section>
  );
}
