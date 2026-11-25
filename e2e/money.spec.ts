import { expect, test } from "./fixtures";
import { newUser } from "./helpers";

test("add a transaction by hand and see it counted in this month's spending", async ({
  page,
}) => {
  await newUser(page, "money");
  await page.goto("/spending");
  await expect(page.getByText("No spending yet this month.")).toBeVisible();

  await page.getByRole("tab", { name: "Manual" }).click();
  await page.getByText("Add a transaction by hand").click();
  await page.locator("#tx-amount").fill("250");
  await page.locator("#tx-merchant").fill("Corner chai shop");
  await page.locator("#tx-category").selectOption({ index: 1 });
  await page.getByRole("button", { name: "Add transaction" }).click();

  // It is listed, and the month's total has moved.
  await expect(page.getByText("Corner chai shop").first()).toBeVisible();
  await expect(page.getByText("₹250").first()).toBeVisible();
  await expect(page.getByText("No spending yet this month.")).toHaveCount(0);
});
