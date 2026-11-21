// `bun run dev`: the ONLY server is the Rust one. Bun just rebuilds web/dist on change and the
// Rust server serves it from disk; open pages reload themselves after each rebuild.
const web = Bun.spawn(["bun", "run", "--cwd", "web", "dev"], { stdio: ["inherit", "inherit", "inherit"] });
const api = Bun.spawn(["cargo", "run"], { stdio: ["inherit", "inherit", "inherit"] });

const stop = () => {
  web.kill();
  api.kill();
};
process.on("SIGINT", stop);
process.on("SIGTERM", stop);

await Promise.race([web.exited, api.exited]);
stop();
process.exit(1);
