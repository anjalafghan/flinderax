import { Link, useLocation, useNavigate } from "react-router-dom";
import { ClipboardCheck, HeartPulse, ListChecks, LogOut, Plus, Sun, UserRound, Users, Wallet, type LucideIcon } from "lucide-react";
import { cn } from "@/utils/cn";
import { Button } from "@/components/ui/button";
import { ModeToggle } from "@/components/ui/mode-toggle";
import { useAuth } from "@/context/AuthContext";

interface Tab {
  to: string;
  label: string;
  icon: LucideIcon;
  /** Paths that light this tab up, so the Money tab stays lit on the Accounts and Spending pages. */
  prefixes: string[];
}

const TABS: Tab[] = [
  { to: "/", label: "Today", icon: Sun, prefixes: ["/", "/cards"] },
  { to: "/money", label: "Money", icon: Wallet, prefixes: ["/money", "/accounts", "/spending"] },
  { to: "/plan", label: "Plan", icon: ListChecks, prefixes: ["/plan", "/what-if", "/working", "/life"] },
  { to: "/joy", label: "People", icon: Users, prefixes: ["/joy", "/people"] },
  { to: "/health", label: "Health", icon: HeartPulse, prefixes: ["/health"] },
  { to: "/review", label: "Review", icon: ClipboardCheck, prefixes: ["/review"] },
];

/** Inside the People tab: the two of you (Dates), and everyone else. */
const PEOPLE_SUBTABS = [
  { to: "/joy", label: "Us", prefixes: ["/joy"] },
  { to: "/people", label: "Circle", prefixes: ["/people"] },
];

/** The tab whose prefixes match the path. "/" only matches exactly, so it doesn't light up everything. */
function activeTab(pathname: string): Tab | undefined {
  return TABS.find((t) =>
    t.prefixes.some((p) => (p === "/" ? pathname === "/" : pathname === p || pathname.startsWith(`${p}/`) || pathname.startsWith(`${p}#`))),
  );
}

/** Us / Circle, shown under the top bar while the People tab is open. */
export function PeopleSubTabs() {
  const { pathname } = useLocation();
  if (activeTab(pathname)?.to !== "/joy") return null;
  return (
    <nav aria-label="People" className="mx-auto mt-4 flex w-full max-w-[1040px] gap-1 px-4 md:px-6">
      {PEOPLE_SUBTABS.map((t) => {
        const on = t.prefixes.some((p) => pathname === p || pathname.startsWith(`${p}/`));
        return (
          <Link
            key={t.to}
            to={t.to}
            aria-current={on ? "page" : undefined}
            className={cn(
              "inline-flex min-h-11 items-center rounded-full px-4 text-sm no-underline transition-colors",
              on ? "bg-[hsl(var(--chip))] font-medium text-foreground" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {t.label}
          </Link>
        );
      })}
    </nav>
  );
}

/** Top bar: italic wordmark, the six tabs as quiet pills from md up, and account actions. */
export function Header() {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { logout } = useAuth();
  const current = activeTab(pathname);

  const handleLogout = () => {
    logout();
    navigate("/auth");
  };

  return (
    <header className="relative z-50 mx-auto w-full max-w-[1040px] px-4 pt-7 md:px-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <Link to="/" className="wordmark text-[34px] leading-none no-underline" aria-label="joy, today">
          joy
        </Link>

        <nav aria-label="Main" className="hidden flex-wrap gap-1 md:flex">
          {TABS.map((t) => (
            <Link
              key={t.to}
              to={t.to}
              aria-current={current?.to === t.to ? "page" : undefined}
              className={cn(
                "inline-flex min-h-11 items-center rounded-full px-4 text-sm no-underline transition-colors",
                current?.to === t.to ? "bg-[hsl(var(--chip))] font-medium text-foreground" : "text-muted-foreground hover:text-foreground",
              )}
            >
              {t.label}
            </Link>
          ))}
        </nav>

        <div className="flex shrink-0 items-center gap-1.5">
          <Button variant="outline" onClick={() => navigate("/cards/new")} className="h-11 gap-2 rounded-full border-border px-3 sm:px-4" aria-label="Add card">
            <Plus className="h-4 w-4" strokeWidth={1.6} />
            <span className="hidden sm:inline">Add card</span>
          </Button>
          <ModeToggle />
          <Button variant="outline" size="icon" onClick={() => navigate("/account")} title="Account" aria-label="Account" className="h-11 w-11 rounded-full border-border">
            <UserRound className="h-[18px] w-[18px]" strokeWidth={1.5} />
          </Button>
          <Button variant="outline" size="icon" onClick={handleLogout} title="Sign out" aria-label="Sign out" className="h-11 w-11 rounded-full border-border">
            <LogOut className="h-[18px] w-[18px]" strokeWidth={1.5} />
          </Button>
        </div>
      </div>
      <PeopleSubTabs />
    </header>
  );
}

/** Phone bottom tab bar. Hidden from md up, where the top nav takes over. */
export function TabBar() {
  const { pathname } = useLocation();
  const current = activeTab(pathname);
  return (
    <nav
      aria-label="Main"
      className="fixed inset-x-3 bottom-[max(0.75rem,env(safe-area-inset-bottom))] z-40 md:hidden"
    >
      <ul className="mx-auto grid max-w-md grid-cols-6 rounded-full border border-border bg-card px-1">
        {TABS.map((t) => {
          const active = current?.to === t.to;
          return (
            <li key={t.to}>
              <Link
                to={t.to}
                aria-current={active ? "page" : undefined}
                className={cn(
                  "flex min-h-[44px] flex-col items-center justify-center gap-0.5 py-2 text-[11px] font-medium transition-colors",
                  active ? "text-foreground" : "text-muted-foreground hover:text-foreground",
                )}
              >
                <t.icon className="h-5 w-5" strokeWidth={active ? 2 : 1.5} aria-hidden />
                <span>{t.label}</span>
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
