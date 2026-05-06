import { invoke } from "@tauri-apps/api/core";

export type NativeTrackerProvider = "linear" | "github" | "plane" | "custom" | "manual";
export type NativeTrackerLocalKind = "task" | "project";
export type NativeTrackerSyncMode = "manual" | "mirror" | "import" | "export";

export interface NativeExternalTrackerBinding {
  id: string;
  project_id: string;
  local_kind: NativeTrackerLocalKind;
  local_id: string;
  provider: NativeTrackerProvider;
  external_id: string;
  external_url: string | null;
  sync_mode: NativeTrackerSyncMode;
  sync_status: string;
  conflict_state: string;
  metadata_json: unknown;
  created_at: string;
  updated_at: string;
}

export interface NativeTrackerSyncOperation {
  binding_id: string;
  local_kind: NativeTrackerLocalKind;
  local_id: string;
  external_id: string;
  operation: string;
  payload: unknown;
}

export interface NativeExternalTrackerSyncRun {
  id: string;
  project_id: string;
  provider: "linear" | "github" | "plane";
  dry_run: boolean;
  status: string;
  operation_count: number;
  degraded_reason: string | null;
  operations: NativeTrackerSyncOperation[];
  created_at: string;
}

interface UpsertNativeExternalTrackerBindingInput {
  projectId: string;
  localKind: NativeTrackerLocalKind;
  localId: string;
  provider: NativeTrackerProvider;
  externalId: string;
  externalUrl?: string;
  syncMode?: NativeTrackerSyncMode;
}

interface RunNativeTrackerSyncInput {
  projectId: string;
  provider: "linear" | "github" | "plane";
  dryRun: boolean;
}

export function upsertNativeExternalTrackerBinding(
  input: UpsertNativeExternalTrackerBindingInput,
): Promise<NativeExternalTrackerBinding> {
  return invoke<NativeExternalTrackerBinding>("upsert_external_tracker_binding", {
    request: {
      projectId: input.projectId,
      localKind: input.localKind,
      localId: input.localId,
      provider: input.provider,
      externalId: input.externalId,
      externalUrl: input.externalUrl,
      syncMode: input.syncMode,
    },
  });
}

export function listNativeExternalTrackerBindings(projectId: string): Promise<NativeExternalTrackerBinding[]> {
  return invoke<NativeExternalTrackerBinding[]>("list_external_tracker_bindings", { projectId });
}

export function runNativeTrackerSync(input: RunNativeTrackerSyncInput): Promise<NativeExternalTrackerSyncRun> {
  return invoke<NativeExternalTrackerSyncRun>("run_tracker_sync", {
    provider: input.provider,
    request: {
      projectId: input.projectId,
      dryRun: input.dryRun,
    },
  });
}
