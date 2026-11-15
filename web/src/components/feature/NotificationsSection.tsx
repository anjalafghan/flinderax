import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import api from "@/services/api";
import { Button } from "@/components/ui/button";
import { urlBase64ToBytes } from "@/utils/push";

const CARD = "rounded-[28px] border border-border bg-card p-6 stack";

const supported = () => typeof window !== "undefined" && "serviceWorker" in navigator && "PushManager" in window && typeof Notification !== "undefined";

/** A nudge in the morning (today) and the evening (tomorrow), with counts only: no names, no amounts. */
export function NotificationsSection() {
  const qc = useQueryClient();
  const status = useQuery({ queryKey: ["push"], queryFn: async () => (await api.get<{ public_key: string | null; subscribed: boolean }>("/api/push")).data, retry: 1 });
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState("");

  if (!status.data) return null;
  const refresh = () => qc.invalidateQueries({ queryKey: ["push"] });

  const turnOn = async () => {
    setBusy(true);
    setNote("");
    try {
      const permission = await Notification.requestPermission();
      if (permission !== "granted") {
        setNote("Notifications are blocked for this site. Allow them in your browser's settings, then try again.");
        return;
      }
      const reg = await navigator.serviceWorker.ready;
      const sub = (await reg.pushManager.getSubscription()) ?? (await reg.pushManager.subscribe({ userVisibleOnly: true, applicationServerKey: urlBase64ToBytes(status.data!.public_key!) as BufferSource }));
      const json = sub.toJSON() as { endpoint?: string; keys?: { p256dh?: string; auth?: string } };
      await api.post("/api/push/subscribe", { endpoint: json.endpoint, p256dh: json.keys?.p256dh, auth: json.keys?.auth });
      await refresh();
    } catch (e) {
      setNote(e instanceof Error ? e.message : "Couldn't turn notifications on.");
    } finally {
      setBusy(false);
    }
  };

  const turnOff = async () => {
    setBusy(true);
    setNote("");
    try {
      const reg = await navigator.serviceWorker.ready;
      const sub = await reg.pushManager.getSubscription();
      if (sub) {
        await api.post("/api/push/unsubscribe", { endpoint: sub.endpoint });
        await sub.unsubscribe();
      }
      await refresh();
    } catch (e) {
      setNote(e instanceof Error ? e.message : "Couldn't turn notifications off.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={CARD}>
      <h2 className="font-display text-2xl">Notifications</h2>
      <p className="text-sm text-muted-foreground">A nudge each morning about today and each evening about tomorrow: bills, dates, birthdays and follow-ups. It only counts them; it never shows names or amounts.</p>
      {!status.data.public_key ? (
        <p className="text-sm">Notifications are not set up on this server yet.</p>
      ) : !supported() ? (
        <p className="text-sm">This browser can't show notifications. On an iPhone, add joy to your Home Screen first, then open it from there.</p>
      ) : (
        <div>
          {status.data.subscribed ? (
            <Button type="button" variant="outline" disabled={busy} onClick={() => void turnOff()}>
              Turn off notifications
            </Button>
          ) : (
            <Button type="button" disabled={busy} onClick={() => void turnOn()}>
              Turn on notifications
            </Button>
          )}
        </div>
      )}
      {note && (
        <p role="alert" className="text-sm text-destructive">
          {note}
        </p>
      )}
    </div>
  );
}
