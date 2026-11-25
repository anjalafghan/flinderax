import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import api from "@/services/api";

export type OutflowKind =
  "premium" | "tax" | "fee" | "maturity" | "wedding" | "other";

export interface LumpyItem {
  on: string;
  label: string;
  amount_paise: number;
  direction: "out" | "in";
  kind: string;
}

export interface MonthRow {
  /** "YYYY-MM". */
  month: string;
  income_paise: number;
  fixed_paise: number;
  /** Everyday spending expected in the month (this month only from today). */
  everyday_paise: number;
  lumpy_out_paise: number;
  lumpy_in_paise: number;
  net_paise: number;
  closing_paise: number;
  short: boolean;
  lumpy: LumpyItem[];
}

export interface YearView {
  months: MonthRow[];
  first_short_month: string | null;
  lowest_paise: number;
  total_lumpy_out_paise: number;
  /** Everyday spending counted for each full month. */
  everyday_monthly_paise: number;
}

export interface Outflow {
  outflow_id: string;
  label: string;
  on_day: string;
  amount_paise: number;
  direction: "out" | "in";
  kind: OutflowKind;
  yearly: boolean;
  next_on: string | null;
  set_aside_monthly_paise: number | null;
}

export interface OutflowInput {
  label: string;
  on_day: string;
  amount_paise: number;
  direction: "out" | "in";
  kind: OutflowKind;
  yearly: boolean;
}

export const KIND_LABEL: Record<OutflowKind, string> = {
  premium: "Insurance premium",
  tax: "Tax",
  fee: "Annual fee",
  maturity: "Maturity (FD / RD)",
  wedding: "Wedding",
  other: "Other",
};

export const yearKey = ["year"] as const;

const data = <T>(p: Promise<{ data: T }>) => p.then((r) => r.data);

export const yearApi = {
  view: () => data(api.get<YearView>("/api/year")),
  outflows: () => data(api.get<Outflow[]>("/api/year/outflows")),
  create: (o: OutflowInput) =>
    data(api.post<{ outflow_id: string }>("/api/year/outflows", o)),
  remove: (id: string) => data(api.post(`/api/year/outflows/${id}/delete`, {})),
};

export const useYear = () =>
  useQuery({ queryKey: [...yearKey, "view"], queryFn: yearApi.view, retry: 1 });
export const useOutflows = () =>
  useQuery({
    queryKey: [...yearKey, "outflows"],
    queryFn: yearApi.outflows,
    retry: 1,
  });

function useYearMutation<V>(fn: (v: V) => Promise<unknown>, success: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      toast.success(success);
      void qc.invalidateQueries({ queryKey: yearKey });
    },
    onError: (e: unknown) => {
      const body = (e as { response?: { data?: unknown } })?.response?.data;
      toast.error(
        typeof body === "string" && body.trim()
          ? body
          : "Something went wrong. Try again.",
      );
    },
  });
}

export const useAddOutflow = () =>
  useYearMutation((o: OutflowInput) => yearApi.create(o), "Added to the year");
export const useRemoveOutflow = () =>
  useYearMutation((id: string) => yearApi.remove(id), "Removed");

const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

/** "2027-03" -> "Mar 2027". */
export function monthLabel(ym: string): string {
  const [y, m] = ym.split("-");
  const name = MONTHS[Number(m) - 1];
  return name ? `${name} ${y}` : ym;
}

/** Share of the tallest bar, 0 to 100, for the month strip. Negative balances count as 0. */
export function barPct(closing: number, max: number): number {
  if (max <= 0 || closing <= 0) return 0;
  return Math.min(100, Math.round((closing * 100) / max));
}
