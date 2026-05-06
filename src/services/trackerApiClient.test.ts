import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  listNativeExternalTrackerBindings,
  runNativeTrackerSync,
  upsertNativeExternalTrackerBinding,
} from "./trackerApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("tracker API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("maps external tracker bindings through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "tracker_binding_1",
        project_id: "proj_local",
        local_kind: "task",
        local_id: "task_1",
        provider: "linear",
        external_id: "LIN-42",
        external_url: "https://linear.app/acme/issue/LIN-42",
        sync_mode: "mirror",
        sync_status: "pending",
        conflict_state: "none",
        metadata_json: {},
        created_at: "2026-05-03T01:00:00Z",
        updated_at: "2026-05-03T01:00:00Z",
      })
      .mockResolvedValueOnce([]);

    const binding = await upsertNativeExternalTrackerBinding({
      projectId: "proj_local",
      localKind: "task",
      localId: "task_1",
      provider: "linear",
      externalId: "LIN-42",
      externalUrl: "https://linear.app/acme/issue/LIN-42",
      syncMode: "mirror",
    });
    const bindings = await listNativeExternalTrackerBindings("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "upsert_external_tracker_binding", {
      request: {
        projectId: "proj_local",
        localKind: "task",
        localId: "task_1",
        provider: "linear",
        externalId: "LIN-42",
        externalUrl: "https://linear.app/acme/issue/LIN-42",
        syncMode: "mirror",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_external_tracker_bindings", { projectId: "proj_local" });
    expect(binding.id).toBe("tracker_binding_1");
    expect(bindings).toEqual([]);
  });

  it("runs provider tracker sync through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "tracker_sync_1",
      project_id: "proj_local",
      provider: "github",
      dry_run: true,
      status: "planned",
      operation_count: 1,
      degraded_reason: null,
      operations: [{ binding_id: "tracker_binding_1", local_kind: "task", local_id: "task_1", external_id: "octo/repo#123", operation: "issueUpdate", payload: {} }],
      created_at: "2026-05-03T01:00:00Z",
    });

    const run = await runNativeTrackerSync({ projectId: "proj_local", provider: "github", dryRun: true });

    expect(invoke).toHaveBeenCalledWith("run_tracker_sync", {
      provider: "github",
      request: {
        projectId: "proj_local",
        dryRun: true,
      },
    });
    expect(run.status).toBe("planned");
  });
});
