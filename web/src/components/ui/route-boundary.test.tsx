import { afterEach, beforeEach, describe, expect, spyOn, test } from "bun:test";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { RouteBoundary } from "./route-boundary";

function Boom({ explode }: { explode: boolean }) {
  if (explode) throw new Error("card exploded");
  return <p>all fine</p>;
}

let quiet: ReturnType<typeof spyOn>;
beforeEach(() => {
  quiet = spyOn(console, "error").mockImplementation(() => {});
});
afterEach(() => {
  cleanup();
  quiet.mockRestore();
});

describe("RouteBoundary", () => {
  test("a crash shows a small message, not a blank page, and the rest of the app stays", () => {
    render(
      <div>
        <nav>menu</nav>
        <RouteBoundary resetKey="/money">
          <Boom explode />
        </RouteBoundary>
      </div>,
    );
    expect(screen.getByText("menu")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toContain("This page hit a problem");
  });

  test("Try again renders the page again", () => {
    // The page recovers (say, the data arrived) between the crash and the tap.
    let broken = true;
    const Flaky = () => (broken ? <Boom explode /> : <Boom explode={false} />);
    render(
      <RouteBoundary resetKey="/money">
        <Flaky />
      </RouteBoundary>,
    );
    broken = false;
    fireEvent.click(screen.getByRole("button", { name: /try again/i }));
    expect(screen.getByText("all fine")).toBeTruthy();
  });

  test("moving to another page clears the error", () => {
    const { rerender } = render(
      <RouteBoundary resetKey="/money">
        <Boom explode />
      </RouteBoundary>,
    );
    expect(screen.getByRole("alert")).toBeTruthy();
    rerender(
      <RouteBoundary resetKey="/plan">
        <Boom explode={false} />
      </RouteBoundary>,
    );
    expect(screen.getByText("all fine")).toBeTruthy();
  });
});
