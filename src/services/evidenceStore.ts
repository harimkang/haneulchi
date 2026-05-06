import { createEvidencePack, type EvidencePack } from "../domain/evidence";

const storagePrefix = "haneulchi:evidence-pack:";

export function loadEvidencePack(id: string): EvidencePack {
  if (typeof window === "undefined" || !hasStorageApi()) {
    return createEvidencePack(id);
  }

  try {
    const raw = window.localStorage.getItem(storageKey(id));
    if (!raw) return createEvidencePack(id);

    const parsed = JSON.parse(raw) as EvidencePack;
    if (!isEvidencePack(parsed, id)) return createEvidencePack(id);

    return parsed;
  } catch {
    return createEvidencePack(id);
  }
}

export function saveEvidencePack(pack: EvidencePack): void {
  if (typeof window === "undefined" || !hasStorageApi()) return;
  try {
    window.localStorage.setItem(storageKey(pack.id), JSON.stringify(pack));
  } catch {
    // Persistence is best-effort in the foundation build.
  }
}

function storageKey(id: string): string {
  return `${storagePrefix}${id}`;
}

function hasStorageApi(): boolean {
  return (
    typeof window.localStorage?.getItem === "function" &&
    typeof window.localStorage?.setItem === "function"
  );
}

function isEvidencePack(value: EvidencePack, id: string): boolean {
  return (
    value !== null &&
    typeof value === "object" &&
    value.id === id &&
    Array.isArray(value.commandBlocks) &&
    Array.isArray(value.contextSources) &&
    Array.isArray(value.tests) &&
    Array.isArray(value.screenshots) &&
    Array.isArray(value.policyEvents)
  );
}
