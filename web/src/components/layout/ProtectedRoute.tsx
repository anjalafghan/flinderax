import { Navigate } from "react-router-dom";
import { useAuth } from "@/context/AuthContext";
import { type ReactNode } from "react";

interface ProtectedRouteProps {
  children: ReactNode;
}

const ProtectedRoute = ({ children }: ProtectedRouteProps) => {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-4 text-foreground">
        <span aria-hidden className="flex gap-2">
          {["bg-sun", "bg-coral", "bg-rose", "bg-grape", "bg-sky"].map(
            (c, i) => (
              <span
                key={c}
                className={`h-2.5 w-2.5 animate-bounce rounded-full ${c}`}
                style={{ animationDelay: `${i * 120}ms` }}
              />
            ),
          )}
        </span>
        <span className="text-sm text-muted-foreground">Loading...</span>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/auth" replace />;
  }

  return <>{children}</>;
};

export default ProtectedRoute;
