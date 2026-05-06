import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getNativeReleaseWorkflowStatus } from "./releaseWorkflowApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("release workflow API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads macOS distribution, Homebrew, and crash reporting workflow diagnostics through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      status: "warning",
      workflows: [
        {
          id: "macos_dmg",
          label: "Signed macOS DMG",
          script: "release:macos:dmg",
          configured: true,
          required_env: ["APPLE_CERTIFICATE", "APPLE_CERTIFICATE_PASSWORD", "APPLE_SIGNING_IDENTITY", "KEYCHAIN_PASSWORD"],
          missing_env: ["APPLE_CERTIFICATE"],
          detail: ".github/workflows/release-macos.yml configured",
        },
        {
          id: "macos_notarization",
          label: "Apple notarization",
          script: "release:macos:notarize",
          configured: true,
          required_env: ["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"],
          missing_env: ["APPLE_ID"],
          detail: "scripts/release/notarize-macos.sh configured",
        },
        {
          id: "macos_artifact_verification",
          label: "macOS artifact verification",
          script: "release:macos:verify",
          configured: true,
          required_env: [],
          missing_env: [],
          detail: "scripts/release/verify-macos-artifacts.sh configured",
        },
        {
          id: "homebrew_cask",
          label: "Homebrew cask",
          script: "release:homebrew:cask",
          configured: true,
          required_env: ["DMG_URL", "HOMEBREW_TAP_REPOSITORY"],
          missing_env: ["DMG_URL"],
          detail: "scripts/release/render-homebrew-cask.sh configured",
        },
        {
          id: "crash_symbols",
          label: "Crash symbols",
          script: "release:symbols:upload",
          configured: true,
          required_env: ["SENTRY_AUTH_TOKEN", "SENTRY_ORG", "SENTRY_PROJECT"],
          missing_env: ["SENTRY_AUTH_TOKEN"],
          detail: "scripts/release/upload-symbols.sh configured",
        },
      ],
    });

    const status = await getNativeReleaseWorkflowStatus();

    expect(invoke).toHaveBeenCalledWith("get_release_workflow_status");
    expect(status.status).toBe("warning");
    expect(status.workflows.map((workflow) => workflow.id)).toEqual([
      "macos_dmg",
      "macos_notarization",
      "macos_artifact_verification",
      "homebrew_cask",
      "crash_symbols",
    ]);
    expect(status.workflows[0].missing_env).toEqual(["APPLE_CERTIFICATE"]);
  });
});
