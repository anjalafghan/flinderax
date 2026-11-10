import { afterEach, beforeEach, describe, expect, mock, test } from "bun:test";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { AuthContext } from "@/context/AuthContext";
import { AccountSection } from "./AccountSection";

type Call = { url: string; body: unknown };
let calls: Call[];
let reply: (url: string) => Response;
const realFetch = globalThis.fetch;
const login = mock(() => {});
const logout = mock(() => {});

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

function view() {
  return render(
    <AuthContext.Provider value={{ isAuthenticated: true, token: "old", login, logout, isLoading: false }}>
      <MemoryRouter initialEntries={["/account"]}>
        <Routes>
          <Route path="/account" element={<AccountSection />} />
          <Route path="/auth" element={<p>sign in page</p>} />
        </Routes>
      </MemoryRouter>
    </AuthContext.Provider>,
  );
}

beforeEach(() => {
  calls = [];
  login.mockClear();
  logout.mockClear();
  reply = () => json({ status: true });
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({ url, body: init?.body ? JSON.parse(String(init.body)) : undefined });
    return reply(url);
  }) as unknown as typeof fetch;
});
afterEach(() => {
  globalThis.fetch = realFetch;
});

const type = (label: string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });

describe("AccountSection", () => {
  test("changing the password stores the fresh token", async () => {
    reply = () => json({ access_token: "fresh", expires_at: 123 });
    view();
    type("Current password", "old password");
    type("New password", "a brand new one");
    type("New password again", "a brand new one");
    fireEvent.click(screen.getByRole("button", { name: "Change password" }));
    await waitFor(() => expect(login).toHaveBeenCalledWith("fresh", 123));
    expect(calls[0]).toEqual({ url: "/api/common/password", body: { current_password: "old password", new_password: "a brand new one" } });
    expect(await screen.findByRole("status")).toHaveTextContent("Password changed");
  });

  test("mismatched new passwords never reach the server", () => {
    view();
    type("Current password", "old password");
    type("New password", "a brand new one");
    type("New password again", "something else");
    fireEvent.click(screen.getByRole("button", { name: "Change password" }));
    expect(screen.getByRole("alert")).toHaveTextContent("don't match");
    expect(calls).toHaveLength(0);
  });

  test("a wrong current password shows the server's sentence and keeps the session", async () => {
    reply = () => new Response("That isn't your current password.", { status: 403 });
    view();
    type("Current password", "nope nope nope");
    type("New password", "a brand new one");
    type("New password again", "a brand new one");
    fireEvent.click(screen.getByRole("button", { name: "Change password" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("isn't your current password");
    expect(login).not.toHaveBeenCalled();
  });

  test("sign out everywhere calls the server, clears the session and goes to sign in", async () => {
    view();
    fireEvent.click(screen.getByRole("button", { name: "Sign out everywhere" }));
    expect(await screen.findByText("sign in page")).toBeInTheDocument();
    expect(calls[0].url).toBe("/api/common/logout-everywhere");
    expect(logout).toHaveBeenCalledTimes(1);
  });

  test("deleting needs the typed word and a password", async () => {
    view();
    fireEvent.click(screen.getByRole("button", { name: "Delete my account…" }));
    const go = screen.getByRole("button", { name: "Delete everything" });
    expect(go).toBeDisabled();
    type("Type delete to confirm", "delete");
    expect(go).toBeDisabled();
    type("Your password", "correct horse");
    expect(go).toBeEnabled();
    fireEvent.click(go);
    expect(await screen.findByText("sign in page")).toBeInTheDocument();
    expect(calls[0]).toEqual({ url: "/api/common/delete-account", body: { password: "correct horse", confirm: "delete" } });
    expect(logout).toHaveBeenCalledTimes(1);
  });

  test("a failed delete keeps you signed in and shows why", async () => {
    reply = () => new Response("That is the last admin account, so it can't be deleted.", { status: 409 });
    view();
    fireEvent.click(screen.getByRole("button", { name: "Delete my account…" }));
    type("Type delete to confirm", "delete");
    type("Your password", "correct horse");
    fireEvent.click(screen.getByRole("button", { name: "Delete everything" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("last admin");
    expect(logout).not.toHaveBeenCalled();
  });
});
