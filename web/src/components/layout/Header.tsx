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

/** Top bar: the logo and account actions everywhere, plus the five tabs from md up. */
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
    <header className="sticky top-0 z-50 w-full border-b bg-background/80 backdrop-blur-md dark:border-white/10">
      <div className="container mx-auto flex h-16 items-center justify-between gap-2 px-4 md:px-6">
        <Link to="/" className="flex shrink-0 items-center gap-2" aria-label="joy, today">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary font-bold text-primary-foreground">j</div>
          <span className="hidden text-xl font-bold tracking-tight text-slate-900 dark:text-white sm:inline">joy</span>
        </Link>

        <nav aria-label="Main" className="hidden items-center gap-1 md:flex">
          {TABS.map((t) => (
            <Link
              key={t.to}
              to={t.to}
              aria-current={current?.to === t.to ? "page" : undefined}
              className={cn(
                "inline-flex h-9 items-center gap-2 rounded-md px-3 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
                current?.to === t.to ? "bg-accent text-accent-foreground" : "text-muted-foreground",
              )}
            >
              <t.icon className="h-4 w-4" aria-hidden />
              {t.label}
            </Link>
          ))}
        </nav>

        <div className="flex shrink-0 items-center gap-2 md:gap-4">
          <Button variant="outline" size="sm" onClick={() => navigate("/cards/new")} className="h-9 px-3 md:px-4" aria-label="Add card">
            <Plus className="h-4 w-4 md:mr-2" />
            <span className="hidden md:inline">Add Card</span>
          </Button>

          <ModeToggle />

          <Button variant="ghost" size="icon" onClick={handleLogout} title="Logout" aria-label="Logout" className="h-9 w-9">
            <LogOut className="h-[1.2rem] w-[1.2rem]" />
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
      className="fixed inset-x-0 bottom-0 z-40 border-t bg-background/95 pb-[env(safe-area-inset-bottom)] backdrop-blur-md dark:border-white/10 md:hidden"
    >
      <ul className="mx-auto grid max-w-md grid-cols-5">
        {TABS.map((t) => {
          const active = current?.to === t.to;
          return (
            <li key={t.to}>
              <Link
                to={t.to}
                aria-current={active ? "page" : undefined}
                className={cn(
                  "flex min-h-[44px] flex-col items-center justify-center gap-0.5 py-2 text-[11px] font-medium transition-colors",
                  active ? "text-primary" : "text-muted-foreground",
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
