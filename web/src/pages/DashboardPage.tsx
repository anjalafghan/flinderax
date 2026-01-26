import { useState } from "react"
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

            <main className="container mx-auto px-4 py-12 md:px-8">
                {/* Catchy Hero Text */}
                <div className="mb-16 text-center space-y-4">
                    <h1 className="text-4xl md:text-5xl font-extrabold tracking-tight">
                        Explore Your Credit Cards.
                    </h1>
                    <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                        Find the perfect card for your needs. Manage your {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(totalDue)} total balance effortlessly.
                    </p>
                </div>

                {/* 3D Carousel Area */}
                <div className="relative mx-auto max-w-5xl h-[400px] flex items-center justify-center perspective-[1000px]">
                    {isLoading ? (
                        <div>Loading...</div>
                    ) : !cards || cards.length === 0 ? (
                        <div className="text-center">
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
                                const scale = isActive ? 1.1 : 0.9;
                                const opacity = isActive ? 1 : 0.6;
                                const x = offset * 120; // Horizontal spacing
                                const rotateY = offset * 25; // Rotation for 3D effect

                                if (isFar && cards.length > 3) return null; // Simple culling

                                return (
                                    <motion.div
                                        key={card.card_id}
                                        className="absolute origin-center cursor-pointer"
                                        initial={false}
                                        animate={{
                                            x: `${x}%`,
                                            scale,
                                            opacity,
                                            zIndex,
                                            rotateY: `${-rotateY}deg`,
                                            filter: isActive ? 'blur(0px)' : 'blur(2px)'
                                        }}
                                        transition={{ type: "spring", stiffness: 300, damping: 30 }}
                                        style={{
                                            width: '340px',
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
                                                    className="absolute -bottom-16 left-0 right-0 flex justify-center"
                                                >
                                                    <Button size="sm" className="bg-primary text-primary-foreground hover:bg-primary/90 rounded-full px-6">
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
                <div className="flex justify-center gap-2 mt-8 mb-16">
                    {cards?.map((_, idx) => (
                        <button
                            key={idx}
                            onClick={() => setActiveIndex(idx)}
                            className={`h-2 w-2 rounded-full transition-colors ${idx === activeIndex ? 'bg-foreground' : 'bg-muted-foreground/30'}`}
                        />
                    ))}
                </div>
            </main>
        </div>
    )
}
