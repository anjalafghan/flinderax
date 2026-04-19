import { useState, useEffect } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { X } from "lucide-react"

import api from "@/services/api"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

interface EditCardModalProps {
    isOpen: boolean
    onClose: () => void
    cardId: string
    currentName: string
    currentBank: string
    currentPrimaryColor: [number, number, number]
    currentSecondaryColor: [number, number, number]
    currentLast4Digits: string | null
}

function rgbToHex(r: number, g: number, b: number): string {
    return "#" + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('')
}

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

export function EditCardModal({
    isOpen,
    onClose,
    cardId,
    currentName,
    currentBank,
    currentPrimaryColor,
    currentSecondaryColor,
    currentLast4Digits
}: EditCardModalProps) {
    const [name, setName] = useState(currentName)
    const [bank, setBank] = useState(currentBank)
    const [primaryColor, setPrimaryColor] = useState(rgbToHex(...currentPrimaryColor))
    const [secondaryColor, setSecondaryColor] = useState(rgbToHex(...currentSecondaryColor))
    const [last4Digits, setLast4Digits] = useState(currentLast4Digits || "")
    const queryClient = useQueryClient()

    useEffect(() => {
        setName(currentName)
        setBank(currentBank)
        setPrimaryColor(rgbToHex(...currentPrimaryColor))
        setSecondaryColor(rgbToHex(...currentSecondaryColor))
        setLast4Digits(currentLast4Digits || "")
    }, [currentName, currentBank, currentPrimaryColor, currentSecondaryColor, currentLast4Digits])

    const updateMutation = useMutation({
        mutationFn: async () => {
            await api.post("/card/update", {
                card_id: cardId,
                card_name: name,
                card_bank: bank,
                card_primary_color: hexToRgb(primaryColor),
                card_secondary_color: hexToRgb(secondaryColor),
                last_4_digits: last4Digits || null,
            })
        },
        onSuccess: () => {
            toast.success("Card updated successfully")
            queryClient.invalidateQueries({ queryKey: ['cards'] })
            queryClient.invalidateQueries({ queryKey: ['card', cardId] })
            onClose()
        },
        onError: (error: any) => {
            toast.error("Failed to update card")
            console.error(error)
        }
    })

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <Card className="w-full max-w-md animate-in fade-in zoom-in duration-200">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-xl font-bold">Edit Card</CardTitle>
                    <Button variant="ghost" size="icon" onClick={onClose} className="h-8 w-8 rounded-full">
                        <X className="h-4 w-4" />
                    </Button>
                </CardHeader>
                <CardContent className="space-y-4 pt-4">
                    <div className="space-y-2">
                        <Label>Card Name / Nickname</Label>
                        <Input value={name} onChange={e => setName(e.target.value)} placeholder="e.g. Travel Rewards" />
                    </div>

                    <div className="space-y-2">
                        <Label>Bank Name</Label>
                        <Input value={bank} onChange={e => setBank(e.target.value)} placeholder="e.g. Chase" />
                    </div>

                    <div className="space-y-2">
                        <Label>Last 4 Digits (Optional)</Label>
                        <Input
                            value={last4Digits}
                            onChange={e => setLast4Digits(e.target.value)}
                            placeholder="e.g. 1234"
                            maxLength={4}
                            pattern="[0-9]{0,4}"
                        />
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

                    <div className="flex gap-2">
                        <Button variant="outline" className="flex-1" onClick={onClose}>Cancel</Button>
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
    )
}