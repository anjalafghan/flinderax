// Bun build for the web app. Output goes to web/dist, which the Rust server serves
// (STATIC_DIR) - there is no separate frontend server, in dev or in production.
//
//   bun build.ts           one-off production build
//   bun build.ts --watch   rebuild on change (dev); refresh the Rust server's page to see it
import { cp, readFile, rm, writeFile } from "node:fs/promises";
import { watch } from "node:fs";
import path from "node:path";
import tailwind from "bun-plugin-tailwind";

const root = import.meta.dir;
const outdir = path.join(root, "dist");
const isWatch = process.argv.includes("--watch");
const siteUrl = (process.env.SITE_URL ?? "").replace(/\/$/, "");

async function build(): Promise<boolean> {
  const started = performance.now();
  await rm(outdir, { recursive: true, force: true });
  const result = await Bun.build({
    entrypoints: [path.join(root, "index.html")],
    outdir,
    target: "browser",
    publicPath: "/",
    minify: !isWatch,
    sourcemap: isWatch ? "inline" : "none",
    splitting: true,
    plugins: [tailwind],
    define: { "process.env.NODE_ENV": JSON.stringify(isWatch ? "development" : "production") },
  });
  if (!result.success) {
    for (const log of result.logs) console.error(log);
    return false;
  }
  await cp(path.join(root, "public"), outdir, { recursive: true });
  const indexPath = path.join(outdir, "index.html");
  await writeFile(indexPath, (await readFile(indexPath, "utf8")).replaceAll("__SITE_URL__", siteUrl));
  console.log(`built web/dist in ${(performance.now() - started).toFixed(0)}ms`);
  return true;
}

const ok = await build();
if (!isWatch) process.exit(ok ? 0 : 1);

let timer: ReturnType<typeof setTimeout> | undefined;
watch(path.join(root, "src"), { recursive: true }, () => {
  clearTimeout(timer);
  timer = setTimeout(() => void build(), 100);
});
watch(path.join(root, "index.html"), () => void build());
console.log("watching web/src ...");
