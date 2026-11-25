import { describe, expect, it } from "bun:test";
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";

import { ChipGroup } from "@/components/ui/chips";
import { Stepper } from "@/components/ui/stepper";
import { isBlankSummary } from "@/utils/setup";
import type { DashboardSummary } from "@/services/types";

function Chips() {
  const [v, setV] = useState("bills");
  return (
    <ChipGroup
      label="Category"
      value={v}
      onChange={setV}
      options={[
        { value: "bills", label: "bills" },
        { value: "gym", label: "gym" },
        { value: "house", label: "house" },
      ]}
    />
  );
}

describe("ChipGroup", () => {
  it("marks one pill checked and changes on tap", () => {
    render(<Chips />);
    expect(
      screen.getByRole("radio", { name: "bills" }).getAttribute("aria-checked"),
    ).toBe("true");
    fireEvent.click(screen.getByRole("radio", { name: "gym" }));
    expect(
      screen.getByRole("radio", { name: "gym" }).getAttribute("aria-checked"),
    ).toBe("true");
    expect(
      screen.getByRole("radio", { name: "bills" }).getAttribute("aria-checked"),
    ).toBe("false");
  });

  it("moves with the arrow keys and wraps around", () => {
    render(<Chips />);
    fireEvent.keyDown(screen.getByRole("radio", { name: "bills" }), {
      key: "ArrowLeft",
    });
    expect(
      screen.getByRole("radio", { name: "house" }).getAttribute("aria-checked"),
    ).toBe("true");
  });
});

function Step() {
  const [n, setN] = useState(1);
  return (
    <Stepper
      label="Repeats"
      value={n}
      onChange={setN}
      min={1}
      max={3}
      format={(x) => (x === 1 ? "Every month" : `Every ${x} months`)}
    />
  );
}

describe("Stepper", () => {
  it("steps within its bounds", () => {
    render(<Step />);
    const fewer = screen.getByRole("button", {
      name: /Fewer/,
    }) as HTMLButtonElement;
    const more = screen.getByRole("button", {
      name: /More/,
    }) as HTMLButtonElement;
    expect(fewer.disabled).toBe(true);
    fireEvent.click(more);
    fireEvent.click(more);
    expect(screen.getByText("Every 3 months")).toBeTruthy();
    expect(more.disabled).toBe(true);
  });
});

describe("isBlankSummary", () => {
  const blank = {
    next_income: null,
    spending: { tracked: false },
    pots: { spending: { account_name: null }, locked: { count: 0 } },
  } as unknown as DashboardSummary;

  it("is true only when there is no income, account or spending", () => {
    expect(isBlankSummary(blank)).toBe(true);
    expect(
      isBlankSummary({
        ...blank,
        next_income: { label: "Salary", arrives: "2026-12-01" },
      }),
    ).toBe(false);
    expect(
      isBlankSummary({
        ...blank,
        spending: { tracked: true },
      } as unknown as DashboardSummary),
    ).toBe(false);
  });
});
