// Pick -> shrink -> upload. Kept.js's `shrink`: big or unusual files are redrawn as a JPEG where
// the browser can decode them (iPhone Safari decodes HEIC); anything that fails goes up as it is.
import { JPEG_QUALITY, fitEdge, needsShrink } from "./helpers";

export async function shrinkImage(file: File): Promise<Blob> {
  try {
    const canDecode = typeof createImageBitmap === "function";
    if (!needsShrink(file.size, file.type, canDecode)) return file;
    const bmp = await createImageBitmap(file);
    const { width, height } = fitEdge(bmp.width, bmp.height);
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) return file;
    ctx.drawImage(bmp, 0, 0, width, height);
    bmp.close?.();
    const blob = await new Promise<Blob | null>((res) => canvas.toBlob(res, "image/jpeg", JPEG_QUALITY));
    return blob ?? file;
  } catch {
    return file;
  }
}
