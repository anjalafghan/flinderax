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

const renderImport = () =>
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
      <StatementImport />
    </QueryClientProvider>,
  );

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
});
