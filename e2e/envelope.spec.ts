import { expect, test } from "./fixtures";
import { newUser, startJarWithSlip } from "./helpers";

test("an envelope opens for someone who is not signed in, and a made-up link does not", async ({
  page,
  browser,
}) => {
  await newUser(page, "envelope");
  await startJarWithSlip(page, "Picnic by the lake");

  // The draw hands out a share text with the envelope link in it.
  const res = await page.request.post("/api/dates/pick", {
    data: {},
    headers: { "X-Joy": "1" },
  });
  expect(res.ok()).toBe(true);
  const link = /(\/joy\/e\/[A-Za-z0-9]{24})/.exec(
    (await res.json()).share_text,
  )?.[1];
  expect(link, "an envelope link").toBeTruthy();

  // A brand-new browser with no session opens it.
  const stranger = await browser.newContext({
    baseURL: "http://127.0.0.1:3123",
  });
  const other = await stranger.newPage();
  await other.goto(link!);
  await expect(other.getByText("Picnic by the lake").first()).toBeVisible();
  await expect(other).not.toHaveURL(/\/auth/);

  // A link nobody made says nothing.
  await other.goto("/joy/e/AAAAAAAAAAAAAAAAAAAAAAAA");
  await expect(other.getByText("Picnic by the lake")).toHaveCount(0);
  await stranger.close();
});
