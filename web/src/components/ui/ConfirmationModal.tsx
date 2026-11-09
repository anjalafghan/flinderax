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
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-foreground/30 p-4 animate-in fade-in duration-200">
            <Card className="w-full max-w-md animate-in zoom-in duration-200 overflow-hidden">
                <div className={cn(
                    "h-1.5 w-full",
                    variant === "destructive" ? "bg-destructive" : "bg-primary"
                )} />
                <CardHeader className="flex flex-row items-start justify-between gap-4 pb-4">
                    <div className="flex min-w-0 items-center gap-3">
                        <div className={cn(
                            "shrink-0 p-2 rounded-full",
                            variant === "destructive" ? "bg-destructive/10 text-destructive" : "bg-primary/10 text-primary"
                        )}>
                            <AlertTriangle className="h-5 w-5" />
                        </div>
                        <CardTitle className="min-w-0 text-[24px]">{title}</CardTitle>
                    </div>
                    <Button variant="ghost" size="icon" onClick={onClose} className="h-9 w-9 shrink-0 rounded-full -mt-1 -mr-1" disabled={isPending}>
                        <X className="h-4 w-4" />
                    </Button>
                </CardHeader>
                <CardContent className="space-y-6">
                    <p className="text-muted-foreground leading-relaxed">
                        {description}
                    </p>

                    <div className="flex flex-col-reverse gap-3 sm:flex-row sm:justify-end">
                        <Button
                            variant="outline"
                            className="h-11 w-full rounded-xl sm:w-auto sm:min-w-28"
                            onClick={onClose}
                            disabled={isPending}
                        >
                            {cancelText}
                        </Button>
                        <Button
                            variant={variant === "destructive" ? "destructive" : "default"}
                            className="h-11 w-full rounded-xl font-semibold sm:w-auto sm:min-w-28"
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
