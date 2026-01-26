import { Routes, Route, Navigate } from 'react-router-dom';
import { Toaster } from 'sonner';
import AuthPage from '@/pages/AuthPage';
import DashboardPage from '@/pages/DashboardPage';
import CreateCardPage from '@/pages/CreateCardPage';
import CardDetailsPage from '@/pages/CardDetailsPage';
import ProtectedRoute from '@/components/layout/ProtectedRoute';

function App() {
  return (
    <>
      <Routes>
        <Route path="/auth" element={<AuthPage />} />
        <Route element={<ProtectedRoute />}>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/cards/new" element={<CreateCardPage />} />
          <Route path="/cards/:id" element={<CardDetailsPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      <Toaster position="top-center" richColors />
    </>
  );
}

export default App;
