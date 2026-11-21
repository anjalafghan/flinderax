/** One kept date's own illustrated keepsake, drawn on the Design canvas (see scenes.gen.ts and
 *  tools/keepsakes). The drawing itself is a file of its own, fetched when the keepsake is shown. */
export interface SceneDef {
  /** Scopes its motion: `.ksc-<id>`. */
  id: string;
  /** The slip's title, lower-cased (the API's `slip_key`); `{b}` and the like stand for a name. */
  key: string;
  /** Which time we kept that slip: Blue Tokai's first visit is a cup, the second a punch card. */
  visit: number;
  /** What the object is, for a screen reader. */
  name: string;
  /** The crop of the 600 × 600 drawing that holds the object. */
  vb: [number, number, number, number];
  /** What touching it does, and what touching it again does. */
  open: string;
  close: string;
  /** The drawing: /keepsakes/<file>, our own static SVG (never user text), content-hashed. */
  file: string;
}
