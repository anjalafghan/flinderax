import { memo, useEffect } from "react"
import { motion } from "framer-motion"
import { ArrowRight } from "lucide-react"
import { useNavigate } from "react-router-dom"

import { Button } from "@/components/ui/button"
import { CreditCard } from "@/components/feature/CreditCard"

// Re-using the interface. In a real app, this should be in a types file.
export interface CardData {
    card_id: string
    card_name: string
    card_bank: string
    card_primary_color: [number, number, number]
    card_secondary_color: [number, number, number]
    last_total_due: number | null
    last_delta: number | null
}

interface CardCarouselProps {
    cards: CardData[]
    activeIndex: number
    setActiveIndex: (index: number | ((prev: number) => number)) => void
    isMobile: boolean
}

export const CardCarousel = memo(function CardCarousel({
    cards,
    activeIndex,
    setActiveIndex,
    isMobile
}: CardCarouselProps) {
    const navigate = useNavigate()

    // Keyboard navigation
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (!cards || cards.length === 0) return

            if (e.key === "ArrowRight") {
                setActiveIndex(prev => (prev + 1) % cards.length)
            } else if (e.key === "ArrowLeft") {
                setActiveIndex(prev => (prev - 1 + cards.length) % cards.length)
            }
        }

        window.addEventListener("keydown", handleKeyDown)
        return () => window.removeEventListener("keydown", handleKeyDown)
    }, [activeIndex, cards, setActiveIndex])

    if (!cards || cards.length === 0) {
        return (
            <div className="text-center px-4">
                <p className="mb-4 text-muted-foreground">You haven't added any cards yet.</p>
                <Button onClick={() => navigate('/cards/new')}>Add Your First Card</Button>
            </div>
        )
    }

    return (
        <div className="relative w-full flex justify-center items-center h-full">
            {cards.map((card: CardData, index: number) => {
                // Calculate position relative to active index
                const N = cards.length;
                let offset = (index - activeIndex) % N;
                if (offset < 0) offset += N;
                if (offset > N / 2) offset -= N;

                const isActive = offset === 0;
                const isFar = Math.abs(offset) > 2; // Increase visibility range for cooler background

                // --- "Coverflow" parameters ---
                const zIndex = isActive ? 50 : 50 - Math.abs(offset);

                // Base scale
                const baseScale = isMobile ? 0.85 : 0.9;
                const activeScale = isMobile ? 1.05 : 1.1;
                const scale = isActive ? activeScale : baseScale;

                // Opacity & Filter
                const opacity = isActive ? 1 : Math.max(0.3, 1 - Math.abs(offset) * 0.3);
                const blurValue = isActive ? 0 : Math.min(4, Math.abs(offset) * 2);

                // Position & Rotation logic
                // Reduced offsets for "overlap" feel
                const xOffsetPct = isMobile ? 65 : 55; // percentage
                const x = offset * xOffsetPct;

                // 3D Depth
                const zDepth = isActive ? 0 : -150 * Math.abs(offset); // Recede background cards
                const rotateY = offset * (isMobile ? -15 : -25); // Inward facing rotation

                if (isFar && cards.length > 5) return null; // Cull only very far cards

                return (
                    <motion.div
                        key={card.card_id}
                        className="absolute origin-center cursor-pointer touch-none will-change-transform"
                        initial={false}
                        animate={{
                            x: `${x}%`,
                            z: zDepth,
                            scale,
                            opacity,
                            zIndex,
                            rotateY: `${rotateY}deg`,
                            filter: `blur(${blurValue}px) brightness(${isActive ? 1 : 0.7})`,
                            boxShadow: isActive
                                ? "0 25px 50px -12px rgba(0, 0, 0, 0.5)"
                                : "0 10px 20px -5px rgba(0, 0, 0, 0.2)"
                        }}
                        transition={{
                            type: "spring",
                            stiffness: 200,
                            damping: 25,
                            mass: 0.8
                        }}
                        drag="x"
                        dragConstraints={{ left: 0, right: 0 }}
                        dragElastic={0.1}
                        onDragEnd={(_, info) => {
                            const swipeThreshold = 40;
                            if (info.offset.x < -swipeThreshold) {
                                setActiveIndex((activeIndex + 1) % cards.length);
                            } else if (info.offset.x > swipeThreshold) {
                                setActiveIndex((activeIndex - 1 + cards.length) % cards.length);
                            }
                        }}
                        style={{
                            width: isMobile ? '280px' : '340px',
                            transformStyle: 'preserve-3d', // Essential for 3D depth
                        }}
                        onClick={() => {
                            if (isActive) {
                                // Persist active card ID before navigating
                                sessionStorage.setItem('dashboard_active_card', card.card_id);
                                navigate(`/cards/${card.card_id}`);
                            }
                            else setActiveIndex(index);
                        }}
                        tabIndex={isActive ? 0 : -1} // Accessibility
                        aria-hidden={!isActive}
                    >
                        <div className="relative group transition-transform duration-300">
                            <CreditCard
                                id={card.card_id}
                                name={card.card_name}
                                bank={card.card_bank}
                                balance={card.last_total_due || 0}
                                lastDelta={card.last_delta || 0}
                                primaryColor={card.card_primary_color}
                                secondaryColor={card.card_secondary_color}
                            />
                            {isActive && (
                                <motion.div
                                    initial={{ opacity: 0, y: 15 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    transition={{ delay: 0.1, duration: 0.3 }}
                                    className="absolute -bottom-14 md:-bottom-20 left-0 right-0 flex justify-center z-50 pointer-events-none"
                                >
                                    <Button size="sm" className="bg-primary text-primary-foreground hover:bg-primary/90 rounded-full px-6 shadow-xl pointer-events-auto">
                                        View Details <ArrowRight className="ml-2 h-4 w-4" />
                                    </Button>
                                </motion.div>
                            )}
                        </div>
                    </motion.div>
                )
            })}
        </div>
    )
})
