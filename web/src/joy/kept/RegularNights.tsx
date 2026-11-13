import { useState } from "react";
import { Link } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getNights, nightsKey } from "../regulars/api";
import { EditAmount } from "../regulars/EditAmount";

/** The regular nights we went to, with their price, so they sit beside the dates we kept. */
export function RegularNights() {
  const qc = useQueryClient();
  const q = useQuery({ queryKey: nightsKey, queryFn: getNights });
  const [editing, setEditing] = useState<number | null>(null);
  const nights = q.data?.nights ?? [];
  if (nights.length === 0) return null;
  const refresh = () => Promise.all([qc.invalidateQueries({ queryKey: nightsKey }), qc.invalidateQueries({ queryKey: ["joy"] })]);
  return (
    <section className="k-nights" aria-labelledby="k-nights-h">
      <h2 id="k-nights-h" className="k-month-h">Regular nights</h2>
      <ul className="k-nights-list">
        {nights.map((n) => (
          <li key={n.visit_id}>
            <p>
              <strong>{n.place || n.title}</strong> · {n.day_label} {n.year} · {n.amount}{" "}
              <Link to={`/joy/regulars/${n.regular_id}`}>{n.title}</Link>
            </p>
            {editing === n.visit_id ? (
              <EditAmount visitId={n.visit_id} initial={n.amount_raw} onCancel={() => setEditing(null)} onSaved={() => { setEditing(null); void refresh(); }} />
            ) : (
              <button type="button" className="btn btn-quiet" onClick={() => setEditing(n.visit_id)}>Edit price</button>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
