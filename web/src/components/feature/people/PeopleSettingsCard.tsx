import { useId } from "react";
import { toast } from "sonner";

import { Label } from "@/components/ui/label";
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
      <div className="flex items-start gap-3">
        <input
          id={`${id}-feed-names`}
          type="checkbox"
          className="mt-1 h-5 w-5 shrink-0 accent-[hsl(var(--primary))]"
          checked={feedNames}
          disabled={!settings.data || save.isPending}
          onChange={(e) =>
            save.mutate(
              { feed_names: e.target.checked },
              {
                onSuccess: () => toast.success(e.target.checked ? "Names will show in your calendar." : "Names are kept out of your calendar."),
                onError: (err) => toast.error(errorMessage(err)),
              },
            )
          }
        />
        <div className="flex flex-col gap-1">
          <Label htmlFor={`${id}-feed-names`} className="text-[15px]">
            Show names in my calendar feed
          </Label>
          <p className={NOTE}>Off keeps names out of your synced calendar.</p>
        </div>
      </div>
    </section>
  );
}
