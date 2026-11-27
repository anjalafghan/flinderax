import { useLayoutEffect, useRef } from "react";
import { Link, useLocation } from "react-router-dom";
import {
  ClipboardCheck,
  Heart,
  Route,
  Settings,
  Sun,
  Wallet,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/utils/cn";
import { ScrollFade } from "@/components/ui/scroll-fade";

/** Tab styling: one ink look for every tab. */
const INK = { text: "text-foreground", hover: "hover:text-foreground", fill: "bg-foreground", glow: "" };
const TAB_TONE: Record<string, { text: string; hover: string; fill: string; glow: string }> = {
  "/": INK, "/money": INK, "/life": INK, "/joy": INK, "/week": INK,
};

interface Tab {
  to: string;
  label: string;
  icon: LucideIcon;
  /** Paths that light this tab up, so the Money tab stays lit on the Cards, Accounts and Spending pages. */
  prefixes: string[];
}

interface SubTab {
  to: string;
  label: string;
  prefixes: string[];
  /** Light up only on this exact path (the Jar would otherwise match every Dates page). */
  exact?: boolean;
}

const TABS: Tab[] = [
  { to: "/", label: "Today", icon: Sun, prefixes: ["/"] },
  {
    to: "/money",
    label: "Money",
    icon: Wallet,
    prefixes: [
      "/money",
      "/cards",
      "/accounts",
      "/spending",
      "/plan",
      "/working",
    ],
  },
  {
    to: "/life",
    label: "Life",
    icon: Route,
    prefixes: [
      "/life",
      "/glance",
      "/wedding",
      "/year",
      "/what-if",
      "/work",
      "/health",
    ],
  },
  { to: "/joy", label: "Us", icon: Heart, prefixes: ["/joy", "/people"] },
  {
    to: "/week",
    label: "Review",
    icon: ClipboardCheck,
    prefixes: ["/week", "/review", "/checkin"],
  },
];

/** The pages inside a tab, keyed by the tab's path. Every tab but Today has them, in the same style. */
const SUBTABS: Record<string, { label: string; items: SubTab[] }> = {
  "/money": {
    label: "Money",
    items: [
      { to: "/money", label: "Overview", prefixes: ["/money"] },
      { to: "/spending", label: "Spending", prefixes: ["/spending"] },
      { to: "/cards", label: "Cards", prefixes: ["/cards"] },
      { to: "/accounts", label: "Accounts", prefixes: ["/accounts"] },
      { to: "/plan", label: "Budget", prefixes: ["/plan", "/working"] },
    ],
  },
  "/life": {
    label: "Life",
    items: [
      { to: "/glance", label: "At a glance", prefixes: ["/glance"] },
      { to: "/life", label: "Life line", prefixes: ["/life", "/wedding"] },
      { to: "/year", label: "Year ahead", prefixes: ["/year"] },
      { to: "/what-if", label: "What if", prefixes: ["/what-if"] },
      { to: "/work", label: "Work", prefixes: ["/work"] },
      { to: "/health", label: "Health", prefixes: ["/health"] },
    ],
  },
  // Named "Dates" for screen readers: it is the date jar's own menu, plus the wider circle.
  "/joy": {
    label: "Dates",
    items: [
      { to: "/joy", label: "Jar", prefixes: ["/joy"], exact: true },
      { to: "/joy/slips", label: "Slips", prefixes: ["/joy/slips"] },
      { to: "/joy/kept", label: "Kept", prefixes: ["/joy/kept"] },
      { to: "/joy/museum", label: "Museum", prefixes: ["/joy/museum"] },
      { to: "/joy/year", label: "Calendar", prefixes: ["/joy/year"] },
      { to: "/joy/regulars", label: "Regulars", prefixes: ["/joy/regulars"] },
      { to: "/joy/middle", label: "Middle", prefixes: ["/joy/middle"] },
      { to: "/people", label: "Circle", prefixes: ["/people"] },
    ],
  },
  "/week": {
    label: "Review",
    items: [
      { to: "/week", label: "This week", prefixes: ["/week"] },
      { to: "/review", label: "Month", prefixes: ["/review"] },
      { to: "/checkin", label: "Quarter", prefixes: ["/checkin"] },
    ],
  },
};

const under = (pathname: string, p: string) =>
  pathname === p || pathname.startsWith(`${p}/`);

/** The tab whose prefixes match the path. "/" only matches exactly, so it doesn't light up everything. */
function activeTab(pathname: string): Tab | undefined {
  return TABS.find((t) =>
    t.prefixes.some((p) => (p === "/" ? pathname === "/" : under(pathname, p))),
  );
}

function subtabOn(t: SubTab, pathname: string): boolean {
  return t.exact
    ? pathname === t.to || pathname === `${t.to}/`
    : t.prefixes.some((p) => under(pathname, p));
}

/** The open tab's pages as an underline bar. On a phone it scrolls sideways and keeps the current one in view. */
export function SubTabs() {
  const { pathname } = useLocation();
  const tab = activeTab(pathname);
  const group = tab && SUBTABS[tab.to];
  const current = useRef<HTMLAnchorElement>(null);
  useLayoutEffect(() => {
    current.current?.scrollIntoView?.({ inline: "nearest", block: "nearest" });
  }, [pathname]);
  if (!group) return null;
  return (
    <nav aria-label={group.label} className="mt-6">
      <div className="-mx-4 border-b border-brand/20 md:mx-0">
        <ScrollFade
          watch={pathname}
          className="scroll-px-10 gap-6 px-4 py-0 md:gap-8 md:px-0"
        >
          {group.items.map((t) => {
            const on = subtabOn(t, pathname);
            return (
              <Link
                key={t.to}
                ref={on ? current : undefined}
                to={t.to}
                viewTransition
                aria-current={on ? "page" : undefined}
                className={cn(
                  "relative -mx-3 inline-flex min-h-11 shrink-0 items-center whitespace-nowrap px-3 text-sm no-underline transition-colors",
                  "after:absolute after:inset-x-3 after:bottom-0 after:h-[3px] after:rounded-full after:bg-[linear-gradient(90deg,hsl(var(--brand)),hsl(var(--brand-2)))] after:transition-transform after:duration-300 after:origin-left",
                  on
                    ? "font-medium text-brand-ink after:scale-x-100"
                    : "text-muted-foreground after:scale-x-0 hover:text-brand-ink hover:after:scale-x-50",
                )}
              >
                {t.label}
              </Link>
            );
          })}
        </ScrollFade>
      </div>
    </nav>
  );
}

/** Top bar: italic wordmark, the main tabs in one tray from md up, and Settings. Theme and sign-out live in Settings. */
export function Header() {
  const { pathname } = useLocation();
  const current = activeTab(pathname);
  const inSettings = pathname === "/account";

  return (
    <header className="relative z-50 mx-auto w-full max-w-[1120px] px-4 pt-5 md:px-7 md:pt-7">
      <div className="flex items-center justify-between gap-4">
        <Link
          to="/"
          className="wordmark inline-flex min-h-11 min-w-11 items-center text-[34px] leading-none no-underline"
          aria-label="joy, today"
        >
          joy
        </Link>

        <nav
          aria-label="Main"
          className="hidden items-center gap-0.5 rounded-full border border-brand/20 bg-card/80 p-1 shadow-[0_10px_30px_-16px_hsl(var(--brand)/.5)] backdrop-blur md:flex"
        >
          {TABS.map((t) => {
            const on = current?.to === t.to;
            return (
              <Link
                key={t.to}
                to={t.to}
                viewTransition
                aria-current={on ? "page" : undefined}
                className={cn(
                  "inline-flex h-9 items-center rounded-full px-4 text-sm no-underline transition-all duration-300",
                  on
                    ? cn(TAB_TONE[t.to]?.fill, TAB_TONE[t.to]?.glow, "font-medium text-background")
                    : cn("text-muted-foreground hover:bg-brand-tint", TAB_TONE[t.to]?.hover),
                )}
              >
                {t.label}
              </Link>
            );
          })}
        </nav>

        <Link
          to="/account"
          title="Settings"
          aria-label="Settings"
          aria-current={inSettings ? "page" : undefined}
          className={cn(
            "group inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-full border border-brand/25 bg-card/70 text-brand-ink no-underline backdrop-blur transition-colors hover:bg-brand-tint",
            inSettings && "bg-brand-gradient border-transparent text-brand-fg",
          )}
        >
          <Settings
            className="h-[18px] w-[18px] transition-transform duration-500 group-hover:rotate-90"
            strokeWidth={1.5}
            aria-hidden
          />
        </Link>
      </div>
      <SubTabs />
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
      <ul className="mx-auto grid max-w-md grid-cols-5 rounded-full border border-brand/20 bg-card/90 p-1 shadow-[0_12px_32px_-12px_hsl(var(--brand)/.45)] backdrop-blur">
        {TABS.map((t) => {
          const active = current?.to === t.to;
          return (
            <li key={t.to}>
              <Link
                to={t.to}
                viewTransition
                aria-current={active ? "page" : undefined}
                className={cn(
                  "flex min-h-[52px] flex-col items-center justify-center gap-0.5 text-[11px] no-underline transition-colors",
                  active
                    ? cn("font-semibold", TAB_TONE[t.to]?.text)
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <span
                  className={cn(
                    "flex h-7 w-12 items-center justify-center rounded-full transition-all duration-300",
                    active ? cn(TAB_TONE[t.to]?.fill, TAB_TONE[t.to]?.glow, "scale-110 text-background") : TAB_TONE[t.to]?.text,
                  )}
                >
                  <t.icon
                    className="h-5 w-5"
                    strokeWidth={active ? 2 : 1.5}
                    aria-hidden
                  />
                </span>
                <span>{t.label}</span>
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
