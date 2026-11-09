import { memo, useEffect, useRef, type PointerEvent } from "react"
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
    last_4_digits: string | null
    last_total_due: number | null
    last_delta: number | null
}

interface CardCarouselProps {
    cards: CardData[]
    activeIndex: number
    setActiveIndex: (index: number | ((prev: number) => number)) => void
    isMobile: boolean
}

/** A swipe farther than this (px) moves to the next or previous card. */
const SWIPE_PX = 40
/** Past this (px) a pointer press counts as a drag, so the click that follows is ignored. */
const DRAG_PX = 5

export const CardCarousel = memo(function CardCarousel({
    cards,
    activeIndex,
    setActiveIndex,
    isMobile
}: CardCarouselProps) {
    const navigate = useNavigate()
    // Swipe state: the CSS transitions replace framer's spring drag, so a swipe is read here.
    const press = useRef<{ x: number; moved: boolean } | null>(null)
    const swallowClick = useRef(false)

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

    const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
        e.currentTarget.setPointerCapture(e.pointerId)
        press.current = { x: e.clientX, moved: false }
        swallowClick.current = false
    }
    const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
        const p = press.current
        if (p && Math.abs(e.clientX - p.x) > DRAG_PX) p.moved = true
    }
    const onPointerUp = (e: PointerEvent<HTMLDivElement>) => {
        const p = press.current
        press.current = null
        if (!p) return
        const dx = e.clientX - p.x
        if (p.moved) swallowClick.current = true
        if (dx < -SWIPE_PX) setActiveIndex(prev => (prev + 1) % cards.length)
        else if (dx > SWIPE_PX) setActiveIndex(prev => (prev - 1 + cards.length) % cards.length)
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

                // Position & Rotation logic
                // Reduced offsets for "overlap" feel
                const xOffsetPct = isMobile ? 65 : 55; // percentage
                const x = offset * xOffsetPct;

                // 3D Depth
                const zDepth = isActive ? 0 : -150 * Math.abs(offset); // Recede background cards
                const rotateY = offset * (isMobile ? -15 : -25); // Inward facing rotation

                if (isFar && cards.length > 5) return null; // Cull only very far cards

                return (
                    <div
                        key={card.card_id}
                        className="absolute origin-center cursor-pointer touch-none will-change-transform transition-[transform,opacity,filter] duration-500 ease-out"
                        style={{
                            width: isMobile ? '280px' : '340px',
                            transform: `translateX(${x}%) translateZ(${zDepth}px) scale(${scale}) rotateY(${rotateY}deg)`,
                            opacity,
                            zIndex,
                            filter: `brightness(${isActive ? 1 : 0.7})`,
                            transformStyle: 'preserve-3d', // Essential for 3D depth
                        }}
                        onPointerDown={onPointerDown}
                        onPointerMove={onPointerMove}
                        onPointerUp={onPointerUp}
                        onPointerCancel={() => { press.current = null }}
                        onClick={() => {
                            if (swallowClick.current) { swallowClick.current = false; return }
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
                                last4Digits={card.last_4_digits}
                            />
                            {isActive && (
                                <div
                                    className="absolute -bottom-14 md:-bottom-20 left-0 right-0 flex justify-center z-50 pointer-events-none transition-[opacity,transform] duration-300 delay-100 starting:opacity-0 starting:translate-y-[15px]"
                                >
                                    <Button size="sm" className="bg-primary text-primary-foreground hover:bg-primary/90 rounded-full px-6 pointer-events-auto">
                                        View Details <ArrowRight className="ml-2 h-4 w-4" />
                                    </Button>
                                </div>
                            )}
                        </div>
                    </div>
                )
            })}
        </div>
    )
})
