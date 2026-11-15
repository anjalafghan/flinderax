import { expect, test } from "./fixtures";
import { newUser } from "./helpers";

test("a payment typed with no signal is kept and sent once the phone is back online", async ({ page }) => {
  await newUser(page, "offline");
  await page.goto("/spending");
  await page.getByRole("tab", { name: "Manual" }).click();
  await page.getByText("Add a transaction by hand").click();

  // No signal for this call: the request never reaches the server.
  await page.route("**/api/ingest/transactions/create", (r) => r.abort("internetdisconnected"));
  await page.locator("#tx-amount").fill("180");
  await page.locator("#tx-merchant").fill("Auto fare");
  await page.locator("#tx-category").selectOption({ index: 1 });
  await page.getByRole("button", { name: "Add transaction" }).click();
  await expect(page.getByText(/saved on this phone/i)).toBeVisible();
  expect(await page.evaluate(() => JSON.parse(localStorage.getItem("joy-outbox-v1") ?? "[]").length)).toBe(1);

  await page.unroute("**/api/ingest/transactions/create");
  await page.evaluate(() => window.dispatchEvent(new Event("online")));
  await expect(page.getByText(/sent 1 saved item/i)).toBeVisible();
  expect(await page.evaluate(() => JSON.parse(localStorage.getItem("joy-outbox-v1") ?? "[]").length)).toBe(0);
  await page.reload();
  await expect(page.getByText("Auto fare").first()).toBeVisible();
});
