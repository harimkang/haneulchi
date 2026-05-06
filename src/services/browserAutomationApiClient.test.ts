import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { planNativeBrowserAutomation } from "./browserAutomationApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("browser automation API client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("plans browser automation runs through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      url: "http://localhost:3000/docs",
      scenario: "smoke",
      status: "planned",
      steps: ["open http://localhost:3000/docs"],
      degraded_reason: null,
    });

    const plan = await planNativeBrowserAutomation({
      projectId: "proj_auth",
      url: "http://localhost:3000/docs",
      scenario: "smoke",
    });

    expect(invoke).toHaveBeenCalledWith("plan_browser_automation", {
      request: {
        projectId: "proj_auth",
        url: "http://localhost:3000/docs",
        scenario: "smoke",
      },
    });
    expect(plan.status).toBe("planned");
  });
});
