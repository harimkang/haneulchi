import { invoke } from "@tauri-apps/api/core";

export interface NativePolicyApproval {
  id: string;
  project_id: string;
  task_id: string | null;
  run_id: string | null;
  action_kind: string;
  command: string | null;
  risk_level: string;
  state: "pending" | "approved" | "denied";
  requested_by: string | null;
  decision_by: string | null;
  decision_note: string | null;
  created_at: string;
  decided_at: string | null;
}

export interface NativePolicyPack {
  id: string;
  project_id: string;
  name: string;
  sandbox_mode: string;
  network: string;
  network_profile: string;
  file_write: string;
  tools: string;
  approval_required: string[];
  forbidden_operations: string[];
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface NativePermissionAudit {
  id: string;
  project_id: string;
  task_id: string | null;
  run_id: string | null;
  policy_pack_id: string | null;
  action_kind: string;
  command: string | null;
  decision: "allowed" | "approval_required" | "forbidden";
  reason: string;
  requested_by: string | null;
  created_at: string;
}

export interface NativePolicyActionEvaluation {
  audit_id: string | null;
  project_id: string;
  policy_pack_id: string | null;
  action_kind: string;
  decision: "allowed" | "approval_required" | "forbidden";
  reason: string;
}

interface CreateNativePolicyApprovalInput {
  projectId: string;
  taskId?: string;
  runId?: string;
  actionKind: string;
  command?: string;
  riskLevel: string;
  requestedBy?: string;
}

interface DecideNativePolicyApprovalInput {
  approvalId: string;
  decision: "approved" | "denied";
  decisionBy?: string;
  decisionNote?: string;
}

interface EvaluateNativePolicyActionInput {
  projectId: string;
  taskId?: string;
  runId?: string;
  actionKind: string;
  command?: string;
  requestedBy?: string;
}

interface UpsertNativePolicyPackInput {
  projectId: string;
  name: string;
  sandboxMode: string;
  network: string;
  networkProfile: string;
  fileWrite: string;
  tools: string;
  approvalRequired: string[];
  forbiddenOperations: string[];
  setActive: boolean;
}

interface ListNativePermissionAuditFilters {
  decision?: "allowed" | "approval_required" | "forbidden";
  actionKind?: string;
  runId?: string;
  taskId?: string;
}

export function createNativePolicyApproval(
  input: CreateNativePolicyApprovalInput,
): Promise<NativePolicyApproval> {
  return invoke<NativePolicyApproval>("create_policy_approval", {
    request: {
      projectId: input.projectId,
      taskId: input.taskId,
      runId: input.runId,
      actionKind: input.actionKind,
      command: input.command,
      riskLevel: input.riskLevel,
      requestedBy: input.requestedBy,
    },
  });
}

export function listNativePolicyApprovals(
  projectId: string,
  state?: "pending" | "approved" | "denied",
): Promise<NativePolicyApproval[]> {
  return invoke<NativePolicyApproval[]>("list_policy_approvals", {
    projectId,
    state,
  });
}

export function listNativePolicyPacks(projectId?: string): Promise<NativePolicyPack[]> {
  return invoke<NativePolicyPack[]>("list_policy_packs", {
    projectId,
  });
}

export function listNativePermissionAudits(
  projectId: string,
  filters: ListNativePermissionAuditFilters = {},
): Promise<NativePermissionAudit[]> {
  return invoke<NativePermissionAudit[]>("list_permission_audit", {
    projectId,
    decision: filters.decision,
    actionKind: filters.actionKind,
    runId: filters.runId,
    taskId: filters.taskId,
  });
}

export function decideNativePolicyApproval(
  input: DecideNativePolicyApprovalInput,
): Promise<NativePolicyApproval> {
  return invoke<NativePolicyApproval>("decide_policy_approval", {
    request: {
      approvalId: input.approvalId,
      decision: input.decision,
      decisionBy: input.decisionBy,
      decisionNote: input.decisionNote,
    },
  });
}

export function evaluateNativePolicyAction(
  input: EvaluateNativePolicyActionInput,
): Promise<NativePolicyActionEvaluation> {
  return invoke<NativePolicyActionEvaluation>("evaluate_policy_action", {
    request: {
      projectId: input.projectId,
      taskId: input.taskId,
      runId: input.runId,
      actionKind: input.actionKind,
      command: input.command,
      requestedBy: input.requestedBy,
    },
  });
}

export function upsertNativePolicyPack(input: UpsertNativePolicyPackInput): Promise<NativePolicyPack> {
  return invoke<NativePolicyPack>("upsert_policy_pack", {
    request: {
      projectId: input.projectId,
      name: input.name,
      sandboxMode: input.sandboxMode,
      network: input.network,
      networkProfile: input.networkProfile,
      fileWrite: input.fileWrite,
      tools: input.tools,
      approvalRequired: input.approvalRequired,
      forbiddenOperations: input.forbiddenOperations,
      setActive: input.setActive,
    },
  });
}
