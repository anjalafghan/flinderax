import { type Page } from "@playwright/test";

import { expect, test } from "./fixtures";
import { newUser } from "./helpers";

/** A date `days` from today as YYYY-MM-DD (local), the way a date input wants it. */
function inDays(days: number): string {
  const d = new Date();
  d.setDate(d.getDate() + days);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** Unsafe requests made with the page's cookie must carry the X-Joy header. */
async function post(page: Page, path: string, data: unknown) {
  const res = await page.request.post(path, {
    data,
    headers: { "X-Joy": "1" },
  });
  expect(res.status(), await res.text()).toBe(200);
  return (await res.json()) as Record<string, unknown>;
}

test("add a large yearly amount and see it in the year ahead", async ({
  page,
}) => {
  await newUser(page, "year");
  await page.goto("/year");
  await expect(
    page.getByRole("heading", { name: "Year ahead", level: 1 }),
  ).toBeVisible();
  await expect(page.getByText("Nothing added yet.")).toBeVisible();

  await page.locator("#out-label").fill("Term insurance premium");
  await page.locator("#out-amount").fill("48000");
  await page.locator("#out-date").fill(inDays(90));
  await page.locator("#out-kind").selectOption("premium");
  await page.getByLabel("Repeats every year").check();
  await page.getByRole("button", { name: "Add", exact: true }).click();

  // It is listed with a monthly set-aside, and it appears in a month of the strip.
  await expect(page.getByText("Nothing added yet.")).toHaveCount(0);
  await expect(page.getByText(/set aside ₹/)).toBeVisible();
  await expect(page.getByText(/Term insurance premium/).first()).toBeVisible();
  await expect(page.getByText("−₹48,000").first()).toBeVisible();

  // Removing it takes it out again.
  await page
    .getByRole("button", { name: "Remove Term insurance premium" })
    .click();
  await expect(page.getByText("Nothing added yet.")).toBeVisible();
});

test("a wedding budget shows its verdict and a due payment, and the payment lands in the year ahead", async ({
  page,
}) => {
  await newUser(page, "wedding");
  // The two of you share a goal: a couple is needed before one can be started.
  await post(page, "/api/couple/create", { packs: [] });
  const goal = await post(page, "/api/couple-goals", {
    name: "Our wedding",
    target_paise: 50_000_000,
    target_date: inDays(300),
  });
  const goalId = goal.goal_id as string;
  const item = await post(page, `/api/couple-goals/${goalId}/items`, {
    name: "Venue hall",
    category: "venue",
    estimate_paise: 20_000_000,
  });
  await post(
    page,
    `/api/couple-goals/${goalId}/items/${item.item_id as string}/milestones`,
    { due_on: inDays(20), amount_paise: 5_000_000 },
  );

  await page.goto(`/wedding/${goalId}`);
  await expect(
    page.getByRole("status", { name: "Budget verdict" }),
  ).toBeVisible();
  await expect(page.getByText("Venue hall").first()).toBeVisible();
  await expect(page.getByText(/Planned ₹2,00,000/)).toBeVisible();

  // The unpaid payment is on the year ahead.
  await page.goto("/year");
  await expect(page.getByText(/Our wedding: Venue hall/).first()).toBeVisible();
  await expect(page.getByText("−₹50,000").first()).toBeVisible();

  // Marking it paid records the money and takes it off the year ahead.
  await page.goto(`/wedding/${goalId}`);
  await page.getByRole("button", { name: /Mark Venue hall .* paid/ }).click();
  await expect(
    page.getByRole("button", { name: "Undo payment" }),
  ).toBeVisible();
  await page.goto("/year");
  await expect(page.getByText(/Our wedding: Venue hall/)).toHaveCount(0);
});

test("set up an education plan and see the monthly top-up with its range", async ({
  page,
}) => {
  await newUser(page, "edu");
  await page.goto("/life");
  const card = page.getByRole("region", { name: /Education fund/ });
  await expect(card).toBeVisible();
  await expect(card.getByText(/A starter plan/)).toBeVisible();

  await card.getByRole("button", { name: "Set up a plan" }).click();
  await page.locator("#edu-year").fill(String(new Date().getFullYear() + 1));
  await card.getByRole("button", { name: "Save plan" }).click();

  // Saved: the starter note is gone and the top-up shows with a low / base / high range.
  await expect(card.getByText(/A starter plan/)).toHaveCount(0);
  await expect(card.getByText("a month")).toBeVisible();
  await expect(card.getByLabel("Range of monthly top-up")).toBeVisible();
  await expect(card.getByText(/not investment advice/)).toBeVisible();

  // It is still there after a reload.
  await page.reload();
  await expect(
    page
      .getByRole("region", { name: /Education fund/ })
      .getByRole("button", { name: "Edit plan" }),
  ).toBeVisible();
});

test("the week page lists what needs a decision, with counts and no names", async ({
  page,
}) => {
  await newUser(page, "decide");
  // A large amount due within two weeks needs a decision.
  await post(page, "/api/year/outflows", {
    label: "Secret premium",
    on_day: inDays(5),
    amount_paise: 4_000_000,
    kind: "premium",
  });
  await page.goto("/week");
  const card = page.getByRole("region", { name: "Needs a decision" });
  await expect(card).toBeVisible();
  await expect(card.getByText("1 large amount coming up")).toBeVisible();
  await expect(card.getByText(/Secret premium/)).toHaveCount(0);
  await card.getByRole("link", { name: /1 large amount coming up/ }).click();
  await expect(page).toHaveURL(/\/year$/);
});
