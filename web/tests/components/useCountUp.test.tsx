import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";

import { useCountUp } from "@/hooks/useCountUp";

function Probe({ n }: { n: number }) {
  return <p>{useCountUp(n)}</p>;
}

describe("useCountUp", () => {
  it("shows the first value as it is, never starting from zero", () => {
    render(<Probe n={1234} />);
    expect(screen.getByText("1234")).toBeTruthy();
  });

  it("ends on the new value after a change", async () => {
    const { rerender } = render(<Probe n={100} />);
    rerender(<Probe n={500} />);
    await new Promise((r) => setTimeout(r, 1000));
    expect(screen.getByText("500")).toBeTruthy();
  });
});
