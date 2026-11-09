import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

mock.module("sonner", () => ({ toast: { success: () => {}, error: () => {}, info: () => {} } }));

import { StatementImport } from "@/components/feature/StatementImport";
import api from "@/services/api";
import { ingestApi } from "@/services/api";
import type { StatementResult } from "@/services/types";

const result: StatementResult = {
  total: 6,
  added: 3,
  duplicates: 0,
  matched_alerts: 3,
  upgraded: 3,
  skipped: 0,
  uncategorized: 0,
  categorized: 6,
  first_date: "2026-11-01",
  last_date: "2026-11-06",
  balance_updated: null,
  chain_ok: true,
  opening_paise: 1_000_000,
  closing_paise: 5_513_050,
  not_on_statement: 1,
  holder_name_suggestion: "ASHA RAO",
};

/** The mapping a 422 would carry: a layout no detector reads, with its first rows. */
const needsMapping = {
  error: "needs_mapping",
  preview: {
    header_row: 0,
    rows: [
      ["Posted On", "Memo", "Out", "In"],
      ["03/11/2026", "Coffee shop", "120.00", ""],
      ["04/11/2026", "Salary", "", "5000.00"],
    ],
    guessed: null,
  },
};

const renderImport = () => {
  // The cards list is a protobuf request; these tests don't need it, so it fails quietly.
  spyOn(api, "getProtobuf").mockRejectedValue(new Error("offline"));
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
      <StatementImport />
    </QueryClientProvider>,
  );
};

describe("StatementImport", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("explains the privacy rule and refuses Excel files without a network call", async () => {
    const upload = spyOn(ingestApi, "uploadStatement");
    spyOn(api, "get").mockResolvedValue({ data: [], status: 200, statusText: "OK" } as never);
    renderImport();
    expect(screen.getByText(/read in memory and discarded straight after import/)).toBeInTheDocument();

    const input = screen.getByLabelText("Statement file");
    fireEvent.change(input, { target: { files: [new File(["x"], "statement.xlsx")] } });
    expect(await screen.findByRole("alert")).toHaveTextContent("Excel files can't be read");
    expect(upload).not.toHaveBeenCalled();
  });

  it("shows the balance check, the holder name offer and the not-on-statement note after an upload", async () => {
    spyOn(api, "get").mockImplementation((url: string) =>
      Promise.resolve({ data: url.includes("holder-names") ? { names: [] } : [], status: 200, statusText: "OK" }) as never,
    );
    const upload = spyOn(ingestApi, "uploadStatement").mockResolvedValue(result);
    renderImport();

    fireEvent.change(screen.getByLabelText("Statement file"), { target: { files: [new File(["csv"], "statement.txt")] } });
    expect(await screen.findByText("Balance check passed")).toBeInTheDocument();
    expect(upload).toHaveBeenCalledTimes(1);
    expect(screen.getByText(/3 bank alerts confirmed/)).toBeInTheDocument();
    expect(screen.getByText((_, el) => el?.tagName === "P" && el.textContent?.startsWith("Is ASHA RAO you?") === true)).toBeInTheDocument();
    expect(screen.getByText(/isn't on the statement/)).toBeInTheDocument();
  });

  it("says when it kept a newer typed balance and how many earlier payments it sorted", async () => {
    spyOn(api, "get").mockImplementation((url: string) =>
      Promise.resolve({ data: url.includes("holder-names") ? { names: [] } : [], status: 200, statusText: "OK" }) as never,
    );
    spyOn(ingestApi, "uploadStatement").mockResolvedValue({
      ...result,
      holder_name_suggestion: null,
      older_recategorized: 4,
      balance_kept: {
        account_name: "Spending",
        balance_paise: 7_000_000,
        set_on: "2026-11-20",
        statement_paise: 5_513_050,
        statement_on: "2026-11-06",
      },
    });
    renderImport();

    fireEvent.change(screen.getByLabelText("Statement file"), { target: { files: [new File(["csv"], "statement.txt")] } });
    expect(await screen.findByText(/Kept the Spending balance of ₹70,000/)).toHaveTextContent("it ends 6 Nov at ₹55,130.50");
    expect(screen.getByText("Also sorted 4 earlier payments with your latest categories.")).toBeInTheDocument();
  });

  it("shows a warning instead of a pass when the balances do not chain", async () => {
    spyOn(api, "get").mockImplementation((url: string) =>
      Promise.resolve({ data: url.includes("holder-names") ? { names: [] } : [], status: 200, statusText: "OK" }) as never,
    );
    spyOn(ingestApi, "uploadStatement").mockResolvedValue({ ...result, chain_ok: false, holder_name_suggestion: null });
    renderImport();

    fireEvent.change(screen.getByLabelText("Statement file"), { target: { files: [new File(["csv"], "statement.csv")] } });
    await waitFor(() => expect(screen.getByText("Balances don't add up")).toBeInTheDocument());
    expect(screen.queryByText("Balance check passed")).not.toBeInTheDocument();
  });

  it("maps the columns of a layout it can't read, and posts the mapping with the import", async () => {
    spyOn(api, "get").mockImplementation((url: string) =>
      Promise.resolve({ data: url.includes("holder-names") ? { names: [] } : [], status: 200, statusText: "OK" }) as never,
    );
    const upload = spyOn(ingestApi, "uploadStatement")
      .mockRejectedValueOnce(new Error(JSON.stringify(needsMapping)))
      .mockResolvedValueOnce(result);
    renderImport();

    fireEvent.change(screen.getByLabelText("Statement file"), { target: { files: [new File(["csv"], "statement.csv")] } });
    expect(await screen.findByRole("heading", { name: "Map the columns" })).toBeInTheDocument();
    // No toast-style error for a layout that can be mapped, and the file's rows are shown.
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.getByText("Coffee shop")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Column 1"), { target: { value: "date" } });
    fireEvent.change(screen.getByLabelText("Column 2"), { target: { value: "narration" } });
    fireEvent.change(screen.getByLabelText("Column 3"), { target: { value: "debit" } });
    fireEvent.change(screen.getByLabelText("Column 4"), { target: { value: "credit" } });
    fireEvent.click(screen.getByRole("button", { name: "Import" }));

    await waitFor(() => expect(screen.getByText("Balance check passed")).toBeInTheDocument());
    expect(upload).toHaveBeenCalledTimes(2);
    const [, target] = upload.mock.calls[1];
    expect(target).toEqual({
      mapping: {
        header_row: 0,
        date: 0,
        narration: 1,
        debit: 2,
        credit: 3,
        amount: null,
        sign: { kind: "separate_columns" },
        balance: null,
        reference: null,
        date_order: "dmy",
      },
      save_profile: true,
      label: "",
    });
    expect(screen.queryByRole("heading", { name: "Map the columns" })).not.toBeInTheDocument();
  });

  it("starts a new mapping for a second file rather than keeping the first file's choices", async () => {
    spyOn(api, "get").mockImplementation((url: string) =>
      Promise.resolve({ data: url.includes("holder-names") ? { names: [] } : [], status: 200, statusText: "OK" }) as never,
    );
    const otherLayout = {
      error: "needs_mapping",
      preview: {
        header_row: 0,
        rows: [
          ["Memo", "Out", "In"],
          ["Coffee shop", "120.00", ""],
        ],
        guessed: null,
      },
    };
    spyOn(ingestApi, "uploadStatement")
      .mockRejectedValueOnce(new Error(JSON.stringify(needsMapping)))
      .mockRejectedValueOnce(new Error(JSON.stringify(otherLayout)));
    renderImport();

    fireEvent.change(screen.getByLabelText("Statement file"), { target: { files: [new File(["csv"], "first.csv")] } });
    await screen.findByRole("heading", { name: "Map the columns" });
    fireEvent.change(screen.getByLabelText("Column 1"), { target: { value: "date" } });
    expect(screen.getByLabelText("Column 1")).toHaveValue("date");

    fireEvent.change(screen.getByLabelText("Statement file"), { target: { files: [new File(["csv"], "second.csv")] } });
    expect(await screen.findByText("Memo")).toBeInTheDocument();
    expect(screen.queryByText("Posted On")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Column 1")).toHaveValue("ignore");
  });

  it("previews a file for mapping without importing it, and asks for the date and description first", async () => {
    spyOn(api, "get").mockImplementation((url: string) =>
      Promise.resolve({ data: url.includes("holder-names") ? { names: [] } : [], status: 200, statusText: "OK" }) as never,
    );
    const upload = spyOn(ingestApi, "uploadStatement");
    const preview = spyOn(ingestApi, "previewStatement").mockResolvedValue({
      preview: needsMapping.preview,
      signature: "0123456789abcdef",
      known_profile: false,
    });
    renderImport();

    fireEvent.change(screen.getByLabelText("Statement file to map"), { target: { files: [new File(["csv"], "statement.csv")] } });
    expect(await screen.findByRole("heading", { name: "Map the columns" })).toBeInTheDocument();
    expect(preview).toHaveBeenCalledTimes(1);
    expect(upload).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Import" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Choose the column with the dates.");
    expect(upload).not.toHaveBeenCalled();
  });
});
