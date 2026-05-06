import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createNativeInitiative, listNativeInitiatives } from "./initiativeApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("initiative API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("lists and creates native initiatives through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([
        {
          id: "init_platform",
          project_id: "proj_local",
          name: "Platform reliability goal",
          description: "Keep platform work visible",
          budget_id: "budget_platform",
          status: "active",
        },
      ])
      .mockResolvedValueOnce({
        id: "init_launch",
        project_id: "proj_local",
        name: "Launch readiness",
        description: null,
        budget_id: null,
        status: "planned",
      });

    const initiatives = await listNativeInitiatives("proj_local");
    const created = await createNativeInitiative({
      projectId: "proj_local",
      name: "Launch readiness",
      status: "planned",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "list_initiatives", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(2, "create_initiative", {
      request: {
        projectId: "proj_local",
        name: "Launch readiness",
        description: undefined,
        budgetId: undefined,
        status: "planned",
      },
    });
    expect(initiatives[0].budget_id).toBe("budget_platform");
    expect(created.id).toBe("init_launch");
  });
});
