import { Link, useParams } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { CatchUpActions } from "@/components/feature/people/CatchUpActions";
import { PersonDates } from "@/components/feature/people/PersonDates";
import { PersonEditForm } from "@/components/feature/people/PersonEditForm";
import { PersonFollowUps } from "@/components/feature/people/PersonFollowUps";
import { PersonGifts } from "@/components/feature/people/PersonGifts";
import { PersonTouches } from "@/components/feature/people/PersonTouches";
import { Retry } from "@/components/feature/people/Retry";
import { NOTE, PANEL, TEXT_LINK } from "@/components/feature/people/ui";
import { StatusPill } from "@/components/feature/people/StatusPill";
import {
  circleLabel,
  everyLabel,
  lastCaughtUpLabel,
  usePerson,
} from "@/services/people";

/** One person: their dates, gifts and catch-ups, and the edit form. Names appear only on this page. */
export default function PersonPage() {
  const { id = "" } = useParams();
  const detail = usePerson(id);
  const data = detail.data;

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page max-w-3xl stack">
        <Link to="/people" className={`${TEXT_LINK} inline-flex min-h-11 items-center`}>
          Back to Circle
        </Link>

        {detail.isLoading && (
          <div className="h-48 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading this person" />
        )}
        {detail.isError && !data && (
          <Retry message="Couldn't load this person." onRetry={() => void detail.refetch()} />
        )}

        {data && (
          <>
            <header className="flex flex-col gap-3">
              <div className="flex flex-wrap items-center gap-3">
                <h1 className="page-title">{data.person.name}</h1>
                <StatusPill status={data.person.status} />
              </div>
              <p className="page-sub">
                {[circleLabel(data.person.circle), everyLabel(data.person.every_days), lastCaughtUpLabel(data.person.last_contact_on)]
                  .filter(Boolean)
                  .join(" · ")}
              </p>
              {data.person.archived && (
                <p className={NOTE}>This person is archived. Bring them back from the edit form below.</p>
              )}
            </header>

            <section aria-labelledby="person-catch-up-title" className={PANEL}>
              <h2 id="person-catch-up-title" className="m-0 font-display text-[32px] leading-none">
                Catch up
              </h2>
              <CatchUpActions personId={data.person.id} name={data.person.name} />
            </section>

            <div className={PANEL}>
              <PersonFollowUps personId={data.person.id} />
              <PersonDates personId={data.person.id} personName={data.person.name} dates={data.dates} />
              <PersonGifts personId={data.person.id} gifts={data.gifts} dates={data.dates} />
              <PersonTouches touches={data.touches} personId={data.person.id} />
              <PersonEditForm person={data.person} />
            </div>
          </>
        )}
      </main>
    </div>
  );
}
