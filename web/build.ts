// Bun build for the web app. Output goes to web/dist, which the Rust server serves
// (STATIC_DIR) - there is no separate frontend server, in dev or in production.
//
//   bun build.ts           one-off production build (minified, console dropped, .br/.gz copies)
//   bun build.ts --watch   rebuild on change (dev); refresh the Rust server's page to see it
import { copyFile, cp, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { watch } from "node:fs";
import { brotliCompressSync, constants as zlib } from "node:zlib";
import path from "node:path";
import tailwind from "bun-plugin-tailwind";
import type { BunPlugin } from "bun";

const root = import.meta.dir;
const outdir = path.join(root, "dist");
const isWatch = process.argv.includes("--watch");
const siteUrl = (process.env.SITE_URL ?? "").replace(/\/$/, "");

// Dates (joy) stylesheets stay out of the app shell. Bun would otherwise fold every lazy route's
// CSS into index.html's one stylesheet (and write copies nothing links to), so "/" downloaded
// ~270 KB of Dates CSS it never uses. Instead they are bundled once into joy-<hash>.css, and each
// joy module that imports a stylesheet awaits a <link> to that file, so Dates loads it on demand.
const JOY_CSS_ENTRY = [
  "src/joy/styles/index.css", // tokens, motion, jar, slips, kept, month, year, regular, envelope
  "src/joy/styles/app.css",
  "src/joy/jar/jar-scope.css",
  "src/joy/kept/kept.css",
  "src/joy/regulars/regulars.css",
  "src/joy/components/photo-viewer.css",
];
const JOY_CSS_FILTER = /[\\/]src[\\/]joy[\\/].*\.css$/;
const JOY_FONT_DIR = "joy-fonts";
const VIRTUAL_JOY_ENTRY = path.join(root, "src", "joy", "__joy-css-entry__.ts");

/** Dates fonts are served as files, not inlined (Bun base64-inlines small url() assets). */
function joyFontsPlugin(): BunPlugin {
  return {
    name: "joy-fonts",
    setup(build) {
      // Left as-is in the css, copied to dist/joy-fonts by copyReferencedFonts.
      build.onResolve({ filter: /^\/joy-fonts\// }, ({ path: p }) => ({ path: p, external: true }));
      build.onLoad({ filter: JOY_CSS_FILTER }, async ({ path: file }) => ({
        loader: "css",
        contents: (await readFile(file, "utf8")).replace(
          /url\((["']?)(?:\.\.\/)+fonts\/([^"')]+)\1\)/g,
          `url("/${JOY_FONT_DIR}/$2")`,
        ),
      }));
    },
  };
}

/** The shell font (JetBrains Mono, used by index.css) is served as a file, not base64-inlined into the shell css. */
function shellFontsPlugin(): BunPlugin {
  return {
    name: "shell-fonts",
    setup(build) {
      build.onResolve({ filter: /JetBrainsMono\.woff2$/ }, () => ({ path: "/fonts/JetBrainsMono.woff2", external: true }));
    },
  };
}

/** Bundles every Dates stylesheet into one css file in outdir; returns its public path. */
async function buildJoyCss(): Promise<string> {
  const result = await Bun.build({
    entrypoints: [VIRTUAL_JOY_ENTRY],
    outdir,
    target: "browser",
    publicPath: "/",
    minify: !isWatch,
    sourcemap: "none",
    naming: { entry: "joy-[hash].[ext]" },
    plugins: [
      joyFontsPlugin(),
      {
        name: "joy-virtual-entry",
        setup(build) {
          build.onResolve({ filter: /__joy-css-entry__/ }, () => ({ path: VIRTUAL_JOY_ENTRY, namespace: "joy-entry" }));
          build.onLoad({ filter: /.*/, namespace: "joy-entry" }, () => ({
            loader: "js",
            contents: JOY_CSS_ENTRY.map((f) => `import ${JSON.stringify(path.join(root, f))};`).join("\n"),
          }));
        },
      },
    ],
  });
  if (!result.success) throw new AggregateError(result.logs, "joy css build failed");
  const css = result.outputs.find((o) => o.path.endsWith(".css"));
  if (!css) throw new Error("joy css build emitted no stylesheet");
  for (const o of result.outputs) if (o.path.endsWith(".js")) await rm(o.path, { force: true });
  return "/" + path.basename(css.path);
}

/** Font files that stylesheets point at by absolute url, mapped to their source folders. */
const FONT_DIRS: Record<string, string> = {
  [JOY_FONT_DIR]: path.join(root, "src", "joy", "fonts"),
  fonts: path.join(root, "src", "fonts"),
};

/** Copies only the fonts the emitted css really references (unused woff2 files stay out of dist). */
async function copyReferencedFonts(dir: string) {
  const css = (await readdir(dir, { recursive: true })).filter((f) => f.endsWith(".css"));
  for (const f of css) {
    const text = await readFile(path.join(dir, f), "utf8");
    for (const m of text.matchAll(/url\(["']?\/(joy-fonts|fonts)\/([^"')]+)["']?\)/g)) {
      const [, group, name] = m;
      await mkdir(path.join(dir, group!), { recursive: true });
      await copyFile(path.join(FONT_DIRS[group!]!, name!), path.join(dir, group!, name!));
    }
  }
}

/** In the main build, each joy stylesheet import becomes "wait for the joy css file once". */
function joyStylesheetPlugin(href: string): BunPlugin {
  const loader = `globalThis.__joyCss ??= new Promise((done) => { const l = document.createElement("link"); l.rel = "stylesheet"; l.href = ${JSON.stringify(href)}; l.onload = l.onerror = () => done(); document.head.append(l); }); await globalThis.__joyCss;`;
  return {
    name: "joy-stylesheet",
    setup(build) {
      build.onLoad({ filter: JOY_CSS_FILTER }, () => ({ loader: "js", contents: loader }));
    },
  };
}

/** Gzip and brotli copies of every text asset, so the server can send them without compressing. */
async function precompress(dir: string): Promise<number> {
  const files = (await readdir(dir, { recursive: true })).filter((f) => /\.(js|css|html|svg)$/.test(f));
  await Promise.all(
    files.map(async (f) => {
      const file = path.join(dir, f);
      const buf = await readFile(file);
      await writeFile(`${file}.gz`, Bun.gzipSync(buf, { level: 9 }));
      await writeFile(
        `${file}.br`,
        brotliCompressSync(buf, {
          params: { [zlib.BROTLI_PARAM_QUALITY]: 11, [zlib.BROTLI_PARAM_SIZE_HINT]: buf.length },
        }),
      );
    }),
  );
  return files.length;
}

async function build(): Promise<boolean> {
  const started = performance.now();
  await rm(outdir, { recursive: true, force: true });
  const joyHref = await buildJoyCss();
  const result = await Bun.build({
    entrypoints: [path.join(root, "index.html")],
    outdir,
    target: "browser",
    publicPath: "/",
    minify: !isWatch,
    sourcemap: isWatch ? "inline" : "none",
    splitting: true,
    // Production drops console calls (dev keeps them for the watch loop).
    ...(isWatch ? {} : { drop: ["console", "debugger"] }),
    plugins: [joyStylesheetPlugin(joyHref), shellFontsPlugin(), tailwind],
    define: { "process.env.NODE_ENV": JSON.stringify(isWatch ? "development" : "production") },
  });
  if (!result.success) {
    for (const log of result.logs) console.error(log);
    return false;
  }
  await cp(path.join(root, "public"), outdir, { recursive: true });
  await copyReferencedFonts(outdir);
  const indexPath = path.join(outdir, "index.html");
  await writeFile(indexPath, (await readFile(indexPath, "utf8")).replaceAll("__SITE_URL__", siteUrl));
  if (!isWatch) await precompress(outdir);
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
