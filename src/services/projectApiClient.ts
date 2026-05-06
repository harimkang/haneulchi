import { invoke } from "@tauri-apps/api/core";

export interface NativeProject {
  id: string;
  key: string;
  name: string;
  path: string;
  color: string | null;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface ProjectTabLayout {
  mode: "grid" | "maximized";
  focusedSessionId: string | null;
  maximizedSessionId: string | null;
  panes: string[];
}

export interface NativeProjectTab {
  id: string;
  project_id: string;
  order_index: number;
  active: boolean;
  layout_json: ProjectTabLayout;
}

export interface NativeProjectFileEntry {
  name: string;
  path: string;
  kind: "directory" | "file";
  git_status?: "added" | "modified" | "deleted" | "renamed" | "untracked" | "changed" | null;
}

export interface NativeProjectFileList {
  project_id: string;
  root_path: string;
  relative_path: string;
  degraded_reason?: string | null;
  entries: NativeProjectFileEntry[];
}

export interface NativeProjectFileSearch {
  project_id: string;
  query: string;
  degraded_reason?: string | null;
  entries: NativeProjectFileEntry[];
}

export interface NativeProjectFilePreview {
  project_id: string;
  path: string;
  name: string;
  language?: string | null;
  body: string;
  size_bytes: number;
  truncated: boolean;
}

export interface NativeProjectDiff {
  project_id: string;
  path?: string | null;
  body: string;
  file_count: number;
  files?: NativeProjectDiffFileSummary[];
  truncated: boolean;
  degraded_reason?: string | null;
}

export interface NativeProjectDiffFileSummary {
  path: string;
  status: "added" | "modified" | "deleted" | "renamed" | "changed" | string;
  additions: number;
  deletions: number;
}

export interface NativeProjectLspDiagnostic {
  path: string;
  line: number;
  severity: string;
  message: string;
}

export interface NativeProjectLspSymbol {
  path: string;
  name: string;
  kind: string;
  line: number;
}

export interface NativeProjectLspDiagnostics {
  project_id: string;
  diagnostics: NativeProjectLspDiagnostic[];
  symbols?: NativeProjectLspSymbol[];
  degraded_reason?: string | null;
}

export interface NativePatchArtifact {
  project_id: string;
  patch_id: string;
  body: string;
  file_count: number;
  status: string;
  degraded_reason?: string | null;
}

export interface NativePrLandingPlan {
  project_id: string;
  provider: string;
  title: string;
  draft: boolean;
  checklist: string[];
  degraded_reason?: string | null;
}

export interface NativeReviewPrLandingPlanReceipt {
  review_id: string;
  evidence_pack_id: string;
  source_task_id: string | null;
  source_run_id: string | null;
  plan: NativePrLandingPlan;
}

export interface NativeProjectDetachPlan {
  project_id: string;
  project_name: string;
  window_id: string;
  status: string;
  degraded_reason?: string | null;
}

export interface NativeProjectTabGroup {
  project_id: string;
  group_name: string;
  created_at?: string;
  updated_at: string;
}

export interface NativeProjectLayoutPreset {
  id: string;
  project_id: string;
  name: string;
  layout_json: ProjectTabLayout;
  created_at?: string;
  updated_at?: string;
}

interface AddNativeProjectInput {
  key: string;
  name: string;
  path: string;
  color?: string;
}

export function listNativeProjects(): Promise<NativeProject[]> {
  return invoke<NativeProject[]>("list_projects");
}

export function addNativeProject(input: AddNativeProjectInput): Promise<NativeProject> {
  return invoke<NativeProject>("add_project", {
    request: {
      key: input.key,
      name: input.name,
      path: input.path,
      color: input.color,
    },
  });
}

export function focusNativeProject(projectId: string): Promise<NativeProject> {
  return invoke<NativeProject>("focus_project", {
    projectId,
  });
}

export function updateNativeProjectLayout(projectId: string, layoutJson: ProjectTabLayout): Promise<NativeProjectTab> {
  return invoke<NativeProjectTab>("update_project_tab_layout", {
    request: {
      projectId,
      layoutJson,
    },
  });
}

export function saveNativeProjectLayoutPreset(
  projectId: string,
  name: string,
  layoutJson: ProjectTabLayout,
): Promise<NativeProjectLayoutPreset> {
  return invoke<NativeProjectLayoutPreset>("save_project_layout_preset", {
    request: {
      projectId,
      name,
      layoutJson,
    },
  });
}

export function listNativeProjectLayoutPresets(projectId: string): Promise<NativeProjectLayoutPreset[]> {
  return invoke<NativeProjectLayoutPreset[]>("list_project_layout_presets", {
    projectId,
  });
}

export function listNativeProjectFiles(projectId: string, relativePath?: string): Promise<NativeProjectFileList> {
  return invoke<NativeProjectFileList>("list_project_files", {
    request: {
      projectId,
      relativePath,
    },
  });
}

export function readNativeProjectFile(projectId: string, path: string): Promise<NativeProjectFilePreview> {
  return invoke<NativeProjectFilePreview>("read_project_file", {
    request: {
      projectId,
      path,
    },
  });
}

export function saveNativeProjectFile(projectId: string, path: string, body: string): Promise<NativeProjectFilePreview> {
  return invoke<NativeProjectFilePreview>("write_project_file", {
    request: {
      projectId,
      path,
      body,
    },
  });
}

export function readNativeProjectDiff(projectId: string, path?: string): Promise<NativeProjectDiff> {
  return invoke<NativeProjectDiff>("read_project_diff", {
    request: {
      projectId,
      path,
    },
  });
}

export function collectNativeProjectLspDiagnostics(projectId: string, path?: string): Promise<NativeProjectLspDiagnostics> {
  return invoke<NativeProjectLspDiagnostics>("collect_project_lsp_diagnostics", {
    request: {
      projectId,
      path,
    },
  });
}

export function exportNativeProjectPatch(projectId: string, path?: string): Promise<NativePatchArtifact> {
  return invoke<NativePatchArtifact>("export_project_patch", {
    request: {
      projectId,
      path,
    },
  });
}

export function importNativeProjectPatch(projectId: string, body: string): Promise<NativePatchArtifact> {
  return invoke<NativePatchArtifact>("import_project_patch", {
    request: {
      projectId,
      body,
    },
  });
}

export function planNativePrLanding(input: { projectId: string; title: string; draft: boolean }): Promise<NativePrLandingPlan> {
  return invoke<NativePrLandingPlan>("plan_pr_landing", {
    request: {
      projectId: input.projectId,
      title: input.title,
      draft: input.draft,
    },
  });
}

export function planNativeReviewPrLanding(input: { reviewId: string; title?: string; draft?: boolean }): Promise<NativeReviewPrLandingPlanReceipt> {
  return invoke<NativeReviewPrLandingPlanReceipt>("plan_review_pr_landing", {
    request: {
      reviewId: input.reviewId,
      title: input.title,
      draft: input.draft,
    },
  });
}

export function planNativeProjectDetach(projectId: string): Promise<NativeProjectDetachPlan> {
  return invoke<NativeProjectDetachPlan>("plan_project_detach", { projectId });
}

export function upsertNativeProjectTabGroup(projectId: string, groupName: string): Promise<NativeProjectTabGroup> {
  return invoke<NativeProjectTabGroup>("upsert_project_tab_group", {
    projectId,
    groupName,
  });
}

export function searchNativeProjectFiles(projectId: string, query: string): Promise<NativeProjectFileSearch> {
  return invoke<NativeProjectFileSearch>("search_project_files", {
    request: {
      projectId,
      query,
    },
  });
}
