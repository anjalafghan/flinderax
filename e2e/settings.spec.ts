import { expect, test } from "./fixtures";
import { newUser, startJarWithSlip } from "./helpers";

test("one settings page holds the account, the budget and the Dates settings", async ({
  page,
}) => {
  await newUser(page, "settings");
  await startJarWithSlip(page, "Anything");

  await page.goto("/account");
  await expect(
    page.getByRole("heading", { name: "Settings", level: 1 }),
  ).toBeVisible();
  // Money plan settings, Dates and partner settings, backup and the account: all on this page.
  await expect(
    page.getByRole("heading", { name: /budget settings/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Your partner", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Your data" })).toBeVisible();
  await expect(page.getByRole("link", { name: /dates settings/i })).toHaveCount(
    0,
  );

  // The old Dates settings address lands here instead of on a second settings page.
  await page.goto("/joy/settings");
  await expect(page).toHaveURL(/\/account$/);
});

test("Dates has the app's own header and a sub-menu, not a second 'joy' header", async ({
  page,
}) => {
  await newUser(page, "navmerge");
  await startJarWithSlip(page, "Anything");
  await page.goto("/joy");
  await expect(page.getByRole("navigation", { name: "Dates" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Dates, home" })).toHaveCount(0);
  // The sub-menu has no Settings entry: settings live in one place.
  await expect(
    page
      .getByRole("navigation", { name: "Dates" })
      .getByRole("link", { name: "Settings" }),
  ).toHaveCount(0);
  // Dates draws in the app's fonts only.
  const hand = await page.evaluate(() =>
    getComputedStyle(document.querySelector(".joy")!).getPropertyValue(
      "--font-hand",
    ),
  );
  expect(hand).not.toContain("Caveat");
});
