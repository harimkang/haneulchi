import { beforeEach, describe, expect, it } from "vitest";
import { attachCommandBlockToEvidencePack, createEvidencePack } from "../domain/evidence";
import { loadEvidencePack, saveEvidencePack } from "./evidenceStore";

describe("evidence store", () => {
  const storage = new Map<string, string>();

  beforeEach(() => {
    storage.clear();
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: {
        getItem: (key: string) => storage.get(key) ?? null,
        setItem: (key: string, value: string) => storage.set(key, value),
        clear: () => storage.clear(),
      },
    });
  });

  it("loads a fallback evidence pack when storage is empty", () => {
    expect(loadEvidencePack("ev_local")).toEqual(createEvidencePack("ev_local"));
  });

  it("saves and reloads evidence pack command block attachments", () => {
    const pack = attachCommandBlockToEvidencePack(createEvidencePack("ev_local"), {
      id: "cmdblk_1",
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 1,
      seqEnd: 2,
      status: "running",
      outputExcerpt: "PASS",
    });

    saveEvidencePack(pack);

    expect(loadEvidencePack("ev_local")).toEqual(pack);
  });

  it("falls back safely when stored data is invalid", () => {
    window.localStorage.setItem("haneulchi:evidence-pack:ev_local", "{bad json");

    expect(loadEvidencePack("ev_local")).toEqual(createEvidencePack("ev_local"));
  });
});
