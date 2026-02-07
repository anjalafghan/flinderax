import { useState, useEffect } from "react"
import { useQuery } from "@tanstack/react-query"

import api from "@/services/api"
import { decodeCardList } from "@/proto/decoder"
import { Header } from "@/components/layout/Header"
import { CardCarousel } from "@/components/feature/CardCarousel"

export interface CardData {
    card_id: string
    card_name: string
    card_bank: string
    card_primary_color: [number, number, number]
    card_secondary_color: [number, number, number]
    last_total_due: number | null
    last_delta: number | null
}

export default function DashboardPage() {
    const [activeIndex, setActiveIndex] = useState(0)
    const [windowWidth, setWindowWidth] = useState(typeof window !== 'undefined' ? window.innerWidth : 1200)

    const [showCarousel, setShowCarousel] = useState(false)

    useEffect(() => {
        const handleResize = () => setWindowWidth(window.innerWidth)
        window.addEventListener('resize', handleResize)

        // Defer heavy carousel rendering to allow LCP element to paint first
        const t = setTimeout(() => {
            setShowCarousel(true)
        }, 100)

        return () => {
            window.removeEventListener('resize', handleResize)
            clearTimeout(t)
        }
    }, [])

    const isMobile = windowWidth < 640



    const { data: cards, isLoading } = useQuery({
        queryKey: ['cards'],
        queryFn: async () => {
            // Fetch as Protobuf
            const buffer = await api.getProtobuf('/card/get_all_cards')
            // Decode
            const decoded = await decodeCardList(buffer)

            // Map Protobuf structure to CardData (handling unpacking colors)
            return decoded.cards.map(c => ({
                card_id: c.card_id,
                card_name: c.card_name,
                card_bank: c.card_bank,
                // Unpack colors from int32 to [r, g, b]
                card_primary_color: unpackColor(c.card_primary_color),
                card_secondary_color: unpackColor(c.card_secondary_color),
                last_total_due: c.last_total_due,
                last_delta: c.last_delta
            })) as CardData[]
        }
    })

    // Helper to unpack color integer back to RGB array
    const unpackColor = (packed: number): [number, number, number] => {
        const r = (packed >> 16) & 0xFF;
        const g = (packed >> 8) & 0xFF;
        const b = packed & 0xFF;
        return [r, g, b];
    }


    // Safe reduce in case cards is undefined (e.g. during loading or error)
    const totalDue = cards?.reduce((acc: number, card: CardData) => acc + (card.last_total_due || 0), 0) || 0

    // Restore active card from session storage
    useEffect(() => {
        if (!cards || cards.length === 0) return;

        const savedCardId = sessionStorage.getItem('dashboard_active_card');
        if (savedCardId) {
            const index = cards.findIndex(c => c.card_id === savedCardId);
            if (index !== -1) {
                setActiveIndex(index);
            }
        }
    }, [cards]);



    return (
        <div className="h-[100dvh] w-full bg-background text-foreground transition-colors duration-300 overflow-hidden flex flex-col">
            <Header />

            <main className="flex-1 container mx-auto px-4 py-4 md:py-12 flex flex-col justify-center overflow-hidden">
                {/* Catchy Hero Text */}
                <div className="mb-4 md:mb-16 text-center space-y-2 md:space-y-4 shrink-0">
                    <h1 className="text-3xl md:text-5xl font-extrabold tracking-tight px-2">
                        Explore Your Credit Cards.
                    </h1>
                    <p className="text-base md:text-lg text-muted-foreground max-w-xl mx-auto px-4">
                        Find the perfect card for your needs. Manage your {new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' }).format(totalDue)} total balance effortlessly.
                    </p>
                </div>

                {/* 3D Carousel Area - Deferred */}
                <div className="relative mx-auto max-w-5xl h-[300px] md:h-[400px] flex items-center justify-center perspective-[1200px] overflow-visible shrink-0">
                    {!showCarousel ? (
                        <div className="text-muted-foreground">Loading visualization...</div>
                    ) : isLoading ? (
                        <div className="text-muted-foreground animate-pulse">Loading cards...</div>
                    ) : (
                        <CardCarousel
                            cards={cards || []}
                            activeIndex={activeIndex}
                            setActiveIndex={setActiveIndex}
                            isMobile={isMobile}
                        />
                    )}
                </div>

                {/* Pagination Dots */}
                <div className="flex justify-center gap-2 mt-4 md:mt-8 mb-4 md:mb-16 shrink-0">
                    {cards?.map((_: CardData, idx: number) => (
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
