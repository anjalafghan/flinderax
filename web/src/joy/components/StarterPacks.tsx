import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { get, keys, post } from "../api";

interface Pack {
  key: string;
  label: string;
  blurb: string;
  total: number;
  added: number;
}

/** Starter slips come in packs a couple chooses: nothing is dropped into the jar that they did not pick. */
export function StarterPacks() {
  const qc = useQueryClient();
  const [note, setNote] = useState("");
  const packs = useQuery({ queryKey: ["joy", "packs"], queryFn: () => get<Pack[]>("/api/dates/packs") });
  const add = useMutation({
    mutationFn: (pack: string) => post<{ added: number }>("/api/dates/packs/add", { pack }),
    onSuccess: (r) => {
      setNote(`${r.added} slips added to the jar.`);
      void qc.invalidateQueries({ queryKey: ["joy", "packs"] });
      void qc.invalidateQueries({ queryKey: keys.slips });
    },
    onError: () => setNote("Couldn't add that pack. Try again."),
  });
  if (!packs.data) return null;
  return (
    <section className="starter-packs" aria-labelledby="packs-h">
      <h2 id="packs-h">Starter slips</h2>
      <p className="soft">Not sure where to start? Add a pack of ready-made ideas. You can change or delete any of them.</p>
      <ul className="pack-list">
        {packs.data.map((p) => (
          <li key={p.key} className="pack">
            <div>
              <strong>{p.label}</strong>
              <span className="soft"> · {p.total} slips</span>
              <p className="soft">{p.blurb}</p>
            </div>
            {p.added >= p.total ? (
              <span className="pack-done">Added</span>
            ) : (
              <button type="button" className="btn" disabled={add.isPending} onClick={() => add.mutate(p.key)}>
                Add<span className="visually-hidden"> {p.label}</span>
              </button>
            )}
          </li>
        ))}
      </ul>
      {note && <p role="status">{note}</p>}
    </section>
  );
}
