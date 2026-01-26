import { memo } from "react"
import { motion } from "framer-motion"
import { ArrowDown, ArrowUp } from "lucide-react"
import { cn } from "@/utils/cn"

interface CreditCardProps {
    id: string
    name: string
    bank: string
    balance: number
    lastDelta: number
    primaryColor: [number, number, number]
    secondaryColor: [number, number, number]
    onClick?: () => void
    variant?: 'platinum' | 'black' | 'custom'
}

export const CreditCard = memo(function CreditCard({
    name,
    bank,
    balance,
    lastDelta,
    primaryColor,
    secondaryColor,
    onClick,
}: CreditCardProps) {
    const p = primaryColor
    const s = secondaryColor

    // We'll use the user's colors but add a "sheen" overlay
    const backgroundStyle = {
        background: `linear-gradient(135deg, rgb(${p[0]}, ${p[1]}, ${p[2]}), rgb(${s[0]}, ${s[1]}, ${s[2]}))`
    }

    return (
        <motion.div
            whileHover={{ scale: 1.02, y: -5, boxShadow: "0 20px 40px -10px rgba(0,0,0,0.3)" }}
            whileTap={{ scale: 0.98 }}
            className="relative aspect-[1.586/1] w-full max-w-[340px] cursor-pointer overflow-hidden rounded-2xl shadow-xl transition-all"
            style={backgroundStyle}
            onClick={onClick}
        >
            {/* Metallic Noise Texture */}
            <div
                className="absolute inset-0 opacity-20 mix-blend-overlay"
                style={{ backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E")` }}
            />

            {/* Glossy Reflective Shine (Diagonal) */}
            <div className="absolute inset-0 bg-gradient-to-tr from-white/0 via-white/10 to-white/0 opacity-50" />

            {/* Radial highlighting for depth */}
            <div className="absolute -top-24 -right-24 h-64 w-64 rounded-full bg-white/20 blur-3xl" />

            <div className="relative flex h-full flex-col justify-between p-4 md:p-6 text-white shadow-sm z-10">
                <div className="flex items-start justify-between">
                    {/* Chip - Realistic style */}
                    <div className="flex gap-3 md:gap-4 items-center">
                        <div className="relative h-7 w-10 md:h-9 md:w-12 overflow-hidden rounded-md bg-gradient-to-br from-yellow-200 via-yellow-400 to-yellow-600 shadow-inner border border-yellow-700/30">
                            <div className="absolute inset-0 opacity-50" style={{ backgroundImage: "repeating-linear-gradient(90deg, transparent, transparent 2px, #000 2px, #000 4px)" }}></div>
                            <div className="absolute top-1/2 w-full h-[1px] bg-black/40"></div>
                        </div>
                        <svg className="h-5 w-5 md:h-6 md:w-6 opacity-80" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <path d="M1 10C1 10 5 14 12 14C19 14 23 10 23 10" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                            <path d="M1 14C1 14 5 18 12 18C19 18 23 14 23 14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                            <path d="M1 6C1 6 5 10 12 10C19 10 23 6 23 6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                        </svg>
                    </div>
                    <span className="font-extrabold tracking-widest uppercase opacity-90 text-[10px] md:text-sm">{bank}</span>
                </div>

                <div className="space-y-3 md:space-y-4">
                    <div className="flex items-end justify-between">
                        <div className="space-y-0.5">
                            <p className="text-[8px] md:text-[10px] uppercase tracking-wider font-semibold opacity-70">
                                {balance < 0 ? "Credit Balance" : "Current Balance"}
                            </p>
                            <h3 className="text-xl md:text-2xl font-bold tracking-tight text-white drop-shadow-md">
                                {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(balance)}
                            </h3>
                        </div>
                        {lastDelta !== 0 && (
                            <div className={cn(
                                "flex items-center gap-1 text-xs md:text-sm font-bold",
                                lastDelta < 0 ? "text-emerald-300" : "text-rose-300"
                            )}>
                                {lastDelta < 0 ? <ArrowDown className="h-3 w-3 md:h-4 md:w-4" /> : <ArrowUp className="h-3 w-3 md:h-4 md:w-4" />}
                                <span className="drop-shadow-sm">{new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(Math.abs(lastDelta))}</span>
                            </div>
                        )}
                    </div>

                    <div className="flex justify-between items-center font-mono text-[10px] md:text-sm opacity-90">
                        <span className="tracking-[0.15em] md:tracking-[0.2em] shadow-black drop-shadow-sm">•••• •••• •••• ••••</span>
                        <span className="font-sans uppercase text-[8px] md:text-[10px] font-bold tracking-widest">{name}</span>
                    </div>
                </div>
            </div>
        </motion.div>
    )
})
