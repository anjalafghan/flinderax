import { describe, expect, test } from "bun:test";

import { statementCsv, type PdfItem } from "@/utils/pdfStatement";

// A made-up statement laid out the way a bank's PDF is: centred headings, left-aligned text,
// right-aligned amounts, long descriptions wrapping onto a second line.
const item = (str: string, x: number, y: number, w = str.length * 4.2): PdfItem => ({ str, x, y, w });
const HEAD = (y: number): PdfItem[] => [
  item("#", 44, y, 4.5), item("Date", 75, y, 17.6), item("Description", 124, y, 43.3), item("Chq/Ref. No.", 280, y, 47.4),
  item("Withdrawal (Dr.)", 355, y, 61.3), item("Deposit (Cr.)", 429, y, 46.7), item("Balance", 498, y, 30.2),
];
const row = (n: number, date: string, desc: string, ref: string, dr: string, cr: string, bal: string, y: number): PdfItem[] => [
  item(String(n), 39, y), item(date, 72, y), item(desc, 119, y), item(ref, 275, y),
  ...(dr ? [item(dr, 416 - dr.length * 4.2, y)] : []),
  ...(cr ? [item(cr, 487 - cr.length * 4.2, y)] : []),
  item(bal, 556 - bal.length * 4.2, y),
];

describe("statementCsv", () => {
  const page1: PdfItem[] = [
    item("Account Statement", 40, 700),
    item("Savings Account Transactions", 220, 495),
    ...HEAD(470),
    item("Opening Balance", 119, 452), item("5,000.00", 529, 452),
    ...row(1, "10 Nov 2025", "UPI/SOMEONE/1/Paid via", "UPI-1", "", "25,000.00", "30,000.00", 438),
    item("CRED", 119, 430),
    ...row(2, "13 Nov 2025", "UPI/SHOP/2/Paid", "UPI-2", "1,700.00", "", "28,300.00", 417),
    item("Statement Generated on 18 Nov 2026", 50, 40), item("Page 1 of 2", 520, 40),
  ];
  const page2: PdfItem[] = [
    item("ANJAL EXAMPLE", 50, 780),
    ...HEAD(740),
    ...row(3, "14 Nov 2025", "Int.Pd:1", "", "", "155.00", "28,455.00", 700),
  ];

  test("reads rows across pages, joins wrapped descriptions, drops the index and footers", () => {
    const csv = statementCsv([page1, page2]);
    expect(csv?.trim().split("\n")).toEqual([
      "Date,Description,Chq/Ref. No.,Withdrawal (Dr.),Deposit (Cr.),Balance",
      '10 Nov 2025,UPI/SOMEONE/1/Paid via CRED,UPI-1,,"25,000.00","30,000.00"',
      '13 Nov 2025,UPI/SHOP/2/Paid,UPI-2,"1,700.00",,"28,300.00"',
      '14 Nov 2025,Int.Pd:1,,,155.00,"28,455.00"',
    ]);
  });

  test("a page with no transactions table gives null", () => {
    expect(statementCsv([[item("Hello", 10, 10), item("Not a statement", 10, 30)]])).toBeNull();
  });
});
