import { Link } from "react-router-dom";
import { ArrowRight, CreditCard, Landmark, Plus, Receipt } from "lucide-react";

import { Header } from "@/components/layout/Header";
import { Card, CardContent } from "@/components/ui/card";

const LINKS = [
  { to: "/accounts", title: "Accounts", hint: "Bank, FD and RD balances", icon: Landmark },
  { to: "/spending", title: "Spending", hint: "What you've spent, and where", icon: Receipt },
  { to: "/", title: "Cards", hint: "Your credit cards and what's due", icon: CreditCard },
  { to: "/cards/new", title: "Add a card", hint: "Start tracking another card", icon: Plus },
];

export default function MoneyPage() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />
      <main className="container mx-auto max-w-3xl space-y-6 px-4 py-6 md:py-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Money</h1>
          <p className="text-muted-foreground">Your accounts, cards and spending in one place.</p>
        </div>
        <ul className="grid gap-3 sm:grid-cols-2">
          {LINKS.map((l) => (
            <li key={l.to}>
              <Link to={l.to} className="block rounded-lg transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                <Card>
                  <CardContent className="flex items-center gap-4 p-4">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
                      <l.icon className="h-5 w-5" aria-hidden />
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="font-semibold">{l.title}</p>
                      <p className="truncate text-sm text-muted-foreground">{l.hint}</p>
                    </div>
                    <ArrowRight className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden />
                  </CardContent>
                </Card>
              </Link>
            </li>
          ))}
        </ul>
      </main>
    </div>
  );
}
