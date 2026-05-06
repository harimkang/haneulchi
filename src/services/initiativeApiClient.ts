import { invoke } from "@tauri-apps/api/core";

export interface NativeInitiativeTokenUsage {
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  cost_usd: number;
}

export interface NativeInitiative {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  budget_id: string | null;
  status: string;
  token_usage?: NativeInitiativeTokenUsage | null;
}

interface CreateNativeInitiativeInput {
  projectId: string;
  name: string;
  description?: string;
  budgetId?: string;
  status?: string;
}

export function listNativeInitiatives(projectId: string): Promise<NativeInitiative[]> {
  return invoke<NativeInitiative[]>("list_initiatives", { projectId });
}

export function createNativeInitiative(input: CreateNativeInitiativeInput): Promise<NativeInitiative> {
  return invoke<NativeInitiative>("create_initiative", {
    request: {
      projectId: input.projectId,
      name: input.name,
      description: input.description,
      budgetId: input.budgetId,
      status: input.status,
    },
  });
}
