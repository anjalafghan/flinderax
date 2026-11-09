import { lazy, Suspense } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import ProtectedRoute from './components/layout/ProtectedRoute'
import { TabBar } from './components/layout/Header'
import { useAuth } from './context/AuthContext'

// Lazy load pages for code splitting
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
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <DashboardPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/cards/new"
            element={
              <ProtectedRoute>
                <CreateCardPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/cards/:id"
            element={
              <ProtectedRoute>
                <CardDetailsPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/accounts"
            element={
              <ProtectedRoute>
                <AccountsPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/plan"
            element={
              <ProtectedRoute>
                <PlanPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/working"
            element={
              <ProtectedRoute>
                <WorkingPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/what-if"
            element={
              <ProtectedRoute>
                <WhatIfPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/spending"
            element={
              <ProtectedRoute>
                <SpendingPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/money"
            element={
              <ProtectedRoute>
                <MoneyPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/spending/sort"
            element={
              <ProtectedRoute>
                <SortPayeesPage />
              </ProtectedRoute>
            }
          />
          {/* Public: the envelope a date arrives in. No login, no app chrome. */}
          <Route path="/joy/e/:token" element={<EnvelopePage />} />
          <Route
            path="/joy/*"
            element={
              <ProtectedRoute>
                <DatesApp />
              </ProtectedRoute>
            }
          />
          <Route
            path="/health"
            element={
              <ProtectedRoute>
                <HealthPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/people"
            element={
              <ProtectedRoute>
                <PeoplePage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/people/:id"
            element={
              <ProtectedRoute>
                <PersonPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/review"
            element={
              <ProtectedRoute>
                <ReviewPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/life"
            element={
              <ProtectedRoute>
                <LifePage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/checkin"
            element={
              <ProtectedRoute>
                <CheckInPage />
              </ProtectedRoute>
            }
          />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Suspense>
      </div>
      {showTabs && <TabBar />}
      <Suspense fallback={null}>
        <Toaster position="top-center" />
      </Suspense>
    </>
  )
}

export default App
