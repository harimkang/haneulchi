import { invoke } from "@tauri-apps/api/core";

export interface NativeBrowserAutomationPlan {
  project_id: string;
  url: string;
  scenario: string;
  status: string;
  steps: string[];
  degraded_reason?: string | null;
}

interface PlanNativeBrowserAutomationInput {
  projectId: string;
  url: string;
  scenario?: string;
}

export function planNativeBrowserAutomation(input: PlanNativeBrowserAutomationInput): Promise<NativeBrowserAutomationPlan> {
  return invoke<NativeBrowserAutomationPlan>("plan_browser_automation", {
    request: {
      projectId: input.projectId,
      url: input.url,
      scenario: input.scenario,
    },
  });
}
