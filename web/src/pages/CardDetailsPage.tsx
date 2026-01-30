import { useState, lazy, Suspense } from "react"
import type { CardData } from "./DashboardPage"
import { useParams, useNavigate } from "react-router-dom"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { ArrowLeft, Trash2, TrendingUp, TrendingDown, Minus, RefreshCcw } from "lucide-react"
import { toast } from "sonner"

import api from "@/services/api"
import { Button } from "@/components/ui/button"
import { CreditCard } from "@/components/feature/CreditCard"
import { Header } from "@/components/layout/Header"
import { cn } from "@/utils/cn"

const UpdateTransactionModal = lazy(() => import("@/components/feature/UpdateTransactionModal").then(m => ({ default: m.UpdateTransactionModal })))
const ConfirmationModal = lazy(() => import("@/components/ui/ConfirmationModal").then(m => ({ default: m.ConfirmationModal })))

export default function CardDetailsPage() {
    const { id } = useParams<{ id: string }>()
    const navigate = useNavigate()
    const queryClient = useQueryClient()
    const [isModalOpen, setIsModalOpen] = useState(false)
    const [isResetConfirmOpen, setIsResetConfirmOpen] = useState(false)
    const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false)

    // Fetch Card Details
    const cardQuery = useQuery({
        queryKey: ['card', id],
        queryFn: async () => {
            const res = await api.post<CardData>('/card/get_card', { card_id: id })
            return res.data
        },
        enabled: !!id
    })

    // Fetch History
    const historyQuery = useQuery({
        queryKey: ['history', id],
        queryFn: async () => {
            const res = await api.post<any[]>('/card/history', { card_id: id })
            return res.data
        },
        enabled: !!id
    })

    const deleteMutation = useMutation({
        mutationFn: async () => {
            await api.post('/card/delete', { card_id: id })
        },
        onSuccess: () => {
            toast.success("Card deleted")
            queryClient.invalidateQueries({ queryKey: ['cards'] })
            navigate('/')
        },
        onError: () => {
            toast.error("Failed to delete card")
            setIsDeleteConfirmOpen(false)
        }
    })

    const resetMutation = useMutation({
        mutationFn: async () => {
            await api.post('/card/reset', { card_id: id })
        },
        onSuccess: () => {
            toast.success("Transaction history reset")
            queryClient.invalidateQueries({ queryKey: ['card', id] })
            queryClient.invalidateQueries({ queryKey: ['history', id] })
            queryClient.invalidateQueries({ queryKey: ['cards'] })
            setIsResetConfirmOpen(false)
        },
        onError: () => {
            toast.error("Failed to reset transactions")
            setIsResetConfirmOpen(false)
        }
    })

    if (cardQuery.isLoading) return <div className="p-8 text-center text-muted-foreground">Loading details...</div>
    if (cardQuery.isError || !cardQuery.data) return <div className="p-8 text-center text-destructive">Card not found</div>

    const card = cardQuery.data

    return (
        <div className="min-h-screen bg-background text-foreground transition-colors duration-300">
            <Header />

            <div className="container mx-auto px-4 py-8 max-w-4xl space-y-8">

                {/* Header Actions */}
                <div className="flex items-center justify-between">
                    <Button variant="ghost" className="pl-0" onClick={() => navigate(-1)}>
                        <ArrowLeft className="mr-2 h-4 w-4" /> Back
                    </Button>
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setIsResetConfirmOpen(true)}
                            disabled={resetMutation.isPending}
                        >
                            <RefreshCcw className={cn("mr-2 h-4 w-4", resetMutation.isPending && "animate-spin")} /> Reset History
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

                <div className="grid gap-6 md:gap-8 md:grid-cols-[1.2fr,1fr]">
                    {/* Left Column: Card Visual & Actions */}
                    <div className="space-y-6">
                        <div className="flex justify-center rounded-xl p-0 md:p-6 transition-all">
                            <CreditCard
                                id={card.card_id}
                                name={card.card_name}
                                bank={card.card_bank}
                                balance={card.last_total_due || 0}
                                lastDelta={card.last_delta || 0}
                                primaryColor={card.card_primary_color}
                                secondaryColor={card.card_secondary_color}
                                variant="custom"
                            />
                        </div>

                        <Button className="w-full text-lg h-14 bg-primary text-primary-foreground hover:bg-primary/90 rounded-2xl shadow-lg" onClick={() => setIsModalOpen(true)}>
                            Update Transaction
                        </Button>
                    </div>

                    {/* Right Column: History */}
                    <div className="space-y-4">
                        <h3 className="text-xl font-bold tracking-tight">Transaction History</h3>

                        {historyQuery.isLoading ? (
                            <div className="text-muted-foreground animate-pulse">Loading history...</div>
                        ) : historyQuery.data && historyQuery.data.length > 0 ? (
                            <div className="space-y-3">
                                {(() => {
                                    // Sort by timestamp descending (newest first)
                                    const sorted = [...historyQuery.data].sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

                                    return sorted.map((tx: any, index: number) => {
                                        const previousTx = sorted[index + 1];
                                        const previousAmount = previousTx ? previousTx.total_due_input : 0;
                                        const delta = tx.total_due_input - previousAmount;

                                        return (
                                            <div key={tx.transaction_id} className="flex items-center justify-between rounded-xl border border-border bg-card p-4 transition-all hover:bg-accent/5">
                                                <div className="space-y-0.5">
                                                    <p className="font-bold text-base">
                                                        {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(tx.total_due_input)}
                                                    </p>
                                                    <p className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
                                                        {new Date(tx.timestamp).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })} at {new Date(tx.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                                    </p>
                                                </div>

                                                <div className={cn(
                                                    "flex items-center text-sm font-bold px-3 py-1 rounded-full",
                                                    delta > 0 ? "text-rose-500 bg-rose-500/10" : delta < 0 ? "text-emerald-500 bg-emerald-500/10" : "text-muted-foreground bg-muted/50"
                                                )}>
                                                    {delta > 0 ? <TrendingUp className="mr-1 h-3.5 w-3.5" /> : delta < 0 ? <TrendingDown className="mr-1 h-3.5 w-3.5" /> : <Minus className="mr-1 h-3.5 w-3.5" />}
                                                    <span className="tabular-nums">{delta > 0 ? '+' : ''}{new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(Math.abs(delta))}</span>
                                                </div>
                                            </div>
                                        )
                                    })
                                })()}
                            </div>
                        ) : (
                            <div className="rounded-2xl border border-dashed border-border p-12 text-center text-muted-foreground bg-muted/20">
                                No transactions yet.
                            </div>
                        )}
                    </div>
                </div>

                <Suspense fallback={null}>
                    <UpdateTransactionModal
                        isOpen={isModalOpen}
                        onClose={() => setIsModalOpen(false)}
                        cardId={id!}
                        currentBalance={card.last_total_due || 0}
                    />

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
                </Suspense>
            </div>
        </div>
    )
}
