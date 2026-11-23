import { expect, test } from "./fixtures";
import { newUser, PASSWORD, register, signIn, uniqueName } from "./helpers";

test("sign in: the session lives in an HttpOnly cookie and the page keeps no token", async ({ page, context }) => {
  await newUser(page, "signin");
  const cookie = (await context.cookies()).find((c) => c.name === "joy_session");
  expect(cookie, "a session cookie").toBeTruthy();
  expect(cookie?.httpOnly).toBe(true);
  expect(cookie?.sameSite).toBe("Strict");
  const kept = await page.evaluate(() => localStorage.getItem("token"));
  expect(kept?.startsWith("s_")).toBe(true);
  expect(kept).not.toContain("v4.local");
  // A reload stays signed in; the page is not sent back to the sign-in screen.
  await page.reload();
  await expect(page).not.toHaveURL(/\/auth/);
});

test("a wrong password stays on the sign-in screen with a message", async ({ page }) => {
  const name = uniqueName("wrongpw");
  await register(page, name);
  await page.goto("/auth");
  await page.locator("#username").fill(name);
  await page.locator("#password").fill(PASSWORD + "x");
  await page.locator("form:visible button[type=submit]").click();
  await expect(page).toHaveURL(/\/auth/);
  await expect(page.getByText(/invalid username or password|check your credentials/i).first()).toBeVisible();
});

test("signing out ends the session", async ({ page, context }) => {
  const name = await newUser(page, "signout");
  await page.goto("/account");
  // The cookie is HttpOnly, so only the server can clear it: this goes through the app's own button.
  const [res] = await Promise.all([
    page.waitForResponse((r) => r.url().endsWith("/api/common/logout")),
    page.getByRole("button", { name: "Sign out", exact: true }).click(),
  ]);
  expect(res.ok()).toBe(true);
  await expect(page).toHaveURL(/\/auth/);
  expect((await context.cookies()).find((c) => c.name === "joy_session"), "the session cookie is gone").toBeUndefined();
  expect(await page.evaluate(() => localStorage.getItem("token"))).toBeNull();
  // Coming back later still lands on sign-in, and signing in again works.
  await page.goto("/");
  await expect(page).toHaveURL(/\/auth/);
  await signIn(page, name);
});
