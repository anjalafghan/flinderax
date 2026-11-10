import api from "@/services/api";

/** Account lifecycle calls. A wrong password comes back as 403 with a plain sentence the form can show. */
export const sessionApi = {
  changePassword: (current_password: string, new_password: string) =>
    api
      .post<{ access_token: string; expires_at: number }>("/api/common/password", { current_password, new_password })
      .then((r) => r.data),
  logoutEverywhere: () => api.post<{ status: boolean }>("/api/common/logout-everywhere", {}).then((r) => r.data),
  deleteAccount: (password: string) =>
    api.post<{ status: boolean }>("/api/common/delete-account", { password, confirm: "delete" }).then((r) => r.data),
};
