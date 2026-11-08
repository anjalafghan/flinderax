import { useState, lazy, Suspense } from "react";
import type { CardData } from "./DashboardPage";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  ArrowLeft,
  Trash2,
  TrendingUp,
  TrendingDown,
  Minus,
  RefreshCcw,
  Settings,
  X,
} from "lucide-react";
import { toast } from "sonner";

import api from "@/services/api";
import { decodeCardHistoryList } from "@/proto/decoder";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CreditCard } from "@/components/feature/CreditCard";
import { Header } from "@/components/layout/Header";
import { useBreakdown } from "@/hooks/usePlanner";
import { CardBreakdownPanel } from "@/components/feature/CardBreakdownPanel";
import { CardSnapshotForm } from "@/components/feature/CardSnapshotForm";
import { CardLoansSection } from "@/components/feature/CardLoansSection";
import { cn } from "@/utils/cn";

const ConfirmationModal = lazy(() =>
  import("@/components/ui/ConfirmationModal").then((m) => ({
    default: m.ConfirmationModal,
  })),
);
const EditCardModal = lazy(() =>
  import("@/components/feature/EditCardModal").then((m) => ({
    default: m.EditCardModal,
  })),
);

const formatINR = (amount: number) =>
  new Intl.NumberFormat("en-IN", { style: "currency", currency: "INR" }).format(amount);

export default function CardDetailsPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [isResetConfirmOpen, setIsResetConfirmOpen] = useState(false);
  const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false);
  const [isEditCardOpen, setIsEditCardOpen] = useState(false);

  // Fetch Card Details
  const cardQuery = useQuery({
    queryKey: ["card", id],
    queryFn: async () => {
      const res = await api.post<CardData>("/api/card/get_card", {
        card_id: id,
      });
      return res.data;
    },
    enabled: !!id,
  });

  const breakdownQuery = useBreakdown(id);

  // Fetch History (protobuf)
  const historyQuery = useQuery({
    queryKey: ["history", id],
    queryFn: async () => {
      const buffer = await api.postProtobuf("/api/card/history", {
        card_id: id,
      });
      const decoded = await decodeCardHistoryList(buffer);
      return decoded.histories;
    },
    enabled: !!id,
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      await api.post("/api/card/delete", { card_id: id });
    },
    onSuccess: () => {
      toast.success("Card deleted");
      queryClient.invalidateQueries({ queryKey: ["cards"] });
      navigate("/");
    },
    onError: () => {
      toast.error("Failed to delete card");
      setIsDeleteConfirmOpen(false);
    },
  });

  const resetMutation = useMutation({
    mutationFn: async () => {
      await api.post("/api/card/reset", { card_id: id });
    },
    onSuccess: () => {
      toast.success("Transaction history reset");
      queryClient.invalidateQueries({ queryKey: ["card", id] });
      queryClient.invalidateQueries({ queryKey: ["history", id] });
      queryClient.invalidateQueries({ queryKey: ["cards"] });
      setIsResetConfirmOpen(false);
    },
    onError: () => {
      toast.error("Failed to reset transactions");
      setIsResetConfirmOpen(false);
    },
  });

  const deleteTransactionMutation = useMutation({
    mutationFn: async (transactionId: string) => {
      await api.post("/api/card/delete_transaction", {
        card_id: id,
        transaction_id: transactionId,
      });
    },
    onSuccess: () => {
      toast.success("Transaction deleted");
      queryClient.invalidateQueries({ queryKey: ["history", id] });
      queryClient.invalidateQueries({ queryKey: ["card", id] });
      queryClient.invalidateQueries({ queryKey: ["cards"] });
    },
    onError: () => {
      toast.error("Failed to delete transaction");
    },
  });

  if (cardQuery.isLoading)
    return (
      <div className="p-8 text-center text-muted-foreground">
        Loading details...
      </div>
    );
  if (cardQuery.isError || !cardQuery.data)
    return (
      <div className="p-8 text-center text-destructive">Card not found</div>
    );

  const card = cardQuery.data;

  return (
    <div className="min-h-screen text-foreground transition-colors duration-300">
      <Header />

      <div className="page stack max-w-4xl">
        {/* Header Actions */}
        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button variant="ghost" className="pl-0" onClick={() => navigate(-1)}>
            <ArrowLeft className="mr-2 h-4 w-4" /> Back
          </Button>
          <div className="flex flex-wrap gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsEditCardOpen(true)}
            >
              <Settings className="mr-2 h-4 w-4" /> Edit Card
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsResetConfirmOpen(true)}
              disabled={resetMutation.isPending}
            >
              <RefreshCcw
                className={cn(
                  "mr-2 h-4 w-4",
                  resetMutation.isPending && "animate-spin",
                )}
              />{" "}
              Reset History
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setIsDeleteConfirmOpen(true)}
              disabled={deleteMutation.isPending}
            >
              <Trash2 className="mr-2 h-4 w-4" /> Delete Card
            </Button>
          </div>
        </div>

        <div className="grid gap-6 md:gap-8 md:grid-cols-[1.2fr_1fr]">
          {/* Left Column: Card Visual & Actions */}
          <div className="flex flex-col gap-4">
            <div className="flex justify-center transition-all md:p-6">
              <CreditCard
                id={card.card_id}
                name={card.card_name}
                bank={card.card_bank}
                balance={card.last_total_due || 0}
                lastDelta={card.last_delta || 0}
                primaryColor={card.card_primary_color}
                secondaryColor={card.card_secondary_color}
                last4Digits={card.last_4_digits}
                variant="custom"
              />
            </div>

            <Button
              className="w-full sm:w-auto"
              onClick={() =>
                document.getElementById("snapshot-form")?.scrollIntoView({ behavior: "smooth", block: "center" })
              }
            >
              Update from bank app
            </Button>
          </div>

          {/* Right Column: History */}
          <Card>
            <CardHeader className="pb-4">
              <CardTitle className="text-[24px]">Transaction History</CardTitle>
            </CardHeader>
            <CardContent>
              {historyQuery.isLoading ? (
                <p className="animate-pulse text-sm text-muted-foreground">Loading history...</p>
              ) : historyQuery.data && historyQuery.data.length > 0 ? (
                <ul className="ledger">
                  {(() => {
                    // Sort by timestamp descending (newest first)
                    const sorted = [...historyQuery.data].sort(
                      (a, b) =>
                        new Date(b.timestamp).getTime() -
                        new Date(a.timestamp).getTime(),
                    );

                    return sorted.map((tx, index: number) => {
                      const previousTx = sorted[index + 1];
                      const previousAmount = previousTx
                        ? previousTx.total_due_input
                        : 0;
                      const delta = tx.total_due_input - previousAmount;

                      return (
                        <li key={tx.transaction_id} className="ledger-row text-sm">
                          <div className="min-w-0">
                            <p className="font-display text-lg">{formatINR(tx.total_due_input)}</p>
                            <p className="text-[13px] leading-snug text-muted-foreground">
                              {new Date(tx.timestamp).toLocaleDateString(
                                undefined,
                                { month: "short", day: "numeric" },
                              )}{" "}
                              at{" "}
                              {new Date(tx.timestamp).toLocaleTimeString([], {
                                hour: "2-digit",
                                minute: "2-digit",
                              })}
                            </p>
                          </div>

                          <div className="flex shrink-0 items-center gap-2">
                            <div
                              className={cn(
                                "flex items-center rounded-full px-3 py-1 text-sm font-semibold",
                                delta > 0
                                  ? "tone-over"
                                  : delta < 0
                                    ? "tone-good"
                                    : "tone-quiet",
                              )}
                            >
                              {delta > 0 ? (
                                <TrendingUp className="mr-1 h-3.5 w-3.5" />
                              ) : delta < 0 ? (
                                <TrendingDown className="mr-1 h-3.5 w-3.5" />
                              ) : (
                                <Minus className="mr-1 h-3.5 w-3.5" />
                              )}
                              <span className="tabular-nums">
                                {delta > 0 ? "+" : ""}
                                {formatINR(Math.abs(delta))}
                              </span>
                            </div>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-9 w-9"
                              aria-label="Delete transaction"
                              onClick={() => {
                                if (window.confirm("Delete this transaction?")) {
                                  deleteTransactionMutation.mutate(tx.transaction_id);
                                }
                              }}
                            >
                              <X className="h-4 w-4" />
                            </Button>
                          </div>
                        </li>
                      );
                    });
                  })()}
                </ul>
              ) : (
                <p className="empty-note">No transactions yet.</p>
              )}
            </CardContent>
          </Card>
        </div>

        {breakdownQuery.data && (
          <>
            <div className="grid gap-6 md:gap-8 md:grid-cols-2">
              <CardBreakdownPanel breakdown={breakdownQuery.data} />
              <CardSnapshotForm cardId={card.card_id} isFirst={breakdownQuery.data.last_updated === null} />
            </div>
            <CardLoansSection cardId={card.card_id} breakdown={breakdownQuery.data} />
          </>
        )}

        <Suspense fallback={null}>
          <ConfirmationModal
            isOpen={isResetConfirmOpen}
            onClose={() => setIsResetConfirmOpen(false)}
            onConfirm={() => resetMutation.mutate()}
            title="Reset Transaction History"
            description="Are you sure you want to reset all transactions for this card? This will set the balance to zero and cannot be undone."
            confirmText="Reset History"
            variant="destructive"
            isPending={resetMutation.isPending}
          />

          <ConfirmationModal
            isOpen={isDeleteConfirmOpen}
            onClose={() => setIsDeleteConfirmOpen(false)}
            onConfirm={() => deleteMutation.mutate()}
            title="Delete Credit Card"
            description={`Are you sure you want to delete "${card.card_name}"? All transaction history and settings for this card will be permanently removed.`}
            confirmText="Delete Card"
            variant="destructive"
            isPending={deleteMutation.isPending}
          />

          <EditCardModal
            key={`${card.card_id}-${isEditCardOpen}`}
            isOpen={isEditCardOpen}
            onClose={() => setIsEditCardOpen(false)}
            cardId={card.card_id}
            currentName={card.card_name}
            currentBank={card.card_bank}
            currentPrimaryColor={card.card_primary_color}
            currentSecondaryColor={card.card_secondary_color}
            currentLast4Digits={card.last_4_digits}
            currentStatementDay={card.statement_day}
            currentDueDay={card.due_day}
            currentCreditLimitPaise={card.credit_limit_paise}
            currentPayFrom={card.pay_from_account_id}
          />
        </Suspense>
      </div>
    </div>
  );
}
