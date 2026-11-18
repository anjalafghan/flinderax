import { Link, useNavigate } from "react-router-dom";
import { LogOut, Moon, Sun } from "lucide-react";

import { Header } from "@/components/layout/Header";
import { AccountSection } from "@/components/feature/AccountSection";
import { IngestSetup } from "@/components/feature/IngestSetup";
import { SettingsSection } from "@/pages/PlanPage";
import { useAuth } from "@/context/AuthContext";
import { useTheme } from "@/context/ThemeContext";
import { cn } from "@/utils/cn";
import { DatesSettingsCards } from "@/joy/pages/SettingsPage";
import "@/joy/styles/index.css";
import "@/joy/styles/app.css";

/** Light or dark, as a two-way switch. It used to be a header button on every page. */
function Appearance() {
  const { theme, setTheme } = useTheme();
  const options = [
    { value: "light", label: "Light", icon: Sun },
    { value: "dark", label: "Dark", icon: Moon },
  ] as const;
  return (
    <div role="radiogroup" aria-label="Theme" className="inline-grid grid-flow-col auto-cols-fr gap-1 rounded-full border border-border bg-card p-1">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={theme === o.value}
          onClick={() => setTheme(o.value)}
          className={cn(
            "inline-flex h-10 items-center justify-center gap-2 rounded-full px-5 text-sm transition-colors",
            theme === o.value ? "bg-foreground font-medium text-background" : "text-muted-foreground hover:text-foreground",
          )}
        >
          <o.icon className="h-4 w-4" strokeWidth={1.6} aria-hidden />
          {o.label}
        </button>
      ))}
    </div>
  );
}

/** One place for settings: the account, alert forwarding, your partner's date share, and links to the rest. */
export default function AccountPage() {
  const { logout } = useAuth();
  const navigate = useNavigate();
  const signOut = () => {
    logout();
    navigate("/auth");
  };
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page page-narrow stack">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-sub">Appearance, alert forwarding, your partner, and your account.</p>
        </div>
        <section aria-label="Appearance" className="space-y-3">
          <h2 className="font-display text-[32px] leading-none">Appearance</h2>
          <Appearance />
        </section>
        <section aria-label="Alert forwarding" className="space-y-3">
          <h2 className="font-display text-[32px] leading-none">Alert forwarding</h2>
          <IngestSetup />
        </section>
        <section aria-label="Budget settings" className="space-y-3">
          <h2 className="font-display text-[32px] leading-none">Budget settings</h2>
          <SettingsSection />
        </section>
        <section aria-label="Partner and Dates" className="space-y-3">
          <div className="joy joy-embed">
            <DatesSettingsCards />
          </div>
        </section>
        <p className="text-sm text-muted-foreground">
          Category rules live with your spending:{" "}
          <Link to="/spending" className="underline underline-offset-4">Money › Spending › Rules</Link>.
        </p>
        <AccountSection />
        <button
          type="button"
          onClick={signOut}
          className="inline-flex h-12 w-full items-center justify-center gap-2 rounded-full border border-border text-sm font-medium text-destructive transition-colors hover:bg-[hsl(var(--chip))]"
        >
          <LogOut className="h-4 w-4" strokeWidth={1.6} aria-hidden />
          Sign out
        </button>
      </main>
    </div>
  );
}
