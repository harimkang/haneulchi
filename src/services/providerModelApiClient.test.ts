import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getNativeProviderModelSettings,
  upsertNativeProviderModelSettings,
} from "./providerModelApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("provider model API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and persists provider model defaults through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        provider: "openai",
        model: "gpt-5.4",
        agent_profile_id: "agent_codex",
      })
      .mockResolvedValueOnce({
        provider: "anthropic",
        model: "claude-3-7-sonnet-latest",
        agent_profile_id: "agent_claude",
      });

    const loaded = await getNativeProviderModelSettings();
    const saved = await upsertNativeProviderModelSettings({
      provider: "anthropic",
      model: "claude-3-7-sonnet-latest",
      agentProfileId: "agent_claude",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "get_provider_model_settings");
    expect(invoke).toHaveBeenNthCalledWith(2, "upsert_provider_model_settings", {
      request: {
        provider: "anthropic",
        model: "claude-3-7-sonnet-latest",
        agentProfileId: "agent_claude",
      },
    });
    expect(loaded.agent_profile_id).toBe("agent_codex");
    expect(saved.agent_profile_id).toBe("agent_claude");
  });
});
