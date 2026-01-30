import { AlertTriangle, X } from "lucide-react"
import { Button } from "./button"
import { Card, CardContent, CardHeader, CardTitle } from "./card"
import { cn } from "@/utils/cn"

interface ConfirmationModalProps {
    isOpen: boolean
    onClose: () => void
    onConfirm: () => void
    title: string
    description: string
    confirmText?: string
    cancelText?: string
    variant?: "destructive" | "primary"
    isPending?: boolean
}

export function ConfirmationModal({
    isOpen,
    onClose,
    onConfirm,
    title,
    description,
    confirmText = "Confirm",
    cancelText = "Cancel",
    variant = "primary",
    isPending = false
}: ConfirmationModalProps) {
    if (!isOpen) return null

    return (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
            <Card className="w-full max-w-md animate-in zoom-in duration-200 shadow-2xl overflow-hidden border-none">
                <div className={cn(
                    "h-1.5 w-full",
                    variant === "destructive" ? "bg-destructive" : "bg-primary"
                )} />
                <CardHeader className="flex flex-row items-start justify-between pb-2">
                    <div className="flex items-center gap-3">
                        <div className={cn(
                            "p-2 rounded-full",
                            variant === "destructive" ? "bg-destructive/10 text-destructive" : "bg-primary/10 text-primary"
                        )}>
                            <AlertTriangle className="h-5 w-5" />
                        </div>
                        <CardTitle className="text-xl font-bold">{title}</CardTitle>
                    </div>
                    <Button variant="ghost" size="icon" onClick={onClose} className="h-8 w-8 rounded-full -mt-1 -mr-1" disabled={isPending}>
                        <X className="h-4 w-4" />
                    </Button>
                </CardHeader>
                <CardContent className="space-y-6 pt-2">
                    <p className="text-muted-foreground leading-relaxed">
                        {description}
                    </p>

                    <div className="flex gap-3 pt-2">
                        <Button
                            variant="outline"
                            className="flex-1 h-11 rounded-xl"
                            onClick={onClose}
                            disabled={isPending}
                        >
                            {cancelText}
                        </Button>
                        <Button
                            variant={variant === "destructive" ? "destructive" : "default"}
                            className="flex-1 h-11 rounded-xl font-semibold shadow-sm"
                            onClick={onConfirm}
                            disabled={isPending}
                        >
                            {isPending ? "Processing..." : confirmText}
                        </Button>
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
