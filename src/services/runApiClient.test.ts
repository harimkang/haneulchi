import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  cancelNativeRun,
  dispatchNativeRun,
  listNativeRuns,
  nativeRunToStateRunSummary,
  recordNativeRunStatusUpdate,
  retryNativeRun,
  updateNativeRunLifecycle,
} from "./runApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("run API client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("records run status updates through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "comment_1",
      task_id: "task_1",
      run_id: "run_1",
      author_type: "agent",
      author_id: "agent_codex",
      body_md: "Investigating OAuth fixture failure.",
      parent_id: null,
      created_at: "2026-04-30T01:00:00Z",
    });

    const comment = await recordNativeRunStatusUpdate({
      runId: "run_1",
      bodyMd: "Investigating OAuth fixture failure.",
      lifecycle: "waiting_input",
      statusDetail: "Needs OAuth test account",
    });

    expect(invoke).toHaveBeenCalledWith("record_run_status_update", {
      request: {
        runId: "run_1",
        bodyMd: "Investigating OAuth fixture failure.",
        lifecycle: "waiting_input",
        statusDetail: "Needs OAuth test account",
      },
    });
    expect(comment.body_md).toBe("Investigating OAuth fixture failure.");
  });

  it("invokes native run lifecycle commands", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "run_1", lifecycle: "queued" })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({ id: "run_1", lifecycle: "review_ready" })
      .mockResolvedValueOnce({ id: "run_1", lifecycle: "cancelled" })
      .mockResolvedValueOnce({ id: "run_1", lifecycle: "queued", retry_count: 1 });

    await dispatchNativeRun({ taskId: "task_1", agentProfileId: "agent_codex", contextPackId: "ctx_default" });
    await listNativeRuns("proj_local");
    await updateNativeRunLifecycle("run_1", "review_ready");
    await cancelNativeRun("run_1");
    await retryNativeRun("run_1");

    expect(invoke).toHaveBeenNthCalledWith(1, "dispatch_run", {
      request: { taskId: "task_1", agentProfileId: "agent_codex", contextPackId: "ctx_default" },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_runs", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(3, "update_run_lifecycle", {
      request: { runId: "run_1", lifecycle: "review_ready" },
    });
    expect(invoke).toHaveBeenNthCalledWith(4, "cancel_run", { runId: "run_1" });
    expect(invoke).toHaveBeenNthCalledWith(5, "retry_run", { runId: "run_1" });
  });

  it("maps native runs into state snapshot summaries", () => {
    expect(
      nativeRunToStateRunSummary({
        id: "run_1",
        task_id: "task_1",
        project_id: "proj_local",
        agent_profile_id: "agent_codex",
        session_id: null,
        workflow_version_id: "workflow_1",
        context_pack_id: "ctx_default",
        workspace_path: "/repo/.haneulchi/worktrees/run_1",
        lifecycle: "queued",
        retry_count: 1,
        next_retry_at: "2026-04-30T01:00:10Z",
        status_detail: "Needs OAuth test account",
        budget_id: null,
        started_at: null,
        ended_at: null,
      }),
    ).toEqual({
      id: "run_1",
      task_id: "task_1",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      workflow_version_id: "workflow_1",
      lifecycle: "queued",
      retry_count: 1,
      next_retry_at: "2026-04-30T01:00:10Z",
      status_detail: "Needs OAuth test account",
      context_pack_id: "ctx_default",
      workspace_path: "/repo/.haneulchi/worktrees/run_1",
    });
  });
});
