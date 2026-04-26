import { useState } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { X, ArrowRight, Check, Clock } from "lucide-react"

import api from "@/services/api"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

interface UpdateTransactionModalProps {
    isOpen: boolean
    onClose: () => void
    cardId: string
    currentBalance: number
}

interface TransactionResponse {
    transaction_id: string
    amount_due: number
    status: boolean
}

interface DeferUpdateResponse {
    batch_id: string
    item_id: string
    total_pending: number
    status: boolean
}

export function UpdateTransactionModal({ isOpen, onClose, cardId, currentBalance }: UpdateTransactionModalProps) {
    const [amount, setAmount] = useState<string>("")
    const [showDeferredConfirm, setShowDeferredConfirm] = useState<{ totalPending: number } | null>(null)
    const [showConfirmTransfer, setShowConfirmTransfer] = useState<{ delta: number } | null>(null)
    const queryClient = useQueryClient()

    const confirmTransferMutation = useMutation({
        mutationFn: async (amountDue: number) => {
            const res = await api.post<TransactionResponse>("/card/insert_transaction", {
                card_id: cardId,
                amount_due: amountDue,
            })
            return res.data
        },
        onSuccess: () => {
            toast.success(`Transfer of ${new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(Math.abs(parseFloat(amount)))} confirmed`)
            handleClose()
        },
        onError: (error: any) => {
            toast.error("Failed to confirm transfer")
            console.error(error)
        }
    })

    const deferMutation = useMutation({
        mutationFn: async (delta: number) => {
            const res = await api.post<DeferUpdateResponse>("/card/defer_update", {
                card_id: cardId,
                amount: delta,
            })
            return res.data
        },
        onSuccess: (data) => {
            if (data.status) {
                setShowDeferredConfirm({ totalPending: data.total_pending })
            }
        },
        onError: (error: any) => {
            toast.error("Failed to defer payment")
            console.error(error)
        }
    })

    const handleClose = () => {
        setAmount("")
        setShowDeferredConfirm(null)
        setShowConfirmTransfer(null)
        onClose()
        queryClient.invalidateQueries({ queryKey: ['card', cardId] })
        queryClient.invalidateQueries({ queryKey: ['history', cardId] })
        queryClient.invalidateQueries({ queryKey: ['cards'] })
        queryClient.invalidateQueries({ queryKey: ['deferred-status'] })
    }

    if (!isOpen) return null

    if (showDeferredConfirm) {
        return (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
                <Card className="w-full max-w-md animate-in fade-in zoom-in duration-200 border-l-4 border-l-blue-500">
                    <CardHeader className="flex flex-row items-start justify-between pb-2">
                        <div>
                            <CardTitle className="text-xl font-bold">Payment Deferred</CardTitle>
                            <p className="text-sm text-muted-foreground mt-1">Amount added to pending batch.</p>
                        </div>
                        <Button variant="ghost" size="icon" onClick={handleClose} className="h-8 w-8 rounded-full">
                            <X className="h-4 w-4" />
                        </Button>
                    </CardHeader>
                    <CardContent className="space-y-6 pt-2">
                        <div className="rounded-lg bg-blue-50 p-4 dark:bg-blue-950/30 border border-blue-100 dark:border-blue-900/50">
                            <div className="flex flex-col gap-1 items-center text-center">
                                <span className="text-sm font-medium text-blue-800 dark:text-blue-200">
                                    Total Pending
                                </span>
                                <span className="text-3xl font-extrabold text-blue-600 dark:text-blue-400">
                                    {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(showDeferredConfirm.totalPending)}
                                </span>
                                <span className="text-xs text-blue-700/80 dark:text-blue-300/80 mt-1">
                                    across all cards
                                </span>
                            </div>
                        </div>

                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <Clock className="h-4 w-4" />
                            <span>Settle all pending payments at once from your dashboard</span>
                        </div>

                        <Button className="w-full bg-blue-600 hover:bg-blue-700 text-white" onClick={handleClose}>
                            Got it <ArrowRight className="ml-2 h-4 w-4" />
                        </Button>
                    </CardContent>
                </Card>
            </div>
        )
    }

    if (showConfirmTransfer) {
        return (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
                <Card className="w-full max-w-md animate-in fade-in zoom-in duration-200 border-l-4 border-l-green-500">
                    <CardHeader className="flex flex-row items-start justify-between pb-2">
                        <div>
                            <CardTitle className="text-xl font-bold">Confirm Transfer</CardTitle>
                            <p className="text-sm text-muted-foreground mt-1">Transfer the amount to your credit card account.</p>
                        </div>
                        <Button variant="ghost" size="icon" onClick={handleClose} className="h-8 w-8 rounded-full">
                            <X className="h-4 w-4" />
                        </Button>
                    </CardHeader>
                    <CardContent className="space-y-6 pt-2">
                        <div className="rounded-lg bg-green-50 p-4 dark:bg-green-950/30 border border-green-100 dark:border-green-900/50">
                            <div className="flex flex-col gap-1 items-center text-center">
                                <span className="text-sm font-medium text-green-800 dark:text-green-200">
                                    You need to transfer
                                </span>
                                <span className="text-3xl font-extrabold text-green-600 dark:text-green-400">
                                    {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(showConfirmTransfer.delta)}
                                </span>
                                <span className="text-xs text-green-700/80 dark:text-green-300/80 mt-1">
                                    from your Savings Account to your Credit Card Payment Account
                                </span>
                            </div>
                        </div>

                        <div className="flex gap-2">
                            <Button variant="outline" className="flex-1" onClick={() => setShowConfirmTransfer(null)}>
                                Cancel
                            </Button>
                            <Button
                                className="flex-1 bg-green-600 hover:bg-green-700"
                                onClick={() => confirmTransferMutation.mutate(parseFloat(amount))}
                                disabled={confirmTransferMutation.isPending}
                            >
                                {confirmTransferMutation.isPending ? "Confirming..." : "I've transferred"}
                                <Check className="ml-2 h-4 w-4" />
                            </Button>
                        </div>
                    </CardContent>
                </Card>
            </div>
        )
    }

    const calculatedDelta = amount ? (parseFloat(amount) - currentBalance) : 0

    const handleConfirmTransfer = () => {
        const amountValue = parseFloat(amount)
        if (calculatedDelta > 0) {
            setShowConfirmTransfer({ delta: calculatedDelta })
        } else if (calculatedDelta < 0) {
            confirmTransferMutation.mutate(amountValue)
        } else {
            toast.info("No change in balance")
            handleClose()
        }
    }

    const handleDeferPayment = () => {
        if (calculatedDelta === 0) {
            toast.info("No change in balance")
            handleClose()
            return
        }
        deferMutation.mutate(parseFloat(amount))
    }

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <Card className="w-full max-w-md animate-in fade-in zoom-in duration-200">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-xl font-bold">Update Balance</CardTitle>
                    <Button variant="ghost" size="icon" onClick={onClose} className="h-8 w-8 rounded-full">
                        <X className="h-4 w-4" />
                    </Button>
                </CardHeader>
                <CardContent className="space-y-6 pt-4">
                    <div className="space-y-2">
                        <Label>New Total Amount Due</Label>
                        <Input
                            type="number"
                            placeholder="0.00"
                            value={amount}
                            onChange={e => setAmount(e.target.value)}
                            autoFocus
                            className="text-2xl font-bold p-6 h-16"
                        />
                        <p className="text-xs text-muted-foreground ml-1">
                            Enter a negative value (e.g. -500) if you have overpaid.
                        </p>
                    </div>

                    <div className="rounded-lg bg-muted p-4 text-center">
                        <span className="text-sm text-muted-foreground">Amount to transfer</span>
                        <div className={`text-2xl font-bold ${calculatedDelta > 0 ? 'text-destructive' : 'text-green-500'}`}>
                            {calculatedDelta > 0 ? '+' : ''}
                            {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(calculatedDelta)}
                        </div>
                    </div>

                    <div className="flex gap-2">
                        <Button variant="outline" className="flex-1" onClick={onClose}>Cancel</Button>
                        <Button
                            className="flex-1"
                            onClick={handleDeferPayment}
                            disabled={!amount || deferMutation.isPending}
                        >
                            {deferMutation.isPending ? "Deferring..." : "Defer Payment"}
                        </Button>
                    </div>

                    {calculatedDelta > 0 && (
                        <Button
                            className="w-full bg-green-600 hover:bg-green-700 text-white"
                            onClick={handleConfirmTransfer}
                        >
                            <Check className="mr-2 h-4 w-4" />
                            I've transferred {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(calculatedDelta)}
                        </Button>
                    )}

                    {calculatedDelta < 0 && (
                        <Button
                            className="w-full bg-green-600 hover:bg-green-700 text-white"
                            onClick={handleConfirmTransfer}
                        >
                            <Check className="mr-2 h-4 w-4" />
                            Confirm Overpayment Refund
                        </Button>
                    )}
                </CardContent>
            </Card>
        </div>
    )
}