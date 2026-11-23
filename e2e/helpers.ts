import { type Page } from "@playwright/test";
import { expect } from "./fixtures";

export const PASSWORD = "correct horse battery";
export const SIGNUP_CODE = "e2e-code";

let counter = Date.now() % 100000;
/** A username nobody has used on this throw-away server. */
export const uniqueName = (prefix: string) => `${prefix}${counter++}`;

/**
 * Creates an account through the Register tab. Registering signs you straight in, so this then signs
 * out again: callers that want to test the Login tab start from a signed-out browser.
 */
export async function register(page: Page, name: string) {
  await page.goto("/auth");
  await page.getByRole("button", { name: "Register", exact: true }).first().click();
  await page.locator("#reg-username").fill(name);
  await page.locator("#reg-code").fill(SIGNUP_CODE);
  await page.locator("#reg-password").fill(PASSWORD);
  const [res] = await Promise.all([
    page.waitForResponse((r) => r.url().includes("/api/common/register")),
    page.locator("form:visible button[type=submit]").click(),
  ]);
  expect(res.status(), await res.text()).toBe(200);
  await expect(page).not.toHaveURL(/\/auth/);
  await page.evaluate(() => localStorage.removeItem("token"));
  await page.context().clearCookies();
}

/** Registers and stays signed in, the way a person does. */
export async function registerAndStay(page: Page, name: string) {
  await page.goto("/auth");
  await page.getByRole("button", { name: "Register", exact: true }).first().click();
  await page.locator("#reg-username").fill(name);
  await page.locator("#reg-code").fill(SIGNUP_CODE);
  await page.locator("#reg-password").fill(PASSWORD);
  await page.locator("form:visible button[type=submit]").click();
  await expect(page).not.toHaveURL(/\/auth/);
}

export async function signIn(page: Page, name: string) {
  await page.goto("/auth");
  await page.getByRole("button", { name: "Login", exact: true }).first().click();
  await page.locator("#username").fill(name);
  await page.locator("#password").fill(PASSWORD);
  await page.locator("form:visible button[type=submit]").click();
  await expect(page).not.toHaveURL(/\/auth/);
}

export async function newUser(page: Page, prefix = "user") {
  const name = uniqueName(prefix);
  await register(page, name);
  await signIn(page, name);
  return name;
}

/** Starts a Dates jar for this signed-in user (no starter slips) and writes one slip. */
export async function startJarWithSlip(page: Page, title: string, cost = "0") {
  await page.goto("/joy");
  await page.getByRole("button", { name: /Start our jar/ }).click();
  await page.getByRole("button", { name: "Continue" }).click();
  await page.goto("/joy/slips/new");
  await page.getByRole("textbox", { name: "The idea" }).fill(title);
  await page.getByRole("spinbutton", { name: "Cost for both of us" }).fill(cost);
  await page.locator("#save-btn").click();
  await expect(page).toHaveURL(/\/joy\/slips$/);
}
