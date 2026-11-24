import { expect, test } from "./fixtures";
import { newUser } from "./helpers";

test("life at a glance shows the road ahead from the life plan", async ({ page }) => {
  await newUser(page, "glance");

  // Nothing dated yet: the road asks for a wedding date instead of guessing one.
  await page.goto("/glance");
  await expect(page.getByRole("heading", { name: "At a glance" })).toBeVisible();
  await expect(page.getByRole("link", { name: /Give the wedding a date/ })).toBeVisible();

  // Reached from the Life tab and from Today.
  await page.goto("/");
  await page.getByRole("link", { name: /Life at a glance/ }).click();
  await expect(page).toHaveURL(/\/glance$/);
  await expect(page.getByRole("link", { name: "At a glance" }).first()).toBeVisible();
});
