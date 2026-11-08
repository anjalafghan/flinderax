import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";

import { HowItsCalculated } from "@/components/feature/HowItsCalculated";

describe("HowItsCalculated", () => {
  it("shows each step, minus signs on deductions and totals as-is", () => {
    render(
      <HowItsCalculated
        lines={[
          { label: "Cash in spendable accounts", amount_paise: 11_216_099, note: null, total: false },
          { label: "Card bills already spent", amount_paise: -7_953_818, note: "keep aside", total: false },
          { label: "Cash you can spend", amount_paise: -1_737_719, note: null, total: true },
        ]}
      />,
    );
    expect(screen.getByText("₹1,12,160.99")).toBeInTheDocument();
    expect(screen.getByText("−₹79,538.18")).toBeInTheDocument();
    expect(screen.getByText("keep aside")).toBeInTheDocument();
    expect(screen.getByText("-₹17,377.19")).toBeInTheDocument(); // a negative total keeps its own sign
  });
});
