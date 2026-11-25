import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "bun:test";
import { AuthProvider } from "./AuthProvider";
import { useAuth } from "./AuthContext";

function Probe() {
  const auth = useAuth();
  return <output>{auth.token ?? "signed-out"}</output>;
}

afterEach(() => {
  cleanup();
  localStorage.clear();
});

describe("cross-tab authentication", () => {
  test("a session change clears cached account data and updates auth state", () => {
    localStorage.setItem("token", "s_account_a");
    const client = new QueryClient();
    client.setQueryData(["account", "private"], { balance: 10 });
    render(
      <QueryClientProvider client={client}>
        <AuthProvider><Probe /></AuthProvider>
      </QueryClientProvider>,
    );
    expect(screen.getByText("s_account_a")).toBeTruthy();

    localStorage.setItem("token", "s_account_b");
    act(() => {
      window.dispatchEvent(new StorageEvent("storage", { key: "token", newValue: "s_account_b" }));
    });
    expect(screen.getByText("s_account_b")).toBeTruthy();
    expect(client.getQueryData(["account", "private"])).toBeUndefined();
  });
});
