import { Header } from "@/components/layout/Header";
import { AccountSection } from "@/components/feature/AccountSection";

/** Password, sessions and deleting the account. */
export default function AccountPage() {
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl stack">
        <div>
          <h1 className="page-title">Account</h1>
          <p className="page-sub">Your password, where you are signed in, and deleting your account.</p>
        </div>
        <AccountSection />
      </main>
    </div>
  );
}
