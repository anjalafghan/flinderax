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
`;

let CardHistoryListType: protobuf.Type | null = null;

// Initialize the protobuf type
async function initProto() {
    if (CardHistoryListType) return CardHistoryListType;

    const root = protobuf.parse(historyProto).root;
    CardHistoryListType = root.lookupType('flinderax_backend.CardHistoryList');
    return CardHistoryListType;
}

export interface CardTransactionHistory {
    transaction_id: string;
    total_due_input: number;
    timestamp: string; // ISO 8601 string
}

export interface CardHistoryList {
    histories: CardTransactionHistory[];
}

/**
 * Decode protobuf binary data into CardHistoryList
 */
export async function decodeCardHistoryList(buffer: ArrayBuffer): Promise<CardHistoryList> {
    const type = await initProto();
    const uint8Array = new Uint8Array(buffer);
    const message = type.decode(uint8Array);
    const object = type.toObject(message, {
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
 * Convert protobuf timestamp (seconds + nanos) to ISO 8601 string
 */
function convertTimestamp(seconds: number, nanos: number): string {
    const milliseconds = seconds * 1000 + Math.floor(nanos / 1000000);
    return new Date(milliseconds).toISOString();
}
