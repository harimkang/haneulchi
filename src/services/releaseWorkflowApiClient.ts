import { invoke } from "@tauri-apps/api/core";

export interface NativeReleaseWorkflowDiagnostic {
  id: string;
  label: string;
  script: string;
  configured: boolean;
  required_env: string[];
  missing_env: string[];
  detail: string;
}

export interface NativeReleaseWorkflowStatus {
  status: string;
  workflows: NativeReleaseWorkflowDiagnostic[];
}

export function getNativeReleaseWorkflowStatus(): Promise<NativeReleaseWorkflowStatus> {
  return invoke<NativeReleaseWorkflowStatus>("get_release_workflow_status");
}
