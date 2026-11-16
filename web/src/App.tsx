import { lazy, Suspense } from 'react'
import { useOutbox } from '@/hooks/useOutbox'
import { RouteBoundary } from '@/components/ui/route-boundary'
import { Routes, Route, Navigate, Outlet, useLocation } from 'react-router-dom'
import ProtectedRoute from './components/layout/ProtectedRoute'
import { TabBar } from './components/layout/Header'
import { useAuth } from './context/AuthContext'

// Lazy load pages for code splitting
const AccountPage = lazy(() => import('./pages/AccountPage'))
const AuthPage = lazy(() => import('./pages/AuthPage'))
const DashboardPage = lazy(() => import('./pages/DashboardPage'))
const CreateCardPage = lazy(() => import('./pages/CreateCardPage'))
const CardDetailsPage = lazy(() => import('./pages/CardDetailsPage'))
const AccountsPage = lazy(() => import('./pages/AccountsPage'))
const PlanPage = lazy(() => import('./pages/PlanPage'))
const WorkingPage = lazy(() => import('./pages/WorkingPage'))
const WhatIfPage = lazy(() => import('./pages/WhatIfPage'))
const SpendingPage = lazy(() => import('./pages/SpendingPage'))
const MoneyPage = lazy(() => import('./pages/MoneyPage'))
const SortPayeesPage = lazy(() => import('./pages/SortPayeesPage'))
const DatesApp = lazy(() => import('./joy/DatesApp'))
const EnvelopePage = lazy(() => import('./joy/pages/EnvelopePage'))
const ReviewPage = lazy(() => import('./pages/ReviewPage'))
const LifePage = lazy(() => import('./pages/LifePage'))
const CheckInPage = lazy(() => import('./pages/CheckInPage'))
const HealthPage = lazy(() => import('./pages/HealthPage'))
const PeoplePage = lazy(() => import('./pages/PeoplePage'))
const PersonPage = lazy(() => import('./pages/PersonPage'))
const WorkPage = lazy(() => import('./pages/WorkPage'))
const WeekPage = lazy(() => import('./pages/WeekPage'))
const YearPage = lazy(() => import('./pages/YearPage'))
const WeddingPage = lazy(() => import('./pages/WeddingPage'))
// Toasts are rare and sonner is ~35 KB, so its Toaster is a lazy chunk, not part of the shell.
const Toaster = lazy(() => import('sonner').then((m) => ({ default: m.Toaster })))

// Simple loading fallback
const PageLoader = () => (
  <div className="flex h-screen items-center justify-center">
    <div className="text-muted-foreground">Loading...</div>
  </div>
)

function App() {
  const { isAuthenticated } = useAuth()
  useOutbox(isAuthenticated)
  const { pathname } = useLocation()
  // The phone tab bar is fixed to the bottom, so pages leave room for it (md+ uses the top nav).
  const showTabs = isAuthenticated && pathname !== '/auth'

  return (
    <>
      <div className={showTabs ? 'pb-[calc(5.5rem+env(safe-area-inset-bottom))] md:pb-0' : undefined}>
      <Suspense fallback={<PageLoader />}>
        <Routes>
          <Route
            path="/auth"
            element={isAuthenticated ? <Navigate to="/" replace /> : <AuthPage />}
          />
          {/* Public: the envelope a date arrives in. No login, no app chrome. */}
          <Route path="/joy/e/:token" element={<EnvelopePage />} />
          {/* Everything below needs a login. */}
          <Route
            element={
              <ProtectedRoute>
                <RouteBoundary resetKey={pathname}>
                  <Outlet />
                </RouteBoundary>
              </ProtectedRoute>
            }
          >
            <Route path="/" element={<DashboardPage />} />
            <Route path="/account" element={<AccountPage />} />
            <Route path="/cards/new" element={<CreateCardPage />} />
            <Route path="/cards/:id" element={<CardDetailsPage />} />
            <Route path="/accounts" element={<AccountsPage />} />
            <Route path="/plan" element={<PlanPage />} />
            <Route path="/working" element={<WorkingPage />} />
            <Route path="/what-if" element={<WhatIfPage />} />
            <Route path="/spending" element={<SpendingPage />} />
            <Route path="/money" element={<MoneyPage />} />
            <Route path="/spending/sort" element={<SortPayeesPage />} />
            <Route path="/joy/*" element={<DatesApp />} />
            <Route path="/health" element={<HealthPage />} />
            <Route path="/people" element={<PeoplePage />} />
            <Route path="/people/:id" element={<PersonPage />} />
            <Route path="/work" element={<WorkPage />} />
            <Route path="/week" element={<WeekPage />} />
            <Route path="/year" element={<YearPage />} />
            <Route path="/wedding/:goalId" element={<WeddingPage />} />
            <Route path="/review" element={<ReviewPage />} />
            <Route path="/life" element={<LifePage />} />
            <Route path="/checkin" element={<CheckInPage />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Suspense>
      </div>
      {showTabs && <TabBar />}
      <Suspense fallback={null}>
        <Toaster position="top-center" duration={2500} />
      </Suspense>
    </>
  )
}

export default App
