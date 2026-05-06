import { invoke } from "@tauri-apps/api/core";

export interface NativeAgentProfile {
  id: string;
  name: string;
  runtime: string;
  command: string;
  args_json: unknown;
  env_policy_json: unknown;
  skills_json: unknown;
  status: "available" | "unavailable" | "paused";
  last_heartbeat_at: string | null;
}

export interface NativeAgentEvent {
  id: string;
  project_id: string;
  session_id: string | null;
  run_id: string | null;
  agent_profile_id: string;
  kind: string;
  severity: string;
  detail: string;
  payload_json: unknown;
  source: string;
  created_at: string;
}

export interface NativeSkillPack {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  skills_json: unknown;
  source_context_pack_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface NativeRuntimePoolItem {
  id: string;
  label: string;
  session_count: number;
  run_count: number;
  blocked_count: number;
}

export interface UpsertNativeAgentProfileInput {
  id: string;
  name: string;
  runtime: string;
  command: string;
  argsJson?: unknown;
  envPolicyJson?: unknown;
  skillsJson?: unknown;
  status?: "available" | "unavailable" | "paused";
}

export interface UpsertNativeSkillPackInput {
  projectId: string;
  name: string;
  description?: string;
  skillsJson: unknown;
  sourceContextPackId?: string;
}

interface IngestNativeAgentEventsInput {
  projectId: string;
  sessionId?: string;
  runId?: string;
  agentProfileId: string;
  adapter: string;
  payload: unknown;
}

export function listNativeAgentProfiles(): Promise<NativeAgentProfile[]> {
  return invoke<NativeAgentProfile[]>("list_agent_profiles");
}

export function scanNativeAgentProfiles(): Promise<NativeAgentProfile[]> {
  return invoke<NativeAgentProfile[]>("scan_agent_profiles");
}

export function upsertNativeAgentProfile(input: UpsertNativeAgentProfileInput): Promise<NativeAgentProfile> {
  return invoke<NativeAgentProfile>("upsert_agent_profile", {
    request: input,
  });
}

export function updateNativeAgentProfileStatus(
  agentId: string,
  status: "available" | "paused" | "unavailable",
): Promise<NativeAgentProfile> {
  return invoke<NativeAgentProfile>("update_agent_profile_status", {
    agentId,
    status,
  });
}

export function heartbeatNativeAgentProfile(agentId: string): Promise<NativeAgentProfile> {
  return invoke<NativeAgentProfile>("heartbeat_agent_profile", {
    agentId,
  });
}

export function listNativeSkillPacks(projectId: string): Promise<NativeSkillPack[]> {
  return invoke<NativeSkillPack[]>("list_skill_packs", {
    projectId,
  });
}

export function listNativeRuntimePool(projectId: string): Promise<NativeRuntimePoolItem[]> {
  return invoke<NativeRuntimePoolItem[]>("list_runtime_pool", {
    projectId,
  });
}

export function upsertNativeSkillPack(input: UpsertNativeSkillPackInput): Promise<NativeSkillPack> {
  return invoke<NativeSkillPack>("upsert_skill_pack", {
    request: input,
  });
}

export function ingestNativeAgentEvents(input: IngestNativeAgentEventsInput): Promise<NativeAgentEvent> {
  return invoke<NativeAgentEvent>("ingest_agent_events", {
    request: {
      projectId: input.projectId,
      sessionId: input.sessionId,
      runId: input.runId,
      agentProfileId: input.agentProfileId,
      adapter: input.adapter,
      payload: input.payload,
    },
  });
}
