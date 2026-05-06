import { invoke } from "@tauri-apps/api/core";
import type { StateBudgetForecast, StateBudgetSummary } from "./stateSnapshotClient";

export interface NativeBudgetSummary {
  workspace: StateBudgetSummary;
  projects: StateBudgetSummary[];
  goals?: StateBudgetSummary[];
  tasks?: StateBudgetSummary[];
  runs?: StateBudgetSummary[];
  agents: StateBudgetSummary[];
}

export interface NativeBudgetForecast {
  workspace?: StateBudgetForecast;
  projects: StateBudgetForecast[];
  goals?: StateBudgetForecast[];
  tasks?: StateBudgetForecast[];
  runs?: StateBudgetForecast[];
  agents: StateBudgetForecast[];
}

export interface NativeBudget {
  id: string;
  scope_type: string;
  scope_id: string | null;
  max_usd: number;
  warn_pct: number;
  hard_limit: boolean;
}

export interface NativeTokenUsage {
  id: string;
  project_id: string | null;
  session_id: string | null;
  task_id: string | null;
  run_id: string | null;
  agent_profile_id: string | null;
  provider: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
  source: string;
}

export interface NativeProviderPrice {
  provider: string;
  model: string;
  input_usd_per_million: number;
  output_usd_per_million: number;
  source: string;
  updated_at: string;
}

export interface NativeProviderPriceUpdateSummary {
  source: string;
  updated: number;
}

interface IngestNativeTokenUsageAdapterInput {
  projectId?: string;
  sessionId?: string;
  taskId?: string;
  runId?: string;
  agentProfileId?: string;
  adapter: string;
  payload: unknown;
}

interface UpsertNativeBudgetInput {
  scopeType: "workspace" | "project" | "goal" | "task" | "run" | "agent";
  scopeId?: string;
  maxUsd: number;
  warnPct: number;
  hardLimit: boolean;
}

interface NativeProviderPriceInput {
  provider: string;
  model: string;
  inputUsdPerMillion: number;
  outputUsdPerMillion: number;
}

interface UpdateNativeProviderPriceTableInput {
  source: string;
  prices: NativeProviderPriceInput[];
}

interface RecordNativeTokenUsageInput {
  projectId?: string;
  sessionId?: string;
  taskId?: string;
  runId?: string;
  agentProfileId?: string;
  provider: string;
  model: string;
  inputTokens: number;
  outputTokens: number;
  costUsd: number;
  source: string;
}

export function getNativeBudgetSummary(): Promise<NativeBudgetSummary> {
  return invoke<NativeBudgetSummary>("get_budget_summary");
}

export function getNativeBudgetForecast(): Promise<NativeBudgetForecast> {
  return invoke<NativeBudgetForecast>("get_budget_forecast");
}

export function listNativeProviderPrices(): Promise<NativeProviderPrice[]> {
  return invoke<NativeProviderPrice[]>("list_provider_prices");
}

export function updateNativeProviderPriceTable(
  input: UpdateNativeProviderPriceTableInput,
): Promise<NativeProviderPriceUpdateSummary> {
  return invoke<NativeProviderPriceUpdateSummary>("update_provider_price_table", {
    request: {
      source: input.source,
      prices: input.prices,
    },
  });
}

export function upsertNativeBudget(input: UpsertNativeBudgetInput): Promise<NativeBudget> {
  return invoke<NativeBudget>("upsert_budget", {
    request: {
      scopeType: input.scopeType,
      scopeId: input.scopeId,
      maxUsd: input.maxUsd,
      warnPct: input.warnPct,
      hardLimit: input.hardLimit,
    },
  });
}

export function recordNativeTokenUsage(input: RecordNativeTokenUsageInput): Promise<NativeTokenUsage> {
  return invoke<NativeTokenUsage>("record_token_usage", {
    request: {
      projectId: input.projectId,
      sessionId: input.sessionId,
      taskId: input.taskId,
      runId: input.runId,
      agentProfileId: input.agentProfileId,
      provider: input.provider,
      model: input.model,
      inputTokens: input.inputTokens,
      outputTokens: input.outputTokens,
      costUsd: input.costUsd,
      source: input.source,
    },
  });
}

export function ingestNativeTokenUsageAdapter(input: IngestNativeTokenUsageAdapterInput): Promise<NativeTokenUsage> {
  return invoke<NativeTokenUsage>("ingest_token_usage_adapter", {
    request: {
      projectId: input.projectId,
      sessionId: input.sessionId,
      taskId: input.taskId,
      runId: input.runId,
      agentProfileId: input.agentProfileId,
      adapter: input.adapter,
      payload: input.payload,
    },
  });
}
