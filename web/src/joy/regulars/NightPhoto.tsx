// Add or remove a night's photo. The picture is shrunk to a JPEG on the phone first, then uploaded with progress.
import { useRef, useState } from "react";
import { useBusy } from "../components/useBusy";
import { deletePhoto, uploadPhoto } from "./api";
import { apiMessage } from "./days";
import { shrinkToJpeg } from "./photo";
import "./regulars.css";

interface Props {
  visitId: number;
  hasPhoto: boolean;
  /** Label for the add control, e.g. "Add a photo" or "Got a photo?" */
  addLabel?: string;
  /** Rendered next to the add control (a spread's "Save" is not needed: choosing a file uploads it). */
  removeLabel?: string;
  onChange: () => void;
}

export function NightPhoto({ visitId, hasPhoto, addLabel = "Add a photo", removeLabel = "Remove photo", onChange }: Props) {
  const { busy, run } = useBusy();
  const [stage, setStage] = useState<"" | "shrink" | "up">("");
  const [pct, setPct] = useState(0);
  const [error, setError] = useState("");
  const input = useRef<HTMLInputElement>(null);

  const pick = (file: File | undefined) =>
    run(async () => {
      if (!file) return;
      setError("");
      try {
        setStage("shrink");
        setPct(0);
        let jpeg: Blob;
        try {
          jpeg = await shrinkToJpeg(file);
        } catch {
          setError("That file doesn’t look like a photo.");
          return;
        }
        setStage("up");
        await uploadPhoto(visitId, jpeg, (f) => setPct(Math.round(f * 100)));
        onChange();
      } catch (e) {
        setError(apiMessage(e, "The photo didn’t go up. Try again."));
      } finally {
        setStage("");
        if (input.current) input.current.value = "";
      }
    });

  const remove = () =>
    run(async () => {
      setError("");
      try {
        await deletePhoto(visitId);
        onChange();
      } catch (e) {
        setError(apiMessage(e));
      }
    });

  return (
    <div className="np">
      {stage ? (
        <div className="np-prog" role="status">
          <span>{stage === "shrink" ? "Getting the photo ready…" : `Uploading… ${pct}%`}</span>
          <progress max={100} value={stage === "up" ? pct : undefined} aria-label="Photo upload" />
        </div>
      ) : hasPhoto ? (
        <button type="button" className="btn btn-quiet" disabled={busy} onClick={() => void remove()}>
          {removeLabel}
        </button>
      ) : (
        <label className="rs-add">
          {addLabel}
          <input ref={input} type="file" accept="image/*" disabled={busy} onChange={(e) => void pick(e.currentTarget.files?.[0])} />
        </label>
      )}
      {error && (
        <p className="np-err" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
