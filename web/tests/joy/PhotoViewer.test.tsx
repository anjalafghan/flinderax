import { describe, expect, it } from "bun:test";
import { fireEvent, render, screen } from "@testing-library/react";
import { PhotoViewer } from "@/joy/components/PhotoViewer";
import { stepTo } from "@/joy/components/viewer";

const photos = [1, 2, 3].map((n) => ({ url: `/p/${n}.jpg`, alt: `Photo ${n} of 3 from Picnic`, caption: "Picnic · 14 Mar" }));

describe("stepTo", () => {
  it("wraps round at both ends", () => {
    expect(stepTo(2, 3, 1)).toBe(0);
    expect(stepTo(0, 3, -1)).toBe(2);
    expect(stepTo(1, 3, 1)).toBe(2);
    expect(stepTo(0, 0, 1)).toBe(0);
  });
});

describe("PhotoViewer", () => {
  it("shows nothing while closed", () => {
    render(<PhotoViewer photos={photos} start={null} onClose={() => {}} />);
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("opens on the photo clicked and steps through all of them with the buttons and arrow keys", () => {
    render(<PhotoViewer photos={photos} start={1} onClose={() => {}} />);
    expect(screen.getByRole("img", { hidden: true })).toHaveAttribute("src", "/p/2.jpg");
    expect(screen.getByText("2 of 3")).toBeInTheDocument();
    expect(screen.getByText("Picnic · 14 Mar")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Next photo", hidden: true }));
    expect(screen.getByText("3 of 3")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Next photo", hidden: true }));
    expect(screen.getByText("1 of 3")).toBeInTheDocument();

    fireEvent.keyDown(screen.getByRole("button", { name: "Close photos", hidden: true }), { key: "ArrowLeft" });
    expect(screen.getByText("3 of 3")).toBeInTheDocument();
    expect(screen.getByRole("img", { hidden: true })).toHaveAttribute("src", "/p/3.jpg");
  });

  it("moves on a sideways swipe", () => {
    render(<PhotoViewer photos={photos} start={0} onClose={() => {}} />);
    const fig = screen.getByRole("img", { hidden: true }).closest("figure")!;
    fireEvent.pointerDown(fig, { clientX: 300 });
    fireEvent.pointerUp(fig, { clientX: 200 });
    expect(screen.getByText("2 of 3")).toBeInTheDocument();
    fireEvent.pointerDown(fig, { clientX: 100 });
    fireEvent.pointerUp(fig, { clientX: 120 });
    expect(screen.getByText("2 of 3")).toBeInTheDocument();
  });

  it("has no arrows or count for a single photo", () => {
    render(<PhotoViewer photos={photos.slice(0, 1)} start={0} onClose={() => {}} />);
    expect(screen.queryByRole("button", { name: "Next photo", hidden: true })).toBeNull();
    expect(screen.queryByText("1 of 1")).toBeNull();
  });
});
