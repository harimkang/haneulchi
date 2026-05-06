import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getRunReplayMetadata,
  getWorkflowRuntimeState,
  reloadWorkflow,
  runWorkflowHook,
  validateWorkflow,
} from "./workflowApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("workflow API client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("invokes workflow status and reload commands", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ valid: true, current_version_id: "workflow_1" })
      .mockResolvedValueOnce({ id: "workflow_2", valid: false });

    await getWorkflowRuntimeState("proj_local");
    await reloadWorkflow({
      projectId: "proj_local",
      sourcePath: "WORKFLOW.md",
      content: "---\nhaneulchi: 1\n---\nUse {task.id}.",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "get_workflow_runtime_state", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(2, "reload_workflow", {
      request: {
        projectId: "proj_local",
        sourcePath: "WORKFLOW.md",
        content: "---\nhaneulchi: 1\n---\nUse {task.id}.",
      },
    });
  });

  it("invokes workflow validation hook runner and replay metadata commands", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ valid: false, diagnostics_json: { errors: [{ code: "frontmatter_missing" }] } })
      .mockResolvedValueOnce({
        run_id: "run_1",
        hook_name: "before_run",
        status: "completed",
        exit_code: 0,
        stdout: "ok",
        stderr: "",
        source_path: "/repo/.haneulchi/hooks/before_run.sh",
        mirrored_path: "/repo/.haneulchi/worktrees/run_1/.haneulchi/hooks/before_run.sh",
        workspace_path: "/repo/.haneulchi/worktrees/run_1",
        env_json: { HANEULCHI_RUN_ID: "run_1" },
      })
      .mockResolvedValueOnce({
        id: "replay_1",
        run_id: "run_1",
        artifact_path: "artifacts/runs/run_1/replay.json",
        body_json: { run_id: "run_1" },
      });

    await validateWorkflow({
      projectId: "proj_local",
      sourcePath: "WORKFLOW.md",
      content: "Use {secret.token}",
    });
    await runWorkflowHook({
      runId: "run_1",
      hookName: "before_run",
      repoRoot: "/repo",
      workspacePath: "/repo/.haneulchi/worktrees/run_1",
    });
    await getRunReplayMetadata("run_1");

    expect(invoke).toHaveBeenNthCalledWith(1, "validate_workflow", {
      request: {
        projectId: "proj_local",
        sourcePath: "WORKFLOW.md",
        content: "Use {secret.token}",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "run_workflow_hook", {
      request: {
        runId: "run_1",
        hookName: "before_run",
        repoRoot: "/repo",
        workspacePath: "/repo/.haneulchi/worktrees/run_1",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "get_run_replay_metadata", { runId: "run_1" });
  });
});
