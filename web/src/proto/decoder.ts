import type { Type } from 'protobufjs';

// Define the protobuf schema inline
const historyProto = `
syntax = "proto3";
package joy_backend;

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
  optional string last_4_digits = 8;
}

message CardList {
  repeated Card cards = 1;
}
`;

let CardHistoryListType: Type | null = null;
let CardListType: Type | null = null;

// Initialize the protobuf type. protobufjs is loaded on first use, so it only ships with the card screens.
async function initProto() {
    if (CardHistoryListType && CardListType) return { CardHistoryListType, CardListType };

    const protobuf = (await import('protobufjs')).default;
    const root = protobuf.parse(historyProto).root;
    CardHistoryListType = root.lookupType('joy_backend.CardHistoryList');
    CardListType = root.lookupType('joy_backend.CardList');
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
    last_4_digits: string | null;
}

export interface CardList {
    cards: Card[];
}

type Raw = Record<string, unknown>;

const isRaw = (v: unknown): v is Raw => typeof v === "object" && v !== null;
const rawList = (v: unknown): Raw[] => (Array.isArray(v) ? v.filter(isRaw) : []);
/** First truthy value among the keys (the `a || b` lookup), or undefined. */
const firstTruthy = (o: Raw, ...keys: string[]): unknown => keys.map((k) => o[k]).find(Boolean);
/** First non-null value among the keys (the `a ?? b` lookup), or undefined. */
const firstDefined = (o: Raw, ...keys: string[]): unknown => keys.map((k) => o[k]).find((v) => v != null);
const str = (v: unknown): string => (typeof v === "string" ? v : "");
const num = (v: unknown): number => (typeof v === "number" ? v : 0);
const strOrNull = (v: unknown): string | null => (typeof v === "string" ? v : null);
const numOrNull = (v: unknown): number | null => (typeof v === "number" ? v : null);

/**
 * Decode protobuf binary data into CardHistoryList
 */
export async function decodeCardHistoryList(buffer: ArrayBuffer): Promise<CardHistoryList> {
    const { CardHistoryListType } = await initProto();
    const uint8Array = new Uint8Array(buffer);
    const message = CardHistoryListType.decode(uint8Array);
    const object: Raw = CardHistoryListType.toObject(message, {
        longs: Number,
        enums: String,
        bytes: String,
    });

    // Convert timestamp fields to ISO strings
    // Note: protobufjs converts snake_case to camelCase by default
    const histories = rawList(object.histories).map((h) => ({
        transaction_id: str(firstTruthy(h, "transactionId", "transaction_id")),
        total_due_input: num(firstTruthy(h, "totalDueInput", "total_due_input")),
        timestamp: convertTimestamp(
            num(firstTruthy(h, "timestampSeconds", "timestamp_seconds")),
            num(firstTruthy(h, "timestampNanos", "timestamp_nanos")),
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
    const object: Raw = CardListType.toObject(message, {
        longs: Number,
        enums: String,
        bytes: String,
        defaults: false,
    });

    const cards = rawList(object.cards).map((c) => ({
        card_id: str(firstTruthy(c, "cardId", "card_id")),
        card_name: str(firstTruthy(c, "cardName", "card_name")),
        card_bank: str(firstTruthy(c, "cardBank", "card_bank")),
        card_primary_color: num(firstTruthy(c, "cardPrimaryColor", "card_primary_color")),
        card_secondary_color: num(firstTruthy(c, "cardSecondaryColor", "card_secondary_color")),
        last_total_due: numOrNull(firstDefined(c, "lastTotalDue", "last_total_due")),
        last_delta: numOrNull(firstDefined(c, "lastDelta", "last_delta")),
        last_4_digits: strOrNull(firstDefined(c, "last_4Digits", "last4Digits", "last_4_digits")),
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
