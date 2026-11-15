import type { ScenarioDraft } from "@/utils/scenario";

/**
 * First answer from the server vs what this browser already has. What-if drafts used to live only in
 * the browser, so a person's old ones are uploaded once instead of being lost; after that the server wins.
 */
export function reconcile(server: ScenarioDraft[], local: ScenarioDraft[]): { drafts: ScenarioDraft[]; push: boolean } {
  if (server.length > 0) return { drafts: server, push: false };
  return { drafts: local, push: local.length > 0 };
}
