// Reads a bank statement PDF into CSV text, in the browser. The importer on the server only
// understands delimited text, so a PDF is turned into the same thing the bank's own CSV would be
// and then goes through the usual preview, checks and import. The file never leaves the device.

/** One run of text on a page, as pdf.js reports it. y grows upward. */
export interface PdfItem {
  str: string;
  x: number;
  y: number;
  /** Width of the run, so the gap to the next one can be told. */
  w: number;
}

export class PdfStatementError extends Error {}

/** Items closer than this in y are one line of the page. */
const LINE_TOLERANCE = 2.5;
/** Header runs closer than this belong to one heading ("Withdrawal" "(Dr.)"). */
const HEADING_GAP = 9;
/** A wrapped description line sits this close (in line heights) under the row it continues. */
const WRAP_LINES = 2.2;

const DATE = /^\s*\d{1,2}[\s/.-]+(?:\d{1,2}|[A-Za-z]{3,9})[\s/.-]+\d{2,4}\b/;
const DATE_HEADING = /^(?:txn\.?\s*|transaction\s*|value\s*)?date$/i;
const TEXT_HEADING = /^(?:description|narration|particulars|transaction (?:details|remarks)|details)$/i;
/** A running-number column. The importer has no use for it. */
const INDEX_HEADING = /^(?:#|s\.?\s?no\.?|sr\.?\s?no\.?|sl\.?\s?no\.?)$/i;

interface Line {
  y: number;
  items: PdfItem[];
}

/** Groups a page's runs into lines, top to bottom, each left to right. */
function toLines(items: PdfItem[]): Line[] {
  const sorted = items.filter((i) => i.str.trim() !== "").sort((a, b) => b.y - a.y || a.x - b.x);
  const lines: Line[] = [];
  for (const item of sorted) {
    const last = lines[lines.length - 1];
    if (last && Math.abs(last.y - item.y) <= LINE_TOLERANCE) last.items.push(item);
    else lines.push({ y: item.y, items: [item] });
  }
  for (const l of lines) l.items.sort((a, b) => a.x - b.x);
  return lines;
}

interface Column {
  x: number;
  /** Where the heading's text ends. */
  end: number;
  /** A run starting at or after this x, and before the next column's, belongs here. */
  from: number;
  name: string;
}

/** The table's headings, when this line is its header row (a date heading and a description heading). */
function headerOf(line: Line): Column[] | null {
  const merged: Column[] = [];
  let right = -Infinity;
  for (const item of line.items) {
    const text = item.str.trim();
    const last = merged[merged.length - 1];
    if (last && item.x - right <= HEADING_GAP) {
      last.name = `${last.name} ${text}`;
      last.end = item.x + item.w;
    } else merged.push({ x: item.x, end: item.x + item.w, from: -Infinity, name: text });
    right = item.x + item.w;
  }
  // Body text sits a little left of its heading and amounts are right-aligned under it, so the
  // line between two columns runs halfway between one heading's end and the next one's start.
  for (let i = 1; i < merged.length; i++) merged[i].from = (merged[i - 1].end + merged[i].x) / 2;
  const hasDate = merged.some((c) => DATE_HEADING.test(c.name));
  const hasText = merged.some((c) => TEXT_HEADING.test(c.name));
  return hasDate && hasText && merged.length >= 4 ? merged : null;
}

/** The index of the column an item starting at x belongs to. */
function columnOf(columns: Column[], x: number): number {
  let found = 0;
  for (let i = 0; i < columns.length; i++) if (columns[i].from <= x) found = i;
  return found;
}

const csvCell = (s: string) => (/[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s);

/**
 * CSV text for the transactions table in `pages`, or null when none is found. Rows that wrap onto
 * more lines keep their continuation in the description. Anything left of the table, and lines far
 * below the last row (page footers), are dropped.
 */
export function statementCsv(pages: PdfItem[][]): string | null {
  let columns: Column[] | null = null;
  let header: Column[] | null = null;
  const rows: string[][] = [];
  let textCol = -1;
  let prevY = 0;
  let lineHeight = 11;

  for (const page of pages) {
    let last: string[] | null = null; // a wrapped line never continues a row from the page before
    let prevInPage: Line | null = null;
    for (const line of toLines(page)) {
      const found = headerOf(line);
      if (found) {
        columns = found;
        header ??= found;
        textCol = found.findIndex((c) => TEXT_HEADING.test(c.name));
        prevInPage = null;
        last = null;
        continue;
      }
      if (!columns) continue;
      const cells: string[] = columns.map(() => "");
      for (const item of line.items) {
        const i = columnOf(columns, item.x);
        cells[i] = cells[i] ? `${cells[i]} ${item.str.trim()}` : item.str.trim();
      }
      const dateAt = columns.findIndex((c) => DATE_HEADING.test(c.name));
      if (dateAt >= 0 && DATE.test(cells[dateAt])) {
        rows.push(cells);
        last = cells;
      } else if (last && cells.every((c, i) => c === "" || i === textCol) && cells[textCol]) {
        // A wrapped description, directly under its row.
        if (prevInPage && prevY - line.y <= lineHeight * WRAP_LINES) last[textCol] = `${last[textCol]} ${cells[textCol]}`;
      } else {
        last = null; // a footer, the opening balance, or a line this table does not own
      }
      if (prevInPage) lineHeight = Math.max(6, Math.min(20, prevInPage.y - line.y)) || lineHeight;
      prevY = line.y;
      prevInPage = line;
    }
  }

  if (!header || rows.length === 0) return null;
  const keep = header.map((c) => !INDEX_HEADING.test(c.name));
  const out = [header.filter((_, i) => keep[i]).map((c) => csvCell(c.name)).join(",")];
  for (const r of rows) out.push(r.filter((_, i) => keep[i]).map(csvCell).join(","));
  return out.join("\n") + "\n";
}

/** Reads a statement PDF and returns its transactions as CSV text. */
export async function pdfToCsv(file: File): Promise<string> {
  const pdfjs = await import("pdfjs-dist");
  pdfjs.GlobalWorkerOptions.workerSrc = "/pdf-worker.mjs";
  let doc;
  try {
    doc = await pdfjs.getDocument({ data: new Uint8Array(await file.arrayBuffer()), isEvalSupported: false }).promise;
  } catch (e) {
    if (e instanceof Error && e.name === "PasswordException") {
      throw new PdfStatementError("This PDF is password protected. Save a copy without the password and try again.");
    }
    throw new PdfStatementError("Couldn't open that PDF.");
  }
  const pages: PdfItem[][] = [];
  for (let n = 1; n <= doc.numPages; n++) {
    const content = await (await doc.getPage(n)).getTextContent();
    pages.push(
      content.items.flatMap((it) => ("str" in it && it.transform ? [{ str: it.str, x: it.transform[4], y: it.transform[5], w: it.width }] : [])),
    );
  }
  void doc.destroy();
  const csv = statementCsv(pages);
  if (!csv) {
    throw new PdfStatementError(
      "Couldn't find a table of transactions in that PDF. If it is a scan, or not a bank statement, download the CSV or text export instead.",
    );
  }
  return csv;
}
