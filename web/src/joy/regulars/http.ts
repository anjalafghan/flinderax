// The Regulars, Year and Month calls use the shared request helpers in ../api.ts (bearer token, 401 -> /auth).
import { get, post, request, upload as uploadWithProgress } from "../api";

export { get, post, request };

/** multipart POST with upload progress (fetch can't report it). `onProgress` gets 0..1. */
export function upload<T>(url: string, form: FormData, onProgress?: (fraction: number) => void): Promise<T> {
  return uploadWithProgress<T>(url, form, {
    onProgress: (loaded, total) => {
      if (total > 0) onProgress?.(loaded / total);
    },
  });
}
