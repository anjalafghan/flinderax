import { Link } from "react-router-dom";
import { daysLeftInMonth, nudgeFor } from "./nudge";

interface Props {
  budget_paise: number | null | undefined;
  spent_paise: number;
  /** Defaults to the days left in this month. */
  daysLeft?: number;
}

/** One quiet line for the Today panel once Dates is at 80% of its month. Renders nothing before that. */
export function DatesNudge({ budget_paise, spent_paise, daysLeft }: Props) {
  const n = nudgeFor(budget_paise, spent_paise, daysLeft ?? daysLeftInMonth());
  if (!n) return null;
  return (
    <p role="status" className="text-sm text-muted-foreground">
      {n.text}.{" "}
      <Link to="/joy" className="underline underline-offset-2">
        Open Dates
      </Link>
    </p>
  );
}

export default DatesNudge;
