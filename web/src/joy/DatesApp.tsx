import { lazy, Suspense, useEffect, useRef } from "react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import { Header } from "@/components/layout/Header";
import "./styles/index.css";
import "./styles/app.css";
import { Gate } from "./components/Gate";
import { pageKey } from "./paths";

const JarPage = lazy(() => import("./jar/JarPage"));
const SlipsPage = lazy(() => import("./pages/SlipsPage"));
const SlipFormPage = lazy(() => import("./pages/SlipFormPage"));
const MiddlePage = lazy(() => import("./pages/MiddlePage"));
const KeptPage = lazy(() => import("./pages/KeptPage"));
const MuseumPage = lazy(() => import("./pages/MuseumPage"));
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
        <Gate>
          <main id="main" tabIndex={-1}>
            <Suspense
              fallback={
                <p className="loading" role="status">
                  Loading…
                </p>
              }
            >
              <Routes>
                <Route index element={<JarIndex />} />
                <Route
                  path="slips"
                  element={
                    <div className="p-slips">
                      <SlipsPage />
                    </div>
                  }
                />
                <Route
                  path="slips/new"
                  element={
                    <div className="p-slips">
                      <SlipFormPage />
                    </div>
                  }
                />
                <Route
                  path="slips/:id"
                  element={
                    <div className="p-slips">
                      <SlipFormPage />
                    </div>
                  }
                />
                <Route path="middle" element={<MiddlePage />} />
                <Route
                  path="settings"
                  element={<Navigate to="/account" replace />}
                />
                <Route path="kept" element={<KeptPage />} />
                <Route path="museum" element={<MuseumPage />} />
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
