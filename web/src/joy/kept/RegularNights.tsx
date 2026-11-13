import { useState } from "react";
import { Link } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { nightsKey, type NightRow } from "../regulars/api";
import { EditAmount } from "../regulars/EditAmount";
import type { Month } from "./types";

export interface MergedMonth extends Month {
  nights: NightRow[];
}

const monthHref = (day: string) => `/joy/year/${day.slice(0, 4)}/${day.slice(5, 7).replace(/^0/, "")}`;
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/** A regular night is a date like any other: it joins its month in the one Kept list. Months newest first. */
export function mergeMonths(months: Month[], nights: NightRow[]): MergedMonth[] {
  const out: MergedMonth[] = months.map((m) => ({ ...m, nights: [] }));
  for (const n of nights) {
    const href = monthHref(n.day);
    let m = out.find((x) => x.href === href);
    if (!m) {
      m = { label: `${MONTHS[Number(n.day.slice(5, 7)) - 1] ?? ""} ${n.day.slice(0, 4)}`, href, items: [], nights: [] };
      out.push(m);
    }
    m.nights.push(n);
  }
  const key = (h: string) => { const [y, mo] = h.split("/").slice(-2).map(Number); return (y ?? 0) * 12 + (mo ?? 0); };
  return out.sort((a, b) => key(b.href) - key(a.href));
}

/** One regular night, shown in the Kept list the same way as any date. */
export function NightItem({ n }: { n: NightRow }) {
  const qc = useQueryClient();
  const [editing, setEditing] = useState(false);
  return (
    <li className="k-night">
      {n.photo && <img className="reg-thumb" src={n.photo} alt={`Photo from ${n.title}`} width={96} height={120} loading="lazy" />}
      <p>
        <strong>{n.place || n.title}</strong> · {n.day_label} {n.year} · {n.amount}{" "}
        <Link to={`/joy/regulars/${n.regular_id}`}>Open</Link>
      </p>
      {editing ? (
        <EditAmount
          visitId={n.visit_id}
          initial={n.amount_raw}
          onCancel={() => setEditing(false)}
          onSaved={() => { setEditing(false); void qc.invalidateQueries({ queryKey: nightsKey }); void qc.invalidateQueries({ queryKey: ["joy"] }); }}
        />
      ) : (
        <button type="button" className="btn btn-quiet" onClick={() => setEditing(true)}>Edit price</button>
      )}
    </li>
  );
}
