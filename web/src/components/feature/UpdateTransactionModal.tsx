import { useState } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { X, ArrowRight } from "lucide-react"

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

export function UpdateTransactionModal({ isOpen, onClose, cardId, currentBalance }: UpdateTransactionModalProps) {
    const [amount, setAmount] = useState<string>("")
    const [showSuggestion, setShowSuggestion] = useState<{ delta: number, newBalance: number } | null>(null)
    const queryClient = useQueryClient()

    const transactionMutation = useMutation({
        mutationFn: async () => {
            const amountFloat = parseFloat(amount)
            if (isNaN(amountFloat)) throw new Error("Invalid amount")

            const res = await api.post("/card/insert_transaction", {
                card_id: cardId,
                amount_due: amountFloat,
            })
            return res.data
        },
        onSuccess: (data) => {
            // Calculate delta locally for immediate feedback if backend doesn't return it conveniently in the response object shape we expect
            // But the endpoint actually returns `amount_due` which is the delta in the `InsertTransactionResponse` struct based on backend code!
            // Wait, let's double check backend.
            // struct InsertTransactionResponse { amount_due: f32 (which is assigned last_delta), ... }

            const delta = data.amount_due;
            const newBalance = parseFloat(amount); // The input total

            if (delta > 0) {
                setShowSuggestion({ delta, newBalance })
                // Don't close immediately if we have a suggestion
                queryClient.invalidateQueries({ queryKey: [] }) // Invalidate all
            } else {
                toast.success("Balance updated successfully")
                handleClose()
            }
        },
        onError: (error: any) => {
            toast.error("Failed to add transaction")
            console.error(error)
        }
    })

    const handleClose = () => {
        setAmount("")
        setShowSuggestion(null)
        onClose()
        // Refresh data on close
        queryClient.invalidateQueries({ queryKey: ['card', cardId] })
        queryClient.invalidateQueries({ queryKey: ['history', cardId] })
        queryClient.invalidateQueries({ queryKey: ['cards'] })
    }

    if (!isOpen) return null

    // If showing suggestion state
    if (showSuggestion) {
        return (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
                <Card className="w-full max-w-md animate-in fade-in zoom-in duration-200 border-l-4 border-l-orange-500">
                    <CardHeader className="flex flex-row items-start justify-between pb-2">
                        <div>
                            <CardTitle className="text-xl font-bold">Transfer Required</CardTitle>
                            <p className="text-sm text-muted-foreground mt-1">Your credit card balance has increased.</p>
                        </div>
                        <Button variant="ghost" size="icon" onClick={handleClose} className="h-8 w-8 rounded-full">
                            <X className="h-4 w-4" />
                        </Button>
                    </CardHeader>
                    <CardContent className="space-y-6 pt-2">
                        <div className="rounded-lg bg-orange-50 p-4 dark:bg-orange-950/30 border border-orange-100 dark:border-orange-900/50">
                            <div className="flex flex-col gap-1 items-center text-center">
                                <span className="text-sm font-medium text-orange-800 dark:text-orange-200">
                                    You need to transfer
                                </span>
                                <span className="text-3xl font-extrabold text-orange-600 dark:text-orange-400">
                                    {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(showSuggestion.delta)}
                                </span>
                                <span className="text-xs text-orange-700/80 dark:text-orange-300/80 mt-1">
                                    form your Savings Account to your Credit Card Payment Account.
                                </span>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <div className="flex justify-between text-sm">
                                <span className="text-muted-foreground">Previous Balance</span>
                                <span>{new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(currentBalance)}</span>
                            </div>
                            <div className="flex justify-between text-sm font-bold">
                                <span>New Total Balance</span>
                                <span>{new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(showSuggestion.newBalance)}</span>
                            </div>
                        </div>

                        <Button className="w-full bg-orange-600 hover:bg-orange-700 text-white" onClick={handleClose}>
                            I've made the transfer <ArrowRight className="ml-2 h-4 w-4" />
                        </Button>
                    </CardContent>
                </Card>
            </div>
        )
    }

    const calculatedDelta = amount ? (parseFloat(amount) - currentBalance) : 0

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
                        <span className="text-sm text-muted-foreground">Change from previous</span>
                        <div className={`text-2xl font-bold ${calculatedDelta > 0 ? 'text-destructive' : 'text-green-500'}`}>
                            {calculatedDelta > 0 ? '+' : ''}
                            {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(calculatedDelta)}
                        </div>
                    </div>

                    <div className="flex gap-2">
                        <Button variant="outline" className="flex-1" onClick={onClose}>Cancel</Button>
                        <Button
                            className="flex-1"
                            onClick={() => transactionMutation.mutate()}
                            disabled={!amount || transactionMutation.isPending}
                        >
                            {transactionMutation.isPending ? "Updating..." : "Update"}
                        </Button>
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
