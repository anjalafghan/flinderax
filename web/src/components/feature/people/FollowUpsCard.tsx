import { FollowUpRow } from "@/components/feature/people/FollowUpRow";
import { NOTE, PANEL, SECTION_TITLE } from "@/components/feature/people/ui";
import { useFollowUps } from "@/services/people";

/** Everything you meant to follow up on, across the circle. Hidden until there is something to do. */
export function FollowUpsCard() {
  const { data } = useFollowUps();
  const open = data?.open ?? [];
  if (open.length === 0) return null;
  const overdue = open.filter((f) => f.state === "overdue").length;
  return (
    <section aria-labelledby="followups-title" className={PANEL}>
      <h2 id="followups-title" className={SECTION_TITLE}>
        Follow-ups
      </h2>
      <p className={NOTE}>
        {overdue > 0 ? `${overdue} overdue. ` : ""}
        {open.length} {open.length === 1 ? "thing" : "things"} to follow up on.
      </p>
      <ul className="ledger">
        {open.map((f) => (
          <FollowUpRow key={f.id} f={f} showName />
        ))}
      </ul>
    </section>
  );
}
