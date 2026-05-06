import { invoke } from "@tauri-apps/api/core";

export interface NativeSecretMetadata {
  id: string;
  project_id: string;
  name: string;
  keychain_ref: string;
  redacted: boolean;
  created_at: string;
  updated_at: string;
}

interface UpsertNativeSecretInput {
  projectId: string;
  name: string;
  value: string;
}

export function upsertNativeSecret(input: UpsertNativeSecretInput): Promise<NativeSecretMetadata> {
  return invoke<NativeSecretMetadata>("upsert_secret", {
    request: {
      projectId: input.projectId,
      name: input.name,
      value: input.value,
    },
  });
}

export function listNativeSecrets(projectId?: string): Promise<NativeSecretMetadata[]> {
  return invoke<NativeSecretMetadata[]>("list_secrets", { projectId });
}
