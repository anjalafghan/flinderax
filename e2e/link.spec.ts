import { expect, test } from "./fixtures";
import { newUser, startJarWithSlip } from "./helpers";

test("link the bank payment that paid for a kept date", async ({ page }) => {
  await newUser(page, "link");
  await startJarWithSlip(page, "Dinner at the lake");

  // A payment, added by hand on today's date.
  await page.goto("/spending");
  await page.getByRole("tab", { name: "Manual" }).click();
  await page.getByText("Add a transaction by hand").click();
  await page.locator("#tx-amount").fill("1230.50");
  await page.locator("#tx-merchant").fill("Lakeside Grill");
  await page.locator("#tx-category").selectOption({ index: 1 });
  await page.getByRole("button", { name: "Add transaction" }).click();
  await expect(page.getByText("Lakeside Grill").first()).toBeVisible();

  // Keep the date, then say which payment it was.
  await page.goto("/joy");
  await page.getByRole("button", { name: "Draw a slip" }).click();
  await page.getByRole("button", { name: "It's a date" }).click();
  await page.goto("/joy/kept");
  await page.getByText("More", { exact: true }).first().click();
  await page.getByText("Which payment was this?").click();
  await page.getByRole("button", { name: /link lakeside grill/i }).click();
  await expect(page.getByText(/paid with Lakeside Grill/i)).toBeVisible();
  await expect(page.getByText(/actually/i).first()).toContainText("1,230.50");
});
