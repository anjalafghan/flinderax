import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

export interface Decision {
  kind: string;
  count: number;
  /** "5 transactions need a category". Counts only, never a name or an amount. */
  text: string;
  /** Where in the app to act on it, e.g. "/review". */
  link: string;
}

export const decisionsKey = ["decisions"] as const;

export const useDecisions = () =>
  useQuery({ queryKey: decisionsKey, queryFn: () => api.get<{ items: Decision[] }>("/api/decisions").then((r) => r.data.items), retry: 1 });
