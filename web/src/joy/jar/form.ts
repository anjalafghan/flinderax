// The sentence ("For Thursday evening we have ...") as state, and the forms it turns into.
// Pure: nothing here touches the DOM, so it is all unit tested.

/** The server's PickForm. Every field is a string (as Joy's form fields were). Empty = unset. */
export interface PickForm {
  budget?: string;
  max_duration?: string;
  settings?: string[];
  tag?: string;
  energy?: string;
  max_km?: string;
  from?: string;
  excluded?: string;
  reason?: string;
  current?: string;
  reroll?: string;
  veto?: string;
  middle?: string;
  when?: string;
  start?: string;
  /** "1": something new, only slips we have never had or booked. */
  fresh?: string;
}

export interface Filters {
  when: string;
  start: "" | "office" | "home";
  budget: string;
  max_duration: string;
  settings: string[];
  tag: string;
  energy: string;
  max_km: string;
  from: string;
  /** "1" = something new. */
  fresh: string;
}

export const EMPTY_FILTERS: Filters = {
  when: "",
  start: "",
  budget: "",
  max_duration: "",
  settings: [],
  tag: "",
  energy: "",
  max_km: "",
  from: "",
  fresh: "",
};

const SCALARS = [
  "when",
  "start",
  "budget",
  "max_duration",
  "tag",
  "energy",
  "max_km",
  "from",
  "fresh",
] as const;

/** Drop the empty fields; the server treats missing and empty alike. */
export function compact(form: PickForm): PickForm {
  const out: PickForm = {};
  for (const [k, v] of Object.entries(form)) {
    if (Array.isArray(v)) {
      const list = v.filter((s) => s !== "");
      if (list.length) (out as Record<string, unknown>)[k] = list;
    } else if (typeof v === "string" && v.trim() !== "") {
      (out as Record<string, unknown>)[k] = v.trim();
    }
  }
  return out;
}

/** The sentence as a form for /match and /pick. `middle` draws from tonight's check-ins instead. */
export function filtersToForm(f: Filters, middle = false): PickForm {
  if (middle) return compact({ middle: "1", when: f.when, start: f.start });
  const form: PickForm = { settings: f.settings };
  for (const k of SCALARS) form[k] = f[k];
  return compact(form);
}

/** Tap a place to circle it, tap again to un-circle it. */
export function toggleSetting(settings: string[], value: string): string[] {
  return settings.includes(value)
    ? settings.filter((s) => s !== value)
    : [...settings, value];
}

/** Tap "office" or "home" again to go back to the usual start. */
export function toggleStart(
  current: Filters["start"],
  value: "office" | "home",
): Filters["start"] {
  return current === value ? "" : value;
}

/** The server's `state` is the kept filters plus where the draw has got to, WITHOUT reroll/veto set. */
export const putBackForm = (
  state: PickForm,
  reason?: PutBackReason,
): PickForm =>
  compact({
    ...state,
    reroll: "1",
    veto: "",
    reason: PUT_BACK_REASONS.some((r) => r.value === reason) ? reason : "",
  });

/** The optional one-tap why. Only "not my thing" teaches the jar what you like. */
export type PutBackReason =
  "not_tonight" | "too_far" | "too_pricey" | "not_my_thing";
export const PUT_BACK_REASONS: { value: PutBackReason; label: string }[] = [
  { value: "not_tonight", label: "Not tonight" },
  { value: "too_far", label: "Too far" },
  { value: "too_pricey", label: "Too pricey" },
  { value: "not_my_thing", label: "Not my thing" },
];
/** "Not for me": out of the draw for both of us; there is no limit on putting back. */
export const vetoForm = (state: PickForm): PickForm =>
  compact({ ...state, veto: "1", reroll: "" });

/** The sentence after a relax: write the loosened filters back so it tells the truth. */
export function applyRelax(f: Filters, relax: PickForm): Filters {
  const next: Filters = { ...f };
  for (const k of SCALARS) {
    if (k === "start")
      next.start =
        relax.start === "office" || relax.start === "home" ? relax.start : "";
    else next[k] = relax[k] ?? "";
  }
  next.settings = relax.settings ?? [];
  return next;
}

/** Today as Joy stamps it ("14 · 10 · 2026") when the server didn't say. */
export const stampDate = (server: string, now = new Date()) =>
  server ||
  [now.getDate(), now.getMonth() + 1, now.getFullYear()]
    .map((n) => String(n).padStart(2, "0"))
    .join(" · ");
