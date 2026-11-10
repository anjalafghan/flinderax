import { lazy, Suspense, useEffect, useRef } from "react";
import { Link, Navigate, Route, Routes, useLocation } from "react-router-dom";
import { Header } from "@/components/layout/Header";
import "./styles/index.css";
import "./styles/app.css";
import { Gate } from "./components/Gate";
import { navFor, pageKey } from "./paths";

const JarPage = lazy(() => import("./jar/JarPage"));
const SlipsPage = lazy(() => import("./pages/SlipsPage"));
const SlipFormPage = lazy(() => import("./pages/SlipFormPage"));
const MiddlePage = lazy(() => import("./pages/MiddlePage"));
const SettingsPage = lazy(() => import("./pages/SettingsPage"));
const KeptPage = lazy(() => import("./pages/KeptPage"));
const RegularsPage = lazy(() => import("./pages/RegularsPage"));
const RegularPage = lazy(() => import("./pages/RegularPage"));
const YearPage = lazy(() => import("./pages/YearPage"));
const MonthPage = lazy(() => import("./pages/MonthPage"));

/** Joy linked the jar to /#middle. The Middle page is its own route now. */
function JarIndex() {
  const { hash } = useLocation();
  if (hash === "#middle") return <Navigate to="/joy/middle" replace />;
  return <JarPage />;
}

function Nav({ page }: { page: ReturnType<typeof pageKey> }) {
  const here = navFor(page);
  const cur = (k: string) => (here === k ? ({ "aria-current": "page" } as const) : {});
  return (
    <>
      <header className="top">
        <Link className="wordmark" to="/joy" aria-label="Dates, home">
          joy
        </Link>
        <nav aria-label="Dates">
          <Link to="/joy" data-nav="jar" {...cur("jar")}>
            Jar
          </Link>
          <Link to="/joy/slips" data-nav="slips" {...cur("slips")}>
            Slips
          </Link>
          <Link to="/joy/kept" data-nav="kept" {...cur("kept")}>
            Kept
          </Link>
          <Link to="/joy/year" data-nav="year" {...cur("year")}>
            Year
          </Link>
          <Link to="/joy/regulars" data-nav="regulars" {...(page === "regulars" ? ({ "aria-current": "page" } as const) : {})}>
            Regulars
          </Link>
          <Link
            to="/joy/middle"
            data-nav="middle"
            className="nav-middle-link"
            title="Meet in the middle: somewhere fair for both of you"
            {...(page === "middle" ? ({ "aria-current": "page" } as const) : {})}
          >
            <svg viewBox="0 0 64 28" aria-hidden="true" focusable="false">
              <circle cx="22" cy="14" r="11" />
              <circle cx="42" cy="14" r="11" />
            </svg>
            Middle
          </Link>
          <Link to="/joy/settings" data-nav="settings" {...(page === "settings" ? ({ "aria-current": "page" } as const) : {})}>
            Settings
          </Link>
        </nav>
      </header>
    </>
  );
}

export default function DatesApp() {
  const { pathname } = useLocation();
  const page = pageKey(pathname);
  const first = useRef(true);

  // A single-page app has no page loads: bring the new page to the top and hand it focus.
  useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    window.scrollTo(0, 0);
    document.getElementById("main")?.focus({ preventScroll: true });
  }, [pathname]);

  return (
    <>
      <Header />
      <div className="joy grain" data-page={page}>
        <a className="visually-hidden" href="#main">
          Skip to content
        </a>
        <Nav page={page} />
        <Gate>
          <main id="main" tabIndex={-1}>
            <Suspense fallback={<p className="loading" role="status">Loading…</p>}>
              <Routes>
                <Route index element={<JarIndex />} />
                <Route path="slips" element={<div className="p-slips"><SlipsPage /></div>} />
                <Route path="slips/new" element={<div className="p-slips"><SlipFormPage /></div>} />
                <Route path="slips/:id" element={<div className="p-slips"><SlipFormPage /></div>} />
                <Route path="middle" element={<MiddlePage />} />
                <Route path="settings" element={<SettingsPage />} />
                <Route path="kept" element={<KeptPage />} />
                <Route path="regulars" element={<RegularsPage />} />
                <Route path="regulars/:id" element={<RegularPage />} />
                <Route path="year" element={<YearPage />} />
                <Route path="year/:y" element={<YearPage />} />
                <Route path="year/:y/:m" element={<MonthPage />} />
                <Route path="*" element={<Navigate to="/joy" replace />} />
              </Routes>
            </Suspense>
          </main>
        </Gate>
      </div>
    </>
  );
}
