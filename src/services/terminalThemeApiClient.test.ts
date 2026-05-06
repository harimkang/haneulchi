import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getNativeTerminalThemeSettings,
  upsertNativeTerminalThemeSettings,
} from "./terminalThemeApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("terminal theme API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and persists terminal theme settings through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        name: "Auth Focus",
        background: "#09111f",
        foreground: "#eaf6ff",
        accent: "#19c37d",
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        name: "Auth Focus",
        background: "#09111f",
        foreground: "#eaf6ff",
        accent: "#19c37d",
      });

    const loaded = await getNativeTerminalThemeSettings("proj_auth");
    const saved = await upsertNativeTerminalThemeSettings({
      projectId: "proj_auth",
      name: "Auth Focus",
      background: "#09111f",
      foreground: "#eaf6ff",
      accent: "#19c37d",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "get_terminal_theme_settings", {
      projectId: "proj_auth",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "upsert_terminal_theme_settings", {
      request: {
        projectId: "proj_auth",
        name: "Auth Focus",
        background: "#09111f",
        foreground: "#eaf6ff",
        accent: "#19c37d",
      },
    });
    expect(loaded.name).toBe("Auth Focus");
    expect(saved.project_id).toBe("proj_auth");
  });
});
