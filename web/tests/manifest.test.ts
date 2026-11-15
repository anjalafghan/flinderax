import { describe, expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const root = path.join(import.meta.dir, "..");
const manifest = () => JSON.parse(readFileSync(path.join(root, "public/manifest.webmanifest"), "utf8"));

describe("web app manifest (installable on a phone's home screen)", () => {
  test("has what browsers need to offer Install", () => {
    const m = manifest();
    expect(m.name).toBe("joy");
    expect(m.display).toBe("standalone");
    expect(m.start_url).toBe("/");
    expect(m.scope).toBe("/");
    const sizes = m.icons.map((i: { sizes: string }) => i.sizes);
    expect(sizes).toContain("192x192");
    expect(sizes).toContain("512x512");
  });

  test("every icon file exists", () => {
    for (const icon of manifest().icons as { src: string }[]) {
      expect(existsSync(path.join(root, "public", icon.src.replace(/^\//, ""))), icon.src).toBe(true);
    }
  });

  test("the page links it and an iOS home-screen icon", () => {
    const html = readFileSync(path.join(root, "index.html"), "utf8");
    expect(html).toContain('rel="manifest" href="/manifest.webmanifest"');
    expect(html).toContain('rel="apple-touch-icon"');
  });

  test("the service worker does not cache the manifest as an app file", () => {
    const sw = readFileSync(path.join(root, "public/sw.js"), "utf8");
    expect(sw).not.toContain("manifest.webmanifest\"");
  });
});
