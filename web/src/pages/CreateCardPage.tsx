import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { useMutation } from "@tanstack/react-query"
import { toast } from "sonner"
import { ArrowLeft } from "lucide-react"

import api from "@/services/api"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

import { CreditCard } from "@/components/feature/CreditCard"

// Helper to convert hex to rgb tuple
function hexToRgb(hex: string): [number, number, number] {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result
        ? [
            parseInt(result[1], 16),
            parseInt(result[2], 16),
            parseInt(result[3], 16)
        ]
        : [0, 0, 0];
}

export default function CreateCardPage() {
    const navigate = useNavigate()

    const [name, setName] = useState("My Card")
    const [bank, setBank] = useState("Bank Name")
    const [primaryColor, setPrimaryColor] = useState("#1e293b")
    const [secondaryColor, setSecondaryColor] = useState("#334155")

    const createCardMutation = useMutation({
        mutationFn: async () => {
            await api.post("/card/create", {
                card_name: name,
                card_bank: bank,
                card_primary_color: hexToRgb(primaryColor),
                card_secondary_color: hexToRgb(secondaryColor),
            })
        },
        onSuccess: () => {
            toast.success("Card added successfully")
            navigate("/")
        },
        onError: (error: any) => {
            toast.error("Failed to create card")
            console.error(error)
        }
    })

    return (
        <div className="min-h-screen bg-gray-50/50 p-6 md:p-10 dark:bg-gray-900/50">
            <div className="mx-auto max-w-2xl space-y-8">
                <div>
                    <Button variant="ghost" className="mb-4 pl-0" onClick={() => navigate(-1)}>
                        <ArrowLeft className="mr-2 h-4 w-4" /> Back to Dashboard
                    </Button>
                    <h1 className="text-3xl font-bold tracking-tight">Add New Card</h1>
                    <p className="text-muted-foreground">Enter card details and customize its look.</p>
                </div>

                <div className="grid gap-8 md:grid-cols-2">
                    {/* Form */}
                    <div className="space-y-6">
                        <div className="space-y-2">
                            <Label>Card Name / Nickname</Label>
                            <Input value={name} onChange={e => setName(e.target.value)} placeholder="e.g. Travel Rewards" />
                        </div>

                        <div className="space-y-2">
                            <Label>Bank Name</Label>
                            <Input value={bank} onChange={e => setBank(e.target.value)} placeholder="e.g. Chase" />
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <Label>Primary Color</Label>
                                <div className="flex gap-2">
                                    <Input
                                        type="color"
                                        value={primaryColor}
                                        onChange={e => setPrimaryColor(e.target.value)}
                                        className="h-10 w-20 p-1 cursor-pointer"
                                    />
                                    <Input
                                        value={primaryColor}
                                        onChange={e => setPrimaryColor(e.target.value)}
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
                                        onChange={e => setSecondaryColor(e.target.value)}
                                        className="h-10 w-20 p-1 cursor-pointer"
                                    />
                                    <Input
                                        value={secondaryColor}
                                        onChange={e => setSecondaryColor(e.target.value)}
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
                            />
                        </div>
                        <p className="text-center text-xs text-muted-foreground">
                            This is how your card will appear on the dashboard.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    )
}
