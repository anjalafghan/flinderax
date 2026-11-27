import { Link, useNavigate } from "react-router-dom";
import {
  LogOut,
  MessageSquareText,
  Moon,
  Palette,
  PiggyBank,
  Sun,
  type LucideIcon,
} from "lucide-react";

import { Header } from "@/components/layout/Header";
import { AccountSection } from "@/components/feature/AccountSection";
import { IngestSetup } from "@/components/feature/IngestSetup";
import { SettingsSection } from "@/pages/PlanPage";
import { useAuth } from "@/context/AuthContext";
import { useTheme } from "@/context/ThemeContext";
import { cn } from "@/utils/cn";
import { DatesSettingsCards } from "@/joy/pages/SettingsPage";
import { ApiTokensCard } from "@/components/feature/ApiTokensCard";
import "@/joy/styles/index.css";
import "@/joy/styles/app.css";

/** Light or dark, as a two-way switch: a sunny pill by day, a starry one by night. */
function Appearance() {
  const { theme, setTheme } = useTheme();
  const options = [
    {
      value: "light",
      label: "Light",
      icon: Sun,
      on: "bg-gradient-to-r from-sun to-coral text-background shadow-[0_8px_20px_-10px_hsl(var(--c-sun))]",
      off: "text-[hsl(var(--foreground))] dark:text-sun",
    },
    {
      value: "dark",
      label: "Dark",
      icon: Moon,
      on: "bg-gradient-to-r from-grape to-sky text-background shadow-[0_8px_20px_-10px_hsl(var(--c-grape))]",
      off: "text-grape",
    },
  ] as const;
  return (
    <div
      role="radiogroup"
      aria-label="Theme"
      className="inline-grid grid-flow-col auto-cols-fr gap-1 rounded-full border border-brand/25 bg-card p-1"
    >
      {options.map((o) => {
        const on = theme === o.value;
        return (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={on}
            onClick={() => setTheme(o.value)}
            className={cn(
              "group inline-flex h-11 items-center justify-center gap-2 rounded-full px-5 text-sm transition-all duration-300",
              on ? cn("font-semibold", o.on) : cn("hover:bg-brand-tint", o.off),
            )}
          >
            <o.icon
              className={cn(
                "h-4 w-4 transition-transform duration-500",
                on ? "rotate-0 scale-110" : "group-hover:rotate-45",
              )}
              strokeWidth={1.8}
              aria-hidden
            />
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

/** A settings heading with its own coloured icon bubble. */
function SectionHeading({
  icon: Icon,
  bubble,
  children,
}: {
  icon: LucideIcon;
  bubble: string;
  children: React.ReactNode;
}) {
  return (
    <h2 className="flex items-center gap-3 font-display text-[32px] leading-none">
      <span
        aria-hidden
        className={cn(
          "flex h-11 w-11 shrink-0 items-center justify-center rounded-full",
          bubble,
        )}
      >
        <Icon className="h-5 w-5" strokeWidth={1.8} />
      </span>
      {children}
    </h2>
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
          <p className="page-sub">
            Appearance, alert forwarding, your partner, and your account.
          </p>
        </div>
        <section aria-label="Appearance" className="space-y-3">
          <SectionHeading
            icon={Palette}
            bubble="bg-gradient-to-br from-sun/30 to-grape/25 text-coral"
          >
            Appearance
          </SectionHeading>
          <Appearance />
        </section>
        <section aria-label="Alert forwarding" className="space-y-3">
          <SectionHeading icon={MessageSquareText} bubble="bg-sky/18 text-sky">
            Alert forwarding
          </SectionHeading>
          <IngestSetup />
        </section>
        <section aria-label="Budget settings" className="space-y-3">
          <SectionHeading icon={PiggyBank} bubble="bg-mint/18 text-mint">
            Budget settings
          </SectionHeading>
          <SettingsSection />
        </section>
        <ApiTokensCard />
        <section aria-label="Partner and Dates" className="space-y-3">
          <div className="joy joy-embed">
            <DatesSettingsCards />
          </div>
        </section>
        <p className="text-sm text-muted-foreground">
          Category rules live with your spending:{" "}
          <Link
            to="/spending"
            className="inline-flex min-h-10 items-center font-medium text-brand-ink underline decoration-brand/40 underline-offset-4 md:min-h-0"
          >
            Money › Spending › Rules
          </Link>
          .
        </p>
        <AccountSection />
        <div className="rounded-[28px] border border-coral/35 bg-gradient-to-br from-coral/12 via-rose/6 to-transparent p-4">
          <button
            type="button"
            onClick={signOut}
            className="group inline-flex h-12 w-full items-center justify-center gap-2 rounded-full border border-coral/40 bg-card text-sm font-semibold text-over transition-colors hover:bg-coral hover:text-background"
          >
            <LogOut
              className="h-4 w-4 transition-transform duration-300 group-hover:translate-x-0.5"
              strokeWidth={1.8}
              aria-hidden
            />
            Sign out
          </button>
        </div>
      </main>
    </div>
  );
}
