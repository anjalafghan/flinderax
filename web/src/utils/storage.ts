// localStorage that never throws. Storage can be blocked (private windows, cleared or denied
// site data), and then reads answer null and writes do nothing, so the page still works.

/** The stored text, or null when absent or storage is blocked. */
export function storageGet(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

/** Stores `value`. Returns false when storage refused it. */
export function storageSet(key: string, value: string): boolean {
  try {
    localStorage.setItem(key, value);
    return true;
  } catch {
    return false;
  }
}

/** Removes `key`; a no-op when storage is blocked. */
export function storageRemove(key: string): void {
  try {
    localStorage.removeItem(key);
  } catch {
    // nothing to remove
  }
}
