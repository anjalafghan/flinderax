import { expect, test } from "./fixtures";
import { newUser, startJarWithSlip } from "./helpers";

test("write a slip, draw it from the jar and keep it as a date", async ({ page }) => {
  await newUser(page, "dates");
  await startJarWithSlip(page, "Sunset at Marine Drive");

  // The slip is listed.
  await expect(page.getByText("Sunset at Marine Drive").first()).toBeVisible();

  // Draw: with one slip in the jar it is the one that comes out.
  await page.goto("/joy");
  await page.getByRole("button", { name: "Draw a slip" }).click();
  await expect(page.getByText("Sunset at Marine Drive").first()).toBeVisible();
  await page.getByRole("button", { name: "It's a date" }).click();

  // Kept lists it as a date.
  await page.goto("/joy/kept");
  await expect(page.getByText("Sunset at Marine Drive").first()).toBeVisible();
});

test("putting a slip back has no limit", async ({ page }) => {
  await newUser(page, "putback");
  await startJarWithSlip(page, "Board games and chai");
  await page.goto("/joy");
  await page.getByRole("button", { name: "Draw a slip" }).click();
  await expect(page.getByRole("button", { name: "Put it back" })).toBeVisible();
  for (let i = 0; i < 5; i++) {
    const back = page.getByRole("button", { name: "Put it back" });
    if (!(await back.isVisible())) break;
    await back.click();
    await page.waitForTimeout(500);
  }
  // Nothing says "no more tries": there is no limit to hide the button behind.
  await expect(page.getByText(/no more tries|more tries/i)).toHaveCount(0);
});

test("mark today as the day we met, straight from the jar", async ({ page }) => {
  await newUser(page, "markday");
  await startJarWithSlip(page, "Chai at the corner stall");

  await page.goto("/joy");
  await page.getByRole("link", { name: /Mark a day to remember/ }).click();
  await expect(page).toHaveURL(/\/joy\/kept#add$/);

  // One tap names it; the day is already today.
  await page.getByRole("button", { name: "The day we met" }).click();
  await expect(page.getByLabel("What happened?")).toHaveValue("The day we met");
  await page.getByRole("button", { name: "Keep this day" }).click();

  await expect(page.getByText("The day we met").first()).toBeVisible();
});
