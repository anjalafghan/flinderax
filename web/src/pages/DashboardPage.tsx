import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ArrowRight, Clock } from "lucide-react";
import { toast } from "sonner";

import api from "@/services/api";
import { decodeCardList } from "@/proto/decoder";
import { Header } from "@/components/layout/Header";
import { CardCarousel } from "@/components/feature/CardCarousel";
import { Button } from "@/components/ui/button";

export interface CardData {
  card_id: string;
  card_name: string;
  card_bank: string;
  card_primary_color: [number, number, number];
  card_secondary_color: [number, number, number];
  last_4_digits: string | null;
  last_total_due: number | null;
  last_delta: number | null;
}

interface DeferredItem {
  item_id: string;
  card_id: string;
  card_name: string | null;
  amount: number;
}

interface DeferredBatchStatus {
  batch_id: string;
  total_amount: number;
  status: string;
  items: DeferredItem[];
}

export default function DashboardPage() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [windowWidth, setWindowWidth] = useState(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );
  const [showSettleConfirm, setShowSettleConfirm] = useState(false);
  const [pendingToSettle, setPendingToSettle] = useState<{
    batchId: string;
    total: number;
  } | null>(null);

  const [showCarousel, setShowCarousel] = useState(false);
  const queryClient = useQueryClient();

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener("resize", handleResize);

    const t = setTimeout(() => {
      setShowCarousel(true);
    }, 100);

    return () => {
      window.removeEventListener("resize", handleResize);
      clearTimeout(t);
    };
  }, []);

  const isMobile = windowWidth < 640;

  const { data: cards, isLoading } = useQuery({
    queryKey: ["cards"],
    queryFn: async () => {
      const buffer = await api.getProtobuf("/api/card/get_all_cards");
      const decoded = await decodeCardList(buffer);

      return decoded.cards.map((c) => ({
        card_id: c.card_id,
        card_name: c.card_name,
        card_bank: c.card_bank,
        card_primary_color: unpackColor(c.card_primary_color),
        card_secondary_color: unpackColor(c.card_secondary_color),
        last_4_digits: c.last_4_digits,
        last_total_due: c.last_total_due,
        last_delta: c.last_delta,
      })) as CardData[];
    },
  });

  const { data: deferredStatus } = useQuery({
    queryKey: ["deferred-status"],
    queryFn: async () => {
      const res = await api.get<DeferredBatchStatus>("/api/card/defer_status");
      return res.data;
    },
  });

  const settleMutation = useMutation({
    mutationFn: async (batchId: string) => {
      const res = await api.post<{
        batch_id: string;
        total_settled: number;
        status: boolean;
      }>("/api/card/defer_settle", { batch_id: batchId });
      return res.data;
    },
    onSuccess: (data) => {
      if (data.status) {
        toast.success(
          `Settled ${new Intl.NumberFormat("en-IN", { style: "currency", currency: "INR" }).format(data.total_settled)} successfully!`,
        );
        setShowSettleConfirm(false);
        setPendingToSettle(null);
        queryClient.invalidateQueries({ queryKey: ["deferred-status"] });
        queryClient.invalidateQueries({ queryKey: ["cards"] });
      }
    },
    onError: (error: any) => {
      toast.error("Failed to settle payments");
      console.error(error);
    },
  });

  const handleSettle = () => {
    if (deferredStatus) {
      setPendingToSettle({
        batchId: deferredStatus.batch_id,
        total: deferredStatus.total_amount,
      });
      setShowSettleConfirm(true);
    }
  };

  const confirmSettle = () => {
    if (pendingToSettle) {
      settleMutation.mutate(pendingToSettle.batchId);
    }
  };

  const unpackColor = (packed: number): [number, number, number] => {
    const r = (packed >> 16) & 0xff;
    const g = (packed >> 8) & 0xff;
    const b = packed & 0xff;
    return [r, g, b];
  };

  const totalDue =
    cards?.reduce(
      (acc: number, card: CardData) => acc + (card.last_total_due || 0),
      0,
    ) || 0;

  useEffect(() => {
    if (!cards || cards.length === 0) return;

    const savedCardId = sessionStorage.getItem("dashboard_active_card");
    if (savedCardId) {
      const index = cards.findIndex((c) => c.card_id === savedCardId);
      if (index !== -1) {
        setActiveIndex(index);
      }
    }
  }, [cards]);

  const hasPendingDeferred = deferredStatus && deferredStatus.total_amount > 0;

  return (
    <div className="h-[100dvh] w-full bg-background text-foreground transition-colors duration-300 overflow-hidden flex flex-col">
      <Header />

      <main className="flex-1 container mx-auto px-4 py-4 md:py-12 flex flex-col justify-center overflow-hidden">
        <div className="mb-4 md:mb-16 text-center space-y-2 md:space-y-4 shrink-0">
          <h1 className="text-3xl md:text-5xl font-extrabold tracking-tight px-2">
            Explore Your Credit Cards.
          </h1>
          <p className="text-base md:text-lg text-muted-foreground max-w-xl mx-auto px-4">
            Find the perfect card for your needs. Manage your{" "}
            {new Intl.NumberFormat("en-IN", {
              style: "currency",
              currency: "INR",
            }).format(totalDue)}{" "}
            total balance effortlessly.
          </p>
        </div>

        <div className="relative mx-auto max-w-5xl h-[300px] md:h-[400px] flex items-center justify-center perspective-[1200px] overflow-visible shrink-0">
          {!showCarousel ? (
            <div className="text-muted-foreground">
              Loading visualization...
            </div>
          ) : isLoading ? (
            <div className="text-muted-foreground animate-pulse">
              Loading cards...
            </div>
          ) : (
            <CardCarousel
              cards={cards || []}
              activeIndex={activeIndex}
              setActiveIndex={setActiveIndex}
              isMobile={isMobile}
            />
          )}
        </div>

        <div className="flex justify-center gap-2 mt-4 md:mt-8 mb-4 md:mb-16 shrink-0">
          {cards?.map((_: CardData, idx: number) => (
            <button
              key={idx}
              onClick={() => setActiveIndex(idx)}
              className={`h-2 rounded-full transition-all ${idx === activeIndex ? "w-6 bg-primary" : "w-2 bg-muted-foreground/30"}`}
              aria-label={`Go to card ${idx + 1}`}
            />
          ))}
        </div>
      </main>

      {hasPendingDeferred && (
        <div className="fixed bottom-0 left-0 right-0 bg-gradient-to-t from-background via-background/95 to-transparent pt-8 pb-4 px-4 border-t border-border/50">
          <div className="container mx-auto max-w-md">
            <div className="rounded-xl border border-orange-200 dark:border-orange-800 bg-orange-50/50 dark:bg-orange-950/30 p-4">
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-full bg-orange-100 dark:bg-orange-900/50">
                    <Clock className="h-5 w-5 text-orange-600 dark:text-orange-400" />
                  </div>
                  <div>
                    <p className="text-sm font-medium text-orange-800 dark:text-orange-200">
                      Pending Payments
                    </p>
                    <p className="text-xl font-bold text-orange-600 dark:text-orange-400">
                      {new Intl.NumberFormat("en-IN", {
                        style: "currency",
                        currency: "INR",
                      }).format(deferredStatus?.total_amount || 0)}
                    </p>
                  </div>
                </div>
                <Button
                  onClick={handleSettle}
                  className="bg-orange-600 hover:bg-orange-700 text-white"
                  disabled={settleMutation.isPending}
                >
                  {settleMutation.isPending ? "Settling..." : "Settle All"}
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {showSettleConfirm && pendingToSettle && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
          <div className="bg-card rounded-xl border border-border p-6 max-w-md w-full animate-in fade-in zoom-in duration-200">
            <h2 className="text-xl font-bold mb-2">Settle All Payments</h2>
            <p className="text-muted-foreground mb-4">
              You are about to settle{" "}
              <span className="font-bold text-foreground">
                {deferredStatus?.items?.length || 0}
              </span>{" "}
              pending payments totaling{" "}
              <span className="font-bold text-green-600">
                {new Intl.NumberFormat("en-IN", {
                  style: "currency",
                  currency: "INR",
                }).format(pendingToSettle.total)}
              </span>
            </p>
            <div className="flex gap-3">
              <Button
                variant="outline"
                className="flex-1"
                onClick={() => {
                  setShowSettleConfirm(false);
                  setPendingToSettle(null);
                }}
              >
                Cancel
              </Button>
              <Button
                className="flex-1 bg-green-600 hover:bg-green-700"
                onClick={confirmSettle}
              >
                Confirm Settlement
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
