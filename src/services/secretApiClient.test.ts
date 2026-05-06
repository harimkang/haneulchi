import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { listNativeSecrets, upsertNativeSecret } from "./secretApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("secret API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("maps Keychain secret upsert and listing through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "secret_proj_local_OPENAI_API_KEY",
        project_id: "proj_local",
        name: "OPENAI_API_KEY",
        keychain_ref: "keychain://proj_local/OPENAI_API_KEY",
        redacted: true,
      })
      .mockResolvedValueOnce([
        {
          id: "secret_proj_local_OPENAI_API_KEY",
          project_id: "proj_local",
          name: "OPENAI_API_KEY",
          keychain_ref: "keychain://proj_local/OPENAI_API_KEY",
          redacted: true,
        },
      ]);

    const secret = await upsertNativeSecret({
      projectId: "proj_local",
      name: "OPENAI_API_KEY",
      value: "sk-hidden",
    });
    const secrets = await listNativeSecrets("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "upsert_secret", {
      request: {
        projectId: "proj_local",
        name: "OPENAI_API_KEY",
        value: "sk-hidden",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_secrets", { projectId: "proj_local" });
    expect(secret.redacted).toBe(true);
    expect(secrets[0].keychain_ref).toBe("keychain://proj_local/OPENAI_API_KEY");
  });
});
