import { expect, test } from "bun:test";
import { nearSlips, resolveArea } from "./near.ts";

const payload = {
  groups: [
    {
      items: [
        { id: 1, title: "NGMA", place: "National Gallery of Modern Art · Fort", cost: "₹150", duration: "1–2 hours" },
        { id: 2, title: "Chowpatty", place: "Girgaon Chowpatty · Girgaon", cost: "Free", duration: "1–2 hours" },
        { id: 3, title: "Versova", place: "Versova beach · Versova", cost: "Free", duration: "1–2 hours" },
        { id: 4, title: "Karaoke", place: "", cost: "Free", duration: "1–2 hours" },
        { id: 5, title: "Pricey", place: "Cafe · Colaba", cost: "₹3,000", duration: "Half a day" },
      ],
    },
  ],
};

test("CSMT resolves and buckets slips", () => {
  expect(resolveArea("I'm at CSMT tomorrow")).toBe("csmt");
  const r = nearSlips(payload, { area: "CSMT" });
  expect(r.near.map((s) => s.id).sort()).toEqual([1, 5]);
  expect(r.short_hop.map((s) => s.id)).toEqual([2]);
  expect(r.anywhere).toEqual([]);
});

test("budget and duration filters, home opt-in", () => {
  const r = nearSlips(payload, { area: "csmt", budget_rupees: 500, max_hours: 3, include_home: true });
  expect(r.near.map((s) => s.id)).toEqual([1]);
  expect(r.anywhere.map((s) => s.id)).toEqual([4]);
});
