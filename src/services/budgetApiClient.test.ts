import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getNativeBudgetForecast,
  getNativeBudgetSummary,
  ingestNativeTokenUsageAdapter,
  listNativeProviderPrices,
  recordNativeTokenUsage,
  updateNativeProviderPriceTable,
  upsertNativeBudget,
} from "./budgetApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("budget API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads native budget summaries through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      workspace: { used_usd: 8.5, state: "warn" },
      projects: [{ scope_id: "proj_local", state: "warn" }],
      tasks: [{ scope_id: "task_budget", state: "warn" }],
      runs: [{ scope_id: "run_budget", state: "warn" }],
      agents: [],
    });

    const summary = await getNativeBudgetSummary();

    expect(invoke).toHaveBeenCalledWith("get_budget_summary");
    expect(summary.projects[0].state).toBe("warn");
    expect(summary.tasks?.[0].scope_id).toBe("task_budget");
    expect(summary.runs?.[0].scope_id).toBe("run_budget");
  });

  it("loads native budget forecasts through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      workspace: {},
      projects: [
        {
          scope_type: "project",
          scope_id: "proj_local",
          average_run_cost_usd: 6,
          estimated_runs_remaining: 1,
          remaining_usd: 8,
        },
      ],
      tasks: [
        {
          scope_type: "task",
          scope_id: "task_budget",
          average_run_cost_usd: 2,
          estimated_runs_remaining: 1,
          remaining_usd: 2,
        },
      ],
      runs: [
        {
          scope_type: "run",
          scope_id: "run_budget",
          average_run_cost_usd: 1,
          estimated_runs_remaining: 1,
          remaining_usd: 1,
        },
      ],
      agents: [],
    });

    const forecast = await getNativeBudgetForecast();

    expect(invoke).toHaveBeenCalledWith("get_budget_forecast");
    expect(forecast.projects[0].scope_id).toBe("proj_local");
    expect(forecast.tasks?.[0].scope_id).toBe("task_budget");
    expect(forecast.runs?.[0].scope_id).toBe("run_budget");
  });

  it("lists and updates provider price tables through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([
        {
          provider: "openai",
          model: "gpt-5.4",
          input_usd_per_million: 5,
          output_usd_per_million: 15,
          source: "local-fixture",
          updated_at: "2026-05-02T01:00:00Z",
        },
      ])
      .mockResolvedValueOnce({ source: "auto-fixture", updated: 1 });

    const prices = await listNativeProviderPrices();
    const update = await updateNativeProviderPriceTable({
      source: "auto-fixture",
      prices: [
        {
          provider: "openai",
          model: "gpt-5.4",
          inputUsdPerMillion: 5,
          outputUsdPerMillion: 15,
        },
      ],
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "list_provider_prices");
    expect(invoke).toHaveBeenNthCalledWith(2, "update_provider_price_table", {
      request: {
        source: "auto-fixture",
        prices: [
          {
            provider: "openai",
            model: "gpt-5.4",
            inputUsdPerMillion: 5,
            outputUsdPerMillion: 15,
          },
        ],
      },
    });
    expect(prices[0].source).toBe("local-fixture");
    expect(update.updated).toBe(1);
  });

  it("maps budget upserts through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "budget_goal_init_platform",
      scope_type: "goal",
      scope_id: "init_platform",
      max_usd: 10,
      warn_pct: 0.8,
      hard_limit: true,
    });

    const budget = await upsertNativeBudget({
      scopeType: "goal",
      scopeId: "init_platform",
      maxUsd: 10,
      warnPct: 0.8,
      hardLimit: true,
    });

    expect(invoke).toHaveBeenCalledWith("upsert_budget", {
      request: {
        scopeType: "goal",
        scopeId: "init_platform",
        maxUsd: 10,
        warnPct: 0.8,
        hardLimit: true,
      },
    });
    expect(budget.id).toBe("budget_goal_init_platform");
  });

  it("maps token usage records through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "usage_1",
      project_id: "proj_local",
      session_id: "pty_1",
      task_id: "task_1",
      run_id: "run_1",
      agent_profile_id: "agent_codex",
      provider: "openai",
      model: "gpt-5.4",
      input_tokens: 1200,
      output_tokens: 800,
      cost_usd: 8.5,
      source: "adapter",
    });

    const usage = await recordNativeTokenUsage({
      projectId: "proj_local",
      sessionId: "pty_1",
      taskId: "task_1",
      runId: "run_1",
      agentProfileId: "agent_codex",
      provider: "openai",
      model: "gpt-5.4",
      inputTokens: 1200,
      outputTokens: 800,
      costUsd: 8.5,
      source: "adapter",
    });

    expect(invoke).toHaveBeenCalledWith("record_token_usage", {
      request: {
        projectId: "proj_local",
        sessionId: "pty_1",
        taskId: "task_1",
        runId: "run_1",
        agentProfileId: "agent_codex",
        provider: "openai",
        model: "gpt-5.4",
        inputTokens: 1200,
        outputTokens: 800,
        costUsd: 8.5,
        source: "adapter",
      },
    });
    expect(usage.cost_usd).toBe(8.5);
  });

  it("maps token usage adapter ingestion through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "usage_1",
      project_id: "proj_local",
      session_id: null,
      task_id: null,
      run_id: null,
      agent_profile_id: "agent_codex",
      provider: "openai",
      model: "gpt-5.4",
      input_tokens: 1200,
      output_tokens: 800,
      cost_usd: 8.5,
      source: "adapter:openai.responses",
    });

    const usage = await ingestNativeTokenUsageAdapter({
      projectId: "proj_local",
      agentProfileId: "agent_codex",
      adapter: "openai.responses",
      payload: {
        model: "gpt-5.4",
        usage: { input_tokens: 1200, output_tokens: 800 },
        cost_usd: 8.5,
      },
    });

    expect(invoke).toHaveBeenCalledWith("ingest_token_usage_adapter", {
      request: {
        projectId: "proj_local",
        sessionId: undefined,
        taskId: undefined,
        runId: undefined,
        agentProfileId: "agent_codex",
        adapter: "openai.responses",
        payload: {
          model: "gpt-5.4",
          usage: { input_tokens: 1200, output_tokens: 800 },
          cost_usd: 8.5,
        },
      },
    });
    expect(usage.source).toBe("adapter:openai.responses");
  });
});
