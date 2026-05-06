import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createNativePolicyApproval,
  decideNativePolicyApproval,
  evaluateNativePolicyAction,
  listNativePermissionAudits,
  listNativePolicyPacks,
  listNativePolicyApprovals,
  upsertNativePolicyPack,
} from "./policyApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("policy API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("maps policy approval requests through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "policy_approval_1",
      project_id: "proj_local",
      task_id: "task_1",
      run_id: "run_1",
      action_kind: "shell_command",
      command: "rm -rf build/cache",
      risk_level: "high",
      state: "pending",
      requested_by: "agent_codex",
      decision_by: null,
      decision_note: null,
      created_at: "2026-05-02T00:00:00Z",
      decided_at: null,
    });

    const approval = await createNativePolicyApproval({
      projectId: "proj_local",
      taskId: "task_1",
      runId: "run_1",
      actionKind: "shell_command",
      command: "rm -rf build/cache",
      riskLevel: "high",
      requestedBy: "agent_codex",
    });

    expect(invoke).toHaveBeenCalledWith("create_policy_approval", {
      request: {
        projectId: "proj_local",
        taskId: "task_1",
        runId: "run_1",
        actionKind: "shell_command",
        command: "rm -rf build/cache",
        riskLevel: "high",
        requestedBy: "agent_codex",
      },
    });
    expect(approval.state).toBe("pending");
  });

  it("loads and decides policy approvals through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([{ id: "policy_approval_1", state: "pending" }])
      .mockResolvedValueOnce({ id: "policy_approval_1", state: "approved" });

    const approvals = await listNativePolicyApprovals("proj_local", "pending");
    const decided = await decideNativePolicyApproval({
      approvalId: "policy_approval_1",
      decision: "approved",
      decisionBy: "human",
      decisionNote: "Allowed.",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "list_policy_approvals", {
      projectId: "proj_local",
      state: "pending",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "decide_policy_approval", {
      request: {
        approvalId: "policy_approval_1",
        decision: "approved",
        decisionBy: "human",
        decisionNote: "Allowed.",
      },
    });
    expect(approvals[0].state).toBe("pending");
    expect(decided.state).toBe("approved");
  });

  it("maps policy pack upsert audit listing and action evaluation through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "policy_pack_1", sandbox_mode: "sandboxed" })
      .mockResolvedValueOnce([{ id: "permission_audit_1", decision: "forbidden" }])
      .mockResolvedValueOnce({
        audit_id: "permission_audit_2",
        project_id: "proj_local",
        policy_pack_id: "policy_pack_1",
        action_kind: "network",
        decision: "forbidden",
        reason: "network profile blocks remote endpoint",
      });

    const pack = await upsertNativePolicyPack({
      projectId: "proj_local",
      name: "Local only",
      sandboxMode: "sandboxed",
      network: "allowed",
      networkProfile: "local-only",
      fileWrite: "ask",
      tools: "ask",
      approvalRequired: ["shell_command"],
      forbiddenOperations: ["deploy"],
      setActive: true,
    });
    const audits = await listNativePermissionAudits("proj_local", {
      decision: "forbidden",
      actionKind: "network",
      runId: "run_policy",
      taskId: "task_policy",
    });
    const evaluation = await evaluateNativePolicyAction({
      projectId: "proj_local",
      actionKind: "network",
      command: "curl https://api.example.com",
      requestedBy: "agent_codex",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "upsert_policy_pack", {
      request: {
        projectId: "proj_local",
        name: "Local only",
        sandboxMode: "sandboxed",
        network: "allowed",
        networkProfile: "local-only",
        fileWrite: "ask",
        tools: "ask",
        approvalRequired: ["shell_command"],
        forbiddenOperations: ["deploy"],
        setActive: true,
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_permission_audit", {
      projectId: "proj_local",
      decision: "forbidden",
      actionKind: "network",
      runId: "run_policy",
      taskId: "task_policy",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "evaluate_policy_action", {
      request: {
        projectId: "proj_local",
        taskId: undefined,
        runId: undefined,
        actionKind: "network",
        command: "curl https://api.example.com",
        requestedBy: "agent_codex",
      },
    });
    expect(pack.sandbox_mode).toBe("sandboxed");
    expect(audits[0].decision).toBe("forbidden");
    expect(evaluation.reason).toBe("network profile blocks remote endpoint");
  });

  it("lists policy packs through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        id: "policy_pack_1",
        project_id: "proj_local",
        name: "Ask before write",
        sandbox_mode: "ask-before-write",
        network: "ask",
        network_profile: "internet",
        file_write: "ask",
        tools: "ask",
        approval_required: ["shell_command"],
        forbidden_operations: [],
        active: true,
        created_at: "2026-05-02T00:00:00Z",
        updated_at: "2026-05-02T00:00:00Z",
      },
    ]);

    const packs = await listNativePolicyPacks("proj_local");

    expect(invoke).toHaveBeenCalledWith("list_policy_packs", { projectId: "proj_local" });
    expect(packs[0].name).toBe("Ask before write");
  });
});
