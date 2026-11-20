import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import api from "@/services/api";

// Analyses: written reports saved to your account, usually by a connector (GET /api/analyses,
// DELETE /api/analyses/{id}). Plain text; never rendered as HTML.

export interface Analysis {
  id: string;
  title: string;
  body: string;
  source: string;
  created_on: string;
}

export const analysesKey = ["analyses"] as const;

export const getAnalyses = () => api.get<{ analyses: Analysis[] }>("/api/analyses").then((r) => r.data);
export const removeAnalysis = (id: string) => api.del<void>(`/api/analyses/${id}`).then((r) => r.data);

export const useAnalyses = () => useQuery({ queryKey: analysesKey, queryFn: getAnalyses, retry: 1 });

export function useRemoveAnalysis() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: removeAnalysis,
    onSuccess: () => void qc.invalidateQueries({ queryKey: analysesKey }),
  });
}
