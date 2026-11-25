import { expect, test } from "./fixtures";
import { newUser, startJarWithSlip } from "./helpers";

test("write a slip, draw it from the jar and keep it as a date", async ({
  page,
}) => {
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

test("putting a slip back has no limit and is never a date", async ({
  page,
}) => {
  test.setTimeout(60_000); // six draws, each with its card animation
  const titles = ["Board games and chai", "Kite flying at Juhu"];
  await newUser(page, "putback");
  await startJarWithSlip(page, titles[0]);
  await page.goto("/joy/slips/new");
  await page.getByRole("textbox", { name: "The idea" }).fill(titles[1]);
  await page.getByRole("spinbutton", { name: "Cost for both of us" }).fill("0");
  await page.locator("#save-btn").click();
  await expect(page).toHaveURL(/\/joy\/slips$/);

  await page.goto("/joy");
  await page.getByRole("button", { name: "Draw a slip" }).click();
  const shown = page.getByText(new RegExp(titles.join("|"))).first();
  const back = page.locator("button:not([disabled])", {
    hasText: /^Put it back$/,
  });
  const startOver = page.getByRole("button", {
    name: "Start over with these filters",
  });
  const pick = () =>
    page.waitForResponse((r) => r.url().endsWith("/api/dates/pick"), {
      timeout: 1_000,
    });
  // Every put-back draws again. Once each slip has been put back the jar offers a fresh round,
  // and that goes on as long as you like: there is no cap to run into.
  await expect(shown).toBeVisible();
  for (let sent = 0; sent < 6;) {
    // A card can still be on screen, looking ready, while it settles or folds away (a click then
    // does nothing), so retry until a click really reaches the server.
    await expect(async () => {
      if (await startOver.isVisible()) {
        const [res] = await Promise.all([
          pick(),
          startOver.click({ timeout: 1_000 }),
        ]);
        expect(res.ok()).toBe(true);
        return;
      }
      const [res] = await Promise.all([pick(), back.click({ timeout: 1_000 })]);
      expect(res.ok(), `put-back ${sent + 1}`).toBe(true);
      sent++;
    }).toPass({ timeout: 15_000 });
  }
  await expect(shown.or(startOver)).toBeVisible();

  // None of that became a date.
  await page.goto("/joy/kept");
  for (const t of titles) await expect(page.getByText(t)).toHaveCount(0);
});

test("mark today as the day we met, straight from the jar", async ({
  page,
}) => {
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
