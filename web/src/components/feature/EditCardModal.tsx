import { useState, useCallback, useMemo } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { X } from "lucide-react";

import api from "@/services/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CreditCard } from "@/components/feature/CreditCard";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { useAccounts } from "@/hooks/usePlanner";
import { paiseToInput, toPaise } from "@/utils/money";

const parseDay = (v: string): number | null => {
  const n = Number(v);
  return v.trim() && Number.isInteger(n) && n >= 1 && n <= 31 ? n : null;
};

interface EditCardModalProps {
  isOpen: boolean;
  onClose: () => void;
  cardId: string;
  currentName: string;
  currentBank: string;
  currentPrimaryColor: [number, number, number];
  currentSecondaryColor: [number, number, number];
  currentLast4Digits: string | null;
  currentStatementDay?: number | null;
  currentDueDay?: number | null;
  currentCreditLimitPaise?: number | null;
  currentPayFrom?: string | null;
}

function rgbToHex(r: number, g: number, b: number): string {
  return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
}

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

export function EditCardModal({
  isOpen,
  onClose,
  cardId,
  currentName,
  currentBank,
  currentPrimaryColor,
  currentSecondaryColor,
  currentLast4Digits,
  currentStatementDay,
  currentDueDay,
  currentCreditLimitPaise,
  currentPayFrom,
}: EditCardModalProps) {
  const [name, setName] = useState(currentName);
  const [bank, setBank] = useState(currentBank);
  const [primaryColor, setPrimaryColor] = useState(() =>
    rgbToHex(...currentPrimaryColor),
  );
  const [secondaryColor, setSecondaryColor] = useState(() =>
    rgbToHex(...currentSecondaryColor),
  );
  const [last4Digits, setLast4Digits] = useState(currentLast4Digits || "");
  const [statementDay, setStatementDay] = useState(currentStatementDay ? String(currentStatementDay) : "");
  const [dueDay, setDueDay] = useState(currentDueDay ? String(currentDueDay) : "");
  const [creditLimit, setCreditLimit] = useState(currentCreditLimitPaise ? paiseToInput(currentCreditLimitPaise) : "");
  const [payFrom, setPayFrom] = useState(currentPayFrom ?? "");
  const { data: accounts } = useAccounts();
  const queryClient = useQueryClient();

  // The form is seeded from props once; the page remounts it (a new `key`) each time it opens.

  const handleNameChange = useCallback((value: string) => setName(value), []);
  const handleBankChange = useCallback((value: string) => setBank(value), []);
  const handleLast4DigitsChange = useCallback(
    (value: string) => setLast4Digits(value),
    [],
  );
  const handlePrimaryColorChange = useCallback(
    (value: string) => setPrimaryColor(value),
    [],
  );
  const handleSecondaryColorChange = useCallback(
    (value: string) => setSecondaryColor(value),
    [],
  );

  const primaryColorRgb = useMemo(() => hexToRgb(primaryColor), [primaryColor]);
  const secondaryColorRgb = useMemo(
    () => hexToRgb(secondaryColor),
    [secondaryColor],
  );
  const previewLast4Digits = useMemo(
    () => last4Digits || currentLast4Digits || null,
    [last4Digits, currentLast4Digits],
  );

  const updateMutation = useMutation({
    mutationFn: async () => {
      if ((statementDay.trim() && parseDay(statementDay) === null) || (dueDay.trim() && parseDay(dueDay) === null)) {
        throw new Error("Statement day and due day must be between 1 and 31");
      }
      if (creditLimit.trim() && toPaise(creditLimit) === null) {
        throw new Error("Credit limit must be an amount");
      }
      await api.post("/api/card/update", {
        card_id: cardId,
        card_name: name,
        card_bank: bank,
        card_primary_color: hexToRgb(primaryColor),
        card_secondary_color: hexToRgb(secondaryColor),
        last_4_digits: last4Digits || null,
        // omitted/empty keeps the stored value
        statement_day: parseDay(statementDay),
        due_day: parseDay(dueDay),
        credit_limit_paise: creditLimit.trim() ? toPaise(creditLimit) : null,
        pay_from_account_id: payFrom || null,
      });
    },
    onSuccess: () => {
      toast.success("Card updated successfully");
      queryClient.invalidateQueries({ queryKey: ["cards"] });
      queryClient.invalidateQueries({ queryKey: ["card", cardId] });
      queryClient.invalidateQueries({ queryKey: ["card-breakdown", cardId] });
      queryClient.invalidateQueries({ queryKey: ["dashboard-summary"] });
      onClose();
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error && error.message.startsWith("Statement") ? error.message : "Failed to update card");
      console.error(error);
    },
  });

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-foreground/30 p-4">
      <Card className="w-full max-w-md animate-in fade-in zoom-in duration-200">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-3xl">Edit Card</CardTitle>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="h-8 w-8 rounded-full"
          >
            <X className="h-4 w-4" />
          </Button>
        </CardHeader>
        <CardContent className="space-y-4 pt-4">
          <div className="space-y-2">
            <Label>Card Name / Nickname</Label>
            <Input
              value={name}
              onChange={(e) => handleNameChange(e.target.value)}
              placeholder="e.g. Travel Rewards"
            />
          </div>

          <div className="space-y-2">
            <Label>Bank Name</Label>
            <Input
              value={bank}
              onChange={(e) => handleBankChange(e.target.value)}
              placeholder="e.g. Chase"
            />
          </div>

          <div className="space-y-2">
            <Label>Last 4 Digits</Label>
            <Input
              value={last4Digits}
              onChange={(e) => handleLast4DigitsChange(e.target.value)}
              placeholder="e.g. 1234"
              maxLength={4}
              pattern="[0-9]{0,4}"
            />
          </div>

          <div className="grid grid-cols-3 gap-3">
            <div className="space-y-2">
              <Label htmlFor="edit-statement-day">Statement day</Label>
              <Input id="edit-statement-day" inputMode="numeric" maxLength={2} value={statementDay} onChange={(e) => setStatementDay(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-due-day">Due day</Label>
              <Input id="edit-due-day" inputMode="numeric" maxLength={2} value={dueDay} onChange={(e) => setDueDay(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-limit">Limit</Label>
              <MoneyInput id="edit-limit" value={creditLimit} onChange={(e) => setCreditLimit(e.target.value)} />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="edit-pay-from">Bill is paid from</Label>
            <Select id="edit-pay-from" value={payFrom} onChange={(e) => setPayFrom(e.target.value)}>
              <option value="">Reserve account</option>
              {accounts?.filter((a) => !a.locked).map((a) => (
                <option key={a.account_id} value={a.account_id}>{a.name}</option>
              ))}
            </Select>
          </div>

          <div className="glass rounded-3xl p-6">
            <Label className="mb-2 text-sm font-medium">Preview</Label>
            <CreditCard
              id="preview"
              name={name || "CARD NAME"}
              bank={bank || "BANK"}
              balance={0}
              lastDelta={0}
              primaryColor={primaryColorRgb}
              secondaryColor={secondaryColorRgb}
              last4Digits={previewLast4Digits}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>Primary Color</Label>
              <div className="flex gap-2">
                <Input
                  type="color"
                  value={primaryColor}
                  onChange={(e) => handlePrimaryColorChange(e.target.value)}
                  className="h-10 w-20 p-1 cursor-pointer"
                />
                <Input
                  value={primaryColor}
                  onChange={(e) => handlePrimaryColorChange(e.target.value)}
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
                  onChange={(e) => handleSecondaryColorChange(e.target.value)}
                  className="h-10 w-20 p-1 cursor-pointer"
                />
                <Input
                  value={secondaryColor}
                  onChange={(e) => handleSecondaryColorChange(e.target.value)}
                  className="uppercase"
                  maxLength={7}
                />
              </div>
            </div>
          </div>

          <div className="flex gap-2">
            <Button variant="outline" className="flex-1" onClick={onClose}>
              Cancel
            </Button>
            <Button
              className="flex-1"
              onClick={() => updateMutation.mutate()}
              disabled={updateMutation.isPending}
            >
              {updateMutation.isPending ? "Saving..." : "Save Changes"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
