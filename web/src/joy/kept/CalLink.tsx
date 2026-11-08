import { useEffect, useRef, type ReactNode } from "react";

/**
 * "Add to calendar". The signed public .ics is fetched ahead (share() must run inside the tap),
 * and where the device can share files it goes to the share sheet so it can reach the other of
 * us; anywhere else it is a plain download.
 */
export function CalLink({ href, className, children }: { href: string; className: string; children: ReactNode }) {
  const file = useRef<File | null>(null);
  useEffect(() => {
    file.current = null;
    if (typeof navigator === "undefined" || typeof navigator.canShare !== "function") return;
    let dead = false;
    fetch(href)
      .then((r) => (r.ok ? r.blob() : null))
      .then((b) => {
        if (!b || dead) return;
        const f = new File([b], "joy-date.ics", { type: "text/calendar" });
        try {
          if (navigator.canShare({ files: [f] })) file.current = f;
        } catch {
          /* download instead */
        }
      })
      .catch(() => {});
    return () => {
      dead = true;
    };
  }, [href]);
  return (
    <a
      className={className}
      href={href}
      download
      onClick={(e) => {
        const f = file.current;
        if (!f || typeof navigator.share !== "function") return;
        e.preventDefault();
        navigator.share({ files: [f], title: "Our date" }).catch((x: unknown) => {
          if (x instanceof Error && x.name === "AbortError") return;
          window.location.href = href;
        });
      }}
    >
      {children}
    </a>
  );
}
