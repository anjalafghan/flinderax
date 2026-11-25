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

  test("reads an HDFC credit card statement: Date & Time, Transaction Description, a '+' run on credits", () => {
    // Positions as pdf.js reports them for the real statement; the summary boxes above are left out.
    const card: PdfItem[] = [
      item("Domestic Transactions", 170, 151, 70),
      item("DATE & TIME", 170, 136, 38), item("TRANSACTION DESCRIPTION", 260, 136, 83),
      item("AMOUNT", 528, 136, 28), item("PI", 567, 136, 6),
      item("12/10/2026| 14:22", 170, 107, 60), item("SPOTIFY SIMUMBAI", 260, 107, 70), item("C 119.00", 524, 107, 30), item("l", 568, 107, 3),
      item("14/10/2026| 09:01", 170, 93, 60), item("BPPY PYMT (Ref# 1)", 260, 93, 80), item("+", 516, 93, 3), item("C 12,000.00", 521, 93, 35), item("l", 568, 93, 3),
      item("15/10/2026| 10:00", 170, 64, 60), item("EMI", 241, 64, 11), item("ASSPLBangalore", 260, 64, 60), item("C 1,500.00", 524, 64, 32), item("l", 568, 64, 3),
    ];
    expect(statementCsv([card])?.trim().split("\n")).toEqual([
      "DATE & TIME,TRANSACTION DESCRIPTION,AMOUNT,PI",
      "12/10/2026 14:22,SPOTIFY SIMUMBAI,C 119.00,l",
      '14/10/2026 09:01,BPPY PYMT (Ref# 1),"+ C 12,000.00",l',
      '15/10/2026 10:00,EMI ASSPLBangalore,"C 1,500.00",l',
    ]);
  });

  test("a page with no transactions table gives null", () => {
    expect(statementCsv([[item("Hello", 10, 10), item("Not a statement", 10, 30)]])).toBeNull();
  });
});
