import protobuf from 'protobufjs';

// Define the protobuf schema inline
const historyProto = `
syntax = "proto3";
package flinderax_backend;

message CardTransactionHistory {
  string transaction_id = 1;
  float total_due_input = 2;
  int64 timestamp_seconds = 3;
  int32 timestamp_nanos = 4;
}

message CardHistoryList {
  repeated CardTransactionHistory histories = 1;
}

message Card {
  string card_id = 1;
  string card_name = 2;
  string card_bank = 3;
  int32 card_primary_color = 4;
  int32 card_secondary_color = 5;
  optional float last_total_due = 6;
  optional float last_delta = 7;
}

message CardList {
  repeated Card cards = 1;
}
`;

let CardHistoryListType: protobuf.Type | null = null;
let CardListType: protobuf.Type | null = null;

// Initialize the protobuf type
async function initProto() {
    if (CardHistoryListType && CardListType) return { CardHistoryListType, CardListType };

    const root = protobuf.parse(historyProto).root;
    CardHistoryListType = root.lookupType('flinderax_backend.CardHistoryList');
    CardListType = root.lookupType('flinderax_backend.CardList');
    return { CardHistoryListType, CardListType };
}

export interface CardTransactionHistory {
    transaction_id: string;
    total_due_input: number;
    timestamp: string; // ISO 8601 string
}

export interface CardHistoryList {
    histories: CardTransactionHistory[];
}

export interface Card {
    card_id: string;
    card_name: string;
    card_bank: string;
    card_primary_color: number;
    card_secondary_color: number;
    last_total_due: number | null;
    last_delta: number | null;
}

export interface CardList {
    cards: Card[];
}

/**
 * Decode protobuf binary data into CardHistoryList
 */
export async function decodeCardHistoryList(buffer: ArrayBuffer): Promise<CardHistoryList> {
    const { CardHistoryListType } = await initProto();
    const uint8Array = new Uint8Array(buffer);
    const message = CardHistoryListType.decode(uint8Array);
    const object = CardHistoryListType.toObject(message, {
        longs: Number,
        enums: String,
        bytes: String,
    }) as any;

    // Convert timestamp fields to ISO strings
    // Note: protobufjs converts snake_case to camelCase by default
    const histories = (object.histories || []).map((h: any) => ({
        transaction_id: h.transactionId || h.transaction_id || '',
        total_due_input: h.totalDueInput || h.total_due_input || 0,
        timestamp: convertTimestamp(
            h.timestampSeconds || h.timestamp_seconds || 0,
            h.timestampNanos || h.timestamp_nanos || 0
        ),
    }));

    return { histories };
}

/**
 * Decode protobuf binary data into CardList
 */
export async function decodeCardList(buffer: ArrayBuffer): Promise<CardList> {
    const { CardListType } = await initProto();
    const uint8Array = new Uint8Array(buffer);
    const message = CardListType.decode(uint8Array);
    const object = CardListType.toObject(message, {
        longs: Number,
        enums: String,
        bytes: String,
        defaults: true, // Include optional fields with defaults if missing
    }) as any;

    const cards = (object.cards || []).map((c: any) => ({
        card_id: c.cardId || c.card_id || '',
        card_name: c.cardName || c.card_name || '',
        card_bank: c.cardBank || c.card_bank || '',
        card_primary_color: c.cardPrimaryColor || c.card_primary_color || 0,
        card_secondary_color: c.cardSecondaryColor || c.card_secondary_color || 0,
        last_total_due: c.lastTotalDue ?? c.last_total_due ?? null,
        last_delta: c.lastDelta ?? c.last_delta ?? null,
    }));

    return { cards };
}

/**
 * Convert protobuf timestamp (seconds + nanos) to ISO 8601 string
 */
function convertTimestamp(seconds: number, nanos: number): string {
    const milliseconds = seconds * 1000 + Math.floor(nanos / 1000000);
    return new Date(milliseconds).toISOString();
}
