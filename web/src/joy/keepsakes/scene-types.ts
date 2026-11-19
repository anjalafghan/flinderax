/** One kept date's own illustrated keepsake, drawn on the Design canvas (see scenes.gen.ts). */
export interface SceneDef {
  /** Scopes its motion in scenes.css: `.ksc-<id>`. */
  id: string;
  /** The slip's title, lower-cased (the API's `slip_key`). */
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
  /** Static SVG markup (our own drawing, never user text). */
  svg: string;
}
