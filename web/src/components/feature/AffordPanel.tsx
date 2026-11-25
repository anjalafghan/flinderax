import { useState } from "react";

import type { AffordAnswer, AffordResult } from "@/services/types";
import { useAfford } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { formatDay } from "@/utils/dates";
import { formatINR, toPaise } from "@/utils/money";
import { cn } from "@/utils/cn";
import { buildAffordPay } from "@/utils/planner";
import StarBorder from "@/components/bits/StarBorder";
import { Check, Clock, PiggyBank, X } from "lucide-react";

const ANSWER: Record<
  AffordAnswer,
  { label: string; box: string; badge: string; Icon: typeof Check }
> = {
  yes: {
    label: "Yes",
    box: "border-good/40 bg-good-tint text-good",
    badge: "bg-gradient-to-br from-mint to-teal",
    Icon: Check,
  },
  yes_from_savings: {
    label: "Yes, from savings",
    box: "border-warn/40 bg-warn-tint text-warn",
    badge: "bg-gradient-to-br from-sky to-grape",
    Icon: PiggyBank,
  },
  wait: {
    label: "Wait",
    box: "border-warn/40 bg-warn-tint text-warn",
    badge: "bg-gradient-to-br from-sun to-coral",
    Icon: Clock,
  },
  no: {
    label: "Not now",
    box: "border-over/40 bg-over-tint text-over",
    badge: "bg-gradient-to-br from-coral to-rose",
    Icon: X,
  },
};

function Result({ r }: { r: AffordResult }) {
  const tone = ANSWER[r.answer];
  const body = (
    <section
      aria-label="Answer"
      data-answer={r.answer}
      className={cn(
        "space-y-3 glass rounded-[28px] p-5 md:p-6 starting:scale-95 starting:opacity-0 transition-[opacity,transform] duration-500",
        tone.box,
      )}
    >
      <div className="flex items-center gap-3">
        <span
          aria-hidden
          className={cn(
            "grid h-12 w-12 shrink-0 place-items-center rounded-2xl text-background shadow-lg",
            tone.badge,
          )}
        >
          <tone.Icon className="h-6 w-6" strokeWidth={2.5} />
        </span>
        <p className="font-display text-3xl leading-none tracking-tight md:text-4xl">
          {tone.label}
        </p>
      </div>
      <p className="text-xl font-bold leading-snug">{r.headline}</p>
      {r.reasons.length > 0 && (
        <ul className="list-disc space-y-1 pl-5 text-sm">
          {r.reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      )}
      {r.wait_until && (
        <p className="text-sm font-medium">
          Wait until {formatDay(r.wait_until)}.
        </p>
      )}
      <p className="text-sm tabular-nums">
        Safe to spend after this:{" "}
        <span className="font-semibold">
          {formatINR(r.safe_to_spend_after_paise)}
        </span>
      </p>
      {r.monthly_paise > 0 && (
        <p className="text-sm tabular-nums">
          {formatINR(r.monthly_paise)}/month
          {r.extra_cost_paise > 0 && (
            <> · costs {formatINR(r.extra_cost_paise)} extra</>
          )}
        </p>
      )}
      <p className="rounded-md bg-background/60 p-3 text-sm text-foreground">
        Sleep on it — come back in 48 hours.
      </p>
    </section>
  );
  if (r.answer !== "yes") return body;
  // A clear yes gets a little star running round its edge.
  return (
    <StarBorder
      as="div"
      color="#2c6b4d"
      trailColor="#2c6b4d"
      radius={28}
      thickness={2}
      stars={2}
      sparkle
      hover="none"
      clickPulse={false}
      theme="light"
      backgroundColor="transparent"
      textColor="inherit"
      borderColor="transparent"
      className="block! w-full p-0! text-left font-normal!"
    >
      {body}
    </StarBorder>
  );
}

export function AffordPanel() {
  const [amount, setAmount] = useState("");
  const [mode, setMode] = useState<"now" | "emi">("now");
  const [emi, setEmi] = useState({ months: "12", rate: "", fee: "" });
  const [error, setError] = useState<string | null>(null);
  const check = useAfford();

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amountPaise = toPaise(amount);
    if (amountPaise === null || amountPaise <= 0)
      return setError("Type what it costs, for example 45000.");
    const built = buildAffordPay(mode, emi);
    if ("error" in built) return setError(built.error);
    setError(null);
    check.mutate({ amount_paise: amountPaise, pay: built.pay });
  };

  const seg = (active: boolean) =>
    cn(
      "h-11 rounded-md border text-sm font-medium transition-colors",
      active
        ? "border-transparent bg-brand-gradient shadow-[0_8px_20px_-10px_hsl(var(--brand)/0.7)]"
        : "bg-background hover:bg-accent",
    );

  return (
    <Card id="afford" className="scroll-mt-24">
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">
          <span className="brand-text">Can I afford it?</span>
        </CardTitle>
        <p className="text-sm text-muted-foreground max-w-prose">
          Check a purchase against your budget before you buy.
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        <form onSubmit={submit} className="form-grid" noValidate>
          <div className="field col-span-full">
            <Label htmlFor="afford-amount">What does it cost?</Label>
            <MoneyInput
              id="afford-amount"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="45000"
            />
          </div>

          <fieldset className="field col-span-full">
            <legend className="text-sm font-medium">How will you pay?</legend>
            <div className="grid grid-cols-2 gap-2">
              <button
                type="button"
                aria-pressed={mode === "now"}
                className={seg(mode === "now")}
                onClick={() => setMode("now")}
              >
                Pay now
              </button>
              <button
                type="button"
                aria-pressed={mode === "emi"}
                className={seg(mode === "emi")}
                onClick={() => setMode("emi")}
              >
                EMI
              </button>
            </div>
          </fieldset>

          {mode === "emi" && (
            <>
              <div className="field">
                <Label htmlFor="afford-months">Months</Label>
                <Input
                  id="afford-months"
                  inputMode="numeric"
                  maxLength={3}
                  value={emi.months}
                  onChange={(e) => setEmi({ ...emi, months: e.target.value })}
                />
              </div>
              <div className="field">
                <Label htmlFor="afford-rate">Interest % a year</Label>
                <Input
                  id="afford-rate"
                  inputMode="decimal"
                  value={emi.rate}
                  onChange={(e) => setEmi({ ...emi, rate: e.target.value })}
                  placeholder="0"
                />
              </div>
              <div className="field col-span-full">
                <Label htmlFor="afford-fee">Processing fee (optional)</Label>
                <MoneyInput
                  id="afford-fee"
                  value={emi.fee}
                  onChange={(e) => setEmi({ ...emi, fee: e.target.value })}
                />
              </div>
            </>
          )}

          {error && (
            <p role="alert" className="col-span-full text-sm text-destructive">
              {error}
            </p>
          )}
          <div className="form-actions">
            <ResetButton
              label="Clear"
              onReset={() => {
                setAmount("");
                setMode("now");
                setEmi({ months: "12", rate: "", fee: "" });
                setError(null);
                check.reset();
              }}
            />
            <Button type="submit" disabled={check.isPending}>
              {check.isPending ? "Checking..." : "Check"}
            </Button>
          </div>
        </form>

        {check.data && <Result r={check.data} />}
      </CardContent>
    </Card>
  );
}
