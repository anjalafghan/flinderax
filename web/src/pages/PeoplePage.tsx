import { Header } from "@/components/layout/Header";
import { AddPersonForm } from "@/components/feature/people/AddPersonForm";
import { CatchUpSection } from "@/components/feature/people/CatchUpSection";
import { CircleList } from "@/components/feature/people/CircleList";
import { FollowUpsCard } from "@/components/feature/people/FollowUpsCard";
import { GiftPlanCard } from "@/components/feature/people/GiftPlanCard";
import { PeopleSettingsCard } from "@/components/feature/people/PeopleSettingsCard";
import { PeopleUpcoming } from "@/components/feature/people/PeopleUpcoming";
import { Retry } from "@/components/feature/people/Retry";
import { usePeople } from "@/services/people";

/** Circle: the people who matter, their dates, and a nudge when it's been a while. */
export default function PeoplePage() {
  const people = usePeople();
  const data = people.data;

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page max-w-3xl stack">
        <header>
          <h1 className="page-title">Circle</h1>
          <p className="page-sub">The people who matter, their dates, and a nudge when it's been a while.</p>
        </header>

        {people.isLoading && (
          <div className="h-48 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your circle" />
        )}
        {people.isError && !data && (
          <Retry message="Couldn't load your circle." onRetry={() => void people.refetch()} />
        )}

        {data && (
          <>
            <PeopleUpcoming upcoming={data.upcoming} />
            <CatchUpSection people={data.catch_up} />
            <FollowUpsCard />
            <CircleList people={data.people} archived={data.archived} />
            <AddPersonForm />
            <GiftPlanCard />
            <PeopleSettingsCard />
          </>
        )}
      </main>
    </div>
  );
}
