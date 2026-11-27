import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { ArrowLeft } from "lucide-react";

import api from "@/services/api";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { Header } from "@/components/layout/Header";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import { CreditCard } from "@/components/feature/CreditCard";
import { InfoTip } from "@/components/ui/info-tip";
import { MoneyInput } from "@/components/ui/money-input";
import { toPaise } from "@/utils/money";
import { Select } from "@/components/ui/select";
import { useAccounts } from "@/hooks/usePlanner";

const parseDay = (v: string): number | null => {
  const n = Number(v);
  return v.trim() && Number.isInteger(n) && n >= 1 && n <= 31 ? n : null;
};

// Helper to convert hex to rgb tuple
function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? [
        parseInt(result[1], 16),
        parseInt(result[2], 16),
        parseInt(result[3], 16),
      ]
    : [0, 0, 0];
}

/** Ready-made card faces: two colours each, for the gradient. */
const LOOKS: { name: string; from: string; to: string }[] = [
  { name: "Midnight", from: "#1e293b", to: "#334155" },
  { name: "Ocean", from: "#0369a1", to: "#14b8a6" },
  { name: "Emerald", from: "#065f46", to: "#10b981" },
  { name: "Sunset", from: "#f97316", to: "#e11d48" },
  { name: "Royal", from: "#4c1d95", to: "#8b5cf6" },
  { name: "Rose gold", from: "#b76e79", to: "#f5c6a5" },
  { name: "Onyx", from: "#0a0a0a", to: "#3f3f46" },
];

export default function CreateCardPage() {
  const navigate = useNavigate();

  const [name, setName] = useState("My Card");
  const [bank, setBank] = useState("Bank Name");
  const [primaryColor, setPrimaryColor] = useState("#1e293b");
  const [secondaryColor, setSecondaryColor] = useState("#334155");
  const [last4Digits, setLast4Digits] = useState("");
  const [statementDay, setStatementDay] = useState("");
  const [dueDay, setDueDay] = useState("");
  const [creditLimit, setCreditLimit] = useState("");
  const [payFrom, setPayFrom] = useState("");
  const { data: accounts } = useAccounts();

  const createCardMutation = useMutation({
    mutationFn: async () => {
      if (
        (statementDay.trim() && parseDay(statementDay) === null) ||
        (dueDay.trim() && parseDay(dueDay) === null)
      ) {
        throw new Error("Statement day and due day must be between 1 and 31");
      }
      if (creditLimit.trim() && toPaise(creditLimit) === null) {
        throw new Error("Credit limit must be an amount");
      }
      await api.post("/api/card/create", {
        card_name: name,
        card_bank: bank,
        card_primary_color: hexToRgb(primaryColor),
        card_secondary_color: hexToRgb(secondaryColor),
        last_4_digits: last4Digits || null,
        statement_day: parseDay(statementDay),
        due_day: parseDay(dueDay),
        credit_limit_paise: creditLimit.trim() ? toPaise(creditLimit) : null,
        pay_from_account_id: payFrom || null,
      });
    },
    onSuccess: () => {
      toast.success("Card added successfully");
      navigate("/cards");
    },
    onError: (error: unknown) => {
      toast.error(
        error instanceof Error && error.message.startsWith("Statement")
          ? error.message
          : "Failed to create card",
      );
      console.error(error);
    },
  });

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <div className="page stack page-narrow">
        <div>
          <Button
            variant="ghost"
            className="mb-4 pl-0"
            onClick={() => navigate("/cards")}
          >
            <ArrowLeft className="mr-2 h-4 w-4" /> Cards
          </Button>
          <h1 className="page-title">Add a card</h1>
          <p className="page-sub">Enter card details and customize its look.</p>
        </div>

        {/* Preview */}
        <Card>
          <CardHeader className="pb-4">
            <CardTitle className="text-[24px]">
            <span className="brand-text">Preview</span>
          </CardTitle>
            <CardDescription className="max-w-prose leading-snug">
              This is how your card will appear on the dashboard.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-center rounded-xl bg-[radial-gradient(90%_90%_at_50%_0%,hsl(var(--brand-tint)),hsl(var(--chip)))] p-4 sm:p-8">
              <CreditCard
                id="preview"
                name={name || "CARD NAME"}
                bank={bank || "BANK"}
                balance={0}
                lastDelta={0}
                primaryColor={hexToRgb(primaryColor)}
                secondaryColor={hexToRgb(secondaryColor)}
                last4Digits={last4Digits || "1234"}
              />
            </div>
          </CardContent>
        </Card>

        {/* Form */}
        <Card>
          <CardContent>
            <div className="form-grid">
              <div className="field">
                <Label htmlFor="card-name">Card Name / Nickname</Label>
                <Input
                  id="card-name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. Travel Rewards"
                />
              </div>

              <div className="field">
                <Label htmlFor="card-bank">Bank Name</Label>
                <Input
                  id="card-bank"
                  value={bank}
                  onChange={(e) => setBank(e.target.value)}
                  placeholder="e.g. Chase"
                />
              </div>

              <div className="field">
                <Label htmlFor="card-last4">Last 4 Digits (Optional)</Label>
                <Input
                  id="card-last4"
                  inputMode="numeric"
                  value={last4Digits}
                  onChange={(e) => setLast4Digits(e.target.value)}
                  placeholder="e.g. 1234"
                  maxLength={4}
                />
              </div>

              <div className="form-grid col-span-full border-t border-border pt-6">
                <div className="col-span-full flex items-center gap-1.5">
                  <p className="text-sm font-medium">
                    Billing cycle (optional)
                  </p>
                  <InfoTip text="Used to work out when each bill is due. Find the statement date and due date on your card's bank page." />
                </div>
                <div className="field">
                  <Label htmlFor="statement-day">Statement day</Label>
                  <Input
                    id="statement-day"
                    inputMode="numeric"
                    maxLength={2}
                    placeholder="e.g. 23"
                    value={statementDay}
                    onChange={(e) => setStatementDay(e.target.value)}
                  />
                </div>
                <div className="field">
                  <Label htmlFor="due-day">Due day</Label>
                  <Input
                    id="due-day"
                    inputMode="numeric"
                    maxLength={2}
                    placeholder="e.g. 13"
                    value={dueDay}
                    onChange={(e) => setDueDay(e.target.value)}
                  />
                </div>
                <div className="field">
                  <Label htmlFor="credit-limit">Credit limit</Label>
                  <MoneyInput
                    id="credit-limit"
                    value={creditLimit}
                    onChange={(e) => setCreditLimit(e.target.value)}
                  />
                </div>
                <div className="field">
                  <Label htmlFor="pay-from">Bill is paid from</Label>
                  <Select
                    id="pay-from"
                    value={payFrom}
                    onChange={(e) => setPayFrom(e.target.value)}
                  >
                    <option value="">Reserve account</option>
                    {accounts
                      ?.filter((a) => !a.locked)
                      .map((a) => (
                        <option key={a.account_id} value={a.account_id}>
                          {a.name}
                        </option>
                      ))}
                  </Select>
                </div>
              </div>

              {/* One tap to a ready-made look; the pickers below fine-tune it. */}
              <fieldset className="field col-span-full">
                <legend className="text-sm font-medium">Quick looks</legend>
                <div className="flex flex-wrap gap-2">
                  {LOOKS.map((l) => (
                    <button
                      key={l.name}
                      type="button"
                      aria-label={`${l.name} look`}
                      aria-pressed={
                        primaryColor === l.from && secondaryColor === l.to
                      }
                      title={l.name}
                      onClick={() => {
                        setPrimaryColor(l.from);
                        setSecondaryColor(l.to);
                      }}
                      className="h-11 w-16 rounded-xl border-2 border-transparent shadow-sm transition-transform hover:-translate-y-0.5 aria-pressed:border-foreground"
                      style={{
                        background: `linear-gradient(135deg, ${l.from}, ${l.to})`,
                      }}
                    />
                  ))}
                </div>
              </fieldset>

              <div className="field">
                <Label htmlFor="primary-color">Primary Color</Label>
                <div className="flex gap-2">
                  <Input
                    id="primary-color"
                    type="color"
                    value={primaryColor}
                    onChange={(e) => setPrimaryColor(e.target.value)}
                    className="h-10 w-20 shrink-0 cursor-pointer p-1"
                  />
                  <Input
                    aria-label="Primary color hex"
                    value={primaryColor}
                    onChange={(e) => setPrimaryColor(e.target.value)}
                    className="min-w-0 flex-1 uppercase"
                    maxLength={7}
                  />
                </div>
              </div>

              <div className="field">
                <Label htmlFor="secondary-color">Secondary Color</Label>
                <div className="flex gap-2">
                  <Input
                    id="secondary-color"
                    type="color"
                    value={secondaryColor}
                    onChange={(e) => setSecondaryColor(e.target.value)}
                    className="h-10 w-20 shrink-0 cursor-pointer p-1"
                  />
                  <Input
                    aria-label="Secondary color hex"
                    value={secondaryColor}
                    onChange={(e) => setSecondaryColor(e.target.value)}
                    className="min-w-0 flex-1 uppercase"
                    maxLength={7}
                  />
                </div>
              </div>

              <div className="form-actions">
                <ResetButton
                  label="Reset"
                  onReset={() => {
                    setName("My Card");
                    setBank("Bank Name");
                    setPrimaryColor("#1e293b");
                    setSecondaryColor("#334155");
                    setLast4Digits("");
                    setStatementDay("");
                    setDueDay("");
                    setCreditLimit("");
                    setPayFrom("");
                  }}
                />
                <Button
                  onClick={() => createCardMutation.mutate()}
                  disabled={createCardMutation.isPending}
                >
                  {createCardMutation.isPending ? "Creating..." : "Create Card"}
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
