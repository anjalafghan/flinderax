import { useId } from "react";
import { toast } from "sonner";

import { NOTE, PANEL, SECTION_TITLE, errorMessage } from "@/components/feature/people/ui";
import { savePeopleSettings, usePeopleMutation, usePeopleSettings } from "@/services/people";

/** The one setting on this screen: whether names appear in the calendar feed. */
export function PeopleSettingsCard() {
  const id = useId();
  const settings = usePeopleSettings();
  const save = usePeopleMutation(savePeopleSettings);
  const feedNames = settings.data?.feed_names ?? false;

  return (
    <section aria-labelledby="people-settings-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="people-settings-title" className={SECTION_TITLE}>
          Settings
        </h2>
      </header>
      <div className="flex flex-col gap-1">
        <label htmlFor={`${id}-feed-names`} className="flex min-h-11 cursor-pointer items-center gap-3 text-[15px] font-medium">
          <input
            id={`${id}-feed-names`}
            type="checkbox"
            aria-describedby={`${id}-feed-hint`}
            className="h-5 w-5 shrink-0 accent-[hsl(var(--primary))]"
            checked={feedNames}
            disabled={!settings.data || save.isPending}
            onChange={(e) => {
              const next = e.target.checked;
              save.mutate(
                { feed_names: next },
                {
                  onSuccess: () => toast.success(next ? "Names will show in your calendar." : "Names are kept out of your calendar."),
                  onError: (err) => toast.error(errorMessage(err)),
                },
              );
            }}
          />
          Show details in my calendar feed
        </label>
        <p id={`${id}-feed-hint`} className={NOTE}>
          Off, events just say a money, health or people task is due. On, they include names, accounts and amounts.
        </p>
      </div>
    </section>
  );
}
