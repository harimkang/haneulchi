import { invoke } from "@tauri-apps/api/core";
import type { StateRunSummary } from "./stateSnapshotClient";

export type RunLifecycle =
  | "queued"
  | "claimed"
  | "starting"
  | "running"
  | "waiting_input"
  | "permission_requested"
  | "blocked"
  | "review_ready"
  | "completed"
  | "failed"
  | "cancelled";

export interface NativeRun {
  id: string;
  task_id: string;
  project_id: string;
  agent_profile_id: string | null;
  session_id: string | null;
  workflow_version_id: string | null;
  context_pack_id: string | null;
  workspace_path: string | null;
  lifecycle: RunLifecycle;
  retry_count: number;
  next_retry_at: string | null;
  status_detail?: string | null;
  budget_id: string | null;
  started_at: string | null;
  ended_at: string | null;
}

export interface NativeRunStatusUpdate {
  id: string;
  task_id: string;
  run_id: string | null;
  author_type: string;
  author_id: string;
  body_md: string;
  parent_id: string | null;
  created_at: string;
}

interface DispatchNativeRunInput {
  taskId: string;
  agentProfileId?: string;
  contextPackId?: string;
  workspacePath?: string;
}

interface RecordNativeRunStatusUpdateInput {
  runId: string;
  bodyMd: string;
  lifecycle?: RunLifecycle;
  statusDetail?: string;
}

export function dispatchNativeRun(input: DispatchNativeRunInput): Promise<NativeRun> {
  return invoke<NativeRun>("dispatch_run", {
    request: {
      taskId: input.taskId,
      agentProfileId: input.agentProfileId,
      contextPackId: input.contextPackId,
      workspacePath: input.workspacePath,
    },
  });
}

export function listNativeRuns(projectId: string): Promise<NativeRun[]> {
  return invoke<NativeRun[]>("list_runs", { projectId });
}

export function updateNativeRunLifecycle(runId: string, lifecycle: RunLifecycle, statusDetail?: string): Promise<NativeRun> {
  const request: { runId: string; lifecycle: RunLifecycle; statusDetail?: string } = {
    runId,
    lifecycle,
  };
  if (statusDetail) {
    request.statusDetail = statusDetail;
  }
  return invoke<NativeRun>("update_run_lifecycle", {
    request,
  });
}

export function cancelNativeRun(runId: string): Promise<NativeRun> {
  return invoke<NativeRun>("cancel_run", { runId });
}

export function retryNativeRun(runId: string): Promise<NativeRun> {
  return invoke<NativeRun>("retry_run", { runId });
}

export function recordNativeRunStatusUpdate(input: RecordNativeRunStatusUpdateInput): Promise<NativeRunStatusUpdate> {
  const request: {
    runId: string;
    bodyMd: string;
    lifecycle?: RunLifecycle;
    statusDetail?: string;
  } = {
    runId: input.runId,
    bodyMd: input.bodyMd,
  };
  if (input.lifecycle) {
    request.lifecycle = input.lifecycle;
  }
  if (input.statusDetail) {
    request.statusDetail = input.statusDetail;
  }
  return invoke<NativeRunStatusUpdate>("record_run_status_update", {
    request,
  });
}

export function nativeRunToStateRunSummary(run: NativeRun): StateRunSummary {
  return {
    id: run.id,
    task_id: run.task_id,
    project_id: run.project_id,
    agent_profile_id: run.agent_profile_id ?? undefined,
    session_id: run.session_id ?? undefined,
    workflow_version_id: run.workflow_version_id ?? undefined,
    lifecycle: run.lifecycle,
    retry_count: run.retry_count,
    next_retry_at: run.next_retry_at ?? undefined,
    status_detail: run.status_detail ?? undefined,
    context_pack_id: run.context_pack_id ?? undefined,
    workspace_path: run.workspace_path ?? undefined,
  };
}
