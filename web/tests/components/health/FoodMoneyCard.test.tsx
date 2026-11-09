import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { toast } from "sonner";

import { FoodMoneyCard } from "@/components/feature/health/FoodMoneyCard";
import { jsonResponse, overview, renderWithProviders, stubFetch } from "./fixture";

describe("FoodMoneyCard", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("states the eating-out share, the orders a week and the six months of bars", () => {
    renderWithProviders(<FoodMoneyCard money={overview.money} />);
    expect(screen.getByText("Eating out is 44% of food over the last 90 days.")).toBeInTheDocument();
    expect(screen.getByText("About 3.5 orders a week over the last 90 days.")).toBeInTheDocument();
    const chart = screen.getByRole("img", { name: /This month eating out is ₹1,200 and groceries ₹3,000/ });
    expect(chart).toBeInTheDocument();
    expect(screen.getAllByText("Jun").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Nov").length).toBeGreaterThan(0);
  });

  it("says so when there is not enough food spending to measure a share", () => {
    renderWithProviders(
      <FoodMoneyCard money={{ ...overview.money, eating_out_share_bps: null, delivery_per_week_x100: 0 }} />,
    );
    expect(screen.getByText("Not enough food spending in the last 90 days to measure yet.")).toBeInTheDocument();
  });

  it("rejects a cap that is not a positive amount, without saving", () => {
    const put = stubFetch(() => jsonResponse({}));
    renderWithProviders(<FoodMoneyCard money={overview.money} />);
    fireEvent.change(screen.getByLabelText(/Monthly cap on eating out/), { target: { value: "lots" } });
    fireEvent.click(screen.getByRole("button", { name: "Save cap" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Enter an amount in rupees, or leave it blank to clear the cap.");
    expect(put).not.toHaveBeenCalled();
  });

  it("saves a cap in paise, and sends null when the field is cleared", async () => {
    const put = stubFetch(() =>
      jsonResponse({ eating_out_cap_paise: 250_000 }),
    );
    const success = spyOn(toast, "success");
    renderWithProviders(<FoodMoneyCard money={overview.money} />);

    fireEvent.change(screen.getByLabelText(/Monthly cap on eating out/), { target: { value: "2500" } });
    fireEvent.click(screen.getByRole("button", { name: "Save cap" }));
    await waitFor(() => expect(success).toHaveBeenCalledWith("Eating-out cap saved."));
    const [url, init] = put.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/diet/settings");
    expect(init.method).toBe("PUT");
    expect(JSON.parse(String(init.body))).toEqual({ eating_out_cap_paise: 250_000 });

    fireEvent.change(screen.getByLabelText(/Monthly cap on eating out/), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Save cap" }));
    await waitFor(() => expect(success).toHaveBeenCalledWith("Eating-out cap cleared."));
    const [, second] = put.mock.calls[1] as unknown as [string, RequestInit];
    expect(JSON.parse(String(second.body))).toEqual({ eating_out_cap_paise: null });
  });
});
