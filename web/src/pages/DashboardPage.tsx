import { useState, useEffect } from "react"
import { useQuery } from "@tanstack/react-query"
import { ArrowRight } from "lucide-react"
import { useNavigate } from "react-router-dom"
import { motion } from "framer-motion"

import api from "@/services/api"
import { Button } from "@/components/ui/button"
import { CreditCard } from "@/components/feature/CreditCard"
import { Header } from "@/components/layout/Header"

interface CardData {
    card_id: string
    card_name: string
    card_bank: string
    card_primary_color: [number, number, number]
    card_secondary_color: [number, number, number]
    last_total_due: number | null
    last_delta: number | null
}

export default function DashboardPage() {
    const navigate = useNavigate()
    const [activeIndex, setActiveIndex] = useState(0)
    const [windowWidth, setWindowWidth] = useState(typeof window !== 'undefined' ? window.innerWidth : 1200)

    useEffect(() => {
        const handleResize = () => setWindowWidth(window.innerWidth)
        window.addEventListener('resize', handleResize)
        return () => window.removeEventListener('resize', handleResize)
    }, [])

    const isMobile = windowWidth < 640

    const { data: cards, isLoading } = useQuery({
        queryKey: ['cards'],
        queryFn: async () => {
            const res = await api.get<CardData[]>('/card/get_all_cards')
            return res.data
        }
    })

    // Safe reduce in case cards is undefined (e.g. during loading or error)
    const totalDue = cards?.reduce((acc, card) => acc + (card.last_total_due || 0), 0) || 0

    return (
        <div className="min-h-screen bg-background text-foreground transition-colors duration-300">
            <Header />

            <main className="container mx-auto px-4 py-8 md:py-12">
                {/* Catchy Hero Text */}
                <div className="mb-10 md:mb-16 text-center space-y-3 md:space-y-4">
                    <h1 className="text-3xl md:text-5xl font-extrabold tracking-tight px-2">
                        Explore Your Credit Cards.
                    </h1>
                    <p className="text-base md:text-lg text-muted-foreground max-w-xl mx-auto px-4">
                        Find the perfect card for your needs. Manage your {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(totalDue)} total balance effortlessly.
                    </p>
                </div>

                {/* 3D Carousel Area */}
                <div className="relative mx-auto max-w-5xl h-[300px] md:h-[400px] flex items-center justify-center perspective-[1000px] overflow-hidden sm:overflow-visible">
                    {isLoading ? (
                        <div className="text-muted-foreground animate-pulse">Loading cards...</div>
                    ) : !cards || cards.length === 0 ? (
                        <div className="text-center px-4">
                            <p className="mb-4 text-muted-foreground">You haven't added any cards yet.</p>
                            <Button onClick={() => navigate('/cards/new')}>Add Your First Card</Button>
                        </div>
                    ) : (
                        <div className="relative w-full flex justify-center items-center">
                            {cards.map((card, index) => {
                                // Calculate position relative to active index
                                const offset = index - activeIndex;
                                const isActive = offset === 0;
                                const isFar = Math.abs(offset) > 1; // Hide cards far away

                                // Visual properties based on position
                                const zIndex = isActive ? 10 : 10 - Math.abs(offset);
                                // Mobile: slightly smaller scale
                                const baseScale = isMobile ? 0.8 : 0.9;
                                const activeScale = isMobile ? 1.0 : 1.1;
                                const scale = isActive ? activeScale : baseScale;

                                const opacity = isActive ? 1 : 0.6;
                                // Mobile: tighter spacing
                                const xOffset = isMobile ? 80 : 120;
                                const x = offset * xOffset;
                                const rotateY = offset * (isMobile ? 15 : 25); // Slighter rotation on mobile

                                if (isFar && cards.length > 3) return null; // Simple culling

                                return (
                                    <motion.div
                                        key={card.card_id}
                                        className="absolute origin-center cursor-pointer touch-none"
                                        initial={false}
                                        animate={{
                                            x: `${x}%`,
                                            scale,
                                            opacity,
                                            zIndex,
                                            rotateY: `${-rotateY}deg`,
                                            filter: isActive ? 'blur(0px)' : (isMobile ? 'blur(1px)' : 'blur(2px)')
                                        }}
                                        transition={{ type: "spring", stiffness: 300, damping: 30 }}
                                        drag="x"
                                        dragConstraints={{ left: 0, right: 0 }}
                                        dragElastic={0.2}
                                        onDragEnd={(_, info) => {
                                            const swipeThreshold = 50;
                                            if (info.offset.x < -swipeThreshold && activeIndex < cards.length - 1) {
                                                setActiveIndex(activeIndex + 1);
                                            } else if (info.offset.x > swipeThreshold && activeIndex > 0) {
                                                setActiveIndex(activeIndex - 1);
                                            }
                                        }}
                                        style={{
                                            width: isMobile ? '280px' : '340px',
                                            perspective: '1000px',
                                            transformStyle: 'preserve-3d'
                                        }}
                                        onClick={() => {
                                            if (isActive) navigate(`/cards/${card.card_id}`);
                                            else setActiveIndex(index);
                                        }}
                                    >
                                        <div className="relative group">
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
                                                    initial={{ opacity: 0, y: 10 }}
                                                    animate={{ opacity: 1, y: 0 }}
                                                    className="absolute -bottom-12 md:-bottom-16 left-0 right-0 flex justify-center"
                                                >
                                                    <Button size="sm" className="bg-primary text-primary-foreground hover:bg-primary/90 rounded-full px-6 shadow-lg">
                                                        View Details <ArrowRight className="ml-2 h-4 w-4" />
                                                    </Button>
                                                </motion.div>
                                            )}
                                        </div>
                                    </motion.div>
                                )
                            })}
                        </div>
                    )}
                </div>

                {/* Pagination Dots */}
                <div className="flex justify-center gap-2 mt-12 md:mt-8 mb-12 md:mb-16">
                    {cards?.map((_, idx) => (
                        <button
                            key={idx}
                            onClick={() => setActiveIndex(idx)}
                            className={`h-2 rounded-full transition-all ${idx === activeIndex ? 'w-6 bg-primary' : 'w-2 bg-muted-foreground/30'}`}
                            aria-label={`Go to card ${idx + 1}`}
                        />
                    ))}
                </div>
            </main>
        </div>
    )
}
