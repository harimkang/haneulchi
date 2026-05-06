import { invoke } from "@tauri-apps/api/core";

export interface NativeTerminalThemeSettings {
  project_id: string | null;
  name: string;
  background: string;
  foreground: string;
  accent: string;
}

export interface UpsertNativeTerminalThemeSettingsInput {
  projectId?: string;
  name: string;
  background: string;
  foreground: string;
  accent: string;
}

export function getNativeTerminalThemeSettings(projectId?: string): Promise<NativeTerminalThemeSettings> {
  return invoke<NativeTerminalThemeSettings>("get_terminal_theme_settings", { projectId });
}

export function upsertNativeTerminalThemeSettings(
  input: UpsertNativeTerminalThemeSettingsInput,
): Promise<NativeTerminalThemeSettings> {
  return invoke<NativeTerminalThemeSettings>("upsert_terminal_theme_settings", {
    request: input,
  });
}
