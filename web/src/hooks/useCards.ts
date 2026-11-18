import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

/** A card as /api/card/get_all_cards sends it; colours are packed 0xRRGGBB integers. */
interface CardWire {
  card_id: string;
  card_name: string;
  card_bank: string;
  card_primary_color: number;
  card_secondary_color: number;
  last_4_digits: string | null;
  last_total_due: number | null;
  last_delta: number | null;
}

export interface CardData {
  card_id: string;
  card_name: string;
  card_bank: string;
  card_primary_color: [number, number, number];
  card_secondary_color: [number, number, number];
  last_4_digits: string | null;
  last_total_due: number | null;
  last_delta: number | null;
  /** Only present on the single-card endpoint, not the card list. */
  statement_day?: number | null;
  due_day?: number | null;
  credit_limit_paise?: number | null;
  pay_from_account_id?: string | null;
}

function unpackColor(packed: number): [number, number, number] {
  return [(packed >> 16) & 0xff, (packed >> 8) & 0xff, packed & 0xff];
}

/** The card list as the carousel wants it. Shared with Today's one-line card summary. */
export function useCards() {
  return useQuery({
    queryKey: ["cards"],
    queryFn: async () => {
      const res = await api.get<{ cards: CardWire[] }>("/api/card/get_all_cards");
      return res.data.cards.map((c) => ({
        card_id: c.card_id,
        card_name: c.card_name,
        card_bank: c.card_bank,
        card_primary_color: unpackColor(c.card_primary_color),
        card_secondary_color: unpackColor(c.card_secondary_color),
        last_4_digits: c.last_4_digits,
        last_total_due: c.last_total_due,
        last_delta: c.last_delta,
      })) as CardData[];
    },
  });
}

/** Card APIs still send rupees, not paise (see card.rs), so this formats rupees. */
export const formatRupees = (rupees: number) =>
  new Intl.NumberFormat("en-IN", { style: "currency", currency: "INR" }).format(rupees);

