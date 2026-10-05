import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { ArrowLeft } from "lucide-react";

import api from "@/services/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import { CreditCard } from "@/components/feature/CreditCard";
import { InfoTip } from "@/components/ui/info-tip";
import { MoneyInput } from "@/components/ui/money-input";
import { toPaise } from "@/utils/money";

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

  const createCardMutation = useMutation({
    mutationFn: async () => {
      if ((statementDay.trim() && parseDay(statementDay) === null) || (dueDay.trim() && parseDay(dueDay) === null)) {
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
      });
    },
    onSuccess: () => {
      toast.success("Card added successfully");
      navigate("/");
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error && error.message.startsWith("Statement") ? error.message : "Failed to create card");
      console.error(error);
    },
  });

  return (
    <div className="min-h-screen bg-gray-50/50 p-6 md:p-10 dark:bg-gray-900/50">
      <div className="mx-auto max-w-2xl space-y-8">
        <div>
          <Button
            variant="ghost"
            className="mb-4 pl-0"
            onClick={() => navigate(-1)}
          >
            <ArrowLeft className="mr-2 h-4 w-4" /> Back to Dashboard
          </Button>
          <h1 className="text-3xl font-bold tracking-tight">Add New Card</h1>
          <p className="text-muted-foreground">
            Enter card details and customize its look.
          </p>
        </div>

        <div className="grid gap-8 md:grid-cols-2">
          {/* Form */}
          <div className="space-y-6">
            <div className="space-y-2">
              <Label>Card Name / Nickname</Label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Travel Rewards"
              />
            </div>

            <div className="space-y-2">
              <Label>Bank Name</Label>
              <Input
                value={bank}
                onChange={(e) => setBank(e.target.value)}
                placeholder="e.g. Chase"
              />
            </div>

            <div className="space-y-2">
              <Label>Last 4 Digits (Optional)</Label>
              <Input
                value={last4Digits}
                onChange={(e) => setLast4Digits(e.target.value)}
                placeholder="e.g. 1234"
                maxLength={4}
              />
            </div>

            <div className="space-y-3 rounded-xl border p-4">
              <div className="flex items-center gap-1.5">
                <p className="text-sm font-medium">Billing cycle (optional)</p>
                <InfoTip text="Used to work out when each bill is due. Find the statement date and due date on your card's bank page." />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="statement-day">Statement day</Label>
                  <Input id="statement-day" inputMode="numeric" maxLength={2} placeholder="e.g. 23" value={statementDay} onChange={(e) => setStatementDay(e.target.value)} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="due-day">Due day</Label>
                  <Input id="due-day" inputMode="numeric" maxLength={2} placeholder="e.g. 13" value={dueDay} onChange={(e) => setDueDay(e.target.value)} />
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="credit-limit">Credit limit</Label>
                <MoneyInput id="credit-limit" value={creditLimit} onChange={(e) => setCreditLimit(e.target.value)} />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Primary Color</Label>
                <div className="flex gap-2">
                  <Input
                    type="color"
                    value={primaryColor}
                    onChange={(e) => setPrimaryColor(e.target.value)}
                    className="h-10 w-20 p-1 cursor-pointer"
                  />
                  <Input
                    value={primaryColor}
                    onChange={(e) => setPrimaryColor(e.target.value)}
                    className="uppercase"
                    maxLength={7}
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label>Secondary Color</Label>
                <div className="flex gap-2">
                  <Input
                    type="color"
                    value={secondaryColor}
                    onChange={(e) => setSecondaryColor(e.target.value)}
                    className="h-10 w-20 p-1 cursor-pointer"
                  />
                  <Input
                    value={secondaryColor}
                    onChange={(e) => setSecondaryColor(e.target.value)}
                    className="uppercase"
                    maxLength={7}
                  />
                </div>
              </div>
            </div>

            <Button
              className="w-full"
              size="lg"
              onClick={() => createCardMutation.mutate()}
              disabled={createCardMutation.isPending}
            >
              {createCardMutation.isPending ? "Creating..." : "Create Card"}
            </Button>
          </div>

          {/* Preview */}
          <div className="flex flex-col gap-4">
            <Label>Preview</Label>
            <div className="flex items-center justify-center rounded-xl bg-gray-100 p-8 dark:bg-gray-800">
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
            <p className="text-center text-xs text-muted-foreground">
              This is how your card will appear on the dashboard.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
