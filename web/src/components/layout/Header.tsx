import { Link, useLocation, useNavigate } from "react-router-dom";
import { ClipboardCheck, Heart, ListChecks, LogOut, Plus, Sun, Wallet, type LucideIcon } from "lucide-react";
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

export const TABS: Tab[] = [
  { to: "/", label: "Today", icon: Sun, prefixes: ["/", "/cards"] },
  { to: "/money", label: "Money", icon: Wallet, prefixes: ["/money", "/accounts", "/spending"] },
  { to: "/plan", label: "Plan", icon: ListChecks, prefixes: ["/plan", "/what-if", "/working"] },
  { to: "/joy", label: "Dates", icon: Heart, prefixes: ["/joy"] },
  { to: "/review", label: "Review", icon: ClipboardCheck, prefixes: ["/review"] },
];

/** The tab whose prefixes match the path. "/" only matches exactly, so it doesn't light up everything. */
export function activeTab(pathname: string): Tab | undefined {
  return TABS.find((t) =>
    t.prefixes.some((p) => (p === "/" ? pathname === "/" : pathname === p || pathname.startsWith(`${p}/`) || pathname.startsWith(`${p}#`))),
  );
}

/** "nov 2026": the quiet dateline beside the wordmark. */
function dateline() {
  return new Date().toLocaleDateString("en-GB", { month: "short", year: "numeric" }).toLowerCase();
}

/** Top bar: wordmark and dateline, the five tabs from md up, and account actions. */
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
    <header className="relative z-50 mx-auto w-full max-w-[1200px] px-4 pt-6 md:px-6 md:pt-8">
      <div className="flex flex-wrap items-center justify-between gap-x-5 gap-y-3 border-b border-border pb-5">
        <Link to="/" className="flex items-baseline gap-3.5" aria-label="joy, today">
          <span className="font-display text-[40px] leading-none">joy</span>
          <span className="eyebrow hidden sm:inline">{dateline()}</span>
        </Link>

        <nav aria-label="Main" className="hidden items-center gap-1.5 md:flex">
          {TABS.map((t) => (
            <Link
              key={t.to}
              to={t.to}
              aria-current={current?.to === t.to ? "page" : undefined}
              className={cn(
                "inline-flex min-h-11 items-center rounded-full border border-transparent px-4 text-sm transition-colors hover:text-foreground",
                current?.to === t.to ? "border-border bg-foreground/[0.09] text-foreground" : "text-muted-foreground",
              )}
            >
              {t.label}
            </Link>
          ))}
        </nav>

        <div className="flex shrink-0 items-center gap-2">
          <Button variant="outline" onClick={() => navigate("/cards/new")} className="h-11 rounded-full px-3 sm:px-[18px]" aria-label="Add card">
            <Plus className="h-4 w-4 sm:mr-2" />
            <span className="hidden sm:inline">Add card</span>
          </Button>
          <ModeToggle />
          <Button variant="outline" size="icon" onClick={handleLogout} title="Sign out" aria-label="Sign out" className="h-11 w-11 rounded-full">
            <LogOut className="h-[18px] w-[18px]" />
          </Button>
        </div>
      </div>
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
      <ul className="glass mx-auto grid max-w-md grid-cols-5 rounded-full bg-card/90 px-1 backdrop-blur-xl">
        {TABS.map((t) => {
          const active = current?.to === t.to;
          return (
            <li key={t.to}>
              <Link
                to={t.to}
                aria-current={active ? "page" : undefined}
                className={cn(
                  "flex min-h-[44px] flex-col items-center justify-center gap-0.5 py-2 text-[11px] font-medium transition-colors",
                  active ? "text-primary [&_svg]:drop-shadow-[0_0_8px_hsl(var(--primary)/0.7)]" : "text-muted-foreground hover:text-foreground",
                )}
              >
                <t.icon className={cn("h-5 w-5", active && "stroke-[2.5]")} aria-hidden />
                <span>{t.label}</span>
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
