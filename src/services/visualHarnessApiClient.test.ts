import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createNativeVisualHarnessLink, listNativeVisualHarnessLinks } from "./visualHarnessApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("visual harness API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("creates and lists visual harness links through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "visual_link_1",
        project_id: "proj_local",
        source_id: "ctx_default",
        target_id: "task_1",
        kind: "context",
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([]);

    const link = await createNativeVisualHarnessLink({
      projectId: "proj_local",
      sourceId: "ctx_default",
      targetId: "task_1",
      kind: "context",
    });
    const links = await listNativeVisualHarnessLinks("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "create_visual_harness_link", {
      request: {
        projectId: "proj_local",
        sourceId: "ctx_default",
        targetId: "task_1",
        kind: "context",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_visual_harness_links", { projectId: "proj_local" });
    expect(link.id).toBe("visual_link_1");
    expect(links).toEqual([]);
  });
});
