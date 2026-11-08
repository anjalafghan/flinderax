// Shrinking a night's photo before it goes up. The sizing maths is pure; the canvas part is not.

export const MAX_SIDE = 1600;
export const JPEG_QUALITY = 0.85;

/** Fit w x h inside a square of `max`, never scaling up. */
export function fitWithin(w: number, h: number, max = MAX_SIDE): { width: number; height: number } {
  if (!(w > 0) || !(h > 0)) return { width: 0, height: 0 };
  const k = Math.min(1, max / Math.max(w, h));
  return { width: Math.max(1, Math.round(w * k)), height: Math.max(1, Math.round(h * k)) };
}

/** Decode, scale to fit and re-encode as JPEG. Throws when the file isn't a picture. */
export async function shrinkToJpeg(file: File): Promise<Blob> {
  let bmp: ImageBitmap;
  try {
    bmp = await createImageBitmap(file, { imageOrientation: "from-image" });
  } catch {
    bmp = await createImageBitmap(file);
  }
  try {
    const { width, height } = fitWithin(bmp.width, bmp.height);
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("canvas");
    ctx.drawImage(bmp, 0, 0, width, height);
    const blob = await new Promise<Blob | null>((ok) => canvas.toBlob(ok, "image/jpeg", JPEG_QUALITY));
    canvas.width = canvas.height = 0;
    if (!blob) throw new Error("encode");
    return blob;
  } finally {
    bmp.close();
  }
}
