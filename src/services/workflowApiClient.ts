import { invoke } from "@tauri-apps/api/core";

export interface WorkflowDiagnostic {
  code: string;
  message: string;
}

export interface WorkflowDiagnostics {
  errors: WorkflowDiagnostic[];
}

export interface WorkflowRuntimeState {
  valid: boolean;
  current_version_id?: string | null;
  last_known_good_version_id?: string | null;
  diagnostics: WorkflowDiagnostics;
}

export interface WorkflowValidationResult {
  project_id: string;
  source_path: string;
  valid: boolean;
  parsed_json: Record<string, unknown>;
  diagnostics_json: WorkflowDiagnostics;
}

export interface PersistedWorkflowVersion {
  id: string;
  project_id: string;
  source_path: string;
  content_hash: string;
  parsed_json: Record<string, unknown>;
  valid: boolean;
  diagnostics_json: WorkflowDiagnostics;
}

export interface WorkflowHookRunResult {
  run_id: string;
  hook_name: string;
  status: string;
  exit_code: number | null;
  stdout: string;
  stderr: string;
  source_path: string | null;
  mirrored_path: string | null;
  workspace_path: string;
  env_json: Record<string, unknown>;
}

export interface PersistedRunReplayMetadata {
  id: string;
  run_id: string;
  artifact_path: string;
  body_json: Record<string, unknown>;
}

interface WorkflowDocumentInput {
  projectId: string;
  sourcePath: string;
  content: string;
}

interface RunWorkflowHookInput {
  runId: string;
  hookName: string;
  repoRoot: string;
  workspacePath?: string | null;
}

export function getWorkflowRuntimeState(projectId: string): Promise<WorkflowRuntimeState> {
  return invoke<WorkflowRuntimeState>("get_workflow_runtime_state", { projectId });
}

export function validateWorkflow(input: WorkflowDocumentInput): Promise<WorkflowValidationResult> {
  return invoke<WorkflowValidationResult>("validate_workflow", {
    request: {
      projectId: input.projectId,
      sourcePath: input.sourcePath,
      content: input.content,
    },
  });
}

export function reloadWorkflow(input: WorkflowDocumentInput): Promise<PersistedWorkflowVersion> {
  return invoke<PersistedWorkflowVersion>("reload_workflow", {
    request: {
      projectId: input.projectId,
      sourcePath: input.sourcePath,
      content: input.content,
    },
  });
}

export function runWorkflowHook(input: RunWorkflowHookInput): Promise<WorkflowHookRunResult> {
  return invoke<WorkflowHookRunResult>("run_workflow_hook", {
    request: {
      runId: input.runId,
      hookName: input.hookName,
      repoRoot: input.repoRoot,
      workspacePath: input.workspacePath,
    },
  });
}

export function getRunReplayMetadata(runId: string): Promise<PersistedRunReplayMetadata | null> {
  return invoke<PersistedRunReplayMetadata | null>("get_run_replay_metadata", { runId });
}
