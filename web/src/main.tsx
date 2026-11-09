import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import App from './App.tsx'
import { AuthProvider } from '@/context/AuthProvider'
import { ThemeProvider } from "@/context/ThemeContext"
import { ErrorBoundary } from '@/components/ui/error-boundary'
import { sweepOfflineCopies } from '@/session'


const queryClient = new QueryClient()

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
            <AuthProvider>
              <App />
            </AuthProvider>
          </ThemeProvider>
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  </React.StrictMode>,
)

// After a deploy, an open tab can ask for code chunks that no longer exist. Reload once to pick up
// the new build instead of showing a broken page; the flag stops a reload loop if it really is down.
window.addEventListener("unhandledrejection", (e) => {
  const msg = String((e.reason as { message?: string } | undefined)?.message ?? e.reason ?? "");
  if (!/dynamically imported module|Importing a module script failed|error loading dynamically/i.test(msg)) return;
  try {
    if (sessionStorage.getItem("joy-chunk-reload")) return;
    sessionStorage.setItem("joy-chunk-reload", "1");
  } catch {
    return;
  }
  window.location.reload();
});
// Clear the guard only once the page has run cleanly for a while (lazy chunks load after "load").
window.addEventListener("load", () => {
  setTimeout(() => {
    try {
      sessionStorage.removeItem("joy-chunk-reload");
    } catch {
      /* storage unavailable */
    }
  }, 10_000);
});

// Saved offline copies of an older login must not outlive it: keep only the current token's.
void sweepOfflineCopies();

// Offline copies of the shell and a few read-only screens (see public/sw.js).
if ("serviceWorker" in navigator && process.env.NODE_ENV === "production") {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch(() => {});
  });
}
