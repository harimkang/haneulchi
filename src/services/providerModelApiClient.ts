import { invoke } from "@tauri-apps/api/core";

export interface NativeProviderModelSettings {
  provider: string;
  model: string;
  agent_profile_id: string;
}

export interface UpsertNativeProviderModelSettingsInput {
  provider: string;
  model: string;
  agentProfileId: string;
}

export function getNativeProviderModelSettings(): Promise<NativeProviderModelSettings> {
  return invoke<NativeProviderModelSettings>("get_provider_model_settings");
}

export function upsertNativeProviderModelSettings(
  input: UpsertNativeProviderModelSettingsInput,
): Promise<NativeProviderModelSettings> {
  return invoke<NativeProviderModelSettings>("upsert_provider_model_settings", {
    request: input,
  });
}
