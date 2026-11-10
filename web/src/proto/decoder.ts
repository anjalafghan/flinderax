// A minimal protobuf wire-format reader for the two messages the server sends (see src/proto/history.proto).
// protobufjs builds its decoders with `new Function`, which the app's Content-Security-Policy (no 'unsafe-eval')
// blocks, so the cards silently failed to load. This reader needs no eval.

class Reader {
    private pos = 0;
    private readonly view: DataView;
    private readonly buf: Uint8Array;
    constructor(buf: Uint8Array) {
        this.buf = buf;
        this.view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
    }
    get done(): boolean {
        return this.pos >= this.buf.length;
    }
    /** Reads a varint as a number (exact up to 2^53, plenty for ids, seconds and packed colours). */
    varint(): number {
        let result = 0;
        let mul = 1;
        for (;;) {
            if (this.pos >= this.buf.length) throw new Error("Truncated protobuf message");
            const b = this.buf[this.pos++];
            result += (b & 0x7f) * mul;
            if (!(b & 0x80)) return result;
            mul *= 128;
        }
    }
    /** int32/int64 are sent as two's-complement varints; a negative needs the 64-bit wrap undone. */
    int(): number {
        const start = this.pos;
        const v = this.varint();
        const len = this.pos - start;
        return len === 10 ? v - 2 ** 64 : v;
    }
    float(): number {
        const v = this.view.getFloat32(this.pos, true);
        this.pos += 4;
        return v;
    }
    bytes(): Uint8Array {
        const len = this.varint();
        const out = this.buf.subarray(this.pos, this.pos + len);
        if (out.length < len) throw new Error("Truncated protobuf message");
        this.pos += len;
        return out;
    }
    string(): string {
        return new TextDecoder().decode(this.bytes());
    }
    skip(wire: number): void {
        if (wire === 0) this.varint();
        else if (wire === 1) this.pos += 8;
        else if (wire === 2) this.bytes();
        else if (wire === 5) this.pos += 4;
        else throw new Error(`Unsupported protobuf wire type ${wire}`);
    }
    /** Calls `onField` for each (field number, wire type); it returns true once it has consumed the value. */
    fields(onField: (field: number, wire: number, r: Reader) => boolean): void {
        while (!this.done) {
            const tag = this.varint();
            const field = Math.floor(tag / 8);
            const wire = tag % 8;
            if (!onField(field, wire, this)) this.skip(wire);
        }
    }
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

function decodeHistory(r: Reader): CardTransactionHistory {
    let transaction_id = "";
    let total_due_input = 0;
    let seconds = 0;
    let nanos = 0;
    r.fields((f, w, rd) => {
        if (f === 1 && w === 2) transaction_id = rd.string();
        else if (f === 2 && w === 5) total_due_input = rd.float();
        else if (f === 3 && w === 0) seconds = rd.int();
        else if (f === 4 && w === 0) nanos = rd.int();
        else return false;
        return true;
    });
    return { transaction_id, total_due_input, timestamp: convertTimestamp(seconds, nanos) };
}

function decodeCard(r: Reader): Card {
    const card: Card = {
        card_id: "",
        card_name: "",
        card_bank: "",
        card_primary_color: 0,
        card_secondary_color: 0,
        last_total_due: null,
        last_delta: null,
        last_4_digits: null,
    };
    r.fields((f, w, rd) => {
        if (f === 1 && w === 2) card.card_id = rd.string();
        else if (f === 2 && w === 2) card.card_name = rd.string();
        else if (f === 3 && w === 2) card.card_bank = rd.string();
        else if (f === 4 && w === 0) card.card_primary_color = rd.int();
        else if (f === 5 && w === 0) card.card_secondary_color = rd.int();
        else if (f === 6 && w === 5) card.last_total_due = rd.float();
        else if (f === 7 && w === 5) card.last_delta = rd.float();
        else if (f === 8 && w === 2) card.last_4_digits = rd.string();
        else return false;
        return true;
    });
    return card;
}

/** Decode protobuf binary data into CardHistoryList. */
export async function decodeCardHistoryList(buffer: ArrayBuffer): Promise<CardHistoryList> {
    const histories: CardTransactionHistory[] = [];
    new Reader(new Uint8Array(buffer)).fields((f, w, rd) => {
        if (f !== 1 || w !== 2) return false;
        histories.push(decodeHistory(new Reader(rd.bytes())));
        return true;
    });
    return { histories };
}

/** Decode protobuf binary data into CardList. */
export async function decodeCardList(buffer: ArrayBuffer): Promise<CardList> {
    const cards: Card[] = [];
    new Reader(new Uint8Array(buffer)).fields((f, w, rd) => {
        if (f !== 1 || w !== 2) return false;
        cards.push(decodeCard(new Reader(rd.bytes())));
        return true;
    });
    return { cards };
}

/**
 * Convert protobuf timestamp (seconds + nanos) to ISO 8601 string
 */
function convertTimestamp(seconds: number, nanos: number): string {
    const milliseconds = seconds * 1000 + Math.floor(nanos / 1000000);
    return new Date(milliseconds).toISOString();
}
