import { useState } from "react";
import { DatabaseBackup, Download, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import api from "@/services/api";

const CARD =
  "rounded-[28px] border border-teal/30 bg-gradient-to-br from-teal/10 via-card to-card p-6 stack shadow-[0_16px_40px_-28px_hsl(var(--c-teal)/0.8)]";
const msg = (e: unknown) =>
  e instanceof Error && e.message
    ? e.message
    : "Something went wrong. Please try again.";

/** Download everything you own as one file, and load such a file back in. */
export function BackupSection() {
  const [file, setFile] = useState<{ name: string; body: unknown } | null>(
    null,
  );
  const [error, setError] = useState("");
  const [done, setDone] = useState("");
  const [busy, setBusy] = useState(false);

  const download = async () => {
    setError("");
    try {
      const { data } = await api.get<unknown>("/api/export");
      const url = URL.createObjectURL(
        new Blob([JSON.stringify(data, null, 1)], { type: "application/json" }),
      );
      const a = document.createElement("a");
      a.href = url;
      a.download = `joy-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(msg(e));
    }
  };

  const choose = async (f: File | undefined) => {
    setError("");
    setDone("");
    setFile(null);
    if (!f) return;
    try {
      setFile({ name: f.name, body: JSON.parse(await f.text()) });
    } catch {
      setError("That file isn't a joy backup (it isn't readable as JSON).");
    }
  };

  const restore = async () => {
    if (!file) return;
    setBusy(true);
    setError("");
    try {
      const { data } = await api.post<{ rows: number }>(
        "/api/import",
        file.body,
      );
      setDone(`Restored ${data.rows} rows. Reload the page to see them.`);
      setFile(null);
    } catch (e) {
      setError(msg(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={CARD}>
      <h2 className="flex items-center gap-3 font-display text-2xl">
        <span
          aria-hidden
          className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-teal/18 text-teal"
        >
          <DatabaseBackup className="h-5 w-5" strokeWidth={1.8} />
        </span>
        Your data
      </h2>
      <p className="text-sm text-muted-foreground">
        Download a backup of your money and people data. Dates data is shared
        with your partner and is kept as it is.
      </p>
      <div>
        <Button
          type="button"
          variant="outline"
          className="border-teal/40 text-teal hover:bg-teal/10"
          onClick={() => void download()}
        >
          <Download className="mr-2 h-4 w-4" aria-hidden />
          Download backup
        </Button>
      </div>
      <div className="stack">
        <label htmlFor="backup-file" className="text-sm font-medium">
          Backup file
        </label>
        <input
          id="backup-file"
          type="file"
          accept="application/json,.json"
          onChange={(e) => void choose(e.target.files?.[0])}
          className="text-sm file:mr-3 file:min-h-11 file:cursor-pointer file:rounded-full file:border-0 file:bg-teal/15 file:px-4 file:font-medium file:text-teal"
        />
        {file && (
          <div className="stack rounded-[18px] border border-sun/40 bg-sun/10 p-4">
            <p className="text-sm">
              This will replace your money and people data with what is in{" "}
              <strong>{file.name}</strong>.
            </p>
            <div>
              <Button
                type="button"
                disabled={busy}
                onClick={() => void restore()}
              >
                <Upload className="mr-2 h-4 w-4" aria-hidden />
                Restore from this file
              </Button>
            </div>
          </div>
        )}
        {done && (
          <p role="status" className="text-sm font-medium text-good">
            {done}
          </p>
        )}
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
      </div>
    </div>
  );
}
