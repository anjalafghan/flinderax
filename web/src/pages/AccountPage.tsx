import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { AccountSection } from "@/components/feature/AccountSection";
import { IngestSetup } from "@/components/feature/IngestSetup";
import { PartnerShare } from "@/components/feature/PartnerShare";

const OTHER_SETTINGS = [
  { to: "/plan", label: "Budget settings", hint: "Spending allowance, savings floor, recovery months" },
  { to: "/joy/settings", label: "Dates settings", hint: "Names, monthly date budget, weather, calendar" },
  { to: "/spending", label: "Category rules", hint: "How spending is sorted (under Rules)" },
];

/** One place for settings: the account, alert forwarding, your partner's date share, and links to the rest. */
export default function AccountPage() {
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl stack">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-sub">Your account, alert forwarding, and where the other settings live.</p>
        </div>
        <section aria-label="Alert forwarding" className="space-y-3">
          <h2 className="font-display text-[32px] leading-none">Alert forwarding</h2>
          <IngestSetup />
        </section>
        <section aria-label="Date budget share">
          <PartnerShare />
        </section>
        <section aria-label="More settings" className="space-y-2">
          <h2 className="font-display text-[32px] leading-none">More settings</h2>
          <ul className="grid gap-2">
            {OTHER_SETTINGS.map((s) => (
              <li key={s.to}>
                <Link to={s.to} className="flex min-h-[56px] flex-col justify-center rounded-2xl border border-border px-4 no-underline hover:bg-[hsl(var(--chip))]">
                  <span className="font-medium">{s.label}</span>
                  <span className="text-sm text-muted-foreground">{s.hint}</span>
                </Link>
              </li>
            ))}
          </ul>
        </section>
        <AccountSection />
      </main>
    </div>
  );
}
