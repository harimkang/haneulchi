import { invoke } from "@tauri-apps/api/core";

export interface NativeVisualHarnessLink {
  id: string;
  project_id: string;
  source_id: string;
  target_id: string;
  kind: string;
  created_at: string;
}

interface CreateNativeVisualHarnessLinkInput {
  projectId: string;
  sourceId: string;
  targetId: string;
  kind: string;
}

export function createNativeVisualHarnessLink(
  input: CreateNativeVisualHarnessLinkInput,
): Promise<NativeVisualHarnessLink> {
  return invoke<NativeVisualHarnessLink>("create_visual_harness_link", {
    request: {
      projectId: input.projectId,
      sourceId: input.sourceId,
      targetId: input.targetId,
      kind: input.kind,
    },
  });
}

export function listNativeVisualHarnessLinks(projectId: string): Promise<NativeVisualHarnessLink[]> {
  return invoke<NativeVisualHarnessLink[]>("list_visual_harness_links", { projectId });
}
