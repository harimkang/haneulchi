import {
  Activity,
  AlertTriangle,
  Bell,
  Blocks,
  CalendarDays,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  CircleDot,
  Clock3,
  Code2,
  Clipboard,
  File,
  FileText,
  Folder,
  GitBranch,
  GitPullRequest,
  KeyRound,
  LayoutGrid,
  ListTodo,
  LocateFixed,
  Maximize2,
  Minimize2,
  PackageCheck,
  Plus,
  RotateCcw,
  StepForward,
  Search,
  ShieldAlert,
  SquarePlus,
  Terminal,
  XCircle,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { TerminalPane } from "./components/TerminalPane";
import { MonacoCodeEditor, MonacoDiffEditor } from "./components/MonacoCodeEditor";
import { CodeMirrorMarkdownEditor } from "./components/CodeMirrorMarkdownEditor";
import { HC_DEFAULT_PROJECT_COLOR, HC_DEFAULT_TERMINAL_THEME } from "./design/haneulchiDesignTokens";
import {
  buildCommandBlockExplanation,
  createCommandBlockState,
  exportCommandBlockBundle,
  formatCommandBlockForClipboard,
  ingestCommandInput,
  ingestCommandOutput,
  listCommandBlocks,
  mergeCommandBlockSummaries,
  mergeCommandBlocks,
  searchCommandBlocks,
  splitCommandBlock,
  updateCommandBlockStatus,
  type CommandBlock,
  type CommandBlockState,
  type CommandBlockExplanation,
} from "./domain/commandBlocks";
import { attachCommandBlockToEvidencePack, type EvidencePack } from "./domain/evidence";
import { createOscEventState, parseOscSequences, type OscEventState } from "./domain/oscEvents";
import { fallbackReadinessSnapshot, type ReadinessSnapshot, type ReadinessStatus } from "./domain/readiness";
import {
  addTask,
  addTaskComment,
  addTaskSubtask,
  advanceTask,
  countTasksByStatus,
  filterTasks,
  getTaskOverview,
  moveTask,
  taskBoardStatuses,
  updateTaskContextPack,
  updateTaskSubtaskStatus,
  updateTaskWorkpad,
  updateTaskPlanningProperties,
  type HaneulchiTask,
  type TaskStatus,
  type TaskState,
} from "./domain/tasks";
import { appendTerminalOutput, bindPtySession, createTerminalSession, markRendererDegraded, terminalSessions, type TerminalSession } from "./domain/terminal";
import { createTerminalTransportState, ingestTerminalPtyOutput } from "./domain/terminalTransport";
import {
  fallbackTerminalPtySnapshot,
  captureTerminalPtyCommand,
  closeTerminalPtySession,
  getTerminalPtySnapshot,
  listenToTerminalPtyOutput,
  resizeTerminalPtySession,
  spawnTerminalPtySession,
  type PtyCommandCapture,
  type SpawnTerminalPtyRequest,
  type TerminalPtySnapshot,
  writeTerminalPtyInput,
} from "./services/terminalPtyClient";
import { loadEvidencePack, saveEvidencePack } from "./services/evidenceStore";
import {
  attachNativeCommandBlockToEvidence,
  explainNativeCommandBlock,
  exportNativeCommandBlockBundle,
  generateNativeEvidencePackForRun,
  markNativeCommandBlock,
  mergeNativeCommandBlocks,
  nativeCommandBlockToCommandBlock,
  recordNativeEvidenceReviewDecision,
  searchNativeCommandBlocks,
  splitNativeCommandBlock,
  upsertNativeCommandBlock,
  type NativeCommandBlockExplanation,
  type NativeEvidencePack,
} from "./services/commandBlockApiClient";
import {
  addNativeTaskComment,
  addNativeTaskSubtask,
  createNativeTaskCycle,
  createNativeTaskModule,
  createNativeReviewFollowUpTask,
  createNativeTask,
  listNativeTaskComments,
  listNativeTaskCycles,
  listNativeTaskModules,
  listNativeTaskSubtasks,
  loadNativeTaskState,
  moveNativeTask,
  nativeTaskToHaneulchiTask,
  saveNativeTaskWorkpad,
  updateNativeTaskContext,
  updateNativeTaskPlanning,
  updateNativeTaskSubtaskStatus,
  type NativeTaskCycle,
  type NativeTaskModule,
} from "./services/taskApiClient";
import { createNativeInitiative, listNativeInitiatives, type NativeInitiative } from "./services/initiativeApiClient";
import { loadTaskState, saveTaskState } from "./services/taskStore";
import {
  cancelNativeRun,
  dispatchNativeRun,
  listNativeRuns,
  nativeRunToStateRunSummary,
  recordNativeRunStatusUpdate,
  retryNativeRun,
  updateNativeRunLifecycle,
  type NativeRun,
  type RunLifecycle,
} from "./services/runApiClient";
import {
  fallbackStateSnapshot,
  getStateSnapshot,
  mergeCommandBlocksIntoStateSnapshot,
  mergeTasksIntoStateSnapshot,
  type StateAttentionItem,
  type StateAgent,
  type StateBenchmarkSuite,
  type StateBudgetForecast,
  type StateBudgetSummary,
  type StateInitiative,
  type StateProject,
  type StateProjectTab,
  type StateSnapshot,
  type StateTaskSummary,
  type StateReview,
  type StateRunSummary,
  type StateSession,
  type StateTokenUsageSummary,
  type StateVisualHarness,
} from "./services/stateSnapshotClient";
import {
  getRunReplayMetadata,
  getWorkflowRuntimeState,
  reloadWorkflow,
  runWorkflowHook,
  validateWorkflow,
  type PersistedRunReplayMetadata,
  type PersistedWorkflowVersion,
  type WorkflowHookRunResult,
  type WorkflowRuntimeState,
  type WorkflowValidationResult,
} from "./services/workflowApiClient";
import {
  answerNativeKnowledgeQuestion,
  exportNativeKnowledgeObsidianMarkdown,
  ingestNativeKnowledgeArtifact,
  listNativeKnowledgeConcepts,
  listNativeKnowledgeExplorations,
  listNativeKnowledgeSources,
  listNativeContextPacks,
  recordNativeKnowledgeLintReport,
  runNativeKnowledgeAutomation,
  saveNativeKnowledgeExploration,
  saveNativeKnowledgePage,
  searchNativeKnowledgePages,
  upsertNativeContextPack,
  upsertNativeKnowledgeSource,
  type NativeContextPack,
  type NativeKnowledgeChatAnswer,
  type NativeKnowledgeConcept,
  type NativeKnowledgeExploration,
  type NativeKnowledgeLintReport,
  type NativeKnowledgeObsidianExport,
  type NativeKnowledgeAutomationRun,
  type NativeKnowledgeIngestionResult,
  type NativeKnowledgePage,
  type NativeKnowledgeSource,
} from "./services/knowledgeApiClient";
import {
  getNativeBudgetSummary,
  getNativeBudgetForecast,
  ingestNativeTokenUsageAdapter,
  listNativeProviderPrices,
  recordNativeTokenUsage,
  updateNativeProviderPriceTable,
  upsertNativeBudget,
  type NativeBudget,
  type NativeBudgetForecast,
  type NativeBudgetSummary,
  type NativeProviderPrice,
  type NativeProviderPriceUpdateSummary,
  type NativeTokenUsage,
} from "./services/budgetApiClient";
import { planNativeBrowserAutomation, type NativeBrowserAutomationPlan } from "./services/browserAutomationApiClient";
import {
  getNativeReleaseWorkflowStatus,
  type NativeReleaseWorkflowStatus,
} from "./services/releaseWorkflowApiClient";
import {
  listNativeBenchmarkRuns,
  listNativeDmgSmokeRuns,
  listNativeDogfoodTelemetryReviews,
  listNativeRecoveryDrillRuns,
  listNativeReleaseGateRuns,
  listNativeTaskLifecycleE2ERuns,
  listNativeTerminalFidelitySmokeRuns,
  listNativeWorkflowNegativeTestRuns,
  runNativeBenchmarks,
  runNativeDmgSmokeTest,
  runNativeDogfoodTelemetryReview,
  runNativeRecoveryDrills,
  runNativeReleaseGates,
  runNativeTaskLifecycleE2E,
  runNativeTerminalFidelitySmoke,
  runNativeWorkflowNegativeTests,
  type NativeBenchmarkRun,
  type NativeDmgSmokeRun,
  type NativeDogfoodTelemetryReview,
  type NativeRecoveryDrillRun,
  type NativeReleaseGateRun,
  type NativeTaskLifecycleE2ERun,
  type NativeTerminalFidelitySmokeRun,
  type NativeWorkflowNegativeTestRun,
} from "./services/qualityApiClient";
import {
  createNativeVisualHarnessLink,
  listNativeVisualHarnessLinks,
  type NativeVisualHarnessLink,
} from "./services/visualHarnessApiClient";
import {
  listNativeExternalTrackerBindings,
  runNativeTrackerSync,
  upsertNativeExternalTrackerBinding,
  type NativeExternalTrackerBinding,
  type NativeExternalTrackerSyncRun,
  type NativeTrackerLocalKind,
  type NativeTrackerProvider,
  type NativeTrackerSyncMode,
} from "./services/trackerApiClient";
import { listNativeSecrets, upsertNativeSecret, type NativeSecretMetadata } from "./services/secretApiClient";
import {
  createNativePolicyApproval,
  decideNativePolicyApproval,
  evaluateNativePolicyAction,
  listNativePolicyApprovals,
  listNativePolicyPacks,
  listNativePermissionAudits,
  upsertNativePolicyPack,
  type NativePermissionAudit,
  type NativePolicyApproval,
  type NativePolicyActionEvaluation,
  type NativePolicyPack,
} from "./services/policyApiClient";
import {
  heartbeatNativeAgentProfile,
  ingestNativeAgentEvents,
  listNativeAgentProfiles,
  listNativeRuntimePool,
  listNativeSkillPacks,
  scanNativeAgentProfiles,
  updateNativeAgentProfileStatus,
  upsertNativeAgentProfile,
  upsertNativeSkillPack,
  type NativeAgentEvent,
  type NativeRuntimePoolItem,
  type NativeSkillPack,
} from "./services/agentApiClient";
import { getNativeProviderModelSettings, upsertNativeProviderModelSettings } from "./services/providerModelApiClient";
import {
  getNativeTerminalThemeSettings,
  upsertNativeTerminalThemeSettings,
  type NativeTerminalThemeSettings,
} from "./services/terminalThemeApiClient";
import {
  attachNativeSessionTask,
  createNativeSession,
  detachNativeSessionTask,
  focusNativeSession,
  killNativeSession,
  launchNativeAgentTerminal,
  listNativeSessions,
  listNativeTerminalStreamChunks,
  recordNativeTerminalStreamChunk,
  recordNativeSessionInput,
  releaseNativeSession,
  takeoverNativeSession,
  type NativeSession,
  type NativeTerminalStreamChunk,
} from "./services/sessionApiClient";
import {
  addNativeProject,
  focusNativeProject,
  listNativeProjectLayoutPresets,
  listNativeProjects,
  listNativeProjectFiles,
  collectNativeProjectLspDiagnostics,
  exportNativeProjectPatch,
  importNativeProjectPatch,
  planNativeProjectDetach,
  planNativePrLanding,
  planNativeReviewPrLanding,
  readNativeProjectDiff,
  readNativeProjectFile,
  saveNativeProjectFile,
  saveNativeProjectLayoutPreset,
  searchNativeProjectFiles,
  updateNativeProjectLayout,
  upsertNativeProjectTabGroup,
  type NativeProjectDiff,
  type NativeProjectFileList,
  type NativeProjectFilePreview,
  type NativeProjectFileSearch,
  type NativeProjectLspDiagnostics,
  type NativePatchArtifact,
  type NativeProjectDetachPlan,
  type NativeProjectLayoutPreset,
  type NativePrLandingPlan,
  type ProjectTabLayout,
} from "./services/projectApiClient";
import "./App.css";

const fallbackWorkspaceTabs = ["Control Tower", "Terminal Deck", "Grid Split", "Board", "Review Queue 3", "Explorer", "Git", "Preview", "Inspector"];

const fallbackProjects = [
  { name: "haneulchi", branch: "main", state: "Active", sessions: 4 },
  { name: "reference: dot-studio", branch: "analysis", state: "Idle", sessions: 1 },
  { name: "docs workspace", branch: "v10-pack", state: "Review", sessions: 2 },
];
const localProjectId = "proj_local";
const localAgentProfileId = "agent_codex";
const defaultContextPackId = "ctx_default";
const defaultDashboardWidgets = {
  agentTeam: true,
  historicalAnalytics: true,
  recentEvidence: true,
};
type BudgetScopeType = "workspace" | "project" | "goal" | "task" | "run" | "agent";
const workflowDebuggerSteps = [
  "Load workflow",
  "Resolve context",
  "Prepare workspace",
  "Run hooks",
  "Launch agent",
  "Generate evidence",
];
const sampleWorkflowDocument = `---
haneulchi: 1
project:
  key: LOCAL
  default_branch: main
workspace:
  strategy: worktree
  base_root: .haneulchi/worktrees
hooks:
  before_run: .haneulchi/hooks/before_run.sh
---

Use {task.id} in {project.name}.
`;
const workflowMarketplacePresets = [
  {
    id: "harness-default",
    name: "Harness Default",
    description: "Worktree agent run with before hook and evidence requirements",
    sourcePath: "marketplace:harness-default/WORKFLOW.md",
    content: `---
haneulchi: 1
project:
  key: LOCAL
  default_branch: main
workspace:
  strategy: worktree
  base_root: .haneulchi/worktrees
context:
  default_pack: ctx_default
hooks:
  before_run: .haneulchi/hooks/before_run.sh
review:
  required_evidence:
    - diff
    - tests
    - transcript
---

Run {task.id} in {project.name}. Attach command blocks and evidence before review.
`,
  },
];

const taskStatusLabels: Record<(typeof taskBoardStatuses)[number], string> = {
  inbox: "Inbox",
  ready: "Ready",
  running: "Running",
  review: "Review",
  blocked: "Blocked",
  done: "Done",
};

const runLifecycleSummaryStatuses = [
  "queued",
  "claimed",
  "starting",
  "running",
  "waiting_input",
  "permission_requested",
  "blocked",
  "review_ready",
  "completed",
  "failed",
  "cancelled",
] as const;

function taskStatusLabel(status: TaskStatus): string {
  return status === "archived" ? "Archived" : taskStatusLabels[status];
}

function formatTaskRowMetadata(task: HaneulchiTask): string {
  return [taskStatusLabel(task.status), task.priority, task.cycle, task.module, ...(task.labels ?? []), task.assignee].filter(Boolean).join(" · ");
}

function parseTaskLabelsDraft(value: string): string[] | undefined {
  const labels = Array.from(new Set(value.split(",").map((label) => label.trim()).filter(Boolean)));
  return labels.length > 0 ? labels : undefined;
}

function statusLabel(status: ReadinessStatus) {
  if (status === "ready") return "Ready";
  if (status === "warning") return "Warning";
  return "Missing";
}

function taskStatusForRunLifecycle(lifecycle: string): TaskStatus | undefined {
  if (["queued", "claimed", "starting", "running", "waiting_input", "permission_requested"].includes(lifecycle)) return "running";
  if (["blocked", "failed", "cancelled"].includes(lifecycle)) return "blocked";
  if (lifecycle === "review_ready") return "review";
  if (lifecycle === "completed") return "done";
  return undefined;
}

function formatLifecycleLabel(lifecycle: string): string {
  return lifecycle.replace(/_/g, " ");
}

function normalizeAttentionSeverity(severity: string | undefined): GlobalAttentionQueueItem["severity"] {
  if (severity === "critical" || severity === "high" || severity === "missing") return "critical";
  if (severity === "warning") return "warning";
  return "info";
}

function attentionIconStatus(severity: GlobalAttentionQueueItem["severity"]) {
  if (severity === "critical") return "missing";
  if (severity === "warning") return "warning";
  return "ready";
}

function runAttentionSeverity(lifecycle: string): GlobalAttentionQueueItem["severity"] | undefined {
  if (["blocked", "failed", "permission_requested"].includes(lifecycle)) return "critical";
  if (["waiting_input", "queued", "starting"].includes(lifecycle)) return "warning";
  return undefined;
}

function budgetAttentionItem(scopeType: string, budget: StateBudgetSummary | undefined): GlobalAttentionQueueItem | undefined {
  if (!budget?.state || !["warn", "exceeded"].includes(budget.state)) return undefined;
  const scopeLabel = budget.scope_id ? `${scopeType} ${budget.scope_id}` : scopeType;
  const used = budget.used_usd ?? 0;
  const percent = typeof budget.max_usd === "number" && budget.max_usd > 0
    ? ` · ${Math.round((used / budget.max_usd) * 100)}%`
    : "";
  const max = typeof budget.max_usd === "number" ? ` of ${formatUsd(budget.max_usd)}` : "";
  return {
    id: `budget:${scopeType}:${budget.scope_id ?? scopeType}`,
    label: `Budget ${budget.state === "exceeded" ? "exceeded" : "warning"}: ${scopeLabel}`,
    detail: `${formatUsd(used)}${max} used${percent}`,
    severity: budget.state === "exceeded" ? "critical" : "warning",
  };
}

function buildBudgetAttentionQueue(snapshot: StateSnapshot): GlobalAttentionQueueItem[] {
  const candidates = [
    budgetAttentionItem("workspace", snapshot.budgets.workspace),
    ...snapshot.budgets.projects.map((budget) => budgetAttentionItem("project", budget)),
    ...(snapshot.budgets.goals ?? []).map((budget) => budgetAttentionItem("goal", budget)),
    ...(snapshot.budgets.tasks ?? []).map((budget) => budgetAttentionItem("task", budget)),
    ...(snapshot.budgets.runs ?? []).map((budget) => budgetAttentionItem("run", budget)),
    ...snapshot.budgets.agents.map((budget) => budgetAttentionItem("agent", budget)),
  ];
  return candidates.filter((item): item is GlobalAttentionQueueItem => Boolean(item));
}

function buildGlobalAttentionQueue(snapshot: StateSnapshot): GlobalAttentionQueueItem[] {
  const snapshotAttention = snapshot.attention.map((item) => ({
    id: `attention:${item.id}`,
    label: item.label,
    detail: item.detail,
    severity: normalizeAttentionSeverity(item.severity),
  }));
  const sessionAttention = snapshot.sessions
    .filter((session) => (session.attention_state ?? "none") !== "none")
    .map((session) => ({
      id: `session:${session.id}`,
      label: `Session ${session.title} ${session.attention_state}`,
      detail: `${session.project_id ?? "workspace"} · ${session.mode}`,
      severity: "warning" as const,
    }));
  const runAttention = snapshot.runs.items.flatMap((run) => {
    const severity = runAttentionSeverity(run.lifecycle);
    if (!severity) return [];
    return [{
      id: `run:${run.id}`,
      label: `Run ${run.id} ${formatLifecycleLabel(run.lifecycle)}`,
      detail: run.status_detail ?? `${run.task_id} · ${run.project_id}`,
      severity,
    }];
  });
  const agentAttention = snapshot.agents
    .filter((agent) => (agent.attention_state ?? "none") !== "none")
    .map((agent) => ({
      id: `agent:${agent.id}`,
      label: `Agent ${agent.label} ${agent.attention_state}`,
      detail: `${agent.notification_count ?? 1} notifications · ${agent.latest_event_detail ?? agent.id}`,
      severity: normalizeAttentionSeverity(agent.attention_severity),
    }));
  const budgetAttentionAlreadyPresent = snapshot.attention.some((item) => item.id.startsWith("budget_"));
  const budgetAttention = budgetAttentionAlreadyPresent ? [] : buildBudgetAttentionQueue(snapshot);
  const severityRank = { critical: 0, warning: 1, info: 2 } satisfies Record<GlobalAttentionQueueItem["severity"], number>;
  return [...snapshotAttention, ...sessionAttention, ...runAttention, ...agentAttention, ...budgetAttention].sort((a, b) => severityRank[a.severity] - severityRank[b.severity]);
}

function upsertRunSummary(snapshot: StateSnapshot, run: StateRunSummary): StateSnapshot {
  const items = snapshot.runs.items.some((item) => item.id === run.id)
    ? snapshot.runs.items.map((item) => (item.id === run.id ? run : item))
    : [...snapshot.runs.items, run];
  const counts = items.reduce<Record<string, number>>((nextCounts, item) => {
    nextCounts[item.lifecycle] = (nextCounts[item.lifecycle] ?? 0) + 1;
    return nextCounts;
  }, {});

  return {
    ...snapshot,
    runs: {
      items,
      counts_by_lifecycle: counts,
    },
  };
}

function formatBudgetSummary(budget: StateBudgetSummary | undefined): string {
  if (!budget) return "Budget workspace · unknown";
  const label = budget.scope_id ?? budget.scope_type ?? "workspace";
  const state = budget.state ?? "unknown";
  const used = formatUsd(budget.used_usd);
  const max = typeof budget.max_usd === "number" ? ` / ${formatUsd(budget.max_usd)}` : "";
  return `Budget ${label} · ${state} · ${used}${max}`;
}

function budgetUsagePercent(budget: StateBudgetSummary | undefined): string {
  if (!budget || typeof budget.used_usd !== "number" || typeof budget.max_usd !== "number" || budget.max_usd <= 0) {
    return "n/a";
  }
  return `${Math.round((budget.used_usd / budget.max_usd) * 100)}%`;
}

function formatBudgetDashboardRow(budget: StateBudgetSummary | undefined, label: string): string {
  const state = budget?.state ?? "unknown";
  const used = formatUsd(budget?.used_usd);
  const max = typeof budget?.max_usd === "number" ? ` / ${formatUsd(budget.max_usd)}` : "";
  const hardLimit = budget?.hard_limit ? " · hard" : "";
  return `${label} · ${state} · ${used}${max} · ${budgetUsagePercent(budget)}${hardLimit}`;
}

function formatBudgetForecastRow(forecast: StateBudgetForecast): string {
  const label = forecast.scope_id ?? forecast.scope_type ?? "workspace";
  const average = typeof forecast.average_run_cost_usd === "number"
    ? `${formatUsd(forecast.average_run_cost_usd)}/run`
    : "insufficient run history";
  const runsRemaining = typeof forecast.estimated_runs_remaining === "number"
    ? `${forecast.estimated_runs_remaining} ${forecast.estimated_runs_remaining === 1 ? "run" : "runs"} left`
    : "runway unknown";
  const remaining = typeof forecast.remaining_usd === "number"
    ? `${formatUsd(forecast.remaining_usd)} remaining`
    : "remaining unknown";
  return `Forecast ${label} · avg ${average} · ${runsRemaining} · ${remaining}`;
}

function countBudgetSummaryScopes(summary: NativeBudgetSummary | undefined): number {
  if (!summary) return 0;
  return (
    summary.projects.length +
    (summary.goals?.length ?? 0) +
    (summary.tasks?.length ?? 0) +
    (summary.runs?.length ?? 0) +
    summary.agents.length
  );
}

function formatProviderPriceTable(priceTable: StateSnapshot["budgets"]["price_table"]): string {
  const count = priceTable?.count ?? 0;
  const source = priceTable?.source ?? "not updated";
  return `Provider prices ${source} · ${count} ${count === 1 ? "model" : "models"}`;
}

function formatProviderPriceRow(price: NativeProviderPrice): string {
  return `${price.provider}/${price.model} · in ${formatUsd(price.input_usd_per_million)}/M · out ${formatUsd(price.output_usd_per_million)}/M`;
}

function formatProviderPriceMetadata(price: NativeProviderPrice): string {
  return `${price.source} · ${price.updated_at}`;
}

function summarizeProviderPriceSource(prices: NativeProviderPrice[]): string {
  const sources = Array.from(new Set(prices.map((price) => price.source).filter(Boolean)));
  if (sources.length === 0) return "not updated";
  if (sources.length === 1) return sources[0] ?? "not updated";
  return "mixed";
}

function formatWorkflowValidationResult(result: WorkflowValidationResult): string {
  const state = result.valid ? "valid" : "invalid";
  const diagnostic = result.diagnostics_json.errors[0]?.message ?? "no diagnostics";
  return `Workflow validation ${state} · ${diagnostic}`;
}

function formatWorkflowRuntimeResult(runtime: WorkflowRuntimeState): string {
  return `Workflow runtime refreshed · ${runtime.current_version_id ?? "none"}`;
}

function formatWorkflowHookRunResult(result: WorkflowHookRunResult): string {
  const exit = result.exit_code === null ? "running" : `exit ${result.exit_code}`;
  return `Hook ${result.hook_name} · ${result.status} · ${exit}`;
}

function formatNativeEvidencePackResult(evidence: NativeEvidencePack): string {
  return `Evidence ${evidence.id} · ${evidence.completeness_state} · ${evidence.artifact_path}`;
}

function formatRunReplayMetadataResult(replay: PersistedRunReplayMetadata): string {
  const keys = Object.keys(replay.body_json).slice(0, 3);
  const summary = keys.length > 0 ? keys.join(", ") : "empty replay body";
  return `Replay ${replay.id} · ${replay.artifact_path} · ${summary}`;
}

function formatNativeBudgetResult(budget: NativeBudget): string {
  const scope = budget.scope_id ?? "workspace";
  const hardLimit = budget.hard_limit ? " · hard" : "";
  return `${budget.scope_type} ${scope} · ${formatUsd(budget.max_usd)} · warn ${Math.round(budget.warn_pct * 100)}%${hardLimit}`;
}

function nativeBudgetToStateSummary(budget: NativeBudget, existing?: StateBudgetSummary): StateBudgetSummary {
  return {
    ...existing,
    id: budget.id,
    scope_type: budget.scope_type,
    scope_id: budget.scope_id,
    used_usd: existing?.used_usd ?? 0,
    max_usd: budget.max_usd,
    warn_pct: budget.warn_pct,
    hard_limit: budget.hard_limit,
    state: existing?.state ?? "ok",
  };
}

function upsertBudgetCollection(items: StateBudgetSummary[] | undefined, budget: NativeBudget): StateBudgetSummary[] {
  const currentItems = items ?? [];
  const targetScopeId = budget.scope_id ?? null;
  const existing = currentItems.find((item) => (item.scope_id ?? null) === targetScopeId);
  const next = nativeBudgetToStateSummary(budget, existing);
  return existing
    ? currentItems.map((item) => ((item.scope_id ?? null) === targetScopeId ? next : item))
    : [...currentItems, next];
}

function upsertNativeBudgetIntoSnapshot(snapshot: StateSnapshot, budget: NativeBudget): StateSnapshot {
  const budgets = { ...snapshot.budgets };
  switch (budget.scope_type) {
    case "workspace":
      budgets.workspace = nativeBudgetToStateSummary(budget, snapshot.budgets.workspace);
      break;
    case "project":
      budgets.projects = upsertBudgetCollection(snapshot.budgets.projects, budget);
      break;
    case "goal":
      budgets.goals = upsertBudgetCollection(snapshot.budgets.goals, budget);
      break;
    case "task":
      budgets.tasks = upsertBudgetCollection(snapshot.budgets.tasks, budget);
      break;
    case "run":
      budgets.runs = upsertBudgetCollection(snapshot.budgets.runs, budget);
      break;
    case "agent":
      budgets.agents = upsertBudgetCollection(snapshot.budgets.agents, budget);
      break;
  }
  return { ...snapshot, budgets };
}

function applyTokenUsageToSnapshot(snapshot: StateSnapshot, usage: NativeTokenUsage): StateSnapshot {
  return {
    ...snapshot,
    budgets: {
      ...snapshot.budgets,
      workspace: {
        ...snapshot.budgets.workspace,
        used_usd: (snapshot.budgets.workspace.used_usd ?? 0) + usage.cost_usd,
      },
      projects: snapshot.budgets.projects.map((budget) =>
        budget.scope_id === usage.project_id
          ? { ...budget, used_usd: (budget.used_usd ?? 0) + usage.cost_usd }
          : budget,
      ),
      goals: snapshot.budgets.goals?.map((budget) => {
        const task = snapshot.tasks.items.find((item) => item.id === usage.task_id);
        return task?.initiative_id && budget.scope_id === task.initiative_id
          ? { ...budget, used_usd: (budget.used_usd ?? 0) + usage.cost_usd }
          : budget;
      }),
      tasks: snapshot.budgets.tasks?.map((budget) =>
        budget.scope_id === usage.task_id
          ? { ...budget, used_usd: (budget.used_usd ?? 0) + usage.cost_usd }
          : budget,
      ),
      runs: snapshot.budgets.runs?.map((budget) =>
        budget.scope_id === usage.run_id
          ? { ...budget, used_usd: (budget.used_usd ?? 0) + usage.cost_usd }
          : budget,
      ),
      agents: snapshot.budgets.agents.map((budget) =>
        budget.scope_id === usage.agent_profile_id
          ? { ...budget, used_usd: (budget.used_usd ?? 0) + usage.cost_usd }
          : budget,
      ),
    },
  };
}

function formatReleaseWorkflowRow(workflow: NativeReleaseWorkflowStatus["workflows"][number]): string {
  return `${workflow.label} · ${workflow.script} · ${workflow.configured ? "configured" : "missing"}`;
}

function formatReleaseWorkflowMissingEnv(workflow: NativeReleaseWorkflowStatus["workflows"][number]): string {
  return workflow.missing_env.length ? `Missing ${workflow.missing_env.join(", ")}` : "Required environment present";
}

function classifyUpdateFeedSignature(signature: unknown): UpdateFeedSignatureState {
  const value = String(signature ?? "").trim();
  if (!value) return "missing";
  return value.includes("SIGNATURE_REQUIRED") ? "blocked" : "signed";
}

function formatUpdateFeedSignatureState(state: UpdateFeedSignatureState): string {
  if (state === "blocked") return "signature placeholder blocks release publishing";
  if (state === "missing") return "signature missing blocks release publishing";
  return "signature present";
}

function formatBenchmarkSummary(snapshot: StateSnapshot): string {
  const benchmarks = snapshot.benchmarks;
  if (!benchmarks?.last_run_id) return "No benchmark run recorded";
  return `${benchmarks.last_run_id} · ${benchmarks.last_status} · ${benchmarks.last_pass_count} pass · ${benchmarks.last_fail_count} fail`;
}

function formatBenchmarkSuiteRow(suite: StateBenchmarkSuite): string {
  return `${suite.name} · ${suite.status} · ${suite.metric_value} ${suite.unit} / ${suite.target_value} ${suite.unit}`;
}

function formatBenchmarkRun(run: NativeBenchmarkRun): string {
  return `${run.id} · ${run.status} · ${run.pass_count} pass · ${run.fail_count} fail`;
}

function formatReleaseGateRun(run: NativeReleaseGateRun): string {
  return `${run.id} · ${run.status} · ${run.pass_count} pass · ${run.fail_count} fail`;
}

function formatReleaseGateScenario(scenario: NativeReleaseGateRun["scenarios"][number]): string {
  return `${scenario.gate_id} · ${scenario.name} · ${scenario.status}`;
}

function formatTerminalSmokeRun(run: NativeTerminalFidelitySmokeRun): string {
  return `${run.id} · ${run.status} · ${run.pass_count} pass · ${run.fail_count} fail · ${run.warning_count} warning`;
}

function formatTerminalFidelityProof(terminalFidelity: StateSnapshot["terminal_fidelity"]): string {
  if (!terminalFidelity?.last_run_id) return "Terminal proof not recorded · run terminal smoke from Quality rail";
  return `Terminal proof ${terminalFidelity.last_run_id} · ${terminalFidelity.last_status} · ${terminalFidelity.last_pass_count} pass · ${terminalFidelity.last_fail_count} fail · ${terminalFidelity.last_warning_count} warning`;
}

function formatTerminalSmokeCase(testCase: NativeTerminalFidelitySmokeRun["cases"][number]): string {
  return `${testCase.case_id} · ${testCase.name} · ${testCase.status}`;
}

function formatTaskLifecycleRun(run: NativeTaskLifecycleE2ERun): string {
  return `${run.id} · ${run.status} · task ${run.task_id} · run ${run.run_id}`;
}

function formatTaskLifecycleTransition(transition: NativeTaskLifecycleE2ERun["transitions"][number]): string {
  return `${transition.step} · ${transition.task_status} · ${transition.run_lifecycle ?? "no run"}`;
}

function formatWorkflowNegativeRun(run: NativeWorkflowNegativeTestRun): string {
  return `${run.id} · ${run.status} · baseline ${run.baseline_workflow_id} · LKG ${run.last_known_good_workflow_id}`;
}

function formatWorkflowNegativeCase(testCase: NativeWorkflowNegativeTestRun["cases"][number]): string {
  return `${testCase.case_id} · ${testCase.status}`;
}

function formatDmgSmokeRun(run: NativeDmgSmokeRun): string {
  return `${run.id} · ${run.status} · ${run.pass_count} pass · ${run.fail_count} fail · blocker ${run.explicit_blocker}`;
}

function formatDmgSmokeCase(testCase: NativeDmgSmokeRun["cases"][number]): string {
  return `${testCase.case_id} · ${testCase.name} · ${testCase.status}`;
}

function formatRecoveryDrillRun(run: NativeRecoveryDrillRun): string {
  return `${run.id} · ${run.status} · ${run.pass_count} pass · ${run.fail_count} fail`;
}

function formatRecoveryDrill(drill: NativeRecoveryDrillRun["drills"][number]): string {
  return `${drill.drill_id} · ${drill.name} · ${drill.status}`;
}

function formatDogfoodReview(review: NativeDogfoodTelemetryReview): string {
  return `${review.id} · ${review.status} · ${review.pass_count} pass · ${review.warning_count} warning · ${review.fail_count} fail`;
}

function formatDogfoodFinding(finding: NativeDogfoodTelemetryReview["findings"][number]): string {
  return `${finding.finding_id} · ${finding.status}`;
}

function formatTrackerBinding(binding: NativeExternalTrackerBinding): string {
  return `${binding.external_id} · ${binding.provider} · ${binding.sync_status}`;
}

function formatTrackerSyncRun(run: NativeExternalTrackerSyncRun): string {
  return `${run.id} · ${run.provider} · ${run.status} · ${run.operation_count} ${run.operation_count === 1 ? "operation" : "operations"}`;
}

function formatOpsBudgetSummary(budget: StateBudgetSummary | undefined): string {
  const state = budget?.state ?? "unknown";
  const used = formatUsd(budget?.used_usd);
  const max = typeof budget?.max_usd === "number" ? ` / ${formatUsd(budget.max_usd)}` : "";
  return `Budget ${state} · ${used}${max}`;
}

function formatUsd(value: number | undefined): string {
  return `$${(value ?? 0).toFixed(2)}`;
}

function formatKnowledgeSummary(snapshot: StateSnapshot): string {
  const recent = snapshot.knowledge.recent_pages[0] ?? "no recent pages";
  return `Knowledge ${snapshot.knowledge.stale_count} stale · ${snapshot.knowledge.gap_count} gaps · ${recent}`;
}

function formatKnowledgeLintReport(report: NativeKnowledgeLintReport): string {
  return `Recorded lint ${report.id} · ${report.stale_count} stale · ${report.gap_count} gaps · ${report.contradiction_count} contradictions`;
}

function isPolicyApprovalAttention(item: StateAttentionItem): boolean {
  return item.id.startsWith("policy_approval_") || item.label.startsWith("Policy approval required:");
}

function nativePolicyApprovalToAttention(approval: NativePolicyApproval): StateAttentionItem {
  const scope = [
    approval.task_id ? `task ${approval.task_id}` : "",
    approval.run_id ? `run ${approval.run_id}` : "",
  ].filter(Boolean);
  const severity = ["critical", "high"].includes(approval.risk_level) ? "critical" : "warning";
  return {
    id: approval.id,
    label: `Policy approval required: ${approval.action_kind}`,
    severity,
    detail: [approval.command ?? approval.action_kind, ...scope].join(" · "),
  };
}

function splitCommaList(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseSkillPackSkillsJson(value: string): string[] {
  const parsed = JSON.parse(value || "[]");
  if (!Array.isArray(parsed) || parsed.some((item) => typeof item !== "string")) {
    throw new Error("skills must be a JSON array of strings");
  }
  return parsed.map((item) => item.trim()).filter(Boolean);
}

function isDangerousTerminalInput(input: string): boolean {
  const lower = input.toLowerCase();
  return lower.includes("sudo ") || lower.includes("rm -rf") || lower.includes("chmod 777") || lower.includes("curl ") && lower.includes("| sh");
}

function displayTerminalInput(input: string): string {
  return input.trim();
}

function nativeAgentProfileToStateAgent(agent: { id: string; name: string; status: string }): StateAgent {
  return {
    id: agent.id,
    label: agent.name,
    available: agent.status === "available",
    token_usage: undefined,
    attention_state: undefined,
    attention_severity: undefined,
    notification_count: undefined,
    last_heartbeat_at: "last_heartbeat_at" in agent && typeof agent.last_heartbeat_at === "string" ? agent.last_heartbeat_at : null,
  };
}

interface SidebarProject {
  id?: string;
  name: string;
  branch: string;
  state: string;
  sessions: number;
}

interface ControlTowerProjectCard {
  id: string;
  name: string;
  state: string;
  sessions: number;
  runs: number;
  alerts: number;
  usageLine?: string;
}

interface GlobalAttentionQueueItem {
  id: string;
  label: string;
  detail: string;
  severity: "critical" | "warning" | "info";
}

interface AgentTeamSummary {
  id: string;
  label: string;
  available: boolean;
  runCount: number;
  blockedCount: number;
  budgetState: string;
  usageLine?: string;
  currentTaskLabel?: string;
  currentRunId?: string;
  recentBlockerRunId?: string;
  recentBlockerLifecycle?: string;
}

interface PendingDangerousInput {
  session: TerminalSession;
  input: string;
  command: string;
}

interface RecentEvidenceActivityItem {
  id: string;
  evidencePackId: string;
  taskLabel: string;
  projectLabel: string;
  reviewState: string;
  completenessState: string;
  runLifecycle: string;
  usageLine?: string;
}

interface RoadmapTimelineItem {
  id: string;
  label: string;
  taskCount: number;
  urgentCount: number;
  statusLine: string;
}

interface CalendarTaskBucket {
  id: string;
  label: string;
  tasks: StateTaskSummary[];
}

interface SkillPackRegistryItem {
  id: string;
  name: string;
  budgetLabel: string;
  sourceLine: string;
  activeWorkloadCount: number;
}

interface RuntimePoolItem {
  id: string;
  label: string;
  sessionCount: number;
  runCount: number;
  blockedCount: number;
}

function nativeRuntimePoolItemToRuntimePoolItem(item: NativeRuntimePoolItem): RuntimePoolItem {
  return {
    id: item.id,
    label: item.label,
    sessionCount: item.session_count,
    runCount: item.run_count,
    blockedCount: item.blocked_count,
  };
}

function formatSkillPackSkills(skillsJson: unknown): string {
  if (!Array.isArray(skillsJson)) return "no skills";
  const skills = skillsJson
    .map((skill) => (typeof skill === "string" ? skill.trim() : ""))
    .filter(Boolean);
  return skills.length > 0 ? skills.join(", ") : "no skills";
}

function nativeSkillPackToRegistryItem(pack: NativeSkillPack, snapshot: StateSnapshot): SkillPackRegistryItem {
  const contextPackId = pack.source_context_pack_id ?? undefined;
  const activeTaskCount = contextPackId
    ? snapshot.tasks.items.filter(
        (task) => task.context_pack_id === contextPackId && !["done", "archived"].includes(task.status),
      ).length
    : 0;
  const activeRunCount = contextPackId
    ? snapshot.runs.items.filter(
        (run) => run.context_pack_id === contextPackId && !["completed", "cancelled"].includes(run.lifecycle),
      ).length
    : 0;
  const sourceLine = [pack.description, pack.source_context_pack_id].filter(Boolean).join(" · ");
  return {
    id: pack.id,
    name: pack.name,
    budgetLabel: formatSkillPackSkills(pack.skills_json),
    sourceLine: sourceLine || "native registry",
    activeWorkloadCount: activeTaskCount + activeRunCount,
  };
}

interface AnalyticsBarItem {
  label: string;
  count: number;
  percent: number;
}

interface HistoricalAnalyticsSummary {
  sampleCount: number;
  runTotal: number;
  runBars: AnalyticsBarItem[];
  reviewTotal: number;
  reviewBars: AnalyticsBarItem[];
  budgetLine: string;
}

interface CommandPaletteItem {
  id: string;
  kind: string;
  label: string;
  detail: string;
  searchText?: string;
  action?: () => void;
}

interface WorkspaceTab {
  id: string;
  label: string;
  active: boolean;
  projectId?: string;
}

const visualQaScreens = [
  { slug: "control-tower", label: "Control Tower" },
  { slug: "explorer", label: "Explorer" },
  { slug: "git", label: "Git" },
  { slug: "inspector", label: "Inspector" },
  { slug: "preview", label: "Preview" },
  { slug: "board", label: "Board" },
  { slug: "terminal-deck", label: "Terminal Deck" },
  { slug: "grid-split", label: "Grid Split" },
] as const;

type VisualQaScreen = (typeof visualQaScreens)[number];

const visualQaScreenBySlug = new Map<string, VisualQaScreen>(visualQaScreens.map((screen) => [screen.slug, screen]));
const visualQaScreenByLabel = new Map<string, VisualQaScreen>(visualQaScreens.map((screen) => [screen.label, screen]));
const visualQaScreenAliases = new Map<string, string>([
  ["control-towel", "control-tower"],
  ["control_towel", "control-tower"],
  ["task-board", "board"],
  ["task_board", "board"],
]);

function getVisualQaScreenFromPathname(pathname: string): VisualQaScreen | undefined {
  const match = pathname.match(/^\/dev\/([^/?#]+)/);
  if (!match) return undefined;
  const requested = decodeURIComponent(match[1]).toLowerCase();
  const slug = visualQaScreenAliases.get(requested) ?? requested.replace(/_/g, "-");
  return visualQaScreenBySlug.get(slug);
}

function getCurrentVisualQaScreen(): VisualQaScreen | undefined {
  if (typeof window === "undefined") return undefined;
  return getVisualQaScreenFromPathname(window.location.pathname);
}

function visualQaWorkspaceTabs(activeScreen: VisualQaScreen): WorkspaceTab[] {
  return visualQaScreens.map((screen) => ({
    id: `visual-${screen.slug}`,
    label: screen.label,
    active: screen.slug === activeScreen.slug,
  }));
}

function VisualQaPanel({
  label,
  children,
  className = "",
}: {
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`hc-visual-panel ${className}`} aria-label={label}>
      <header>
        <span>{label}</span>
      </header>
      {children}
    </section>
  );
}

function VisualQaRightRail() {
  const panels = [
    {
      label: "Attention Center",
      icon: <AlertTriangle size={14} />,
      rows: ["Critical 0", "Warning 2", "Workflow review pending"],
    },
    {
      label: "Review Queue",
      icon: <Activity size={14} />,
      rows: ["RG-08 visual QA evidence", "Terminal proof ready", "Policy approval clear"],
    },
    {
      label: "Recent Sessions",
      icon: <Terminal size={14} />,
      rows: ["codex-agent active", "preview server idle", "release gate waiting"],
    },
  ];

  return (
    <>
      {panels.map((panel) => (
        <section
          className="hc-rail-panel hc-visual-rail-panel"
          aria-label={panel.label}
          data-testid="visual-qa-right-rail-panel"
          key={panel.label}
        >
          <header>
            <span>{panel.label}</span>
            {panel.icon}
          </header>
          <div className="hc-visual-rail-list">
            {panel.rows.map((row) => (
              <span key={row}>{row}</span>
            ))}
          </div>
        </section>
      ))}
    </>
  );
}

function VisualQaWorkspaceContent({ screen }: { screen: VisualQaScreen }) {
  return (
    <section
      className={`hc-visual-qa-screen hc-visual-qa-screen-${screen.slug}`}
      data-testid={`visual-qa-${screen.slug}`}
      aria-label={`${screen.label} reference screen`}
    >
      {renderVisualQaScreen(screen)}
    </section>
  );
}

function renderVisualQaScreen(screen: VisualQaScreen): ReactNode {
  switch (screen.slug) {
    case "control-tower":
      return <VisualQaControlTower />;
    case "terminal-deck":
      return <VisualQaTerminalDeck />;
    case "grid-split":
      return <VisualQaGridSplit />;
    case "explorer":
      return <VisualQaExplorer />;
    case "git":
      return <VisualQaGit />;
    case "inspector":
      return <VisualQaInspector />;
    case "preview":
      return <VisualQaPreview />;
    case "board":
      return <VisualQaBoard />;
  }
}

function VisualQaScreenToolbar({ title, meta }: { title: string; meta: string }) {
  return (
    <div className="hc-visual-screen-toolbar">
      <strong>{title}</strong>
      <span>{meta}</span>
    </div>
  );
}

function VisualQaControlTower() {
  return (
    <>
      <VisualQaScreenToolbar title="Control Tower" meta="1536 x 864 visual reference" />
      <section className="hc-visual-kpi-strip" aria-label="KPI Strip">
        <span className="hc-visual-section-label">KPI Strip</span>
        {["Active Runs 12", "Review Ready 4", "Budget 62%", "Knowledge 3 gaps"].map((item) => (
          <article key={item}>
            <span>{item}</span>
            <strong>{item.split(" ")[0]}</strong>
          </article>
        ))}
      </section>
      <div className="hc-visual-control-layout">
        <VisualQaPanel label="Session Map" className="hc-visual-session-map-panel">
          <div className="hc-visual-session-map">
            {["CLI", "Agent", "Preview", "Release"].map((item) => (
              <span key={item}>{item}</span>
            ))}
          </div>
        </VisualQaPanel>
        <VisualQaPanel label="Active Projects" className="hc-visual-active-projects">
          <table>
            <tbody>
              {["haneulchi main active", "docs workspace review", "reference analysis idle"].map((row) => (
                <tr key={row}>
                  <td><span className="hc-status-dot ready" /></td>
                  <td>{row}</td>
                  <td>ok</td>
                </tr>
              ))}
            </tbody>
          </table>
        </VisualQaPanel>
        <VisualQaPanel label="Orchestration Timeline" className="hc-visual-timeline">
          {["Plan", "Run", "Verify", "Review", "Release"].map((item) => (
            <span key={item}>{item}</span>
          ))}
        </VisualQaPanel>
      </div>
    </>
  );
}

function VisualQaTerminalDeck() {
  return (
    <>
      <VisualQaScreenToolbar title="Terminal Deck" meta="1536 x 1024 visual reference" />
      <section className="hc-visual-terminal-grid" aria-label="Visual terminal grid">
        {terminalSessions.map((session, index) => (
          <article className="hc-terminal-pane hc-visual-terminal-pane" aria-label={`Terminal ${index + 1}`} key={session.id}>
            <header>
              <span className={`hc-status-dot ${session.status}`} />
              <strong>Terminal {index + 1}</strong>
              <small>{session.title}</small>
            </header>
            <pre data-dynamic="terminal-output">
              {session.lines.map((line) => (
                <span key={line}>{line}</span>
              ))}
            </pre>
          </article>
        ))}
      </section>
      <section className="hc-bottom-panel hc-visual-bottom-logs" aria-label="Bottom Logs">
        <header>
          <span>Bottom Logs</span>
          <small>176px evidence strip</small>
        </header>
        <div>
          <span data-dynamic="timestamp">12:35 command block indexed</span>
          <span data-dynamic="timestamp">12:36 visual shell benchmark queued</span>
          <span data-dynamic="timestamp">12:37 release gate evidence attached</span>
        </div>
      </section>
    </>
  );
}

function VisualQaGridSplit() {
  return (
    <>
      <VisualQaScreenToolbar title="Grid Split" meta="Terminal, Explorer, Git, Preview" />
      <div className="hc-visual-grid-split">
        <VisualQaPanel label="Terminal Logs">
          <pre data-dynamic="terminal-output">$ npm test{"\n"}PASS visual shell contract{"\n"}logs pinned inside panel</pre>
        </VisualQaPanel>
        <VisualQaPanel label="Explorer">
          <span>src/App.tsx</span>
          <span>src/App.css</span>
          <span>tests/designTokenCompliance.test.ts</span>
        </VisualQaPanel>
        <VisualQaPanel label="Git">
          <span>main ahead 2</span>
          <span>modified App.tsx</span>
          <span>staged visual route test</span>
        </VisualQaPanel>
        <VisualQaPanel label="Preview">
          <div className="hc-visual-browser-surface">
            <span>http://localhost:1420/dev/grid-split</span>
            <strong>Preview</strong>
          </div>
        </VisualQaPanel>
      </div>
    </>
  );
}

function VisualQaExplorer() {
  return (
    <>
      <VisualQaScreenToolbar title="Explorer" meta="File tree and detail reference" />
      <div className="hc-visual-explorer-toolbar">
        <label>
          <span>Project Selector</span>
          <select aria-label="Visual project selector" defaultValue="haneulchi">
            <option>haneulchi</option>
          </select>
        </label>
        <label>
          <span>Branch Selector</span>
          <select aria-label="Visual branch selector" defaultValue="main">
            <option>main</option>
          </select>
        </label>
        <label>
          <Search size={12} />
          <input aria-label="Visual file search" defaultValue="App" />
        </label>
        <button type="button">Filters</button>
      </div>
      <div className="hc-visual-explorer-layout">
        <VisualQaPanel label="File Tree">
          {["M src/App.tsx", "A src/App.css", "? tests/visualShell.test.ts"].map((item) => (
            <span className="hc-visual-file-row" key={item}>{item}</span>
          ))}
        </VisualQaPanel>
        <VisualQaPanel label="File Detail">
          <div className="hc-visual-mode-tabs">
            {["Summary", "Git Blame", "History"].map((tab) => (
              <button type="button" key={tab}>{tab}</button>
            ))}
          </div>
          <pre>function App() {"{"} return visual QA shell; {"}"}</pre>
        </VisualQaPanel>
      </div>
    </>
  );
}

function VisualQaGit() {
  return (
    <>
      <VisualQaScreenToolbar title="Git" meta="Commit graph and changes reference" />
      <div className="hc-visual-git-toolbar">
        <span>repo haneulchi</span>
        <span>branch main</span>
        <span>ahead 2 behind 0</span>
        {["Fetch", "Compare", "Pull"].map((action) => (
          <button type="button" key={action}>{action}</button>
        ))}
      </div>
      <div className="hc-visual-git-layout">
        <VisualQaPanel label="Commit Graph">
          {["main visual QA routes", "release gates parity", "tracker binding fix"].map((commit, index) => (
            <div className="hc-visual-commit-row" key={commit}>
              <span>{index + 1}</span>
              <span className="hc-visual-avatar" data-dynamic="avatar" aria-hidden="true">{commit[0].toUpperCase()}</span>
              <strong>{commit}</strong>
              <small>codex</small>
            </div>
          ))}
        </VisualQaPanel>
        <VisualQaPanel label="Changes">
          {["Staged src/App.test.tsx", "Unstaged src/App.tsx", "Untracked visual baselines"].map((change) => (
            <span key={change}>{change}</span>
          ))}
          <label>
            <span>Commit composer</span>
            <input aria-label="Visual commit message" defaultValue="Add visual QA routes" />
          </label>
          <footer>Stash footer</footer>
        </VisualQaPanel>
        <VisualQaPanel label="Pull Requests">
          <span>Draft PR visual shell evidence</span>
        </VisualQaPanel>
      </div>
    </>
  );
}

function VisualQaInspector() {
  return (
    <>
      <VisualQaScreenToolbar title="Inspector" meta="Session inspector reference" />
      <div className="hc-visual-mode-tabs hc-visual-inspector-tabs">
        {["Session", "Task", "Worktree", "Agent", "Environment"].map((tab) => (
          <button type="button" key={tab}>{tab}</button>
        ))}
      </div>
      <div className="hc-visual-inspector-grid">
        {["Overview", "Recent Commands", "Recent Events", "Artifacts", "Environment"].map((card) => (
          <VisualQaPanel label={card} key={card}>
            <span>{card} card visible</span>
            <small>Previous / Next / Reopen</small>
          </VisualQaPanel>
        ))}
      </div>
    </>
  );
}

function VisualQaPreview() {
  return (
    <>
      <VisualQaScreenToolbar title="Preview" meta="Browser and event reference" />
      <div className="hc-visual-preview-layout">
        <aside className="hc-visual-routes-sidebar" aria-label="Routes">
          <strong>Routes</strong>
          {["/", "/docs", "/dev/preview"].map((route) => (
            <span key={route}>{route}</span>
          ))}
        </aside>
        <section className="hc-visual-preview-main" aria-label="Web preview surface">
          <header>
            <span>Address</span>
            <input aria-label="Visual preview address" defaultValue="http://localhost:1420/dev/preview" />
          </header>
          <div className="hc-visual-mode-tabs">
            {["Web Preview", "Markdown", "Diff Preview"].map((tab) => (
              <button type="button" key={tab}>{tab}</button>
            ))}
          </div>
          <div className="hc-visual-browser-surface">
            <strong>Web Preview</strong>
            <span>Large preview area</span>
          </div>
          <footer>
            <span>Preview Events</span>
            <span>Network</span>
          </footer>
        </section>
      </div>
    </>
  );
}

function VisualQaBoard() {
  return (
    <>
      <VisualQaScreenToolbar title="Board" meta="Task board reference" />
      <div className="hc-visual-board-toolbar">
        <span>Board Selector</span>
        <span>Filters</span>
        <span>Group sort</span>
        <button type="button">
          <Plus size={13} />
          New Task
        </button>
      </div>
      <section className="hc-visual-board-columns" aria-label="Visual task board">
        {["Backlog", "In Progress", "In Review", "Done"].map((column, index) => (
          <article className="hc-visual-board-column" key={column}>
            <header>
              <span>{column}</span>
              <small>{index + 2}</small>
            </header>
            <div className="hc-visual-task-card">
              <strong>HC-{index + 101} Visual QA route</strong>
              <span>high · haneulchi · shell · codex</span>
            </div>
            <div className="hc-visual-task-card">
              <strong>HC-{index + 201} Dense task card</strong>
              <span>medium · design · review</span>
            </div>
          </article>
        ))}
      </section>
    </>
  );
}

interface ProjectTabGroupAssignment {
  projectId: string;
  group: string;
}

interface ProjectDetachPlan {
  projectId: string;
  projectName: string;
  windowId: string;
  status: "planned";
}

interface ProjectLayoutPreset {
  id: string;
  projectId: string;
  name: string;
  layoutJson: ProjectTabLayout;
}

function nativeCommandBlockExplanationToCommandBlockExplanation(
  explanation: NativeCommandBlockExplanation,
): CommandBlockExplanation {
  return {
    id: explanation.id,
    commandBlockId: explanation.command_block_id,
    command: explanation.command,
    summary: explanation.summary,
    evidence: explanation.evidence,
    provider: explanation.provider ?? undefined,
    model: explanation.model ?? undefined,
    agentProfileId: explanation.agent_profile_id ?? undefined,
    prompt: explanation.prompt ?? undefined,
    diagnostics: explanation.diagnostics,
  };
}

function stateProjectsToSidebarProjects(projects: StateProject[], sessions: StateSession[]): SidebarProject[] {
  if (projects.length === 0) return fallbackProjects;
  return projects.map((project) => ({
    id: project.id,
    name: project.name,
    branch: project.state,
    state: project.state,
    sessions: sessions.filter((session) => session.project_id === project.id).length,
  }));
}

function buildControlTowerProjectCards(snapshot: StateSnapshot): ControlTowerProjectCard[] {
  const projects = snapshot.projects.length > 0
    ? snapshot.projects
      : fallbackProjects.map((project, index) => ({
          id: `fallback_${index}`,
          name: project.name,
          state: project.state.toLowerCase(),
          token_usage: undefined,
        }));

  return projects.map((project) => {
    const sessions = snapshot.sessions.filter((session) => session.project_id === project.id);
    const runs = snapshot.runs.items.filter((run) => run.project_id === project.id);
    const activeRuns = runs.filter((run) => !["completed", "cancelled"].includes(run.lifecycle));
    const sessionAlerts = sessions.filter((session) => (session.attention_state ?? "none") !== "none").length;
    const runAlerts = runs.filter((run) => ["blocked", "failed"].includes(run.lifecycle)).length;
    return {
      id: project.id,
      name: project.name,
      state: project.state,
      sessions: sessions.length,
      runs: activeRuns.length,
      alerts: sessionAlerts + runAlerts,
      usageLine: formatTokenUsageSummary(project.token_usage),
    };
  });
}

function buildAgentTeamSummaries(snapshot: StateSnapshot): AgentTeamSummary[] {
  return snapshot.agents.map((agent) => {
    const assignedRuns = snapshot.runs.items.filter(
      (run) => run.agent_profile_id === agent.id && !["completed", "cancelled"].includes(run.lifecycle),
    );
    const assignedTasks = snapshot.tasks.items.filter((task) => taskAssignedToAgent(task, agent.id));
    const blockerRun = assignedRuns.find((run) => ["blocked", "failed", "permission_requested"].includes(run.lifecycle));
    const currentRun = assignedRuns.find((run) => !["blocked", "failed", "permission_requested"].includes(run.lifecycle)) ?? assignedRuns[0];
    const currentRunTask = currentRun ? snapshot.tasks.items.find((task) => task.id === currentRun.task_id) : undefined;
    const currentTask = currentRunTask ?? assignedTasks[0];
    const budget = snapshot.budgets.agents.find((item) => item.scope_id === agent.id);
    return {
      id: agent.id,
      label: agent.label,
      available: agent.available,
      runCount: assignedRuns.length,
      blockedCount: assignedRuns.filter((run) => ["blocked", "failed", "permission_requested"].includes(run.lifecycle)).length,
      budgetState: budget?.state ?? "unknown",
      usageLine: formatTokenUsageSummary(agent.token_usage),
      currentTaskLabel: currentTask?.title,
      currentRunId: currentRun?.id,
      recentBlockerRunId: blockerRun?.id,
      recentBlockerLifecycle: blockerRun?.lifecycle,
    };
  });
}

function taskAssignedToAgent(task: StateTaskSummary, agentId: string): boolean {
  if (task.assignee_type && task.assignee_type !== "agent") return false;
  return (task.assignee_id ?? task.assignee) === agentId;
}

function buildRecentEvidenceActivity(snapshot: StateSnapshot): RecentEvidenceActivityItem[] {
  return snapshot.reviews.map((review) => {
    const task = review.task_id ? snapshot.tasks.items.find((item) => item.id === review.task_id) : undefined;
    const run = review.run_id ? snapshot.runs.items.find((item) => item.id === review.run_id) : undefined;
    const projectId = run?.project_id ?? task?.project_id;
    const project = projectId ? snapshot.projects.find((item) => item.id === projectId) : undefined;
    return {
      id: review.id,
      evidencePackId: review.evidence_pack_id,
      taskLabel: task?.title ?? review.task_id ?? "Unlinked task",
      projectLabel: project?.name ?? projectId ?? "Workspace",
      reviewState: review.state,
      completenessState: review.completeness_state,
      runLifecycle: run ? formatLifecycleLabel(run.lifecycle) : "no run",
      usageLine: formatEvidenceTokenUsage(review.token_usage),
    };
  });
}

function taskPlanningCycle(task: StateTaskSummary): string {
  return task.cycle ?? task.cycle_id ?? "Backlog";
}

function taskPlanningModule(task: StateTaskSummary): string {
  return task.module ?? task.module_id ?? "General";
}

function initiativeRoadmapLabel(initiative: StateInitiative): string {
  const usage = formatTokenUsageSummary(initiative.token_usage);
  return usage ? `${initiative.name} · ${usage}` : initiative.name;
}

function taskPlanningInitiative(task: StateTaskSummary, initiativesById: Map<string, StateInitiative>): string | undefined {
  const initiative = task.initiative ?? task.initiative_id;
  if (!initiative) return undefined;
  const persistedInitiative = initiativesById.get(initiative);
  return persistedInitiative ? initiativeRoadmapLabel(persistedInitiative) : initiative;
}

function buildRoadmapTimelineItems(tasks: StateTaskSummary[], initiatives: StateInitiative[] = []): RoadmapTimelineItem[] {
  const initiativesById = new Map(initiatives.map((initiative) => [initiative.id, initiative]));
  const groups = new Map<string, StateTaskSummary[]>();
  for (const task of tasks) {
    const initiative = taskPlanningInitiative(task, initiativesById);
    const label = initiative
      ? `${initiative} · ${taskPlanningCycle(task)} · ${taskPlanningModule(task)}`
      : `${taskPlanningCycle(task)} · ${taskPlanningModule(task)}`;
    groups.set(label, [...(groups.get(label) ?? []), task]);
  }

  return Array.from(groups.entries())
    .map(([label, groupTasks]) => {
      const statusCounts = groupTasks.reduce<Record<string, number>>((counts, task) => {
        counts[task.status] = (counts[task.status] ?? 0) + 1;
        return counts;
      }, {});
      const statusLine = Object.entries(statusCounts)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([status, count]) => `${status} ${count}`)
        .join(" · ");
      return {
        id: label,
        label,
        taskCount: groupTasks.length,
        urgentCount: groupTasks.filter((task) => task.priority === "urgent").length,
        statusLine,
      };
    })
    .sort((left, right) => left.label.localeCompare(right.label));
}

function nativeInitiativeToStateInitiative(initiative: NativeInitiative): StateInitiative {
  return {
    id: initiative.id,
    project_id: initiative.project_id,
    name: initiative.name,
    description: initiative.description ?? undefined,
    budget_id: initiative.budget_id ?? undefined,
    status: initiative.status,
    token_usage: initiative.token_usage ?? undefined,
  };
}

function mergeInitiativesIntoStateSnapshot(snapshot: StateSnapshot, initiatives: NativeInitiative[]): StateSnapshot {
  if (initiatives.length === 0) return snapshot;
  const existing = snapshot.initiatives ?? [];
  const byId = new Map<string, StateInitiative>();
  const order: string[] = [];
  for (const initiative of existing) {
    byId.set(initiative.id, initiative);
    order.push(initiative.id);
  }
  for (const initiative of initiatives.map(nativeInitiativeToStateInitiative)) {
    if (!byId.has(initiative.id)) {
      order.push(initiative.id);
    }
    byId.set(initiative.id, initiative);
  }
  return {
    ...snapshot,
    initiatives: order.map((id) => byId.get(id)).filter((initiative): initiative is StateInitiative => Boolean(initiative)),
  };
}

function buildCalendarTaskBuckets(tasks: StateTaskSummary[]): CalendarTaskBucket[] {
  const groups = new Map<string, StateTaskSummary[]>();
  for (const task of tasks.filter((item) => item.cycle || item.cycle_id)) {
    const label = taskPlanningCycle(task);
    groups.set(label, [...(groups.get(label) ?? []), task]);
  }

  return Array.from(groups.entries())
    .map(([label, groupTasks]) => ({
      id: label,
      label,
      tasks: groupTasks.slice().sort((left, right) => left.title.localeCompare(right.title)),
    }))
    .sort((left, right) => left.label.localeCompare(right.label));
}

function buildSkillPackRegistryItems(
  nativeSkillPacks: NativeSkillPack[],
  contextPacks: NativeContextPack[],
  snapshot: StateSnapshot,
): SkillPackRegistryItem[] {
  const byId = new Map<string, SkillPackRegistryItem>();
  for (const pack of nativeSkillPacks) {
    byId.set(pack.id, nativeSkillPackToRegistryItem(pack, snapshot));
  }
  for (const pack of contextPacks) {
    const sources = contextPackSources(pack);
    const activeTaskCount = snapshot.tasks.items.filter(
      (task) => task.context_pack_id === pack.id && !["done", "archived"].includes(task.status),
    ).length;
    const activeRunCount = snapshot.runs.items.filter(
      (run) => run.context_pack_id === pack.id && !["completed", "cancelled"].includes(run.lifecycle),
    ).length;
    if (byId.has(pack.id)) continue;
    byId.set(pack.id, {
      id: pack.id,
      name: pack.name,
      budgetLabel: contextPackBudgetLabel(pack),
      sourceLine: sources.map((source) => source.id ?? source.path ?? source.type ?? "source").join(", ") || "no sources",
      activeWorkloadCount: activeTaskCount + activeRunCount,
    });
  }
  return Array.from(byId.values());
}

function runtimePoolLabel(mode: string): string {
  if (mode === "shell") return "Local";
  if (mode === "ssh") return "Remote SSH";
  if (mode === "agent" || mode === "cloud") return "Cloud agents";
  return mode;
}

function buildRuntimePoolItems(snapshot: StateSnapshot): RuntimePoolItem[] {
  const groups = new Map<string, { sessions: StateSession[]; runs: StateRunSummary[] }>();
  for (const session of snapshot.sessions) {
    const group = groups.get(session.mode) ?? { sessions: [], runs: [] };
    group.sessions.push(session);
    groups.set(session.mode, group);
  }
  for (const run of snapshot.runs.items) {
    const session = run.session_id ? snapshot.sessions.find((item) => item.id === run.session_id) : undefined;
    const mode = session?.mode ?? "agent";
    const group = groups.get(mode) ?? { sessions: [], runs: [] };
    group.runs.push(run);
    groups.set(mode, group);
  }

  return Array.from(groups.entries())
    .map(([mode, group]) => ({
      id: mode,
      label: runtimePoolLabel(mode),
      sessionCount: group.sessions.length,
      runCount: group.runs.filter((run) => !["completed", "cancelled"].includes(run.lifecycle)).length,
      blockedCount: group.runs.filter((run) => ["blocked", "failed", "permission_requested"].includes(run.lifecycle)).length,
    }))
    .sort((left, right) => {
      const order = ["shell", "ssh", "agent"];
      const leftOrder = order.indexOf(left.id);
      const rightOrder = order.indexOf(right.id);
      if (leftOrder !== rightOrder) return (leftOrder === -1 ? order.length : leftOrder) - (rightOrder === -1 ? order.length : rightOrder);
      return left.id.localeCompare(right.id);
    });
}

function formatEvidenceTokenUsage(usage: StateReview["token_usage"]): string | undefined {
  if (!usage || typeof usage.cost_usd !== "number") return undefined;
  const totalTokens = typeof usage.total_tokens === "number"
    ? usage.total_tokens
    : (usage.input_tokens ?? 0) + (usage.output_tokens ?? 0);
  return `${formatUsd(usage.cost_usd)} · ${totalTokens} tokens`;
}

function formatTaskTokenUsage(task: StateTaskSummary): string | undefined {
  return formatTokenUsageSummary(task.token_usage);
}

function formatTokenUsageSummary(usage: StateTokenUsageSummary | undefined): string | undefined {
  if (!usage || typeof usage.cost_usd !== "number") return undefined;
  const totalTokens = typeof usage.total_tokens === "number"
    ? usage.total_tokens
    : usage.input_tokens + usage.output_tokens;
  if (totalTokens <= 0 && usage.cost_usd <= 0) return undefined;
  return `${formatUsd(usage.cost_usd)} · ${totalTokens} tokens`;
}

function formatCalendarTaskLabel(task: StateTaskSummary): string {
  const usage = formatTaskTokenUsage(task);
  return usage ? `${task.title} · ${usage}` : task.title;
}

function buildAnalyticsBars(counts: Record<string, number>, total: number): AnalyticsBarItem[] {
  return Object.entries(counts)
    .filter(([, count]) => count > 0)
    .map(([label, count]) => ({
      label: formatLifecycleLabel(label),
      count,
      percent: total > 0 ? Math.round((count / total) * 100) : 0,
    }));
}

function buildHistoricalAnalyticsSummary(snapshot: StateSnapshot, budget: StateBudgetSummary | undefined): HistoricalAnalyticsSummary {
  const runTotal = snapshot.runs.items.length || Object.values(snapshot.runs.counts_by_lifecycle).reduce((sum, count) => sum + count, 0);
  const reviewCounts = snapshot.reviews.reduce<Record<string, number>>((counts, review) => {
    counts[review.completeness_state] = (counts[review.completeness_state] ?? 0) + 1;
    return counts;
  }, {});
  const reviewTotal = snapshot.reviews.length;
  return {
    sampleCount: 1,
    runTotal,
    runBars: buildAnalyticsBars(snapshot.runs.counts_by_lifecycle, runTotal),
    reviewTotal,
    reviewBars: buildAnalyticsBars(reviewCounts, reviewTotal),
    budgetLine: formatOpsBudgetSummary(budget),
  };
}

function visualHarnessNodePosition(index: number, total: number): { x: number; y: number } {
  if (total <= 1) return { x: 160, y: 90 };
  const columns = Math.min(3, total);
  const rows = Math.ceil(total / columns);
  const column = index % columns;
  const row = Math.floor(index / columns);
  const xStep = 240 / Math.max(columns - 1, 1);
  const yStep = rows <= 1 ? 0 : 86 / Math.max(rows - 1, 1);
  return {
    x: 40 + column * xStep,
    y: rows <= 1 ? 90 : 48 + row * yStep,
  };
}

function shortenVisualHarnessLabel(label: string): string {
  return label.length > 18 ? `${label.slice(0, 15)}...` : label;
}

function nativeVisualHarnessLinkToEdge(
  link: NativeVisualHarnessLink,
  status = "persisted",
): StateVisualHarness["edges"][number] {
  return {
    id: link.id,
    source_id: link.source_id,
    target_id: link.target_id,
    kind: link.kind,
    status,
  };
}

function VisualHarnessCanvas({
  graph,
  onConnectNodes,
}: {
  graph?: StateVisualHarness;
  onConnectNodes?: (sourceId: string, targetId: string) => void;
}): ReactNode {
  const [dragSourceId, setDragSourceId] = useState<string | undefined>();
  const nodes = (graph?.nodes ?? []).slice(0, 9);
  const edges = (graph?.edges ?? []).slice(0, 12);
  const positions = new Map(nodes.map((node, index) => [node.id, visualHarnessNodePosition(index, nodes.length)]));

  return (
    <svg
      className="hc-visual-harness-canvas"
      role="img"
      aria-label="Visual harness dependency graph"
      viewBox="0 0 320 180"
      preserveAspectRatio="xMidYMid meet"
    >
      <defs>
        <marker id="visual-harness-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" />
        </marker>
      </defs>
      <rect className="hc-visual-harness-canvas-bg" x="1" y="1" width="318" height="178" rx="8" />
      {edges.map((edge) => {
        const source = positions.get(edge.source_id);
        const target = positions.get(edge.target_id);
        if (!source || !target) return null;
        return (
          <line
            key={`visual-canvas-edge-${edge.id}`}
            className={`hc-visual-harness-edge hc-visual-harness-edge-${edge.kind}`}
            aria-label={`${edge.source_id} to ${edge.target_id} ${edge.kind} edge`}
            x1={source.x}
            y1={source.y}
            x2={target.x}
            y2={target.y}
            markerEnd="url(#visual-harness-arrow)"
          />
        );
      })}
      {nodes.map((node, index) => {
        const position = positions.get(node.id) ?? visualHarnessNodePosition(index, nodes.length);
        return (
          <g
            key={`visual-canvas-node-${node.kind}-${node.id}`}
            className={`hc-visual-harness-node hc-visual-harness-node-${node.kind}`}
            aria-label={`${node.label} node`}
            tabIndex={0}
            transform={`translate(${position.x} ${position.y})`}
            onPointerDown={() => setDragSourceId(node.id)}
            onPointerUp={() => {
              if (dragSourceId && dragSourceId !== node.id) {
                onConnectNodes?.(dragSourceId, node.id);
              }
              setDragSourceId(undefined);
            }}
          >
            <circle r="19" />
            <text y="-2">{shortenVisualHarnessLabel(node.label)}</text>
            <text y="11" className="hc-visual-harness-node-meta">{node.kind}</text>
          </g>
        );
      })}
    </svg>
  );
}

function projectIdFromStateProjectTab(tab: StateProjectTab, projects: StateProject[]): string | undefined {
  if (tab.project_id) return tab.project_id;
  const inferredProjectId = tab.id.startsWith("tab_") ? tab.id.slice(4) : undefined;
  const projectIds = new Set(projects.map((project) => project.id));
  return inferredProjectId && projectIds.has(inferredProjectId) ? inferredProjectId : undefined;
}

function stateProjectTabsToWorkspaceTabs(
  tabs: StateProjectTab[],
  projects: StateProject[],
  activeWorkspaceSurface: string,
): WorkspaceTab[] {
  if (tabs.length === 0) {
    return fallbackWorkspaceTabs.map((label) => ({
      id: label,
      label,
      active: label === activeWorkspaceSurface,
    }));
  }
  return tabs.map((tab) => {
    return {
      id: tab.id,
      label: tab.label,
      active: tab.active,
      projectId: projectIdFromStateProjectTab(tab, projects),
    };
  });
}

function stateSessionToTerminalSession(session: StateSession): TerminalSession {
  const attentionState = session.attention_state ?? "none";
  const status = session.state === "completed" ? "missing" : attentionState === "none" ? "ready" : "warning";
  const totalTokens = session.token_usage
    ? session.token_usage.total_tokens ?? session.token_usage.input_tokens + session.token_usage.output_tokens
    : undefined;
  const tokenUsageLine = session.token_usage
    ? `tokens: ${totalTokens} · cost: ${formatUsd(session.token_usage.cost_usd)}`
    : undefined;
  return createTerminalSession({
    id: session.id,
    ptyId: session.pane_id ?? session.id,
    title: session.title,
    cwd: session.cwd ?? "",
    branch: session.branch ?? "",
    status,
    renderer: {
      kind: "webgl",
      degraded: false,
    },
    seedLines: [
      `${session.mode} · ${session.state}`,
      `${session.cwd ?? ""} ${session.branch ?? ""}`.trim(),
      `attention: ${attentionState}`,
      `budget: ${session.token_budget_state ?? "unknown"}`,
      tokenUsageLine,
    ].filter((line): line is string => Boolean(line && line.length > 0)),
  });
}

function formatSessionTokenUsage(session: StateSession): string {
  if (!session.token_usage) return "";
  const totalTokens = session.token_usage.total_tokens ?? session.token_usage.input_tokens + session.token_usage.output_tokens;
  return ` · ${totalTokens} tokens · ${formatUsd(session.token_usage.cost_usd)}`;
}

function formatSessionStackMetadata(session: StateSession): string | undefined {
  const parts = [
    session.cwd,
    session.branch ? `branch ${session.branch}` : undefined,
    session.task_id ? `task ${session.task_id}` : undefined,
    session.run_id ? `run ${session.run_id}` : undefined,
    session.agent_profile_id ? `agent ${session.agent_profile_id}` : undefined,
    session.ports?.length ? `ports ${session.ports.join(", ")}` : undefined,
    session.updated_at ? `heartbeat ${session.updated_at}` : undefined,
  ].filter((part): part is string => Boolean(part && part.trim()));
  return parts.length ? parts.join(" · ") : undefined;
}

function redactTerminalTranscriptPreview(value: string): string {
  return value
    .replace(/\b([A-Z0-9_]*(?:API_KEY|TOKEN|SECRET|PASSWORD))=([^\s]+)/gi, "$1=[redacted]")
    .replace(/\bsk-[A-Za-z0-9_-]{8,}\b/g, "sk-[redacted]");
}

function formatTerminalTranscriptChunk(chunk: NativeTerminalStreamChunk): string {
  const firstLine = redactTerminalTranscriptPreview(chunk.body).trim().split(/\r?\n/).find(Boolean);
  return `transcript ${chunk.seq_start}-${chunk.seq_end} · ${firstLine || chunk.artifact_path}`;
}

function formatPtyCommandCapture(capture: PtyCommandCapture): string {
  return `Capture exit ${capture.exitCode} · ${capture.exitSuccess ? "success" : "failed"}`;
}

function splitTerminalCaptureArgs(value: string): string[] {
  return value.trim().split(/\s+/).filter(Boolean);
}

function mergeStateSessionsIntoTerminalDeck(sessions: StateSession[], localSessions: TerminalSession[]): TerminalSession[] {
  if (sessions.length === 0) return localSessions;
  return sessions.map((session) => {
    const base = stateSessionToTerminalSession(session);
    const local = localSessions.find((candidate) => candidate.id === session.id || candidate.ptyId === base.ptyId);
    if (!local) return base;
    return {
      ...base,
      ptyId: local.ptyId ?? base.ptyId,
      renderer: local.renderer.degraded ? local.renderer : base.renderer,
      lines: local.lines.length > base.lines.length ? local.lines : base.lines,
    };
  });
}

function nativeSessionToStateSession(session: NativeSession): StateSession {
  return {
    id: session.id,
    project_id: session.project_id,
    pane_id: session.pane_id ?? session.id,
    mode: session.mode,
    title: session.title,
    cwd: session.cwd ?? "",
    branch: session.branch ?? "",
    agent_profile_id: session.agent_profile_id,
    task_id: session.task_id,
    run_id: session.run_id,
    state: session.state,
    attention_state: session.attention_state,
    token_budget_state: session.token_budget_state,
    created_at: session.created_at,
    updated_at: session.updated_at,
  };
}

function renderMarkdownPreview(markdown: string) {
  const nodes: ReactNode[] = [];
  let listItems: string[] = [];
  const flushList = () => {
    if (listItems.length === 0) return;
    const items = listItems;
    listItems = [];
    nodes.push(
      <ul key={`list-${nodes.length}`}>
        {items.map((item, index) => (
          <li key={`${item}-${index}`}>{item}</li>
        ))}
      </ul>,
    );
  };

  markdown.split(/\r?\n/).forEach((rawLine, index) => {
    const line = rawLine.trim();
    if (!line) {
      flushList();
      return;
    }
    if (line.startsWith("## ")) {
      flushList();
      nodes.push(<h2 key={`h2-${index}`}>{line.slice(3).trim()}</h2>);
      return;
    }
    if (line.startsWith("# ")) {
      flushList();
      nodes.push(<h1 key={`h1-${index}`}>{line.slice(2).trim()}</h1>);
      return;
    }
    if (line.startsWith("- ")) {
      listItems.push(line.slice(2).trim());
      return;
    }
    flushList();
    nodes.push(<p key={`p-${index}`}>{line}</p>);
  });
  flushList();

  return nodes.length > 0 ? nodes : <p>No workpad notes</p>;
}

function contextPackSources(pack: NativeContextPack): Array<{ type?: string; id?: string; path?: string }> {
  const sources = (pack.sources_json as { sources?: unknown[] } | undefined)?.sources;
  if (!Array.isArray(sources)) return [];
  return sources.filter((source): source is { type?: string; id?: string; path?: string } =>
    typeof source === "object" && source !== null,
  );
}

function contextPackBudgetLabel(pack: NativeContextPack): string {
  const hint = (pack.sources_json as { budget?: { max_tokens_hint?: unknown } } | undefined)?.budget?.max_tokens_hint;
  return typeof hint === "number" ? `${hint} tokens` : "no token hint";
}

function projectQuickPreviewKind(file: NativeProjectFilePreview) {
  const path = file.path.toLowerCase();
  if (file.language === "markdown" || path.endsWith(".md") || path.endsWith(".markdown")) return "markdown";
  if (file.language === "html" || path.endsWith(".html") || path.endsWith(".htm")) return "html";
  if (file.language === "log" || path.endsWith(".log")) return "log";
  if (file.language === "json" || path.endsWith(".json")) return "json";
  if (file.language === "yaml" || path.endsWith(".yaml") || path.endsWith(".yml")) return "yaml";
  if (file.language === "pdf" || path.endsWith(".pdf")) return "pdf";
  if (file.language === "image" || /\.(png|jpe?g|gif|webp|avif|bmp|svg)$/.test(path)) return "image";
  return "text";
}

function renderProjectQuickPreview(file: NativeProjectFilePreview) {
  const kind = projectQuickPreviewKind(file);
  if (kind === "markdown") {
    return <div className="hc-quick-preview-markdown">{renderMarkdownPreview(file.body)}</div>;
  }
  if (kind === "log") {
    const lines = file.body.split(/\r?\n/).filter((line, index, all) => line.length > 0 || index < all.length - 1);
    return (
      <ol className="hc-log-preview">
        {lines.slice(0, 80).map((line, index) => (
          <li key={`${index}-${line}`}>
            <span>{index + 1}</span>
            <code>{line}</code>
          </li>
        ))}
      </ol>
    );
  }
  if (kind === "html") {
    return (
      <iframe
        className="hc-html-preview-frame"
        title={`HTML preview ${file.path}`}
        sandbox=""
        srcDoc={file.body}
      />
    );
  }
  if (kind === "image") {
    return <img className="hc-image-preview" src={file.body} alt={`Image preview ${file.path}`} />;
  }
  if (kind === "pdf") {
    return <iframe className="hc-pdf-preview-frame" title={`PDF preview ${file.path}`} src={file.body} />;
  }
  if (kind === "json") {
    return <pre className="hc-structured-preview">{formatJsonPreview(file.body)}</pre>;
  }
  if (kind === "yaml") {
    return <pre className="hc-structured-preview">{file.body}</pre>;
  }
  return <pre>{file.body}</pre>;
}

function shouldRenderMonacoEditor(file: NativeProjectFilePreview) {
  const kind = projectQuickPreviewKind(file);
  return kind !== "pdf" && kind !== "image" && kind !== "log";
}

function formatJsonPreview(body: string) {
  try {
    return JSON.stringify(JSON.parse(body), null, 2);
  } catch {
    return body;
  }
}

function normalizeLocalhostPreviewUrl(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    return { error: "Enter a localhost URL" };
  }
  try {
    const url = new URL(trimmed);
    const hostname = url.hostname.toLowerCase();
    const allowedHost = hostname === "localhost" || hostname === "127.0.0.1" || hostname === "[::1]" || hostname === "::1";
    if ((url.protocol !== "http:" && url.protocol !== "https:") || !allowedHost) {
      return { error: "Only localhost HTTP(S) URLs are allowed" };
    }
    return { url: url.toString() };
  } catch {
    return { error: "Enter a valid localhost URL" };
  }
}

interface TaskSavedView {
  id: string;
  label: string;
  query: string;
}

function taskSavedViewsKey(projectId: string) {
  return `haneulchi:task-views:${projectId}`;
}

function createTaskSavedView(query: string): TaskSavedView | undefined {
  const normalizedQuery = query.trim();
  if (!normalizedQuery) return undefined;
  const id = `view_${normalizedQuery.toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "") || "filter"}`;
  return {
    id,
    label: normalizedQuery,
    query: normalizedQuery,
  };
}

type ReleaseChannel = "stable" | "beta";
type UpdateFeedSignatureState = "blocked" | "missing" | "signed";

interface UpdateFeedCheck {
  channel: ReleaseChannel;
  version: string;
  platform: string;
  url?: string;
  signatureState: UpdateFeedSignatureState;
}

const releaseChannelStorageKey = "haneulchi:release-channel";
const terminalThemeStorageKey = "haneulchi:terminal-theme";
const providerModelStorageKey = "haneulchi:provider-model";
const projectTabGroupsStorageKey = "haneulchi:project-tab-groups";
const projectDetachPlansStorageKey = "haneulchi:project-detach-plans";
const projectLayoutPresetsStorageKey = "haneulchi:project-layout-presets";

interface TerminalTheme {
  name: string;
  background: string;
  foreground: string;
  accent: string;
}

interface ProviderModelSettings {
  provider: string;
  model: string;
  agentProfileId: string;
}

const defaultProviderModelSettings: ProviderModelSettings = {
  provider: "openai",
  model: "gpt-5.4",
  agentProfileId: "agent_codex",
};

function loadReleaseChannel(): ReleaseChannel {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return "stable";
  const stored = window.localStorage.getItem(releaseChannelStorageKey);
  return stored === "beta" ? "beta" : "stable";
}

function saveReleaseChannel(channel: ReleaseChannel) {
  if (typeof window === "undefined" || typeof window.localStorage?.setItem !== "function") return;
  window.localStorage.setItem(releaseChannelStorageKey, channel);
}

function isHexColor(value: unknown): value is string {
  return typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value.trim());
}

function parseTerminalTheme(value: string): TerminalTheme {
  const parsed = JSON.parse(value) as Partial<TerminalTheme>;
  if (!parsed.name?.trim()) throw new Error("theme name is required");
  if (!isHexColor(parsed.background)) throw new Error("theme background must be a #rrggbb color");
  if (!isHexColor(parsed.foreground)) throw new Error("theme foreground must be a #rrggbb color");
  if (!isHexColor(parsed.accent)) throw new Error("theme accent must be a #rrggbb color");
  return {
    name: parsed.name.trim(),
    background: parsed.background.trim(),
    foreground: parsed.foreground.trim(),
    accent: parsed.accent.trim(),
  };
}

function loadTerminalTheme(): TerminalTheme | undefined {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return undefined;
  const stored = window.localStorage.getItem(terminalThemeStorageKey);
  if (!stored) return undefined;
  try {
    return parseTerminalTheme(stored);
  } catch {
    return undefined;
  }
}

function projectTerminalThemeStorageKey(projectId: string): string {
  return `${terminalThemeStorageKey}:${projectId}`;
}

function loadProjectTerminalTheme(projectId: string): TerminalTheme | undefined {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return undefined;
  const stored = window.localStorage.getItem(projectTerminalThemeStorageKey(projectId));
  if (!stored) return undefined;
  try {
    return parseTerminalTheme(stored);
  } catch {
    return undefined;
  }
}

function terminalThemeToJson(theme: TerminalTheme) {
  return JSON.stringify(theme, null, 2);
}

function terminalThemeStyle(theme: TerminalTheme | undefined): CSSProperties | undefined {
  if (!theme) return undefined;
  return {
    "--hc-terminal-bg": theme.background,
    "--hc-text-terminal": theme.foreground,
    "--hc-accent-primary": theme.accent,
  } as CSSProperties;
}

function nativeTerminalThemeToTerminalTheme(theme: NativeTerminalThemeSettings): TerminalTheme {
  return {
    name: theme.name,
    background: theme.background,
    foreground: theme.foreground,
    accent: theme.accent,
  };
}

function sshSessionCwd(target: string, remotePath: string) {
  const normalizedTarget = target.trim();
  const normalizedPath = remotePath.trim() || "~";
  const path = normalizedPath.startsWith("/") ? normalizedPath : `/${normalizedPath}`;
  return `ssh://${normalizedTarget}${path}`;
}

function shellQuote(value: string) {
  return `'${value.replace(/'/g, `'\\''`)}'`;
}

function parseSshSessionCwd(cwd: string): { target: string; remotePath: string } | undefined {
  if (!cwd.startsWith("ssh://")) return undefined;
  const withoutScheme = cwd.slice("ssh://".length);
  const pathStart = withoutScheme.indexOf("/");
  const target = (pathStart === -1 ? withoutScheme : withoutScheme.slice(0, pathStart)).trim();
  const rawPath = pathStart === -1 ? "~" : withoutScheme.slice(pathStart);
  if (!target) return undefined;
  const remotePath = rawPath === "/~" ? "~" : rawPath.startsWith("/~/") ? `~/${rawPath.slice(3)}` : rawPath || "~";
  return { target, remotePath };
}

function terminalPtyRequestForSession(session: TerminalSession): SpawnTerminalPtyRequest {
  const ssh = parseSshSessionCwd(session.cwd);
  if (ssh) {
    return {
      title: session.title,
      command: "ssh",
      args: ["-t", ssh.target, `cd ${shellQuote(ssh.remotePath)} && exec \${SHELL:-sh} -l`],
      cols: 100,
      rows: 30,
    };
  }
  return {
    title: session.title,
    command: "sh",
    args: ["-lc", "printf 'live PTY connected\\r\\n'; sleep 1; printf 'stream complete\\r\\n'"],
    cols: 100,
    rows: 30,
  };
}

function parseProviderModelSettings(value: string): ProviderModelSettings {
  const parsed = JSON.parse(value) as Partial<ProviderModelSettings>;
  const provider = parsed.provider?.trim();
  const model = parsed.model?.trim();
  const agentProfileId = parsed.agentProfileId?.trim();
  if (!provider) throw new Error("provider is required");
  if (!model) throw new Error("model is required");
  if (!agentProfileId) throw new Error("agent profile is required");
  return { provider, model, agentProfileId };
}

function loadProviderModelSettings(): ProviderModelSettings {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return defaultProviderModelSettings;
  const stored = window.localStorage.getItem(providerModelStorageKey);
  if (!stored) return defaultProviderModelSettings;
  try {
    return parseProviderModelSettings(stored);
  } catch {
    return defaultProviderModelSettings;
  }
}

function saveProviderModelSettings(settings: ProviderModelSettings) {
  if (typeof window === "undefined" || typeof window.localStorage?.setItem !== "function") return;
  window.localStorage.setItem(providerModelStorageKey, JSON.stringify(settings));
}

function providerModelSettingsFromSnapshot(snapshot: StateSnapshot): ProviderModelSettings {
  const providerModel = snapshot.provider_model ?? {
    provider: defaultProviderModelSettings.provider,
    model: defaultProviderModelSettings.model,
    agent_profile_id: defaultProviderModelSettings.agentProfileId,
  };
  return {
    provider: providerModel.provider,
    model: providerModel.model,
    agentProfileId: providerModel.agent_profile_id,
  };
}

function parseProjectTabGroups(value: string): ProjectTabGroupAssignment[] {
  const parsed = JSON.parse(value) as Partial<ProjectTabGroupAssignment>[];
  if (!Array.isArray(parsed)) return [];
  return parsed
    .map((item) => ({
      projectId: item.projectId?.trim() ?? "",
      group: item.group?.trim() ?? "",
    }))
    .filter((item) => item.projectId && item.group);
}

function loadProjectTabGroups(): ProjectTabGroupAssignment[] {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return [];
  const stored = window.localStorage.getItem(projectTabGroupsStorageKey);
  if (!stored) return [];
  try {
    return parseProjectTabGroups(stored);
  } catch {
    return [];
  }
}

function saveProjectTabGroups(groups: ProjectTabGroupAssignment[]) {
  if (typeof window === "undefined" || typeof window.localStorage?.setItem !== "function") return;
  window.localStorage.setItem(projectTabGroupsStorageKey, JSON.stringify(groups));
}

function parseProjectDetachPlans(value: string): ProjectDetachPlan[] {
  const parsed = JSON.parse(value) as Partial<ProjectDetachPlan>[];
  if (!Array.isArray(parsed)) return [];
  return parsed
    .map((item) => ({
      projectId: item.projectId?.trim() ?? "",
      projectName: item.projectName?.trim() ?? "",
      windowId: item.windowId?.trim() ?? "",
      status: item.status === "planned" ? "planned" as const : undefined,
    }))
    .filter((item): item is ProjectDetachPlan => Boolean(item.projectId && item.projectName && item.windowId && item.status));
}

function loadProjectDetachPlans(): ProjectDetachPlan[] {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return [];
  const stored = window.localStorage.getItem(projectDetachPlansStorageKey);
  if (!stored) return [];
  try {
    return parseProjectDetachPlans(stored);
  } catch {
    return [];
  }
}

function saveProjectDetachPlans(plans: ProjectDetachPlan[]) {
  if (typeof window === "undefined" || typeof window.localStorage?.setItem !== "function") return;
  window.localStorage.setItem(projectDetachPlansStorageKey, JSON.stringify(plans));
}

function isProjectTabLayout(value: unknown): value is ProjectTabLayout {
  if (!value || typeof value !== "object") return false;
  const layout = value as Partial<ProjectTabLayout>;
  return (
    (layout.mode === "grid" || layout.mode === "maximized") &&
    (typeof layout.focusedSessionId === "string" || layout.focusedSessionId === null) &&
    (typeof layout.maximizedSessionId === "string" || layout.maximizedSessionId === null) &&
    Array.isArray(layout.panes) &&
    layout.panes.every((pane) => typeof pane === "string")
  );
}

function parseProjectLayoutPresets(value: string): ProjectLayoutPreset[] {
  const parsed = JSON.parse(value) as Partial<ProjectLayoutPreset>[];
  if (!Array.isArray(parsed)) return [];
  return parsed
    .map((item) => ({
      id: item.id?.trim() ?? "",
      projectId: item.projectId?.trim() ?? "",
      name: item.name?.trim() ?? "",
      layoutJson: item.layoutJson,
    }))
    .filter((item): item is ProjectLayoutPreset => Boolean(item.id && item.projectId && item.name && isProjectTabLayout(item.layoutJson)));
}

function loadProjectLayoutPresets(): ProjectLayoutPreset[] {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return [];
  const stored = window.localStorage.getItem(projectLayoutPresetsStorageKey);
  if (!stored) return [];
  try {
    return parseProjectLayoutPresets(stored);
  } catch {
    return [];
  }
}

function saveProjectLayoutPresets(presets: ProjectLayoutPreset[]): string | undefined {
  if (typeof window === "undefined" || typeof window.localStorage?.setItem !== "function") return undefined;
  try {
    window.localStorage.setItem(projectLayoutPresetsStorageKey, JSON.stringify(presets));
    return undefined;
  } catch (error) {
    return error instanceof Error ? error.message : "local storage unavailable";
  }
}

function nativeProjectLayoutPresetToProjectLayoutPreset(preset: NativeProjectLayoutPreset): ProjectLayoutPreset {
  return {
    id: preset.id,
    projectId: preset.project_id,
    name: preset.name,
    layoutJson: preset.layout_json,
  };
}

function upsertProjectLayoutPreset(presets: ProjectLayoutPreset[], preset: ProjectLayoutPreset): ProjectLayoutPreset[] {
  const presetName = preset.name.toLowerCase();
  return [
    ...presets.filter((item) => !(item.projectId === preset.projectId && (item.id === preset.id || item.name.toLowerCase() === presetName))),
    preset,
  ];
}

function mergeProjectLayoutPresets(
  presets: ProjectLayoutPreset[],
  projectId: string,
  loadedPresets: ProjectLayoutPreset[],
): ProjectLayoutPreset[] {
  const loadedNames = new Set(loadedPresets.map((preset) => preset.name.toLowerCase()));
  return [
    ...presets.filter((item) => !(item.projectId === projectId && loadedNames.has(item.name.toLowerCase()))),
    ...loadedPresets,
  ];
}

function providerTokenUsageAdapter(provider: string): string {
  return provider === "openai" ? "openai.responses" : "local.usage-json";
}

function providerModelPayload(settings: ProviderModelSettings): string {
  return JSON.stringify(
    {
      provider: settings.provider,
      model: settings.model,
      input_tokens: 0,
      output_tokens: 0,
    },
    null,
    2,
  );
}

function loadTaskSavedViews(projectId: string): TaskSavedView[] {
  if (typeof window === "undefined" || typeof window.localStorage?.getItem !== "function") return [];
  try {
    const parsed = JSON.parse(window.localStorage.getItem(taskSavedViewsKey(projectId)) ?? "[]") as TaskSavedView[];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (view) =>
        view &&
        typeof view.id === "string" &&
        typeof view.label === "string" &&
        typeof view.query === "string" &&
        view.id.trim() &&
        view.label.trim() &&
        view.query.trim(),
    );
  } catch {
    return [];
  }
}

function saveTaskSavedViews(projectId: string, views: TaskSavedView[]): string | undefined {
  if (typeof window === "undefined" || typeof window.localStorage?.setItem !== "function") return undefined;
  try {
    window.localStorage.setItem(taskSavedViewsKey(projectId), JSON.stringify(views));
    return undefined;
  } catch (error) {
    return error instanceof Error ? error.message : "local storage unavailable";
  }
}

const gitStatusBadgeLabels: Record<string, string> = {
  added: "A",
  modified: "M",
  deleted: "D",
  renamed: "R",
  untracked: "?",
  changed: "C",
};

const diffableGitStatuses = new Set(["added", "modified", "deleted", "renamed", "changed"]);

function App() {
  const [readinessSnapshot, setReadinessSnapshot] = useState<ReadinessSnapshot>(fallbackReadinessSnapshot);
  const [ptySnapshot, setPtySnapshot] = useState<TerminalPtySnapshot>(fallbackTerminalPtySnapshot);
  const [terminalDeckSessions, setTerminalDeckSessions] = useState<TerminalSession[]>(terminalSessions);
  const [activeWorkspaceSurface, setActiveWorkspaceSurface] = useState("Terminal Deck");
  const [commandBlockState, setCommandBlockState] = useState<CommandBlockState>(createCommandBlockState());
  const [commandBlockQuery, setCommandBlockQuery] = useState("");
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [commandPaletteQuery, setCommandPaletteQuery] = useState("");
  const [focusedCommandBlockId, setFocusedCommandBlockId] = useState<string | undefined>();
  const [commandBlockExplanation, setCommandBlockExplanation] = useState<CommandBlockExplanation | undefined>();
  const [exportedCommandBlockId, setExportedCommandBlockId] = useState<string | undefined>();
  const [commandBlockPersistError, setCommandBlockPersistError] = useState<string | undefined>();
  const [commandBlockPersistStatus, setCommandBlockPersistStatus] = useState<string | undefined>();
  const [commandBlockEvidenceError, setCommandBlockEvidenceError] = useState<string | undefined>();
  const [commandBlockSearchError, setCommandBlockSearchError] = useState<string | undefined>();
  const activeProjectIdRef = useRef(localProjectId);
  const sessionItemCountRef = useRef(terminalSessions.length);
  const [focusedTerminalSessionId, setFocusedTerminalSessionId] = useState<string | undefined>();
  const [notificationJumpSessionId, setNotificationJumpSessionId] = useState<string | undefined>();
  const [notificationJumpStatus, setNotificationJumpStatus] = useState<string | undefined>();
  const [maximizedTerminalSessionId, setMaximizedTerminalSessionId] = useState<string | undefined>();
  const [sessionControlError, setSessionControlError] = useState<string | undefined>();
  const [terminalTranscriptChunksBySession, setTerminalTranscriptChunksBySession] = useState<Record<string, NativeTerminalStreamChunk[]>>({});
  const [newProjectKey, setNewProjectKey] = useState("");
  const [newProjectName, setNewProjectName] = useState("");
  const [newProjectPath, setNewProjectPath] = useState("");
  const [newProjectColor, setNewProjectColor] = useState(HC_DEFAULT_PROJECT_COLOR);
  const [copiedCommandBlockId, setCopiedCommandBlockId] = useState<string | undefined>();
  const [notificationDrawerOpen, setNotificationDrawerOpen] = useState(false);
  const [compactRightRailOpen, setCompactRightRailOpen] = useState(false);
  const [dashboardWidgets, setDashboardWidgets] = useState(defaultDashboardWidgets);
  const [workflowMarketplaceStatus, setWorkflowMarketplaceStatus] = useState<string | undefined>();
  const [workflowMarketplaceError, setWorkflowMarketplaceError] = useState<string | undefined>();
  const [workflowValidationResult, setWorkflowValidationResult] = useState<WorkflowValidationResult | undefined>();
  const [workflowRuntimeResult, setWorkflowRuntimeResult] = useState<WorkflowRuntimeState | undefined>();
  const [workflowHookRunId, setWorkflowHookRunId] = useState("run_1");
  const [workflowHookName, setWorkflowHookName] = useState("before_run");
  const [workflowHookRepoRoot, setWorkflowHookRepoRoot] = useState("");
  const [workflowHookWorkspacePath, setWorkflowHookWorkspacePath] = useState("");
  const [workflowHookResult, setWorkflowHookResult] = useState<WorkflowHookRunResult | undefined>();
  const [workflowControlError, setWorkflowControlError] = useState<string | undefined>();
  const [releaseChannel, setReleaseChannel] = useState<ReleaseChannel>(() => loadReleaseChannel());
  const [updateFeedCheck, setUpdateFeedCheck] = useState<UpdateFeedCheck | undefined>();
  const [updateFeedError, setUpdateFeedError] = useState<string | undefined>();
  const [releaseWorkflowStatus, setReleaseWorkflowStatus] = useState<NativeReleaseWorkflowStatus | undefined>();
  const [releaseWorkflowError, setReleaseWorkflowError] = useState<string | undefined>();
  const [terminalTheme, setTerminalTheme] = useState<TerminalTheme | undefined>(() => loadTerminalTheme());
  const [terminalThemeJson, setTerminalThemeJson] = useState(() => terminalThemeToJson(loadTerminalTheme() ?? HC_DEFAULT_TERMINAL_THEME));
  const [activeProjectTerminalTheme, setActiveProjectTerminalTheme] = useState<TerminalTheme | undefined>();
  const [terminalThemeStatus, setTerminalThemeStatus] = useState<string | undefined>();
  const [terminalThemeError, setTerminalThemeError] = useState<string | undefined>();
  const [sshSessionTitle, setSshSessionTitle] = useState("SSH staging");
  const [sshTarget, setSshTarget] = useState("");
  const [sshRemotePath, setSshRemotePath] = useState("~");
  const [sshBranchLabel, setSshBranchLabel] = useState("");
  const [sshSessionStatus, setSshSessionStatus] = useState<string | undefined>();
  const [sshSessionError, setSshSessionError] = useState<string | undefined>();
  const [providerModelSettings, setProviderModelSettings] = useState<ProviderModelSettings>(() => loadProviderModelSettings());
  const [providerModelProvider, setProviderModelProvider] = useState(() => loadProviderModelSettings().provider);
  const [providerModelName, setProviderModelName] = useState(() => loadProviderModelSettings().model);
  const [providerModelAgent, setProviderModelAgent] = useState(() => loadProviderModelSettings().agentProfileId);
  const [providerModelStatus, setProviderModelStatus] = useState<string | undefined>();
  const [providerModelError, setProviderModelError] = useState<string | undefined>();
  const [projectTabGroups, setProjectTabGroups] = useState<ProjectTabGroupAssignment[]>(() => loadProjectTabGroups());
  const [projectTabGroupDraft, setProjectTabGroupDraft] = useState("");
  const [projectDetachPlans, setProjectDetachPlans] = useState<ProjectDetachPlan[]>(() => loadProjectDetachPlans());
  const [projectLayoutPresets, setProjectLayoutPresets] = useState<ProjectLayoutPreset[]>(() => loadProjectLayoutPresets());
  const [projectLayoutPresetName, setProjectLayoutPresetName] = useState("");
  const [projectWindowStatus, setProjectWindowStatus] = useState<string | undefined>();
  const [projectControlError, setProjectControlError] = useState<string | undefined>();
  const [knowledgeSources, setKnowledgeSources] = useState<NativeKnowledgeSource[]>([]);
  const [knowledgeSourcePath, setKnowledgeSourcePath] = useState("");
  const [knowledgeSourceStatus, setKnowledgeSourceStatus] = useState<string | undefined>();
  const [knowledgeSourceError, setKnowledgeSourceError] = useState<string | undefined>();
  const [knowledgePages, setKnowledgePages] = useState<NativeKnowledgePage[]>([]);
  const [knowledgePageSlug, setKnowledgePageSlug] = useState("");
  const [knowledgePageTitle, setKnowledgePageTitle] = useState("");
  const [knowledgePageBody, setKnowledgePageBody] = useState("");
  const [knowledgePageStatus, setKnowledgePageStatus] = useState<string | undefined>();
  const [knowledgePageError, setKnowledgePageError] = useState<string | undefined>();
  const [knowledgeConcepts, setKnowledgeConcepts] = useState<NativeKnowledgeConcept[]>([]);
  const [knowledgeConceptError, setKnowledgeConceptError] = useState<string | undefined>();
  const [knowledgeObsidianExport, setKnowledgeObsidianExport] = useState<NativeKnowledgeObsidianExport | undefined>();
  const [knowledgeObsidianExportError, setKnowledgeObsidianExportError] = useState<string | undefined>();
  const [knowledgeQuestion, setKnowledgeQuestion] = useState("");
  const [knowledgeChatContextPackId, setKnowledgeChatContextPackId] = useState("");
  const [knowledgeChatAnswer, setKnowledgeChatAnswer] = useState<NativeKnowledgeChatAnswer | undefined>();
  const [knowledgeChatError, setKnowledgeChatError] = useState<string | undefined>();
  const [knowledgeExplorations, setKnowledgeExplorations] = useState<NativeKnowledgeExploration[]>([]);
  const [knowledgeExplorationTitle, setKnowledgeExplorationTitle] = useState("");
  const [knowledgeExplorationQuestion, setKnowledgeExplorationQuestion] = useState("");
  const [knowledgeExplorationAnswer, setKnowledgeExplorationAnswer] = useState("");
  const [knowledgeExplorationPageId, setKnowledgeExplorationPageId] = useState("");
  const [knowledgeExplorationContextPackId, setKnowledgeExplorationContextPackId] = useState("");
  const [knowledgeExplorationStatus, setKnowledgeExplorationStatus] = useState<string | undefined>();
  const [knowledgeExplorationError, setKnowledgeExplorationError] = useState<string | undefined>();
  const [knowledgeAutomationWatch, setKnowledgeAutomationWatch] = useState(false);
  const [knowledgeAutomationRun, setKnowledgeAutomationRun] = useState<NativeKnowledgeAutomationRun | undefined>();
  const [knowledgeAutomationError, setKnowledgeAutomationError] = useState<string | undefined>();
  const [knowledgeLintStaleCount, setKnowledgeLintStaleCount] = useState("0");
  const [knowledgeLintGapCount, setKnowledgeLintGapCount] = useState("0");
  const [knowledgeLintContradictionCount, setKnowledgeLintContradictionCount] = useState("0");
  const [knowledgeLintBody, setKnowledgeLintBody] = useState("");
  const [knowledgeLintReport, setKnowledgeLintReport] = useState<NativeKnowledgeLintReport | undefined>();
  const [knowledgeLintError, setKnowledgeLintError] = useState<string | undefined>();
  const [knowledgeIngestionKind, setKnowledgeIngestionKind] = useState("markdown");
  const [knowledgeIngestionPath, setKnowledgeIngestionPath] = useState("");
  const [knowledgeIngestionTitle, setKnowledgeIngestionTitle] = useState("");
  const [knowledgeIngestionBody, setKnowledgeIngestionBody] = useState("");
  const [knowledgeIngestionLastPath, setKnowledgeIngestionLastPath] = useState("");
  const [knowledgeIngestionResult, setKnowledgeIngestionResult] = useState<NativeKnowledgeIngestionResult | undefined>();
  const [knowledgeIngestionError, setKnowledgeIngestionError] = useState<string | undefined>();
  const [tokenUsageAdapter, setTokenUsageAdapter] = useState("openai.responses");
  const [tokenUsageAdapterAgent, setTokenUsageAdapterAgent] = useState("");
  const [tokenUsageAdapterPayload, setTokenUsageAdapterPayload] = useState("");
  const [tokenUsageAdapterResult, setTokenUsageAdapterResult] = useState<NativeTokenUsage | undefined>();
  const [tokenUsageAdapterError, setTokenUsageAdapterError] = useState<string | undefined>();
  const [manualTokenUsageProvider, setManualTokenUsageProvider] = useState("openai");
  const [manualTokenUsageModel, setManualTokenUsageModel] = useState("gpt-5.4");
  const [manualTokenUsageAgent, setManualTokenUsageAgent] = useState("");
  const [manualTokenUsageSession, setManualTokenUsageSession] = useState("");
  const [manualTokenUsageTask, setManualTokenUsageTask] = useState("");
  const [manualTokenUsageRun, setManualTokenUsageRun] = useState("");
  const [manualTokenUsageInputTokens, setManualTokenUsageInputTokens] = useState("");
  const [manualTokenUsageOutputTokens, setManualTokenUsageOutputTokens] = useState("");
  const [manualTokenUsageCostUsd, setManualTokenUsageCostUsd] = useState("");
  const [manualTokenUsageSource, setManualTokenUsageSource] = useState("manual");
  const [tokenUsageRecordResult, setTokenUsageRecordResult] = useState<NativeTokenUsage | undefined>();
  const [tokenUsageRecordError, setTokenUsageRecordError] = useState<string | undefined>();
  const [budgetSummaryResult, setBudgetSummaryResult] = useState<NativeBudgetSummary | undefined>();
  const [budgetForecastResult, setBudgetForecastResult] = useState<NativeBudgetForecast | undefined>();
  const [providerPrices, setProviderPrices] = useState<NativeProviderPrice[]>([]);
  const [providerPriceUpdateSource, setProviderPriceUpdateSource] = useState("local-fixture");
  const [providerPriceUpdatePayload, setProviderPriceUpdatePayload] = useState("");
  const [providerPriceUpdateResult, setProviderPriceUpdateResult] = useState<NativeProviderPriceUpdateSummary | undefined>();
  const [budgetScopeType, setBudgetScopeType] = useState<BudgetScopeType>("project");
  const [budgetScopeId, setBudgetScopeId] = useState(localProjectId);
  const [budgetMaxUsd, setBudgetMaxUsd] = useState("20");
  const [budgetWarnPct, setBudgetWarnPct] = useState("0.8");
  const [budgetHardLimit, setBudgetHardLimit] = useState(true);
  const [budgetSetResult, setBudgetSetResult] = useState<NativeBudget | undefined>();
  const [budgetWorkflowError, setBudgetWorkflowError] = useState<string | undefined>();
  const [releaseGateRun, setReleaseGateRun] = useState<NativeReleaseGateRun | undefined>();
  const [releaseGateHistory, setReleaseGateHistory] = useState<NativeReleaseGateRun[]>([]);
  const [releaseGateError, setReleaseGateError] = useState<string | undefined>();
  const [terminalSmokeRun, setTerminalSmokeRun] = useState<NativeTerminalFidelitySmokeRun | undefined>();
  const [terminalSmokeHistory, setTerminalSmokeHistory] = useState<NativeTerminalFidelitySmokeRun[]>([]);
  const [terminalSmokeError, setTerminalSmokeError] = useState<string | undefined>();
  const [taskLifecycleRun, setTaskLifecycleRun] = useState<NativeTaskLifecycleE2ERun | undefined>();
  const [taskLifecycleHistory, setTaskLifecycleHistory] = useState<NativeTaskLifecycleE2ERun[]>([]);
  const [taskLifecycleError, setTaskLifecycleError] = useState<string | undefined>();
  const [workflowNegativeRun, setWorkflowNegativeRun] = useState<NativeWorkflowNegativeTestRun | undefined>();
  const [workflowNegativeHistory, setWorkflowNegativeHistory] = useState<NativeWorkflowNegativeTestRun[]>([]);
  const [workflowNegativeError, setWorkflowNegativeError] = useState<string | undefined>();
  const [dmgSmokePath, setDmgSmokePath] = useState("");
  const [dmgSmokeAppBundlePath, setDmgSmokeAppBundlePath] = useState("");
  const [dmgSmokeRun, setDmgSmokeRun] = useState<NativeDmgSmokeRun | undefined>();
  const [dmgSmokeHistory, setDmgSmokeHistory] = useState<NativeDmgSmokeRun[]>([]);
  const [dmgSmokeError, setDmgSmokeError] = useState<string | undefined>();
  const [recoveryDrillRun, setRecoveryDrillRun] = useState<NativeRecoveryDrillRun | undefined>();
  const [recoveryDrillHistory, setRecoveryDrillHistory] = useState<NativeRecoveryDrillRun[]>([]);
  const [recoveryDrillError, setRecoveryDrillError] = useState<string | undefined>();
  const [benchmarkRun, setBenchmarkRun] = useState<NativeBenchmarkRun | undefined>();
  const [benchmarkHistory, setBenchmarkHistory] = useState<NativeBenchmarkRun[]>([]);
  const [benchmarkError, setBenchmarkError] = useState<string | undefined>();
  const [dogfoodReview, setDogfoodReview] = useState<NativeDogfoodTelemetryReview | undefined>();
  const [dogfoodReviewHistory, setDogfoodReviewHistory] = useState<NativeDogfoodTelemetryReview[]>([]);
  const [dogfoodReviewError, setDogfoodReviewError] = useState<string | undefined>();
  const [visualHarnessSource, setVisualHarnessSource] = useState("");
  const [visualHarnessTarget, setVisualHarnessTarget] = useState("");
  const [visualHarnessKind, setVisualHarnessKind] = useState("context");
  const [visualHarnessLink, setVisualHarnessLink] = useState<NativeVisualHarnessLink | undefined>();
  const [visualHarnessError, setVisualHarnessError] = useState<string | undefined>();
  const [trackerProvider, setTrackerProvider] = useState<NativeTrackerProvider>("linear");
  const [trackerLocalKind, setTrackerLocalKind] = useState<NativeTrackerLocalKind>("task");
  const [trackerLocalId, setTrackerLocalId] = useState("");
  const [trackerExternalId, setTrackerExternalId] = useState("");
  const [trackerExternalUrl, setTrackerExternalUrl] = useState("");
  const [trackerSyncMode, setTrackerSyncMode] = useState<NativeTrackerSyncMode>("mirror");
  const [trackerDryRun, setTrackerDryRun] = useState(true);
  const [trackerBinding, setTrackerBinding] = useState<NativeExternalTrackerBinding | undefined>();
  const [trackerSyncRun, setTrackerSyncRun] = useState<NativeExternalTrackerSyncRun | undefined>();
  const [trackerError, setTrackerError] = useState<string | undefined>();
  const [secretName, setSecretName] = useState("");
  const [secretValue, setSecretValue] = useState("");
  const [secretResult, setSecretResult] = useState<NativeSecretMetadata | undefined>();
  const [secretInventory, setSecretInventory] = useState<NativeSecretMetadata[]>([]);
  const [secretError, setSecretError] = useState<string | undefined>();
  const [policyPackName, setPolicyPackName] = useState("");
  const [policyPackSandboxMode, setPolicyPackSandboxMode] = useState("ask-before-write");
  const [policyPackNetwork, setPolicyPackNetwork] = useState("ask");
  const [policyPackNetworkProfile, setPolicyPackNetworkProfile] = useState("internet");
  const [policyPackFileWrite, setPolicyPackFileWrite] = useState("ask");
  const [policyPackApprovals, setPolicyPackApprovals] = useState("");
  const [policyPackForbidden, setPolicyPackForbidden] = useState("");
  const [policyPackResult, setPolicyPackResult] = useState<NativePolicyPack | undefined>();
  const [policyPacks, setPolicyPacks] = useState<NativePolicyPack[]>([]);
  const [policyPackError, setPolicyPackError] = useState<string | undefined>();
  const [policyApprovalError, setPolicyApprovalError] = useState<string | undefined>();
  const [policyActionKind, setPolicyActionKind] = useState("network");
  const [policyActionCommand, setPolicyActionCommand] = useState("curl http://127.0.0.1:3000");
  const [policyApprovalRiskLevel, setPolicyApprovalRiskLevel] = useState("high");
  const [policyEvaluation, setPolicyEvaluation] = useState<NativePolicyActionEvaluation | undefined>();
  const [permissionAuditDecision, setPermissionAuditDecision] = useState("forbidden");
  const [permissionAuditActionKind, setPermissionAuditActionKind] = useState("network");
  const [permissionAuditRunId, setPermissionAuditRunId] = useState("");
  const [permissionAuditTaskId, setPermissionAuditTaskId] = useState("");
  const [permissionAuditEvents, setPermissionAuditEvents] = useState<NativePermissionAudit[]>([]);
  const [permissionAuditError, setPermissionAuditError] = useState<string | undefined>();
  const [contextPacks, setContextPacks] = useState<NativeContextPack[]>([]);
  const [contextPackName, setContextPackName] = useState("");
  const [contextPackDescription, setContextPackDescription] = useState("");
  const [contextPackSourceId, setContextPackSourceId] = useState("");
  const [contextPackMaxTokens, setContextPackMaxTokens] = useState("");
  const [contextPackStatus, setContextPackStatus] = useState<string | undefined>();
  const [contextPackError, setContextPackError] = useState<string | undefined>();
  const [nativeSkillPacks, setNativeSkillPacks] = useState<NativeSkillPack[]>([]);
  const [skillPackName, setSkillPackName] = useState("");
  const [skillPackDescription, setSkillPackDescription] = useState("");
  const [skillPackSkillsJson, setSkillPackSkillsJson] = useState("[\"code-review\"]");
  const [skillPackContextPackId, setSkillPackContextPackId] = useState("");
  const [skillPackStatus, setSkillPackStatus] = useState<string | undefined>();
  const [skillPackError, setSkillPackError] = useState<string | undefined>();
  const [evidencePack, setEvidencePack] = useState<EvidencePack>(() => loadEvidencePack("ev_local"));
  const [stateSnapshot, setStateSnapshot] = useState<StateSnapshot>(fallbackStateSnapshot);
  const [projectFileList, setProjectFileList] = useState<NativeProjectFileList | undefined>();
  const [projectFileSearch, setProjectFileSearch] = useState<NativeProjectFileSearch | undefined>();
  const [projectFileSearchQuery, setProjectFileSearchQuery] = useState("");
  const [projectFilePreview, setProjectFilePreview] = useState<NativeProjectFilePreview | undefined>();
  const [projectFileDraft, setProjectFileDraft] = useState("");
  const [projectFileSaveStatus, setProjectFileSaveStatus] = useState<string | undefined>();
  const [projectDiff, setProjectDiff] = useState<NativeProjectDiff | undefined>();
  const [projectLspDiagnostics, setProjectLspDiagnostics] = useState<NativeProjectLspDiagnostics | undefined>();
  const [exportedPatch, setExportedPatch] = useState<NativePatchArtifact | undefined>();
  const [importedPatch, setImportedPatch] = useState<NativePatchArtifact | undefined>();
  const [prLandingPlan, setPrLandingPlan] = useState<NativePrLandingPlan | undefined>();
  const [patchImportBody, setPatchImportBody] = useState("");
  const [prLandingTitle, setPrLandingTitle] = useState("");
  const [projectFileError, setProjectFileError] = useState<string | undefined>();
  const [projectFileSearchError, setProjectFileSearchError] = useState<string | undefined>();
  const [projectFilePreviewError, setProjectFilePreviewError] = useState<string | undefined>();
  const [projectFileSaveError, setProjectFileSaveError] = useState<string | undefined>();
  const [projectDiffError, setProjectDiffError] = useState<string | undefined>();
  const [projectLspError, setProjectLspError] = useState<string | undefined>();
  const [patchWorkflowError, setPatchWorkflowError] = useState<string | undefined>();
  const [pendingDangerousInput, setPendingDangerousInput] = useState<PendingDangerousInput | undefined>();
  const [dangerousInputError, setDangerousInputError] = useState<string | undefined>();
  const [terminalInputRecordError, setTerminalInputRecordError] = useState<string | undefined>();
  const [terminalSessionCreateError, setTerminalSessionCreateError] = useState<string | undefined>();
  const [terminalOutputListenerError, setTerminalOutputListenerError] = useState<string | undefined>();
  const [terminalCaptureCommand, setTerminalCaptureCommand] = useState("pwd");
  const [terminalCaptureArgs, setTerminalCaptureArgs] = useState("");
  const [terminalCaptureResult, setTerminalCaptureResult] = useState<PtyCommandCapture | undefined>();
  const [terminalCaptureError, setTerminalCaptureError] = useState<string | undefined>();
  const [localhostPreviewDraft, setLocalhostPreviewDraft] = useState("http://localhost:3000");
  const [localhostPreviewUrl, setLocalhostPreviewUrl] = useState<string | undefined>();
  const [localhostPreviewError, setLocalhostPreviewError] = useState<string | undefined>();
  const [browserAutomationPlan, setBrowserAutomationPlan] = useState<NativeBrowserAutomationPlan | undefined>();
  const [browserAutomationError, setBrowserAutomationError] = useState<string | undefined>();
  const [taskState, setTaskState] = useState<TaskState>(() => loadTaskState(localProjectId));
  const [taskStateProjectId, setTaskStateProjectId] = useState(localProjectId);
  const [quickTaskTitle, setQuickTaskTitle] = useState("");
  const [quickTaskError, setQuickTaskError] = useState<string | undefined>();
  const [taskLoadError, setTaskLoadError] = useState<string | undefined>();
  const [taskFilterQuery, setTaskFilterQuery] = useState("");
  const [taskViewError, setTaskViewError] = useState<string | undefined>();
  const [taskSavedViews, setTaskSavedViews] = useState<TaskSavedView[]>(() => loadTaskSavedViews(localProjectId));
  const [selectedTaskId, setSelectedTaskId] = useState<string | undefined>();
  const [taskWorkpadDraft, setTaskWorkpadDraft] = useState("");
  const [taskWorkpadMode, setTaskWorkpadMode] = useState<"edit" | "preview">("edit");
  const [taskWorkpadError, setTaskWorkpadError] = useState<string | undefined>();
  const [newTaskComment, setNewTaskComment] = useState("");
  const [taskCommentError, setTaskCommentError] = useState<string | undefined>();
  const [newTaskSubtaskTitle, setNewTaskSubtaskTitle] = useState("");
  const [taskSubtaskError, setTaskSubtaskError] = useState<string | undefined>();
  const [taskCycles, setTaskCycles] = useState<NativeTaskCycle[]>([]);
  const [taskModules, setTaskModules] = useState<NativeTaskModule[]>([]);
  const [taskPlanningStatus, setTaskPlanningStatus] = useState<string | undefined>();
  const [taskCycleDraft, setTaskCycleDraft] = useState("");
  const [taskModuleDraft, setTaskModuleDraft] = useState("");
  const [taskInitiativeDraft, setTaskInitiativeDraft] = useState("");
  const [taskLabelsDraft, setTaskLabelsDraft] = useState("");
  const [taskDueDateDraft, setTaskDueDateDraft] = useState("");
  const [taskEstimateDraft, setTaskEstimateDraft] = useState("");
  const [taskAssigneeDraft, setTaskAssigneeDraft] = useState("");
  const [taskContextPackDraft, setTaskContextPackDraft] = useState("");
  const [taskContextStatus, setTaskContextStatus] = useState<string | undefined>();
  const [taskContextError, setTaskContextError] = useState<string | undefined>();
  const [taskPlanningError, setTaskPlanningError] = useState<string | undefined>();
  const [roadmapInitiativeNameDraft, setRoadmapInitiativeNameDraft] = useState("");
  const [roadmapInitiativeStatusDraft, setRoadmapInitiativeStatusDraft] = useState("planned");
  const [roadmapInitiativeStatus, setRoadmapInitiativeStatus] = useState<string | undefined>();
  const [roadmapInitiativeError, setRoadmapInitiativeError] = useState<string | undefined>();
  const [taskDispatchError, setTaskDispatchError] = useState<string | undefined>();
  const [runLifecycleError, setRunLifecycleError] = useState<string | undefined>();
  const [runStatusDetailDrafts, setRunStatusDetailDrafts] = useState<Record<string, string>>({});
  const [runStatusUpdateDrafts, setRunStatusUpdateDrafts] = useState<Record<string, string>>({});
  const [runStatusUpdateStatus, setRunStatusUpdateStatus] = useState<string | undefined>();
  const [generatedEvidencePacksByRun, setGeneratedEvidencePacksByRun] = useState<Record<string, NativeEvidencePack>>({});
  const [evidenceGenerationError, setEvidenceGenerationError] = useState<string | undefined>();
  const [runReplayMetadataByRun, setRunReplayMetadataByRun] = useState<Record<string, PersistedRunReplayMetadata>>({});
  const [runReplayError, setRunReplayError] = useState<string | undefined>();
  const [reviewStateFilter, setReviewStateFilter] = useState("all");
  const [reviewCompletenessFilter, setReviewCompletenessFilter] = useState("all");
  const [reviewActionStatus, setReviewActionStatus] = useState<string | undefined>();
  const [reviewActionError, setReviewActionError] = useState<string | undefined>();
  const [agentAdapterId, setAgentAdapterId] = useState("agent_custom");
  const [agentAdapterName, setAgentAdapterName] = useState("Custom CLI");
  const [agentAdapterRuntime, setAgentAdapterRuntime] = useState("generic-cli");
  const [agentAdapterCommand, setAgentAdapterCommand] = useState("custom-agent");
  const [agentAdapterArgsJson, setAgentAdapterArgsJson] = useState("[]");
  const [agentAdapterEnvPolicyJson, setAgentAdapterEnvPolicyJson] = useState("{\"inherit\":true}");
  const [agentAdapterSkillsJson, setAgentAdapterSkillsJson] = useState("[]");
  const [agentAdapterStatus, setAgentAdapterStatus] = useState<string | undefined>();
  const [agentAdapterError, setAgentAdapterError] = useState<string | undefined>();
  const [agentDirectoryError, setAgentDirectoryError] = useState<string | undefined>();
  const [agentLaunchStatus, setAgentLaunchStatus] = useState<string | undefined>();
  const [agentLaunchError, setAgentLaunchError] = useState<string | undefined>();
  const [nativeRuntimePoolItems, setNativeRuntimePoolItems] = useState<NativeRuntimePoolItem[]>([]);
  const [runtimePoolStatus, setRuntimePoolStatus] = useState<string | undefined>();
  const [runtimePoolError, setRuntimePoolError] = useState<string | undefined>();
  const [agentEventAdapter, setAgentEventAdapter] = useState("raw-jsonl");
  const [agentEventProfile, setAgentEventProfile] = useState(localAgentProfileId);
  const [agentEventSession, setAgentEventSession] = useState("");
  const [agentEventRun, setAgentEventRun] = useState("");
  const [agentEventPayload, setAgentEventPayload] = useState("{\"type\":\"status\",\"status\":\"needs_input\",\"message\":\"Waiting for review\"}\n");
  const [agentEventResult, setAgentEventResult] = useState<NativeAgentEvent | undefined>();
  const [agentEventError, setAgentEventError] = useState<string | undefined>();
  const [oscEventState, setOscEventState] = useState<OscEventState>(createOscEventState());
  const quickTaskInputRef = useRef<HTMLInputElement | null>(null);
  const knowledgeSourceInputRef = useRef<HTMLInputElement | null>(null);
  const terminalTransportStateRef = useRef(createTerminalTransportState());
  const oscEventStateRef = useRef(oscEventState);
  const stateSessionsRef = useRef<StateSession[]>([]);
  const renderedTerminalDeckSessionsRef = useRef<TerminalSession[]>(terminalSessions);
  const ptySessionOwnerRef = useRef<Record<string, string>>({});
  const readinessSummary = readinessSnapshot.summary;
  const commandBlocks = listCommandBlocks(commandBlockState);
  const visibleCommandBlocks = searchCommandBlocks(commandBlockState, commandBlockQuery);
  const focusedCommandBlock = focusedCommandBlockId ? commandBlockState.blocks[focusedCommandBlockId] : undefined;
  const taskCounts = countTasksByStatus(taskState);
  const filteredTasks = filterTasks(taskState, taskFilterQuery);
  const visibleTasks = filteredTasks;
  const selectedTaskOverview = selectedTaskId ? getTaskOverview(taskState, selectedTaskId) : undefined;
  const renderedTaskRows = selectedTaskOverview ? visibleTasks.filter((task) => task.id !== selectedTaskOverview.id) : visibleTasks;
  const taskRowsByStatus = taskBoardStatuses.map((status) => ({
    status,
    tasks: renderedTaskRows.filter((task) => task.status === status),
  }));
  const controlPlaneSnapshot = mergeCommandBlocksIntoStateSnapshot(mergeTasksIntoStateSnapshot(stateSnapshot, taskState), commandBlocks);
  const trackedTaskCount = controlPlaneSnapshot.tasks.items.length;
  const runItems = controlPlaneSnapshot.runs.items;
  const reviewItems = controlPlaneSnapshot.reviews;
  const globalAttentionItems = buildGlobalAttentionQueue(controlPlaneSnapshot);
  const criticalAttentionCount = globalAttentionItems.filter((item) => item.severity === "critical").length;
  const warningAttentionCount = globalAttentionItems.filter((item) => item.severity === "warning").length;
  const filteredReviewItems = reviewItems.filter((review) => {
    const matchesState = reviewStateFilter === "all" || review.state === reviewStateFilter;
    const matchesCompleteness = reviewCompletenessFilter === "all" || review.completeness_state === reviewCompletenessFilter;
    return matchesState && matchesCompleteness;
  });
  const policyApprovalItems = controlPlaneSnapshot.attention.filter(isPolicyApprovalAttention);
  const securityState = controlPlaneSnapshot.security ?? {
    keychain: "unknown",
    secret_count: 0,
    redaction: { status: "inactive", protected_secret_count: 0 },
    permission_audit: { recent_count: 0, allowed_count: 0, approval_required_count: 0, forbidden_count: 0, latest_decision: null, latest_action_kind: null },
    diagnostics: { status: "warning", pending_policy_approvals: 0, checks: [] },
  };
  const redactionStatus = securityState.redaction?.status ?? (securityState.secret_count > 0 ? "active" : "inactive");
  const protectedSecretCount = securityState.redaction?.protected_secret_count ?? securityState.secret_count;
  const permissionAudit = securityState.permission_audit ?? {
    recent_count: 0,
    allowed_count: 0,
    approval_required_count: 0,
    forbidden_count: 0,
    latest_decision: null,
    latest_action_kind: null,
  };
  const securityDiagnostics = securityState.diagnostics ?? {
    status: controlPlaneSnapshot.health.api === "ok" && controlPlaneSnapshot.health.db === "ok" ? "ok" : "warning",
    pending_policy_approvals: policyApprovalItems.length,
    checks: [
      {
        id: "keychain",
        label: "Keychain",
        status: securityState.keychain === "unknown" || securityState.keychain === "unavailable" ? "warning" : "ok",
        detail: `${securityState.keychain} secret storage`,
      },
      {
        id: "redaction",
        label: "Secret redaction",
        status: redactionStatus === "active" || securityState.secret_count === 0 ? "ok" : "warning",
        detail: `${protectedSecretCount} protected ${protectedSecretCount === 1 ? "value" : "values"}`,
      },
      {
        id: "policy-approvals",
        label: "Policy approvals",
        status: policyApprovalItems.length === 0 ? "ok" : "warning",
        detail: `${policyApprovalItems.length} pending ${policyApprovalItems.length === 1 ? "approval" : "approvals"}`,
      },
    ],
  };
  const activePolicyPack = securityState.policy_pack ?? {};
  const agentItems = controlPlaneSnapshot.agents;
  const agentItemsById = new Map(agentItems.map((agent) => [agent.id, agent]));
  const taskDispatchBlockedLabels = new Map(
    renderedTaskRows.map((task) => {
      const assignedAgent = task.assignee ? agentItemsById.get(task.assignee) : undefined;
      const label = task.status === "ready" && assignedAgent && !assignedAgent.available
        ? `${assignedAgent.label} unavailable`
        : undefined;
      return [task.id, label] as const;
    }),
  );
  const agentTeamItems = buildAgentTeamSummaries(controlPlaneSnapshot);
  const roadmapTimelineItems = buildRoadmapTimelineItems(
    controlPlaneSnapshot.tasks.items,
    controlPlaneSnapshot.initiatives ?? [],
  );
  const calendarTaskBuckets = buildCalendarTaskBuckets(controlPlaneSnapshot.tasks.items);
  const skillPackRegistryItems = buildSkillPackRegistryItems(nativeSkillPacks, contextPacks, controlPlaneSnapshot);
  const runtimePoolItems = nativeRuntimePoolItems.length > 0
    ? nativeRuntimePoolItems.map(nativeRuntimePoolItemToRuntimePoolItem)
    : buildRuntimePoolItems(controlPlaneSnapshot);
  const availableAgentCount = agentTeamItems.filter((agent) => agent.available).length;
  const pausedAgentCount = agentTeamItems.length - availableAgentCount;
  const activeAgentRunCount = agentTeamItems.reduce((count, agent) => count + agent.runCount, 0);
  const blockedAgentRunCount = agentTeamItems.reduce((count, agent) => count + agent.blockedCount, 0);
  const recentEvidenceItems = buildRecentEvidenceActivity(controlPlaneSnapshot);
  const evidenceActivityDegraded = controlPlaneSnapshot.health.api !== "ok" || controlPlaneSnapshot.health.db !== "ok";
  const taskAssigneeOptions = [
    ...agentItems,
    ...(agentItems.some((agent) => agent.id === localAgentProfileId)
      ? []
      : [{ id: localAgentProfileId, label: "Codex", available: true }]),
  ];
  const sessionItems = controlPlaneSnapshot.sessions;
  const unreadSessionItems = sessionItems.filter((session) => session.attention_state === "unread");
  const nextUnreadSession = unreadSessionItems[0] ?? sessionItems.find((session) => (session.attention_state ?? "none") !== "none");
  const notificationHealthDegraded = controlPlaneSnapshot.health.api !== "ok";
  const sidebarProjects = stateProjectsToSidebarProjects(controlPlaneSnapshot.projects, sessionItems);
  const controlTowerProjectCards = buildControlTowerProjectCards(controlPlaneSnapshot);
  const routeVisualQaScreen = getCurrentVisualQaScreen();
  const activeWorkspaceVisualQaScreen =
    activeWorkspaceSurface === "Terminal Deck" ? undefined : visualQaScreenByLabel.get(activeWorkspaceSurface);
  const visualQaScreen = routeVisualQaScreen ?? activeWorkspaceVisualQaScreen;
  const workspaceTabs = visualQaScreen
    ? visualQaWorkspaceTabs(visualQaScreen)
    : stateProjectTabsToWorkspaceTabs(controlPlaneSnapshot.project_tabs, controlPlaneSnapshot.projects, activeWorkspaceSurface);
  const workspaceTabShortcutKey = workspaceTabs.map((tab) => `${tab.id}:${tab.projectId ?? ""}`).join("\0");
  const renderedTerminalDeckSessions = mergeStateSessionsIntoTerminalDeck(sessionItems, terminalDeckSessions);
  const terminalDeckLayoutPaneKey = renderedTerminalDeckSessions.map((session) => session.id).join("\0");
  const validFocusedTerminalSessionId = focusedTerminalSessionId && renderedTerminalDeckSessions.some((session) => [session.id, session.ptyId].includes(focusedTerminalSessionId))
    ? focusedTerminalSessionId
    : undefined;
  const validNotificationJumpSessionId = notificationJumpSessionId && renderedTerminalDeckSessions.some((session) => [session.id, session.ptyId].includes(notificationJumpSessionId))
    ? notificationJumpSessionId
    : undefined;
  const activeTerminalSessionId = validNotificationJumpSessionId ?? validFocusedTerminalSessionId ?? renderedTerminalDeckSessions[0]?.id;
  const visibleTerminalDeckSessions = maximizedTerminalSessionId
    ? renderedTerminalDeckSessions.filter((session) => session.id === maximizedTerminalSessionId)
    : renderedTerminalDeckSessions;
  const activeProjectId =
    workspaceTabs.find((tab) => tab.active && tab.projectId)?.projectId ??
    controlPlaneSnapshot.projects.find((project) => project.state === "active")?.id ??
    localProjectId;
  const missingRoadmapInitiativeKey = Array.from(new Set(
    controlPlaneSnapshot.tasks.items
      .map((task) => task.initiative_id ?? task.initiative)
      .filter((initiativeId): initiativeId is string => Boolean(initiativeId))
      .filter((initiativeId) => !(controlPlaneSnapshot.initiatives ?? []).some((initiative) => initiative.id === initiativeId)),
  )).sort().join("\0");
  const projectSessionItems = sessionItems.filter((session) =>
    session.project_id ? session.project_id === activeProjectId : activeProjectId === localProjectId,
  );
  activeProjectIdRef.current = activeProjectId;
  sessionItemCountRef.current = sessionItems.length;
  stateSessionsRef.current = sessionItems;
  renderedTerminalDeckSessionsRef.current = renderedTerminalDeckSessions;
  const activeSnapshotProject = controlPlaneSnapshot.projects.find((project) => project.id === activeProjectId);
  const activeProjectTab = controlPlaneSnapshot.project_tabs.find((tab) => tab.active);
  const activeProjectTabProjectId = activeProjectTab
    ? projectIdFromStateProjectTab(activeProjectTab, controlPlaneSnapshot.projects)
    : undefined;
  const activeProjectTabGroup = activeProjectId
    ? projectTabGroups.find((assignment) => assignment.projectId === activeProjectId)?.group ??
      controlPlaneSnapshot.project_tabs.find((tab) => tab.project_id === activeProjectId)?.group_name
    : undefined;
  const activeProjectName = activeSnapshotProject?.name ?? activeProjectTab?.label ?? "No active project";
  const activeProjectDetachPlan = activeProjectId
    ? projectDetachPlans.find((plan) => plan.projectId === activeProjectId)
    : undefined;
  const activeProjectLayoutPresets = activeProjectId
    ? projectLayoutPresets.filter((preset) => preset.projectId === activeProjectId)
    : [];
  const effectiveTerminalTheme = activeProjectTerminalTheme ?? terminalTheme;
  const projectWindowDegraded = controlPlaneSnapshot.health.api !== "ok" || controlPlaneSnapshot.health.db !== "ok";
  const runCounts = controlPlaneSnapshot.runs.counts_by_lifecycle;
  const primaryBudget =
    controlPlaneSnapshot.budgets.projects[0] ??
    controlPlaneSnapshot.budgets.goals?.[0] ??
    controlPlaneSnapshot.budgets.tasks?.[0] ??
    controlPlaneSnapshot.budgets.runs?.[0] ??
    controlPlaneSnapshot.budgets.agents[0] ??
    controlPlaneSnapshot.budgets.workspace;
  const historicalAnalytics = buildHistoricalAnalyticsSummary(controlPlaneSnapshot, primaryBudget);
  const budgetDashboardDegraded = controlPlaneSnapshot.health.db !== "ok";
  const analyticsDegraded = controlPlaneSnapshot.health.api !== "ok" || controlPlaneSnapshot.health.db !== "ok";
  const allOptionalDashboardWidgetsHidden = !dashboardWidgets.agentTeam && !dashboardWidgets.historicalAnalytics && !dashboardWidgets.recentEvidence;
  const workflowDiagnostics = controlPlaneSnapshot.workflow.diagnostics?.errors ?? [];
  const opsWorkflowIssueCount = Math.max(controlPlaneSnapshot.workflow.invalid_projects.length, workflowDiagnostics.length);
  const workflowStatusLine = `${controlPlaneSnapshot.workflow.valid ? "Valid" : "Invalid"} · current ${controlPlaneSnapshot.workflow.current_version_id ?? "none"} · LKG ${controlPlaneSnapshot.workflow.last_known_good_version_id ?? "none"}`;
  const hasKnowledgeWarnings =
    controlPlaneSnapshot.knowledge.stale_count > 0 || controlPlaneSnapshot.knowledge.gap_count > 0;
  const recentKnowledgePage = controlPlaneSnapshot.knowledge.recent_pages[0] ?? "none";
  const commandPaletteKnowledgeError = knowledgePageError?.startsWith("Markdown knowledge pages unavailable · ")
    ? knowledgePageError.replace("Markdown knowledge pages unavailable · ", "Knowledge search unavailable · ")
    : undefined;
  const taskFilterLabel = `Task filter ${filteredTasks.length} ${filteredTasks.length === 1 ? "match" : "matches"}`;
  const visibleProjectFileEntries = projectFileSearchQuery.trim() && projectFileSearch ? projectFileSearch.entries : projectFileList?.entries;
  const projectFileEntryCount = visibleProjectFileEntries?.length ?? 0;

  function applyNativeStateSnapshot(snapshot: StateSnapshot) {
    setStateSnapshot(snapshot);
    setCommandBlockState((state) =>
      mergeCommandBlockSummaries(
        state,
        snapshot.command_blocks.recent.map((block) => ({
          id: block.id,
          sessionId: block.session_id,
          command: block.command,
          status: block.status,
          seqStart: block.seq_start,
          seqEnd: block.seq_end,
          cwd: "native state snapshot",
          branch: "persisted",
        })),
      ),
    );
    const nativeProviderModel = providerModelSettingsFromSnapshot(snapshot);
    setProviderModelSettings(nativeProviderModel);
    setProviderModelProvider(nativeProviderModel.provider);
    setProviderModelName(nativeProviderModel.model);
    setProviderModelAgent(nativeProviderModel.agentProfileId);
    saveProviderModelSettings(nativeProviderModel);
  }

  function reloadNativeStateSnapshot(projectId: string) {
    return getStateSnapshot(projectId).then(applyNativeStateSnapshot);
  }

  function handleStateSnapshotReloadError(error: unknown) {
    const message = error instanceof Error ? error.message : "native state snapshot API unavailable";
    setProjectControlError(`Project state unavailable · ${message}`);
  }

  const handleLoadProjects = () => {
    setProjectControlError(undefined);
    listNativeProjects()
      .then((projects) => {
        setStateSnapshot((snapshot) => {
          const activeLoadedProjectId =
            projects.find((project) => project.status === "active")?.id ??
            snapshot.project_tabs.find((tab) => tab.active)?.project_id ??
            projects[0]?.id;
          return {
            ...snapshot,
            projects: projects.map((project) => ({
              id: project.id,
              name: project.name,
              state: project.status,
            })),
            project_tabs: projects.map((project) => {
              const existingTab = snapshot.project_tabs.find((tab) => tab.project_id === project.id);
              return {
                id: existingTab?.id ?? `tab_${project.id}`,
                project_id: project.id,
                label: project.name,
                active: project.id === activeLoadedProjectId,
                ...(existingTab?.layout_json ? { layout_json: existingTab.layout_json } : {}),
                ...(existingTab?.group_name ? { group_name: existingTab.group_name } : {}),
              };
            }),
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native project API unavailable";
        setProjectControlError(`Project list unavailable · ${message}`);
      });
  };

  const handleAddProject = () => {
    setProjectControlError(undefined);
    const nextNumber = controlPlaneSnapshot.projects.length + 1;
    const key = newProjectKey.trim() || `LOCAL${nextNumber}`;
    const name = newProjectName.trim() || `Workspace ${nextNumber}`;
    const path = newProjectPath.trim() || ".";
    const color = newProjectColor.trim() || HC_DEFAULT_PROJECT_COLOR;
    addNativeProject({
      key,
      name,
      path,
      color,
    })
      .then((project) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          projects: [
            ...snapshot.projects.map((item) => ({ ...item, state: "idle" })),
            { id: project.id, name: project.name, state: project.status },
          ],
          project_tabs: [
            ...snapshot.project_tabs.map((tab) => ({ ...tab, active: false })),
            { id: `tab_${project.id}`, project_id: project.id, label: project.name, active: true },
          ],
        }));
        setNewProjectKey("");
        setNewProjectName("");
        setNewProjectPath("");
        setNewProjectColor(HC_DEFAULT_PROJECT_COLOR);
        void reloadNativeStateSnapshot(project.id).catch(handleStateSnapshotReloadError);
      })
      .catch(handleProjectControlError);
  };

  const handleFocusProject = (project: SidebarProject) => {
    if (!project.id) return;
    setProjectControlError(undefined);
    focusNativeProject(project.id)
      .then((focusedProject) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          projects: snapshot.projects.map((item) => ({
            ...item,
            state: item.id === focusedProject.id ? focusedProject.status : "idle",
          })),
          project_tabs: snapshot.project_tabs.map((tab) => ({
            ...tab,
            active: tab.id === `tab_${focusedProject.id}`,
          })),
        }));
        void reloadNativeStateSnapshot(focusedProject.id).catch(handleStateSnapshotReloadError);
      })
      .catch(handleProjectControlError);
  };

  const handleFocusWorkspaceTab = (tab: WorkspaceTab) => {
    if (!tab.projectId) {
      if (fallbackWorkspaceTabs.includes(tab.label)) {
        setActiveWorkspaceSurface(tab.label);
      }
      return;
    }
    setProjectControlError(undefined);
    focusNativeProject(tab.projectId)
      .then((focusedProject) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          projects: snapshot.projects.map((item) => ({
            ...item,
            state: item.id === focusedProject.id ? focusedProject.status : "idle",
          })),
          project_tabs: snapshot.project_tabs.map((item) => ({
            ...item,
            active: item.project_id === focusedProject.id || item.id === `tab_${focusedProject.id}`,
          })),
        }));
        void reloadNativeStateSnapshot(focusedProject.id).catch(handleStateSnapshotReloadError);
      })
      .catch(handleProjectControlError);
  };

  function handleProjectControlError(error: unknown) {
    const message = error instanceof Error ? error.message : "native project API unavailable";
    setProjectControlError(`Project control unavailable · ${message}`);
  }

  const handleSaveProjectTabGroup = () => {
    if (!activeProjectId) {
      setProjectWindowStatus("No active project tab");
      return;
    }
    const group = projectTabGroupDraft.trim();
    const nextGroups = group
      ? [
          ...projectTabGroups.filter((assignment) => assignment.projectId !== activeProjectId),
          { projectId: activeProjectId, group },
        ]
      : projectTabGroups.filter((assignment) => assignment.projectId !== activeProjectId);
    setProjectTabGroups(nextGroups);
    saveProjectTabGroups(nextGroups);
    setProjectWindowStatus(group ? `Grouped ${activeProjectName} under ${group}` : `Cleared group for ${activeProjectName}`);
    if (group) {
      void upsertNativeProjectTabGroup(activeProjectId, group).catch((error) => {
        const message = error instanceof Error ? error.message : "native project tab group API unavailable";
        setProjectWindowStatus(`Project tab group saved locally · ${message}`);
      });
    }
  };

  const applyProjectDetachPlan = (nativePlan: NativeProjectDetachPlan) => {
    const plan: ProjectDetachPlan = {
      projectId: nativePlan.project_id,
      projectName: nativePlan.project_name,
      windowId: nativePlan.window_id,
      status: nativePlan.status === "planned" ? "planned" : "planned",
    };
    const nextPlans = [
      ...projectDetachPlans.filter((candidate) => candidate.projectId !== plan.projectId),
      plan,
    ];
    setProjectDetachPlans(nextPlans);
    saveProjectDetachPlans(nextPlans);
    setProjectWindowStatus(`Detached ${plan.projectName} to window ${plan.windowId}`);
  };

  const handleDetachActiveProject = () => {
    if (!activeProjectId || !activeSnapshotProject) {
      setProjectWindowStatus("No active project tab");
      return;
    }
    void planNativeProjectDetach(activeProjectId)
      .then(applyProjectDetachPlan)
      .catch(() => {
        const plan: ProjectDetachPlan = {
          projectId: activeProjectId,
          projectName: activeSnapshotProject.name,
          windowId: `win_${activeProjectId}`,
          status: "planned",
        };
        const nextPlans = [
          ...projectDetachPlans.filter((candidate) => candidate.projectId !== activeProjectId),
          plan,
        ];
        setProjectDetachPlans(nextPlans);
        saveProjectDetachPlans(nextPlans);
        setProjectWindowStatus(`Detached ${plan.projectName} to window ${plan.windowId}`);
      });
  };

  const commandPaletteItems: CommandPaletteItem[] = [
    ...sidebarProjects.map((project) => ({
      id: `project:${project.id ?? project.name}`,
      kind: "Project",
      label: project.name,
      detail: `${project.state} · ${project.sessions} ${project.sessions === 1 ? "session" : "sessions"}`,
      action: () => handleFocusProject(project),
    })),
    ...visibleTasks.map((task) => ({
      id: `task:${task.id}`,
      kind: "Task",
      label: task.title,
      detail: `${task.status} · ${task.priority}`,
      action: () => handleOpenTaskDrawer(task.id),
    })),
    ...commandBlocks.map((block) => ({
      id: `command:${block.id}`,
      kind: "Command",
      label: block.command,
      detail: `${block.status} · ${block.cwd}`,
      action: () => setFocusedCommandBlockId(block.id),
    })),
    ...knowledgePages.map((page) => ({
      id: `knowledge-page:${page.id}`,
      kind: "Knowledge",
      label: page.title,
      detail: `${page.freshness_state} · ${page.slug}`,
      searchText: `${page.slug} ${page.title} ${page.body_md}`,
      action: () => {
        setKnowledgePageSlug(page.slug);
        setKnowledgePageTitle(page.title);
        setKnowledgePageBody(page.body_md);
      },
    })),
    ...controlPlaneSnapshot.knowledge.recent_pages.map((page) => ({
      id: `knowledge:${page}`,
      kind: "Knowledge",
      label: page,
      detail: "Recent knowledge page",
    })),
  ];
  const filteredCommandPaletteItems = commandPaletteItems
    .filter((item) => {
      const query = commandPaletteQuery.trim().toLowerCase();
      if (!query) return true;
      return `${item.kind} ${item.label} ${item.detail} ${item.searchText ?? ""}`.toLowerCase().includes(query);
    })
    .slice(0, 10);

  const runCommandPaletteItem = (item: CommandPaletteItem) => {
    item.action?.();
    setCommandPaletteOpen(false);
    setCommandPaletteQuery("");
  };

  const handleJumpToUnread = useCallback(() => {
    if (!nextUnreadSession) return;
    setNotificationJumpSessionId(nextUnreadSession.id);
    setFocusedTerminalSessionId(nextUnreadSession.id);
    setNotificationJumpStatus(`Focused ${nextUnreadSession.title}`);
    setNotificationDrawerOpen(false);
  }, [nextUnreadSession]);

  const setDashboardWidgetVisible = (widget: keyof typeof defaultDashboardWidgets, visible: boolean) => {
    setDashboardWidgets((current) => ({
      ...current,
      [widget]: visible,
    }));
  };

  function loadKnowledgeSourceIndex() {
    setKnowledgeSourceError(undefined);
    void listNativeKnowledgeSources(localProjectId)
      .then((sources) => {
        setKnowledgeSources(sources);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge API unavailable";
        setKnowledgeSourceError(`Knowledge source index unavailable · ${message}`);
      });
  }

  function handleIndexKnowledgeSource() {
    const pathOrRef = knowledgeSourcePath.trim();
    if (!pathOrRef) {
      setKnowledgeSourceStatus(undefined);
      setKnowledgeSourceError("Knowledge source path is required");
      return;
    }
    setKnowledgeSourceStatus(undefined);
    setKnowledgeSourceError(undefined);
    void upsertNativeKnowledgeSource({
      projectId: localProjectId,
      kind: "file",
      pathOrRef,
      fingerprint: `local:${pathOrRef}`,
      status: "current",
    })
      .then((source) => {
        setKnowledgeSources((sources) => {
          const withoutExisting = sources.filter((candidate) =>
            !(candidate.kind === source.kind && candidate.path_or_ref === source.path_or_ref),
          );
          return [source, ...withoutExisting];
        });
        setKnowledgeSourcePath("");
        setKnowledgeSourceStatus(`Indexed ${source.path_or_ref}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge API unavailable";
        setKnowledgeSourceError(`Knowledge source index unavailable · ${message}`);
      });
  }

  function loadMarkdownKnowledgePages() {
    setKnowledgePageError(undefined);
    void searchNativeKnowledgePages(localProjectId)
      .then((pages) => {
        setKnowledgePages(pages);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge API unavailable";
        setKnowledgePageError(`Markdown knowledge pages unavailable · ${message}`);
      });
  }

  function handleSaveKnowledgePage() {
    const slug = knowledgePageSlug.trim();
    const title = knowledgePageTitle.trim();
    const bodyMd = knowledgePageBody.trim();
    if (!slug || !title || !bodyMd) {
      setKnowledgePageStatus(undefined);
      setKnowledgePageError("Knowledge page slug title and markdown are required");
      return;
    }
    setKnowledgePageStatus(undefined);
    setKnowledgePageError(undefined);
    void saveNativeKnowledgePage({
      projectId: localProjectId,
      slug,
      title,
      bodyMd,
      sourceIds: [],
      freshnessState: "current",
    })
      .then((page) => {
        setKnowledgePages((pages) => [page, ...pages.filter((candidate) => candidate.slug !== page.slug)]);
        setKnowledgePageSlug("");
        setKnowledgePageTitle("");
        setKnowledgePageBody("");
        setKnowledgePageStatus(`Saved ${page.slug}`);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          knowledge: {
            ...snapshot.knowledge,
            recent_pages: [page.slug, ...snapshot.knowledge.recent_pages.filter((slug) => slug !== page.slug)].slice(0, 5),
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge API unavailable";
        setKnowledgePageError(`Knowledge page save failed · ${message}`);
      });
  }

  function loadKnowledgeConcepts() {
    setKnowledgeConceptError(undefined);
    void listNativeKnowledgeConcepts(localProjectId)
      .then((concepts) => {
        setKnowledgeConcepts(concepts);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge concept API unavailable";
        setKnowledgeConceptError(`Knowledge concepts unavailable · ${message}`);
      });
  }

  function handleExportKnowledgeObsidianMarkdown() {
    setKnowledgeObsidianExport(undefined);
    setKnowledgeObsidianExportError(undefined);
    void exportNativeKnowledgeObsidianMarkdown(localProjectId)
      .then((exported) => {
        setKnowledgeObsidianExport(exported);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge export API unavailable";
        setKnowledgeObsidianExportError(`Knowledge export unavailable · ${message}`);
      });
  }

  function handleAnswerKnowledgeQuestion() {
    const question = knowledgeQuestion.trim();
    if (!question) {
      setKnowledgeChatAnswer(undefined);
      setKnowledgeChatError("Knowledge question is required");
      return;
    }
    setKnowledgeChatAnswer(undefined);
    setKnowledgeChatError(undefined);
    void answerNativeKnowledgeQuestion({
      projectId: localProjectId,
      question,
      contextPackId: knowledgeChatContextPackId || undefined,
    })
      .then((answer) => {
        setKnowledgeChatAnswer(answer);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge chat API unavailable";
        setKnowledgeChatError(`Knowledge chat unavailable · ${message}`);
      });
  }

  function loadKnowledgeExplorations() {
    setKnowledgeExplorationError(undefined);
    void listNativeKnowledgeExplorations(localProjectId)
      .then((explorations) => {
        setKnowledgeExplorations(explorations);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge exploration API unavailable";
        setKnowledgeExplorationError(`Knowledge explorations unavailable · ${message}`);
      });
  }

  function handleSaveKnowledgeExploration() {
    const title = knowledgeExplorationTitle.trim();
    const question = knowledgeExplorationQuestion.trim();
    const answerMd = knowledgeExplorationAnswer.trim();
    if (!title || !question || !answerMd) {
      setKnowledgeExplorationStatus(undefined);
      setKnowledgeExplorationError("Knowledge exploration title question and answer are required");
      return;
    }
    setKnowledgeExplorationStatus(undefined);
    setKnowledgeExplorationError(undefined);
    void saveNativeKnowledgeExploration({
      projectId: localProjectId,
      title,
      question,
      answerMd,
      pageIds: knowledgeExplorationPageId ? [knowledgeExplorationPageId] : [],
      contextPackId: knowledgeExplorationContextPackId || undefined,
    })
      .then((exploration) => {
        setKnowledgeExplorations((explorations) => [
          exploration,
          ...explorations.filter((candidate) => candidate.id !== exploration.id),
        ]);
        setKnowledgeExplorationTitle("");
        setKnowledgeExplorationQuestion("");
        setKnowledgeExplorationAnswer("");
        setKnowledgeExplorationPageId("");
        setKnowledgeExplorationContextPackId("");
        setKnowledgeExplorationStatus(`Saved exploration ${exploration.id}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge exploration API unavailable";
        setKnowledgeExplorationError(`Knowledge exploration save failed · ${message}`);
      });
  }

  function handleRunKnowledgeAutomation() {
    setKnowledgeAutomationError(undefined);
    void runNativeKnowledgeAutomation({
      projectId: localProjectId,
      watch: knowledgeAutomationWatch,
    })
      .then((run) => {
        setKnowledgeAutomationRun(run);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          knowledge: {
            ...snapshot.knowledge,
            stale_count: run.stale_count,
            gap_count: run.gap_count,
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge automation unavailable";
        setKnowledgeAutomationRun(undefined);
        setKnowledgeAutomationError(`Knowledge automation unavailable · ${message}`);
      });
  }

  function handleRecordKnowledgeLintReport() {
    const staleCount = Number(knowledgeLintStaleCount);
    const gapCount = Number(knowledgeLintGapCount);
    const contradictionCount = Number(knowledgeLintContradictionCount);
    const bodyMd = knowledgeLintBody.trim();
    if (
      !Number.isFinite(staleCount) ||
      !Number.isFinite(gapCount) ||
      !Number.isFinite(contradictionCount) ||
      staleCount < 0 ||
      gapCount < 0 ||
      contradictionCount < 0
    ) {
      setKnowledgeLintReport(undefined);
      setKnowledgeLintError("Knowledge lint counts must be non-negative numbers");
      return;
    }
    if (!bodyMd) {
      setKnowledgeLintReport(undefined);
      setKnowledgeLintError("Knowledge lint body is required");
      return;
    }
    setKnowledgeLintReport(undefined);
    setKnowledgeLintError(undefined);
    void recordNativeKnowledgeLintReport({
      projectId: localProjectId,
      staleCount,
      gapCount,
      contradictionCount,
      bodyMd,
    })
      .then((report) => {
        setKnowledgeLintReport(report);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          knowledge: {
            ...snapshot.knowledge,
            stale_count: report.stale_count,
            gap_count: report.gap_count,
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge lint unavailable";
        setKnowledgeLintError(`Knowledge lint unavailable · ${message}`);
      });
  }

  function handleIngestKnowledgeArtifact() {
    const pathOrRef = knowledgeIngestionPath.trim();
    const title = knowledgeIngestionTitle.trim();
    if (!pathOrRef || !knowledgeIngestionBody.trim()) {
      setKnowledgeIngestionResult(undefined);
      setKnowledgeIngestionLastPath("");
      setKnowledgeIngestionError("Ingestion artifact path and body are required");
      return;
    }
    setKnowledgeIngestionResult(undefined);
    setKnowledgeIngestionLastPath("");
    setKnowledgeIngestionError(undefined);
    void ingestNativeKnowledgeArtifact({
      projectId: localProjectId,
      kind: knowledgeIngestionKind,
      pathOrRef,
      title: title || undefined,
      bodyMd: knowledgeIngestionBody,
      maxChunkChars: 1200,
    })
      .then((result) => {
        setKnowledgeIngestionResult(result);
        setKnowledgeIngestionLastPath(pathOrRef);
        setKnowledgeIngestionPath("");
        setKnowledgeIngestionTitle("");
        setKnowledgeIngestionBody("");
        setKnowledgeSources((sources) => [
          {
            id: result.source_id,
            project_id: result.project_id,
            kind: result.modality,
            path_or_ref: pathOrRef,
            fingerprint: result.fingerprint,
            status: "current",
          },
          ...sources.filter((source) => source.id !== result.source_id),
        ]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          knowledge: {
            ...snapshot.knowledge,
            recent_pages: [result.slug, ...snapshot.knowledge.recent_pages.filter((slug) => slug !== result.slug)].slice(0, 5),
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native knowledge ingestion unavailable";
        setKnowledgeIngestionResult(undefined);
        setKnowledgeIngestionLastPath("");
        setKnowledgeIngestionError(`Knowledge ingestion unavailable · ${message}`);
      });
  }

  function handleIngestTokenUsageAdapter() {
    let payload: unknown;
    try {
      payload = JSON.parse(tokenUsageAdapterPayload);
    } catch {
      setTokenUsageAdapterResult(undefined);
      setTokenUsageAdapterError("Token usage adapter payload must be valid JSON");
      return;
    }
    setTokenUsageAdapterError(undefined);
    setTokenUsageAdapterResult(undefined);
    const agentProfileId = tokenUsageAdapterAgent.trim() || undefined;
    void ingestNativeTokenUsageAdapter({
      projectId: localProjectId,
      agentProfileId,
      adapter: tokenUsageAdapter,
      payload,
    })
      .then((usage) => {
        setTokenUsageAdapterResult(usage);
        setTokenUsageAdapterPayload("");
        setStateSnapshot((snapshot) => applyTokenUsageToSnapshot(snapshot, usage));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native token usage adapter unavailable";
        setTokenUsageAdapterResult(undefined);
        setTokenUsageAdapterError(`Token usage adapter unavailable · ${message}`);
      });
  }

  function handleRecordManualTokenUsage() {
    const provider = manualTokenUsageProvider.trim();
    const model = manualTokenUsageModel.trim();
    const source = manualTokenUsageSource.trim();
    const inputTokens = Number.parseInt(manualTokenUsageInputTokens, 10);
    const outputTokens = Number.parseInt(manualTokenUsageOutputTokens, 10);
    const costUsd = Number(manualTokenUsageCostUsd);
    if (!provider || !model || !source) {
      setTokenUsageRecordResult(undefined);
      setTokenUsageRecordError("Manual token usage provider model and source are required");
      return;
    }
    if (!Number.isFinite(inputTokens) || inputTokens < 0 || !Number.isFinite(outputTokens) || outputTokens < 0) {
      setTokenUsageRecordResult(undefined);
      setTokenUsageRecordError("Manual token usage tokens must be zero or greater");
      return;
    }
    if (!Number.isFinite(costUsd) || costUsd < 0) {
      setTokenUsageRecordResult(undefined);
      setTokenUsageRecordError("Manual token usage cost must be zero or greater");
      return;
    }
    setTokenUsageRecordError(undefined);
    setTokenUsageRecordResult(undefined);
    void recordNativeTokenUsage({
      projectId: localProjectId,
      sessionId: manualTokenUsageSession.trim() || undefined,
      taskId: manualTokenUsageTask.trim() || undefined,
      runId: manualTokenUsageRun.trim() || undefined,
      agentProfileId: manualTokenUsageAgent.trim() || undefined,
      provider,
      model,
      inputTokens,
      outputTokens,
      costUsd,
      source,
    })
      .then((usage) => {
        setTokenUsageRecordResult(usage);
        setStateSnapshot((snapshot) => applyTokenUsageToSnapshot(snapshot, usage));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native token usage API unavailable";
        setTokenUsageRecordResult(undefined);
        setTokenUsageRecordError(`Manual token usage unavailable · ${message}`);
      });
  }

  function handleRefreshBudgetForecast() {
    setBudgetWorkflowError(undefined);
    setBudgetForecastResult(undefined);
    void getNativeBudgetForecast()
      .then((forecast) => {
        setBudgetForecastResult(forecast);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          budgets: {
            ...snapshot.budgets,
            forecasts: forecast,
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native budget forecast unavailable";
        setBudgetForecastResult(undefined);
        setBudgetWorkflowError(`Budget forecast unavailable · ${message}`);
      });
  }

  function handleLoadBudgetSummary() {
    setBudgetWorkflowError(undefined);
    setBudgetSummaryResult(undefined);
    void getNativeBudgetSummary()
      .then((summary) => {
        setBudgetSummaryResult(summary);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          budgets: {
            ...snapshot.budgets,
            workspace: summary.workspace,
            projects: summary.projects,
            goals: summary.goals ?? [],
            tasks: summary.tasks ?? [],
            runs: summary.runs ?? [],
            agents: summary.agents,
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native budget summary unavailable";
        setBudgetWorkflowError(`Budget summary unavailable · ${message}`);
      });
  }

  function handleSetBudget() {
    const maxUsd = Number(budgetMaxUsd);
    const warnPct = Number(budgetWarnPct);
    const scopeId = budgetScopeId.trim();
    if (!Number.isFinite(maxUsd) || maxUsd <= 0) {
      setBudgetSetResult(undefined);
      setBudgetWorkflowError("Budget max USD must be greater than 0");
      return;
    }
    if (!Number.isFinite(warnPct) || warnPct <= 0 || warnPct > 1) {
      setBudgetSetResult(undefined);
      setBudgetWorkflowError("Budget warning percent must be between 0 and 1");
      return;
    }
    if (budgetScopeType !== "workspace" && !scopeId) {
      setBudgetSetResult(undefined);
      setBudgetWorkflowError("Budget scope id is required");
      return;
    }
    setBudgetWorkflowError(undefined);
    setBudgetSetResult(undefined);
    void upsertNativeBudget({
      scopeType: budgetScopeType,
      scopeId: budgetScopeType === "workspace" ? undefined : scopeId,
      maxUsd,
      warnPct,
      hardLimit: budgetHardLimit,
    })
      .then((budget) => {
        setBudgetSetResult(budget);
        setStateSnapshot((snapshot) => upsertNativeBudgetIntoSnapshot(snapshot, budget));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native budget API unavailable";
        setBudgetSetResult(undefined);
        setBudgetWorkflowError(`Budget set unavailable · ${message}`);
      });
  }

  function handleLoadProviderPrices() {
    setBudgetWorkflowError(undefined);
    void listNativeProviderPrices()
      .then((prices) => {
        setProviderPrices(prices);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          budgets: {
            ...snapshot.budgets,
            price_table: {
              count: prices.length,
              source: summarizeProviderPriceSource(prices),
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native provider price API unavailable";
        setProviderPrices([]);
        setBudgetWorkflowError(`Provider price list unavailable · ${message}`);
      });
  }

  function handleUpdateProviderPrices() {
    let prices: unknown;
    try {
      prices = JSON.parse(providerPriceUpdatePayload);
    } catch {
      setProviderPriceUpdateResult(undefined);
      setBudgetWorkflowError("Provider price payload must be a JSON array");
      return;
    }
    if (!Array.isArray(prices)) {
      setProviderPriceUpdateResult(undefined);
      setBudgetWorkflowError("Provider price payload must be a JSON array");
      return;
    }
    const source = providerPriceUpdateSource.trim();
    if (!source) {
      setProviderPriceUpdateResult(undefined);
      setBudgetWorkflowError("Provider price update source is required");
      return;
    }
    setBudgetWorkflowError(undefined);
    setProviderPriceUpdateResult(undefined);
    void updateNativeProviderPriceTable({
      source,
      prices: prices as Parameters<typeof updateNativeProviderPriceTable>[0]["prices"],
    })
      .then((result) => {
        setProviderPriceUpdateResult(result);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          budgets: {
            ...snapshot.budgets,
            price_table: {
              count: result.updated,
              source: result.source,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native provider price update unavailable";
        setProviderPriceUpdateResult(undefined);
        setBudgetWorkflowError(`Provider price update unavailable · ${message}`);
      });
  }

  function handleRunReleaseGates() {
    setReleaseGateError(undefined);
    setReleaseGateRun(undefined);
    void runNativeReleaseGates(activeProjectId)
      .then((run) => {
        setReleaseGateRun(run);
        setReleaseGateHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          release_gates: {
            last_run_id: run.id,
            last_status: run.status,
            last_pass_count: run.pass_count,
            last_fail_count: run.fail_count,
            last_warning_count: run.warning_count,
            diagnostics: {
              status: run.status,
              scenario_count: run.scenario_count,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native release gate API unavailable";
        setReleaseGateRun(undefined);
        setReleaseGateError(`Release gate runner unavailable · ${message}`);
      });
  }

  function handleLoadReleaseGateHistory() {
    setReleaseGateError(undefined);
    void listNativeReleaseGateRuns(activeProjectId)
      .then((runs) => {
        setReleaseGateHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            release_gates: {
              last_run_id: latest.id,
              last_status: latest.status,
              last_pass_count: latest.pass_count,
              last_fail_count: latest.fail_count,
              last_warning_count: latest.warning_count,
              diagnostics: {
                status: latest.status,
                scenario_count: latest.scenario_count,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native release gate API unavailable";
        setReleaseGateHistory([]);
        setReleaseGateError(`Release gate history unavailable · ${message}`);
      });
  }

  function handleRunTerminalFidelitySmoke() {
    setTerminalSmokeError(undefined);
    setTerminalSmokeRun(undefined);
    void runNativeTerminalFidelitySmoke(activeProjectId)
      .then((run) => {
        setTerminalSmokeRun(run);
        setTerminalSmokeHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          terminal_fidelity: {
            last_run_id: run.id,
            last_status: run.status,
            last_pass_count: run.pass_count,
            last_fail_count: run.fail_count,
            last_warning_count: run.warning_count,
            diagnostics: {
              status: run.status,
              case_count: run.case_count,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native terminal smoke API unavailable";
        setTerminalSmokeRun(undefined);
        setTerminalSmokeError(`Terminal fidelity smoke unavailable · ${message}`);
      });
  }

  function handleLoadTerminalSmokeHistory() {
    setTerminalSmokeError(undefined);
    void listNativeTerminalFidelitySmokeRuns(activeProjectId)
      .then((runs) => {
        setTerminalSmokeHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            terminal_fidelity: {
              last_run_id: latest.id,
              last_status: latest.status,
              last_pass_count: latest.pass_count,
              last_fail_count: latest.fail_count,
              last_warning_count: latest.warning_count,
              diagnostics: {
                status: latest.status,
                case_count: latest.case_count,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native terminal smoke API unavailable";
        setTerminalSmokeHistory([]);
        setTerminalSmokeError(`Terminal smoke history unavailable · ${message}`);
      });
  }

  function handleRunTaskLifecycleE2E() {
    setTaskLifecycleError(undefined);
    setTaskLifecycleRun(undefined);
    void runNativeTaskLifecycleE2E(activeProjectId)
      .then((run) => {
        setTaskLifecycleRun(run);
        setTaskLifecycleHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          task_lifecycle: {
            last_run_id: run.id,
            last_status: run.status,
            last_task_id: run.task_id,
            last_agent_run_id: run.run_id,
            last_evidence_pack_id: run.evidence_pack_id,
            diagnostics: {
              status: run.status,
              transition_count: run.transitions.length,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task lifecycle API unavailable";
        setTaskLifecycleRun(undefined);
        setTaskLifecycleError(`Task lifecycle E2E unavailable · ${message}`);
      });
  }

  function handleLoadTaskLifecycleHistory() {
    setTaskLifecycleError(undefined);
    void listNativeTaskLifecycleE2ERuns(activeProjectId)
      .then((runs) => {
        setTaskLifecycleHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            task_lifecycle: {
              last_run_id: latest.id,
              last_status: latest.status,
              last_task_id: latest.task_id,
              last_agent_run_id: latest.run_id,
              last_evidence_pack_id: latest.evidence_pack_id,
              diagnostics: {
                status: latest.status,
                transition_count: latest.transitions.length,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task lifecycle API unavailable";
        setTaskLifecycleHistory([]);
        setTaskLifecycleError(`Task lifecycle history unavailable · ${message}`);
      });
  }

  function handleRunWorkflowNegativeTests() {
    setWorkflowNegativeError(undefined);
    setWorkflowNegativeRun(undefined);
    void runNativeWorkflowNegativeTests(activeProjectId)
      .then((run) => {
        setWorkflowNegativeRun(run);
        setWorkflowNegativeHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          workflow_negative: {
            last_run_id: run.id,
            last_status: run.status,
            last_baseline_workflow_id: run.baseline_workflow_id,
            last_invalid_workflow_id: run.invalid_workflow_id,
            last_known_good_workflow_id: run.last_known_good_workflow_id,
            diagnostics: {
              status: run.status,
              case_count: run.cases.length,
              dispatch_run_id: run.dispatch_run_id,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native workflow negative API unavailable";
        setWorkflowNegativeRun(undefined);
        setWorkflowNegativeError(`Workflow negative tests unavailable · ${message}`);
      });
  }

  function handleLoadWorkflowNegativeHistory() {
    setWorkflowNegativeError(undefined);
    void listNativeWorkflowNegativeTestRuns(activeProjectId)
      .then((runs) => {
        setWorkflowNegativeHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            workflow_negative: {
              last_run_id: latest.id,
              last_status: latest.status,
              last_baseline_workflow_id: latest.baseline_workflow_id,
              last_invalid_workflow_id: latest.invalid_workflow_id,
              last_known_good_workflow_id: latest.last_known_good_workflow_id,
              diagnostics: {
                status: latest.status,
                case_count: latest.cases.length,
                dispatch_run_id: latest.dispatch_run_id,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native workflow negative API unavailable";
        setWorkflowNegativeHistory([]);
        setWorkflowNegativeError(`Workflow negative history unavailable · ${message}`);
      });
  }

  function handleRunDmgSmokeTest() {
    const dmgPath = dmgSmokePath.trim() || undefined;
    const appBundlePath = dmgSmokeAppBundlePath.trim() || undefined;
    setDmgSmokeError(undefined);
    setDmgSmokeRun(undefined);
    void runNativeDmgSmokeTest(activeProjectId, dmgPath, appBundlePath)
      .then((run) => {
        setDmgSmokeRun(run);
        setDmgSmokeHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          distribution: {
            last_dmg_smoke_run_id: run.id,
            last_status: run.status,
            explicit_blocker: run.explicit_blocker,
            last_pass_count: run.pass_count,
            last_fail_count: run.fail_count,
            last_warning_count: run.warning_count,
            diagnostics: {
              status: run.status,
              case_count: run.case_count,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native DMG smoke API unavailable";
        setDmgSmokeRun(undefined);
        setDmgSmokeError(`DMG smoke test unavailable · ${message}`);
      });
  }

  function handleLoadDmgSmokeHistory() {
    setDmgSmokeError(undefined);
    void listNativeDmgSmokeRuns(activeProjectId)
      .then((runs) => {
        setDmgSmokeHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            distribution: {
              last_dmg_smoke_run_id: latest.id,
              last_status: latest.status,
              explicit_blocker: latest.explicit_blocker,
              last_pass_count: latest.pass_count,
              last_fail_count: latest.fail_count,
              last_warning_count: latest.warning_count,
              diagnostics: {
                status: latest.status,
                case_count: latest.case_count,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native DMG smoke API unavailable";
        setDmgSmokeHistory([]);
        setDmgSmokeError(`DMG smoke history unavailable · ${message}`);
      });
  }

  function handleRunRecoveryDrills() {
    setRecoveryDrillError(undefined);
    setRecoveryDrillRun(undefined);
    void runNativeRecoveryDrills(activeProjectId)
      .then((run) => {
        setRecoveryDrillRun(run);
        setRecoveryDrillHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          recovery: {
            last_run_id: run.id,
            last_status: run.status,
            last_pass_count: run.pass_count,
            last_fail_count: run.fail_count,
            last_warning_count: run.warning_count,
            diagnostics: {
              status: run.status,
              drill_count: run.drill_count,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native recovery drill API unavailable";
        setRecoveryDrillRun(undefined);
        setRecoveryDrillError(`Recovery drills unavailable · ${message}`);
      });
  }

  function handleLoadRecoveryDrillHistory() {
    setRecoveryDrillError(undefined);
    void listNativeRecoveryDrillRuns(activeProjectId)
      .then((runs) => {
        setRecoveryDrillHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            recovery: {
              last_run_id: latest.id,
              last_status: latest.status,
              last_pass_count: latest.pass_count,
              last_fail_count: latest.fail_count,
              last_warning_count: latest.warning_count,
              diagnostics: {
                status: latest.status,
                drill_count: latest.drill_count,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native recovery drill API unavailable";
        setRecoveryDrillHistory([]);
        setRecoveryDrillError(`Recovery drill history unavailable · ${message}`);
      });
  }

  function handleRunBenchmarks() {
    setBenchmarkError(undefined);
    setBenchmarkRun(undefined);
    void runNativeBenchmarks(activeProjectId)
      .then((run) => {
        setBenchmarkRun(run);
        setBenchmarkHistory((runs) => [run, ...runs.filter((candidate) => candidate.id !== run.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          benchmarks: {
            last_run_id: run.id,
            last_status: run.status,
            last_pass_count: run.pass_count,
            last_fail_count: run.fail_count,
            last_warning_count: run.warning_count,
            suites: run.suites,
            diagnostics: {
              status: run.status,
              suite_count: run.suite_count,
              duration_ms: run.duration_ms,
              created_at: run.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native benchmark API unavailable";
        setBenchmarkRun(undefined);
        setBenchmarkError(`Benchmark runner unavailable · ${message}`);
      });
  }

  function handleLoadBenchmarkHistory() {
    setBenchmarkError(undefined);
    void listNativeBenchmarkRuns(activeProjectId)
      .then((runs) => {
        setBenchmarkHistory(runs);
        const latest = runs[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            benchmarks: {
              last_run_id: latest.id,
              last_status: latest.status,
              last_pass_count: latest.pass_count,
              last_fail_count: latest.fail_count,
              last_warning_count: latest.warning_count,
              suites: latest.suites,
              diagnostics: {
                status: latest.status,
                suite_count: latest.suite_count,
                duration_ms: latest.duration_ms,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native benchmark API unavailable";
        setBenchmarkHistory([]);
        setBenchmarkError(`Benchmark history unavailable · ${message}`);
      });
  }

  function handleRunDogfoodTelemetryReview() {
    setDogfoodReviewError(undefined);
    setDogfoodReview(undefined);
    void runNativeDogfoodTelemetryReview(activeProjectId)
      .then((review) => {
        setDogfoodReview(review);
        setDogfoodReviewHistory((reviews) => [review, ...reviews.filter((candidate) => candidate.id !== review.id)]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          dogfood: {
            last_review_id: review.id,
            last_status: review.status,
            last_evidence_pack_id: review.evidence_pack_id,
            last_pass_count: review.pass_count,
            last_warning_count: review.warning_count,
            last_fail_count: review.fail_count,
            diagnostics: {
              status: review.status,
              finding_count: review.finding_count,
              created_at: review.created_at,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native dogfood telemetry API unavailable";
        setDogfoodReview(undefined);
        setDogfoodReviewError(`Dogfood telemetry review unavailable · ${message}`);
      });
  }

  function handleLoadDogfoodTelemetryHistory() {
    setDogfoodReviewError(undefined);
    void listNativeDogfoodTelemetryReviews(activeProjectId)
      .then((reviews) => {
        setDogfoodReviewHistory(reviews);
        const latest = reviews[0];
        if (latest) {
          setStateSnapshot((snapshot) => ({
            ...snapshot,
            dogfood: {
              last_review_id: latest.id,
              last_status: latest.status,
              last_evidence_pack_id: latest.evidence_pack_id,
              last_pass_count: latest.pass_count,
              last_warning_count: latest.warning_count,
              last_fail_count: latest.fail_count,
              diagnostics: {
                status: latest.status,
                finding_count: latest.finding_count,
                created_at: latest.created_at,
              },
            },
          }));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native dogfood telemetry API unavailable";
        setDogfoodReviewHistory([]);
        setDogfoodReviewError(`Dogfood telemetry history unavailable · ${message}`);
      });
  }

  function persistVisualHarnessLink(sourceId: string, targetId: string, kind: string) {
    if (!sourceId || !targetId || !kind) {
      setVisualHarnessLink(undefined);
      setVisualHarnessError("Visual harness source target and kind are required");
      return;
    }
    setVisualHarnessError(undefined);
    setVisualHarnessLink(undefined);
    void createNativeVisualHarnessLink({
      projectId: activeProjectId,
      sourceId,
      targetId,
      kind,
    })
      .then((link) => {
        setVisualHarnessLink(link);
        setStateSnapshot((snapshot) => {
          const current = snapshot.visual_harness ?? { nodes: [], edges: [], diagnostics: {} };
          const edges = [
            ...current.edges.filter((edge) => edge.id !== link.id),
            {
              id: link.id,
              source_id: link.source_id,
              target_id: link.target_id,
              kind: link.kind,
              status: "manual",
            },
          ];
          return {
            ...snapshot,
            visual_harness: {
              ...current,
              edges,
              diagnostics: {
                ...current.diagnostics,
                status: current.nodes.length === 0 && edges.length === 0 ? "empty" : "ok",
                node_count: current.nodes.length,
                edge_count: edges.length,
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native visual harness API unavailable";
        setVisualHarnessLink(undefined);
        setVisualHarnessError(`Visual harness link unavailable · ${message}`);
      });
  }

  function handleCreateVisualHarnessLink() {
    persistVisualHarnessLink(
      visualHarnessSource.trim(),
      visualHarnessTarget.trim(),
      visualHarnessKind.trim(),
    );
  }

  function handleCreateVisualHarnessLinkFromCanvas(sourceId: string, targetId: string) {
    setVisualHarnessSource(sourceId);
    setVisualHarnessTarget(targetId);
    persistVisualHarnessLink(sourceId, targetId, visualHarnessKind.trim() || "dependency");
  }

  function handleLoadVisualHarnessLinks() {
    setVisualHarnessError(undefined);
    void listNativeVisualHarnessLinks(activeProjectId)
      .then((links) => {
        setStateSnapshot((snapshot) => {
          const current = snapshot.visual_harness ?? { nodes: [], edges: [], diagnostics: {} };
          const loadedEdges = links.map((link) => nativeVisualHarnessLinkToEdge(link));
          const loadedIds = new Set(loadedEdges.map((edge) => edge.id));
          const edges = [
            ...current.edges.filter((edge) => !loadedIds.has(edge.id)),
            ...loadedEdges,
          ];
          return {
            ...snapshot,
            visual_harness: {
              ...current,
              edges,
              diagnostics: {
                ...current.diagnostics,
                status: current.nodes.length === 0 && edges.length === 0 ? "empty" : "ok",
                node_count: current.nodes.length,
                edge_count: edges.length,
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native visual harness API unavailable";
        setVisualHarnessError(`Visual harness links unavailable · ${message}`);
      });
  }

  function handleBindExternalTracker() {
    const localId = trackerLocalId.trim();
    const externalId = trackerExternalId.trim();
    if (!localId || !externalId) {
      setTrackerBinding(undefined);
      setTrackerError("Tracker local id and external id are required");
      return;
    }
    const provider = trackerProvider;
    const syncMode = trackerSyncMode;
    setTrackerError(undefined);
    setTrackerBinding(undefined);
    void upsertNativeExternalTrackerBinding({
      projectId: activeProjectId,
      localKind: trackerLocalKind,
      localId,
      provider,
      externalId,
      externalUrl: trackerExternalUrl.trim() || undefined,
      syncMode,
    })
      .then((binding) => {
        setTrackerBinding(binding);
        setStateSnapshot((snapshot) => {
          const current = snapshot.tracker ?? { binding_count: 0, bindings: [], diagnostics: {} };
          const bindings = [
            binding,
            ...current.bindings.filter((item) => item.id !== binding.id),
          ].map((item) => ({
            id: item.id,
            local_kind: item.local_kind,
            local_id: item.local_id,
            provider: item.provider,
            external_id: item.external_id,
            external_url: item.external_url,
            sync_mode: item.sync_mode,
            sync_status: item.sync_status,
            conflict_state: item.conflict_state,
          }));
          return {
            ...snapshot,
            tracker: {
              ...current,
              binding_count: bindings.length,
              bindings,
              diagnostics: {
                ...current.diagnostics,
                status: bindings.some((item) => item.conflict_state !== "none") ? "conflict" : "pending",
                pending_count: bindings.filter((item) => item.sync_status === "pending").length,
                conflict_count: bindings.filter((item) => item.conflict_state !== "none").length,
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native tracker binding API unavailable";
        setTrackerBinding(undefined);
        setTrackerError(`Tracker binding unavailable · ${message}`);
      });
  }

  function handleLoadTrackerBindings() {
    setTrackerError(undefined);
    void listNativeExternalTrackerBindings(activeProjectId)
      .then((nativeBindings) => {
        const bindings = nativeBindings.map((binding) => ({
          id: binding.id,
          local_kind: binding.local_kind,
          local_id: binding.local_id,
          provider: binding.provider,
          external_id: binding.external_id,
          external_url: binding.external_url,
          sync_mode: binding.sync_mode,
          sync_status: binding.sync_status,
          conflict_state: binding.conflict_state,
        }));
        setStateSnapshot((snapshot) => {
          const current = snapshot.tracker ?? { binding_count: 0, bindings: [], diagnostics: {} };
          return {
            ...snapshot,
            tracker: {
              ...current,
              binding_count: bindings.length,
              bindings,
              diagnostics: {
                ...current.diagnostics,
                status: bindings.some((item) => item.conflict_state !== "none")
                  ? "conflict"
                  : bindings.some((item) => item.sync_status === "pending")
                    ? "pending"
                    : bindings.length > 0
                      ? "bound"
                      : "unbound",
                pending_count: bindings.filter((item) => item.sync_status === "pending").length,
                conflict_count: bindings.filter((item) => item.conflict_state !== "none").length,
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native tracker binding API unavailable";
        setTrackerError(`Tracker bindings unavailable · ${message}`);
      });
  }

  function handleRunTrackerSync() {
    if (trackerProvider === "custom" || trackerProvider === "manual") {
      setTrackerSyncRun(undefined);
      setTrackerError("Tracker sync runs support linear github and plane providers");
      return;
    }
    setTrackerError(undefined);
    setTrackerSyncRun(undefined);
    void runNativeTrackerSync({
      projectId: activeProjectId,
      provider: trackerProvider,
      dryRun: trackerDryRun,
    })
      .then((run) => {
        setTrackerSyncRun(run);
        setStateSnapshot((snapshot) => {
          const current = snapshot.tracker ?? { binding_count: 0, bindings: [], diagnostics: {} };
          return {
            ...snapshot,
            tracker: {
              ...current,
              diagnostics: {
                ...current.diagnostics,
                [run.provider]: {
                  last_run_id: run.id,
                  last_status: run.status,
                  last_operation_count: run.operation_count,
                  degraded_reason: run.degraded_reason,
                },
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native tracker sync API unavailable";
        setTrackerSyncRun(undefined);
        setTrackerError(`Tracker sync unavailable · ${message}`);
      });
  }

  function handleSaveSecret() {
    const name = secretName.trim();
    const value = secretValue.trim();
    if (!name || !value) {
      setSecretResult(undefined);
      setSecretError("Secret name and value are required");
      return;
    }
    setSecretError(undefined);
    setSecretResult(undefined);
    void upsertNativeSecret({
      projectId: localProjectId,
      name,
      value,
      })
      .then((secret) => {
        setSecretResult(secret);
        setSecretInventory((secrets) => {
          const existingIndex = secrets.findIndex((candidate) => candidate.id === secret.id);
          if (existingIndex === -1) {
            return [...secrets, secret];
          }
          return secrets.map((candidate, index) => (index === existingIndex ? secret : candidate));
        });
        setSecretValue("");
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          security: {
            keychain: snapshot.security?.keychain ?? "unknown",
            secret_count: Math.max(snapshot.security?.secret_count ?? 0, 1),
            redaction: {
              status: "active",
              protected_secret_count: Math.max(snapshot.security?.redaction?.protected_secret_count ?? 0, 1),
            },
            permission_audit: snapshot.security?.permission_audit,
            diagnostics: snapshot.security?.diagnostics,
            policy_pack: snapshot.security?.policy_pack,
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native secret API unavailable";
        setSecretResult(undefined);
        setSecretError(`Keychain secret unavailable · ${message}`);
      });
  }

  function handleLoadSecrets() {
    setSecretError(undefined);
    void listNativeSecrets(localProjectId)
      .then((secrets) => {
        const protectedCount = secrets.filter((secret) => secret.redacted).length;
        setSecretInventory(secrets);
        setStateSnapshot((snapshot) => {
          const currentSecurity = snapshot.security;
          return {
            ...snapshot,
            security: {
              ...(currentSecurity ?? {}),
              keychain: currentSecurity?.keychain ?? "local",
              secret_count: secrets.length,
              redaction: {
                ...(currentSecurity?.redaction ?? {}),
                status: secrets.length > 0 ? "active" : "inactive",
                protected_secret_count: protectedCount,
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native secret API unavailable";
        setSecretInventory([]);
        setSecretError(`Keychain secret list unavailable · ${message}`);
      });
  }

  function loadContextPackIndex() {
    setContextPackError(undefined);
    void listNativeContextPacks(localProjectId)
      .then((packs) => {
        setContextPacks(packs);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native context API unavailable";
        setContextPackError(`Context packs unavailable · ${message}`);
      });
  }

  function handleSaveContextPack() {
    const name = contextPackName.trim();
    const sourceId = contextPackSourceId.trim();
    const maxTokensHint = contextPackMaxTokens.trim() ? Number(contextPackMaxTokens.trim()) : undefined;
    if (!name || !sourceId) {
      setContextPackStatus(undefined);
      setContextPackError("Context pack name and source are required");
      return;
    }
    if (typeof maxTokensHint === "number" && (!Number.isFinite(maxTokensHint) || maxTokensHint <= 0)) {
      setContextPackStatus(undefined);
      setContextPackError("Context pack max tokens must be positive");
      return;
    }
    setContextPackStatus(undefined);
    setContextPackError(undefined);
    void upsertNativeContextPack({
      projectId: localProjectId,
      name,
      description: contextPackDescription.trim() || undefined,
      sourcesJson: [{ type: "knowledge_page", id: sourceId }],
      maxTokensHint,
    })
      .then((pack) => {
        setContextPacks((packs) => [pack, ...packs.filter((candidate) => candidate.id !== pack.id)]);
        setContextPackName("");
        setContextPackDescription("");
        setContextPackSourceId("");
        setContextPackMaxTokens("");
        setContextPackStatus(`Saved context pack ${pack.id}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native context API unavailable";
        setContextPackError(`Context pack save failed · ${message}`);
      });
  }

  useEffect(() => {
    let active = true;

    invoke<ReadinessSnapshot>("get_readiness_snapshot")
      .then((snapshot) => {
        if (active) {
          setReadinessSnapshot(snapshot);
        }
      })
      .catch(() => {
        if (active) {
          setReadinessSnapshot(fallbackReadinessSnapshot);
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    loadKnowledgeSourceIndex();
  }, []);

  useEffect(() => {
    loadMarkdownKnowledgePages();
  }, []);

  useEffect(() => {
    loadKnowledgeConcepts();
  }, []);

  useEffect(() => {
    loadKnowledgeExplorations();
  }, []);

  useEffect(() => {
    loadContextPackIndex();
  }, []);

  useEffect(() => {
    let active = true;
    let dispose: (() => void) | undefined;

    listenToTerminalPtyOutput((event) => {
      if (!active) return;
      const ownerSessionId =
        ptySessionOwnerRef.current[event.sessionId] ??
        renderedTerminalDeckSessionsRef.current.find((session) => session.ptyId === event.sessionId)?.id;
      const shouldRecordStreamChunk =
        Boolean(ownerSessionId) && stateSessionsRef.current.some((session) => session.id === ownerSessionId);
      if (shouldRecordStreamChunk && event.chunk.length > 0) {
        void recordNativeTerminalStreamChunk({
          sessionId: ownerSessionId,
          seqStart: event.seq,
          seqEnd: event.seq,
          body: event.chunk,
        }).catch(() => undefined);
      }
      const result = ingestTerminalPtyOutput(terminalTransportStateRef.current, event);
      terminalTransportStateRef.current = result.state;
      if (result.releasedChunks.length === 0) return;
      const parsedChunks: string[] = [];
      let nextOscState = oscEventStateRef.current;
      for (const chunk of result.releasedChunks) {
        const parsed = parseOscSequences(nextOscState, chunk);
        nextOscState = parsed.state;
        if (parsed.displayText.length > 0) {
          parsedChunks.push(parsed.displayText);
        }
      }
      oscEventStateRef.current = nextOscState;
      setOscEventState(nextOscState);
      if (parsedChunks.length === 0) return;
      setCommandBlockState((state) =>
        parsedChunks.reduce((nextState, chunk) => {
          const result = ingestCommandOutput(nextState, { ...event, chunk });
          if (result.updatedBlock) {
            persistCommandBlock(result.updatedBlock);
          }
          return result.state;
        }, state),
      );
      setTerminalDeckSessions((sessions) =>
        sessions.map((session) => (session.ptyId === event.sessionId ? appendTerminalOutput(session, parsedChunks) : session)),
      );
    })
      .then((unlisten) => {
        if (active) {
          setTerminalOutputListenerError(undefined);
          dispose = unlisten;
        } else {
          unlisten();
        }
      })
      .catch((error) => {
        if (!active) return;
        const message = error instanceof Error ? error.message : "native terminal output listener unavailable";
        setTerminalOutputListenerError(`Terminal output listener unavailable · ${message}`);
      });

    return () => {
      active = false;
      dispose?.();
    };
  }, []);

  function handleRunTerminalSession(session: TerminalSession) {
    spawnTerminalPtySession(terminalPtyRequestForSession(session))
      .then((ptySession) => {
        ptySessionOwnerRef.current = {
          ...ptySessionOwnerRef.current,
          [ptySession.id]: session.id,
        };
        setTerminalDeckSessions((sessions) =>
          sessions.some((candidate) => candidate.id === session.id)
            ? sessions.map((candidate) =>
                candidate.id === session.id
                  ? appendTerminalOutput(bindPtySession(candidate, ptySession.id), [`PTY ${ptySession.id} started`])
                  : candidate,
              )
            : [...sessions, appendTerminalOutput(bindPtySession(session, ptySession.id), [`PTY ${ptySession.id} started`])],
        );
        return getTerminalPtySnapshot();
      })
      .then((snapshot) => setPtySnapshot(snapshot))
      .catch((error) => {
        setTerminalDeckSessions((sessions) =>
          sessions.some((candidate) => candidate.id === session.id)
            ? sessions.map((candidate) =>
                candidate.id === session.id ? appendTerminalOutput(candidate, [`PTY start failed: ${String(error)}`]) : candidate,
              )
            : [...sessions, appendTerminalOutput(session, [`PTY start failed: ${String(error)}`])],
        );
      });
  }

  function handleSplitTerminalSession(session: TerminalSession) {
    const source = sessionItems.find((candidate) => candidate.id === session.id || candidate.pane_id === session.ptyId);
    setTerminalSessionCreateError(undefined);
    void createNativeSession({
      projectId: source?.project_id ?? localProjectId,
      mode: source?.mode ?? "shell",
      title: `${session.title} split`,
      cwd: session.cwd,
      branch: session.branch,
    })
      .then((created) => {
        const nextSession = nativeSessionToStateSession(created);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          sessions: snapshot.sessions.some((item) => item.id === nextSession.id)
            ? snapshot.sessions.map((item) => (item.id === nextSession.id ? nextSession : item))
            : [...snapshot.sessions, nextSession],
        }));
        setFocusedTerminalSessionId(nextSession.id);
      })
      .catch(handleTerminalSessionCreateError);
  }

  function handleTerminalRendererDegraded(session: TerminalSession, reason: string) {
    const normalizedReason = reason.startsWith("Renderer degraded:") ? reason : `Renderer degraded: ${reason}`;
    setTerminalDeckSessions((sessions) => {
      const hasSession = sessions.some((candidate) => candidate.id === session.id || candidate.ptyId === session.ptyId);
      if (!hasSession) return [...sessions, markRendererDegraded(session, normalizedReason)];
      return sessions.map((candidate) =>
        candidate.id === session.id || candidate.ptyId === session.ptyId
          ? markRendererDegraded(candidate, normalizedReason)
          : candidate,
      );
    });
  }

  function handleCreateTerminalSession() {
    const title = `Terminal session ${sessionItemCountRef.current + 1}`;
    setTerminalSessionCreateError(undefined);
    void createNativeSession({
      projectId: activeProjectIdRef.current,
      mode: "shell",
      title,
    })
      .then((created) => {
        const nextSession = nativeSessionToStateSession(created);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          sessions: snapshot.sessions.some((item) => item.id === nextSession.id)
            ? snapshot.sessions.map((item) => (item.id === nextSession.id ? nextSession : item))
            : [...snapshot.sessions, nextSession],
        }));
        setFocusedTerminalSessionId(nextSession.id);
      })
      .catch(handleTerminalSessionCreateError);
  }

  function handleTerminalSessionCreateError(error: unknown) {
    const message = error instanceof Error ? error.message : "native session API unavailable";
    setTerminalSessionCreateError(`Terminal session unavailable · ${message}`);
  }

  function handleCreateSshTerminalSession() {
    const title = sshSessionTitle.trim();
    const target = sshTarget.trim();
    if (!title || !target) {
      setSshSessionStatus(undefined);
      setSshSessionError("SSH session title and target are required");
      return;
    }
    setSshSessionStatus(undefined);
    setSshSessionError(undefined);
    void createNativeSession({
      projectId: activeProjectId,
      mode: "ssh",
      title,
      cwd: sshSessionCwd(target, sshRemotePath),
      branch: sshBranchLabel.trim() || undefined,
    })
      .then((created) => {
        const nextSession = nativeSessionToStateSession(created);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          sessions: snapshot.sessions.some((item) => item.id === nextSession.id)
            ? snapshot.sessions.map((item) => (item.id === nextSession.id ? nextSession : item))
            : [...snapshot.sessions, nextSession],
        }));
        setFocusedTerminalSessionId(nextSession.id);
        setSshSessionStatus(`SSH session ${nextSession.title} ready`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native session API unavailable";
        setSshSessionError(`SSH session unavailable · ${message}`);
      });
  }

  function handleCaptureTerminalCommand() {
    const command = terminalCaptureCommand.trim();
    if (!command) {
      setTerminalCaptureResult(undefined);
      setTerminalCaptureError("Capture PTY command is required");
      return;
    }
    setTerminalCaptureResult(undefined);
    setTerminalCaptureError(undefined);
    void captureTerminalPtyCommand({
      command,
      args: splitTerminalCaptureArgs(terminalCaptureArgs),
      cols: 120,
      rows: 30,
    })
      .then((result) => setTerminalCaptureResult(result))
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native terminal capture API unavailable";
        setTerminalCaptureError(`Terminal capture unavailable · ${message}`);
      });
  }

  function buildProjectLayoutJson(focusedSessionId: string | undefined, maximizedSessionId: string | undefined): ProjectTabLayout {
    return {
      mode: maximizedSessionId ? "maximized" : "grid",
      focusedSessionId: focusedSessionId ?? null,
      maximizedSessionId: maximizedSessionId ?? null,
      panes: renderedTerminalDeckSessions.map((session) => session.id),
    };
  }

  function persistProjectLayoutJson(projectId: string, layoutJson: ProjectTabLayout) {
    setStateSnapshot((snapshot) => ({
      ...snapshot,
      project_tabs: snapshot.project_tabs.map((tab) =>
        projectIdFromStateProjectTab(tab, snapshot.projects) === projectId ? { ...tab, layout_json: layoutJson } : tab,
      ),
    }));
    setProjectControlError(undefined);
    void updateNativeProjectLayout(projectId, layoutJson)
      .then((projectTab) => {
        if (!projectTab) return;
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          project_tabs: snapshot.project_tabs.map((tab) =>
            tab.id === projectTab.id || projectIdFromStateProjectTab(tab, snapshot.projects) === projectTab.project_id
              ? { ...tab, layout_json: projectTab.layout_json }
              : tab,
          ),
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native project layout API unavailable";
        setProjectControlError(`Project layout saved locally · ${message}`);
      });
  }

  function persistActiveProjectLayout(focusedSessionId: string | undefined, maximizedSessionId: string | undefined) {
    const projectId = activeProjectTabProjectId ?? activeProjectId;
    if (!projectId || !controlPlaneSnapshot.projects.some((project) => project.id === projectId)) return;
    persistProjectLayoutJson(projectId, buildProjectLayoutJson(focusedSessionId, maximizedSessionId));
  }

  function handleSaveProjectLayoutPreset() {
    const projectId = activeProjectTabProjectId ?? activeProjectId;
    if (!projectId || !controlPlaneSnapshot.projects.some((project) => project.id === projectId)) {
      setProjectWindowStatus("No active project tab");
      return;
    }
    const name = projectLayoutPresetName.trim();
    if (!name) {
      setProjectWindowStatus("Layout preset name required");
      return;
    }
    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "") || "preset";
    const layoutJson = buildProjectLayoutJson(activeTerminalSessionId, maximizedTerminalSessionId);
    const preset: ProjectLayoutPreset = {
      id: `layout_${projectId}_${slug}`,
      projectId,
      name,
      layoutJson,
    };
    const nextPresets = upsertProjectLayoutPreset(projectLayoutPresets, preset);
    const storageError = saveProjectLayoutPresets(nextPresets);
    setProjectLayoutPresets(nextPresets);
    setProjectLayoutPresetName("");
    setProjectWindowStatus(storageError ? `Layout preset saved in session · ${storageError}` : `Saved layout preset ${name}`);
    void saveNativeProjectLayoutPreset(projectId, name, layoutJson)
      .then((nativePreset) => {
        const persistedPreset = nativeProjectLayoutPresetToProjectLayoutPreset(nativePreset);
        setProjectLayoutPresets((currentPresets) => {
          const mergedPresets = upsertProjectLayoutPreset(currentPresets, persistedPreset);
          saveProjectLayoutPresets(mergedPresets);
          return mergedPresets;
        });
        setProjectWindowStatus(`Saved layout preset ${name} · native`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native project layout preset API unavailable";
        setProjectWindowStatus(`Layout preset saved locally · ${message}`);
      });
  }

  function handleLoadProjectLayoutPresets() {
    const projectId = activeProjectTabProjectId ?? activeProjectId;
    if (!projectId || !controlPlaneSnapshot.projects.some((project) => project.id === projectId)) {
      setProjectWindowStatus("No active project tab");
      return;
    }
    void listNativeProjectLayoutPresets(projectId)
      .then((nativePresets) => {
        const loadedPresets = nativePresets
          .map(nativeProjectLayoutPresetToProjectLayoutPreset)
          .filter((preset) => isProjectTabLayout(preset.layoutJson));
        setProjectLayoutPresets((currentPresets) => {
          const mergedPresets = mergeProjectLayoutPresets(currentPresets, projectId, loadedPresets);
          const storageError = saveProjectLayoutPresets(mergedPresets);
          setProjectWindowStatus(
            storageError
              ? `Loaded ${loadedPresets.length} layout ${loadedPresets.length === 1 ? "preset" : "presets"} · ${storageError}`
              : `Loaded ${loadedPresets.length} layout ${loadedPresets.length === 1 ? "preset" : "presets"}`,
          );
          return mergedPresets;
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native project layout preset API unavailable";
        setProjectWindowStatus(`Layout preset load unavailable · ${message}`);
      });
  }

  function handleApplyProjectLayoutPreset(preset: ProjectLayoutPreset) {
    const projectId = activeProjectTabProjectId ?? activeProjectId;
    if (!projectId || preset.projectId !== projectId) {
      setProjectWindowStatus("Layout preset unavailable for active project");
      return;
    }
    const paneIds = new Set(renderedTerminalDeckSessions.map((session) => session.id));
    const focusedSessionId =
      preset.layoutJson.focusedSessionId && paneIds.has(preset.layoutJson.focusedSessionId)
        ? preset.layoutJson.focusedSessionId
        : undefined;
    const maximizedSessionId =
      preset.layoutJson.mode === "maximized" &&
      preset.layoutJson.maximizedSessionId &&
      paneIds.has(preset.layoutJson.maximizedSessionId)
        ? preset.layoutJson.maximizedSessionId
        : undefined;
    setFocusedTerminalSessionId(focusedSessionId);
    setMaximizedTerminalSessionId(maximizedSessionId);
    setProjectWindowStatus(`Applied layout preset ${preset.name}`);
    persistProjectLayoutJson(projectId, preset.layoutJson);
  }

  function handleOpenProjectFile(path: string) {
    if (!activeSnapshotProject) return;
    setProjectFilePreviewError(undefined);
    setProjectFileSaveError(undefined);
    setProjectFileSaveStatus(undefined);
    setProjectLspDiagnostics(undefined);
    setProjectLspError(undefined);
    setProjectFilePreview(undefined);
    setProjectFileDraft("");
    void readNativeProjectFile(activeSnapshotProject.id, path)
      .then((preview) => {
        setProjectFilePreview(preview);
        setProjectFileDraft(preview.body);
      })
      .catch((error) => {
        setProjectFilePreview(undefined);
        setProjectFileDraft("");
        setProjectFilePreviewError(String(error));
      });
  }

  function handleSaveProjectFile() {
    if (!activeSnapshotProject || !projectFilePreview) {
      setProjectFileSaveError("No source file selected");
      return;
    }
    setProjectFileSaveError(undefined);
    setProjectFileSaveStatus(undefined);
    void saveNativeProjectFile(activeSnapshotProject.id, projectFilePreview.path, projectFileDraft)
      .then((preview) => {
        setProjectFilePreview(preview);
        setProjectFileDraft(preview.body);
        setProjectFileSaveStatus(`Saved ${preview.path}`);
      })
      .catch((error) => {
        setProjectFileSaveError(String(error));
      });
  }

  function handleOpenProjectDiff(path?: string) {
    if (!activeSnapshotProject) return;
    setProjectDiffError(undefined);
    setProjectDiff(undefined);
    setExportedPatch(undefined);
    setImportedPatch(undefined);
    setPrLandingPlan(undefined);
    void readNativeProjectDiff(activeSnapshotProject.id, path)
      .then((diff) => setProjectDiff(diff))
      .catch((error) => {
        setProjectDiff(undefined);
        setProjectDiffError(String(error));
      });
  }

  function handleLoadLocalhostPreview() {
    const result = normalizeLocalhostPreviewUrl(localhostPreviewDraft);
    if (result.error) {
      setLocalhostPreviewUrl(undefined);
      setLocalhostPreviewError(result.error);
      return;
    }
    setLocalhostPreviewError(undefined);
    setLocalhostPreviewUrl(result.url);
  }

  function handleOpenTerminalLink(url: string) {
    const result = normalizeLocalhostPreviewUrl(url);
    if (result.error) {
      setLocalhostPreviewUrl(undefined);
      setLocalhostPreviewError(`Terminal link blocked · ${result.error}`);
      return;
    }
    if (!result.url) return;
    setLocalhostPreviewDraft(result.url);
    setLocalhostPreviewError(undefined);
    setLocalhostPreviewUrl(result.url);
  }

  function handlePlanBrowserAutomation() {
    const result = normalizeLocalhostPreviewUrl(localhostPreviewUrl ?? localhostPreviewDraft);
    if (result.error || !result.url || !activeSnapshotProject) {
      setBrowserAutomationPlan(undefined);
      setBrowserAutomationError(result.error ?? "No active project for browser automation");
      return;
    }
    setBrowserAutomationError(undefined);
    setBrowserAutomationPlan(undefined);
    void planNativeBrowserAutomation({
      projectId: activeSnapshotProject.id,
      url: result.url,
      scenario: "smoke",
    })
      .then((plan) => setBrowserAutomationPlan(plan))
      .catch((error) => {
        setBrowserAutomationPlan(undefined);
        const message = error instanceof Error ? error.message : String(error);
        setBrowserAutomationError(`Browser automation unavailable · ${message}`);
      });
  }

  function handleRunLspDiagnostics() {
    if (!activeSnapshotProject) {
      setProjectLspError("No active project for LSP diagnostics");
      return;
    }
    setProjectLspError(undefined);
    setProjectLspDiagnostics(undefined);
    void collectNativeProjectLspDiagnostics(activeSnapshotProject.id, projectFilePreview?.path)
      .then((diagnostics) => setProjectLspDiagnostics(diagnostics))
      .catch((error) => {
        setProjectLspDiagnostics(undefined);
        const message = error instanceof Error ? error.message : String(error);
        setProjectLspError(`LSP diagnostics unavailable · ${message}`);
      });
  }

  function handleExportProjectPatch() {
    if (!activeSnapshotProject) {
      setPatchWorkflowError("No active project for patch export");
      return;
    }
    setPatchWorkflowError(undefined);
    setExportedPatch(undefined);
    void exportNativeProjectPatch(activeSnapshotProject.id, projectDiff?.path ?? undefined)
      .then((patch) => {
        setExportedPatch(patch);
        setPatchImportBody(patch.body);
      })
      .catch((error) => {
        setExportedPatch(undefined);
        const message = error instanceof Error ? error.message : String(error);
        setPatchWorkflowError(`Patch export unavailable · ${message}`);
      });
  }

  function handleImportProjectPatch() {
    if (!activeSnapshotProject) {
      setPatchWorkflowError("No active project for patch import");
      return;
    }
    setPatchWorkflowError(undefined);
    setImportedPatch(undefined);
    void importNativeProjectPatch(activeSnapshotProject.id, patchImportBody)
      .then((patch) => setImportedPatch(patch))
      .catch((error) => {
        setImportedPatch(undefined);
        const message = error instanceof Error ? error.message : String(error);
        setPatchWorkflowError(`Patch import unavailable · ${message}`);
      });
  }

  function handlePlanPrLanding() {
    if (!activeSnapshotProject) {
      setPatchWorkflowError("No active project for PR planning");
      return;
    }
    setPatchWorkflowError(undefined);
    setPrLandingPlan(undefined);
    void planNativePrLanding({
      projectId: activeSnapshotProject.id,
      title: prLandingTitle.trim() || `${activeSnapshotProject.name} changes`,
      draft: true,
    })
      .then((plan) => setPrLandingPlan(plan))
      .catch((error) => {
        setPrLandingPlan(undefined);
        const message = error instanceof Error ? error.message : String(error);
        setPatchWorkflowError(`PR planning unavailable · ${message}`);
      });
  }

  function focusRelativeTerminalSession(direction: 1 | -1) {
    if (renderedTerminalDeckSessions.length === 0) return;
    const currentIndex = Math.max(
      0,
      renderedTerminalDeckSessions.findIndex((session) => session.id === activeTerminalSessionId),
    );
    const nextIndex = (currentIndex + direction + renderedTerminalDeckSessions.length) % renderedTerminalDeckSessions.length;
    const nextSession = renderedTerminalDeckSessions[nextIndex];
    setFocusedTerminalSessionId(nextSession.id);
    const nextMaximizedSessionId = maximizedTerminalSessionId ? nextSession.id : undefined;
    if (maximizedTerminalSessionId) {
      setMaximizedTerminalSessionId(nextSession.id);
    }
    persistActiveProjectLayout(nextSession.id, nextMaximizedSessionId);
  }

  function toggleMaximizedTerminalSession() {
    if (maximizedTerminalSessionId) {
      setMaximizedTerminalSessionId(undefined);
      persistActiveProjectLayout(activeTerminalSessionId, undefined);
      return;
    }
    if (activeTerminalSessionId) {
      setMaximizedTerminalSessionId(activeTerminalSessionId);
      persistActiveProjectLayout(activeTerminalSessionId, activeTerminalSessionId);
    }
  }

  function persistCommandBlock(block: CommandBlock) {
    void upsertNativeCommandBlock(block)
      .then(() => {
        setCommandBlockPersistError(undefined);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        setCommandBlockPersistStatus(undefined);
        setCommandBlockPersistError(`Command block parser degraded · ${message}`);
      });
  }

  function retryCommandBlockPersistence() {
    const blocks = listCommandBlocks(commandBlockState);
    if (blocks.length === 0) return;
    void Promise.all(blocks.map((block) => upsertNativeCommandBlock(block)))
      .then(() => {
        setCommandBlockPersistError(undefined);
        setCommandBlockPersistStatus("Command block persistence recovered");
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        setCommandBlockPersistStatus(undefined);
        setCommandBlockPersistError(`Command block parser degraded · ${message}`);
      });
  }

  function sendTerminalInput(session: TerminalSession, input: string) {
    if (!session.ptyId) return;
    setCommandBlockState((state) => {
      const result = ingestCommandInput(state, {
        sessionId: session.ptyId ?? session.id,
        input,
        cwd: session.cwd,
        branch: session.branch,
      });
      if (result.createdBlock) {
        persistCommandBlock(result.createdBlock);
      }
      return result.state;
    });
    void writeTerminalPtyInput(session.ptyId, input).catch((error) => {
      setTerminalDeckSessions((sessions) =>
        sessions.some((candidate) => candidate.id === session.id)
          ? sessions.map((candidate) =>
              candidate.id === session.id ? appendTerminalOutput(candidate, [`PTY input failed: ${String(error)}`]) : candidate,
            )
          : [...sessions, appendTerminalOutput(session, [`PTY input failed: ${String(error)}`])],
      );
    });
  }

  function handleTerminalResize(session: TerminalSession, cols: number, rows: number) {
    if (!session.ptyId) return;
    void resizeTerminalPtySession(session.ptyId, cols, rows).catch((error) => {
      setTerminalDeckSessions((sessions) =>
        sessions.some((candidate) => candidate.id === session.id)
          ? sessions.map((candidate) =>
              candidate.id === session.id ? appendTerminalOutput(candidate, [`PTY resize failed: ${String(error)}`]) : candidate,
            )
          : [...sessions, appendTerminalOutput(session, [`PTY resize failed: ${String(error)}`])],
      );
    });
  }

  function removeTerminalPane(session: TerminalSession) {
    setTerminalDeckSessions((sessions) => sessions.filter((candidate) => candidate.id !== session.id));
    setFocusedTerminalSessionId((focused) => ([session.id, session.ptyId].includes(focused ?? "") ? undefined : focused));
    setMaximizedTerminalSessionId((maximized) => (maximized === session.id ? undefined : maximized));
  }

  function handleTerminalClose(session: TerminalSession) {
    if (!session.ptyId) {
      removeTerminalPane(session);
      return;
    }
    void closeTerminalPtySession(session.ptyId)
      .then(() => {
        const owners = { ...ptySessionOwnerRef.current };
        delete owners[session.ptyId as string];
        ptySessionOwnerRef.current = owners;
        setPtySnapshot((snapshot) => {
          const sessions = (snapshot.sessions ?? []).filter((candidate) => candidate.id !== session.ptyId);
          return { total: sessions.length, sessions };
        });
        removeTerminalPane(session);
      })
      .catch((error) => {
        setTerminalDeckSessions((sessions) =>
          sessions.some((candidate) => candidate.id === session.id)
            ? sessions.map((candidate) =>
                candidate.id === session.id ? appendTerminalOutput(candidate, [`PTY close failed: ${String(error)}`]) : candidate,
              )
            : [...sessions, appendTerminalOutput(session, [`PTY close failed: ${String(error)}`])],
        );
      });
  }

  function handleTerminalInput(session: TerminalSession, input: string) {
    if (isDangerousTerminalInput(input)) {
      setDangerousInputError(undefined);
      setPendingDangerousInput({
        session,
        input,
        command: displayTerminalInput(input),
      });
      return;
    }
    setTerminalInputRecordError(undefined);
    void recordNativeSessionInput({
      sessionId: session.id,
      text: input,
      allowDangerous: false,
    }).catch((error) => {
      const message = error instanceof Error ? error.message : "native session input API unavailable";
      setTerminalInputRecordError(`Terminal input recording unavailable · ${message}`);
    });
    sendTerminalInput(session, input);
  }

  function handleAllowDangerousInput() {
    const pending = pendingDangerousInput;
    if (!pending) return;
    setDangerousInputError(undefined);
    void recordNativeSessionInput({
      sessionId: pending.session.id,
      text: pending.input,
      allowDangerous: true,
    })
      .then(() => {
        sendTerminalInput(pending.session, pending.input);
        setPendingDangerousInput(undefined);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        setDangerousInputError(`Dangerous action approval failed · ${message}`);
      });
  }

  function handleAttachCommandBlock(blockId: string) {
    const block = commandBlockState.blocks[blockId];
    if (!block) return;
    setCommandBlockEvidenceError(undefined);
    setEvidencePack((pack) => attachCommandBlockToEvidencePack(pack, block));
    void attachNativeCommandBlockToEvidence({
      evidencePackId: evidencePack.id,
      commandBlockId: block.id,
    }).catch((error) => {
      const message = error instanceof Error ? error.message : "native evidence API unavailable";
      setCommandBlockEvidenceError(`Evidence attachment saved locally · ${message}`);
    });
  }

  function handleCopyCommandBlock(blockId: string) {
    const block = commandBlockState.blocks[blockId];
    if (!block || typeof navigator === "undefined" || !navigator.clipboard?.writeText) return;

    void navigator.clipboard
      .writeText(formatCommandBlockForClipboard(block))
      .then(() => setCopiedCommandBlockId(blockId))
      .catch(() => setCopiedCommandBlockId(undefined));
  }

  function handleExplainCommandBlock(blockId: string) {
    const block = commandBlockState.blocks[blockId];
    if (!block) return;
    setFocusedCommandBlockId(blockId);
    const fallbackExplanation = buildCommandBlockExplanation(block, providerModelSettings);
    void explainNativeCommandBlock(block.id, {
      provider: providerModelSettings.provider,
      model: providerModelSettings.model,
      agentProfileId: providerModelSettings.agentProfileId,
    })
      .then((explanation) => setCommandBlockExplanation(nativeCommandBlockExplanationToCommandBlockExplanation(explanation)))
      .catch(() => setCommandBlockExplanation(fallbackExplanation));
  }

  function handleExportCommandBlockBundle(blockId: string) {
    const block = commandBlockState.blocks[blockId];
    if (!block || typeof navigator === "undefined" || !navigator.clipboard?.writeText) return;
    void exportNativeCommandBlockBundle(block.id)
      .catch(() => exportCommandBlockBundle(block))
      .then((bundle) => navigator.clipboard.writeText(JSON.stringify(bundle, null, 2)))
      .then(() => setExportedCommandBlockId(blockId))
      .catch(() => setExportedCommandBlockId(undefined));
  }

  function handleJumpToCommandBlock(blockId: string) {
    if (!commandBlockState.blocks[blockId]) return;
    setFocusedCommandBlockId(blockId);
  }

  function persistManualCommandBlockChanges(...blocks: (CommandBlock | undefined)[]) {
    blocks.forEach((block) => {
      if (block) persistCommandBlock(block);
    });
  }

  function handleMarkCommandBlock(blockId: string, status: "completed" | "failed") {
    setCommandBlockState((state) => {
      const result = updateCommandBlockStatus(state, blockId, status);
      if (result.updatedBlock) {
        void markNativeCommandBlock(blockId, status).catch(() => persistManualCommandBlockChanges(result.updatedBlock));
      }
      return result.state;
    });
    setFocusedCommandBlockId(blockId);
  }

  function handleMergeCommandBlockWithPrevious(blockId: string) {
    setCommandBlockState((state) => {
      const blocks = listCommandBlocks(state);
      const blockIndex = blocks.findIndex((block) => block.id === blockId);
      const previousBlock = blockIndex > 0 ? blocks[blockIndex - 1] : undefined;
      if (!previousBlock) return state;
      const result = mergeCommandBlocks(state, previousBlock.id, blockId);
      if (result.mergedBlock) {
        void mergeNativeCommandBlocks(previousBlock.id, blockId).catch(() =>
          persistManualCommandBlockChanges(result.mergedBlock),
        );
        setFocusedCommandBlockId(result.mergedBlock.id);
      }
      return result.state;
    });
  }

  function handleSplitCommandBlock(blockId: string) {
    setCommandBlockState((state) => {
      const result = splitCommandBlock(state, blockId);
      if (result.updatedBlock && result.createdBlock) {
        void splitNativeCommandBlock(blockId).catch(() =>
          persistManualCommandBlockChanges(result.updatedBlock, result.createdBlock),
        );
        setFocusedCommandBlockId(result.createdBlock.id);
      }
      return result.state;
    });
  }

  function handleQuickTaskCreate() {
    const title = quickTaskTitle;
    setQuickTaskError(undefined);
    const result = addTask(taskState, {
      title,
      projectId: activeProjectId,
    });
    if (!result.createdTask) return;
    const optimisticTask = result.createdTask;

    setTaskState(result.state);
    setQuickTaskTitle("");
    void createNativeTask({
      projectId: activeProjectId,
      title,
    })
      .then((nativeTask) => {
        const task = nativeTaskToHaneulchiTask(nativeTask);
        setTaskState((state) => {
          const nextTasks = { ...state.tasks };
          delete nextTasks[optimisticTask.id];
          return {
            tasks: {
              ...nextTasks,
              [task.id]: task,
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setQuickTaskError(`Quick task saved locally · ${message}`);
      });
  }

  function handleSaveTaskView() {
    const view = createTaskSavedView(taskFilterQuery);
    if (!view) return;
    setTaskViewError(undefined);
    setTaskSavedViews((views) => {
      const nextViews = [view, ...views.filter((item) => item.id !== view.id)].slice(0, 6);
      const error = saveTaskSavedViews(taskStateProjectId, nextViews);
      if (error) setTaskViewError(`Task view saved for this session · ${error}`);
      return nextViews;
    });
  }

  function handleDeleteTaskView(viewId: string) {
    setTaskViewError(undefined);
    setTaskSavedViews((views) => {
      const nextViews = views.filter((view) => view.id !== viewId);
      const error = saveTaskSavedViews(taskStateProjectId, nextViews);
      if (error) setTaskViewError(`Task view removed for this session · ${error}`);
      return nextViews;
    });
  }

  function handleAdvanceTask(taskId: string) {
    const advanced = advanceTask(taskState, taskId);
    const nextStatus = advanced.tasks[taskId]?.status;
    setTaskViewError(undefined);
    setTaskState(advanced);
    if (!nextStatus || nextStatus === taskState.tasks[taskId]?.status) return;
    void moveNativeTask(taskId, nextStatus)
      .then((nativeTask) => {
        const task = nativeTaskToHaneulchiTask(nativeTask);
        setTaskState((state) => ({
          tasks: {
            ...state.tasks,
            [task.id]: {
              ...state.tasks[task.id],
              ...task,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskViewError(`Task status saved locally · ${message}`);
      });
  }

  function handleOpenTaskDrawer(taskId: string) {
    setSelectedTaskId(taskId);
    setTaskCommentError(undefined);
    setTaskSubtaskError(undefined);
    void listNativeTaskComments(taskId)
      .then((nativeComments) => {
        setTaskState((state) => {
          const task = state.tasks[taskId];
          if (!task) return state;
          return {
            tasks: {
              ...state.tasks,
              [taskId]: {
                ...task,
                comments: nativeComments.map((comment) => ({
                  id: comment.id,
                  author: comment.author_id || comment.author_type,
                  body: comment.body_md,
                })),
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskCommentError(`Task comments unavailable · ${message}`);
      });
    void listNativeTaskSubtasks(taskId)
      .then((nativeSubtasks) => {
        setTaskState((state) => {
          const task = state.tasks[taskId];
          if (!task) return state;
          return {
            tasks: {
              ...state.tasks,
              [taskId]: {
                ...task,
                subtasks: nativeSubtasks.map((subtask) => ({
                  id: subtask.id,
                  title: subtask.title,
                  status: subtask.status,
                })),
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskSubtaskError(`Task subtasks unavailable · ${message}`);
      });
  }

  function handleSaveTaskWorkpad() {
    if (!selectedTaskOverview) return;
    const taskId = selectedTaskOverview.id;
    const body = taskWorkpadDraft;
    setTaskWorkpadError(undefined);
    setTaskState((state) => updateTaskWorkpad(state, selectedTaskOverview.id, taskWorkpadDraft));
    void saveNativeTaskWorkpad({ taskId, body })
      .then((nativeWorkpad) => {
        setTaskState((state) => updateTaskWorkpad(state, taskId, nativeWorkpad.body_md));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskWorkpadError(`Task workpad save failed · ${message}`);
      });
  }

  function handleAddTaskComment() {
    if (!selectedTaskOverview) return;
    const body = newTaskComment;
    const result = addTaskComment(taskState, selectedTaskOverview.id, {
      author: "human",
      body,
    });
    if (!result.createdComment) return;
    const optimisticComment = result.createdComment;

    setTaskCommentError(undefined);
    setTaskState(result.state);
    setNewTaskComment("");
    void addNativeTaskComment({
      taskId: selectedTaskOverview.id,
      body,
    })
      .then((nativeComment) => {
        setTaskState((state) => {
          const task = state.tasks[selectedTaskOverview.id];
          if (!task) return state;
          const comments = task.comments ?? [];
          return {
            tasks: {
              ...state.tasks,
              [task.id]: {
                ...task,
                comments: comments.map((comment) =>
                  comment.id === optimisticComment.id
                    ? { id: nativeComment.id, author: nativeComment.author_id, body: nativeComment.body_md }
                    : comment,
                ),
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskCommentError(`Task comment save failed · ${message}`);
      });
  }

  function handleAddTaskSubtask() {
    if (!selectedTaskOverview) return;
    const taskId = selectedTaskOverview.id;
    const result = addTaskSubtask(taskState, taskId, { title: newTaskSubtaskTitle });
    if (!result.createdSubtask) return;
    setTaskSubtaskError(undefined);
    setTaskState(result.state);
    setNewTaskSubtaskTitle("");
    void addNativeTaskSubtask({ taskId, title: result.createdSubtask.title })
      .then((nativeSubtask) => {
        setTaskState((state) => {
          const task = state.tasks[taskId];
          if (!task) return state;
          return {
            tasks: {
              ...state.tasks,
              [taskId]: {
                ...task,
                subtasks: (task.subtasks ?? []).map((subtask) =>
                  subtask.id === result.createdSubtask?.id
                    ? { id: nativeSubtask.id, title: nativeSubtask.title, status: nativeSubtask.status }
                    : subtask,
                ),
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskSubtaskError(`Task subtask save failed · ${message}`);
      });
  }

  function handleUpdateTaskSubtaskStatus(subtaskId: string, status: "open" | "done") {
    if (!selectedTaskOverview) return;
    const taskId = selectedTaskOverview.id;
    setTaskSubtaskError(undefined);
    setTaskState((state) => updateTaskSubtaskStatus(state, taskId, subtaskId, status));
    void updateNativeTaskSubtaskStatus({ taskId, subtaskId, status })
      .then((nativeSubtask) => {
        setTaskState((state) => {
          const task = state.tasks[taskId];
          if (!task) return state;
          return {
            tasks: {
              ...state.tasks,
              [taskId]: {
                ...task,
                subtasks: (task.subtasks ?? []).map((subtask) =>
                  subtask.id === nativeSubtask.id
                    ? { id: nativeSubtask.id, title: nativeSubtask.title, status: nativeSubtask.status }
                    : subtask,
                ),
              },
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskSubtaskError(`Task subtask update failed · ${message}`);
      });
  }

  function handleSaveTaskPlanningProperties() {
    if (!selectedTaskOverview) return;
    const taskId = selectedTaskOverview.id;
    const labels = parseTaskLabelsDraft(taskLabelsDraft);
    setTaskPlanningError(undefined);
    setTaskPlanningStatus(undefined);
    setTaskState((state) =>
      updateTaskPlanningProperties(state, taskId, {
        cycle: taskCycleDraft,
        module: taskModuleDraft,
        initiative: taskInitiativeDraft,
        labels,
        dueDate: taskDueDateDraft,
        estimate: taskEstimateDraft,
        assignee: taskAssigneeDraft,
      }),
    );
    void updateNativeTaskPlanning({
      taskId,
      cycle: taskCycleDraft,
      module: taskModuleDraft,
      initiative: taskInitiativeDraft,
      labels,
      dueDate: taskDueDateDraft,
      estimate: taskEstimateDraft,
      assignee: taskAssigneeDraft,
    })
      .then((nativeTask) => {
        const task = nativeTaskToHaneulchiTask(nativeTask);
        setTaskState((state) => ({
          tasks: {
            ...state.tasks,
            [task.id]: {
              ...state.tasks[task.id],
              ...task,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskPlanningError(`Task planning unavailable · ${message}`);
      });
  }

  function handleCreateTaskCycle() {
    const name = taskCycleDraft.trim();
    if (!name) {
      setTaskPlanningStatus(undefined);
      setTaskPlanningError("Task cycle name is required");
      return;
    }

    setTaskPlanningError(undefined);
    setTaskPlanningStatus("Creating task cycle");
    void createNativeTaskCycle({ projectId: activeProjectId, name })
      .then((cycle) => {
        setTaskCycles((cycles) => [...cycles.filter((item) => item.id !== cycle.id), cycle]);
        setTaskCycleDraft(cycle.name);
        setTaskPlanningStatus(`Created cycle ${cycle.name} · ${cycle.status}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskPlanningStatus(undefined);
        setTaskPlanningError(`Task cycle unavailable · ${message}`);
      });
  }

  function handleCreateTaskModule() {
    const name = taskModuleDraft.trim();
    if (!name) {
      setTaskPlanningStatus(undefined);
      setTaskPlanningError("Task module name is required");
      return;
    }

    setTaskPlanningError(undefined);
    setTaskPlanningStatus("Creating task module");
    void createNativeTaskModule({ projectId: activeProjectId, name })
      .then((taskModule) => {
        setTaskModules((modules) => [...modules.filter((item) => item.id !== taskModule.id), taskModule]);
        setTaskModuleDraft(taskModule.name);
        setTaskPlanningStatus(`Created module ${taskModule.name} · ${taskModule.status}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskPlanningStatus(undefined);
        setTaskPlanningError(`Task module unavailable · ${message}`);
      });
  }

  function handleCreateRoadmapInitiative() {
    const name = roadmapInitiativeNameDraft.trim();
    const status = roadmapInitiativeStatusDraft.trim() || "planned";
    if (!name) {
      setRoadmapInitiativeStatus(undefined);
      setRoadmapInitiativeError("Roadmap initiative name is required");
      return;
    }

    setRoadmapInitiativeError(undefined);
    setRoadmapInitiativeStatus("Creating roadmap initiative");
    void createNativeInitiative({ projectId: activeProjectId, name, status })
      .then((initiative) => {
        setStateSnapshot((snapshot) => mergeInitiativesIntoStateSnapshot(snapshot, [initiative]));
        setRoadmapInitiativeNameDraft("");
        setRoadmapInitiativeStatusDraft(initiative.status || status);
        setRoadmapInitiativeStatus(`Created ${initiative.name} · ${initiative.status}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native initiative API unavailable";
        setRoadmapInitiativeStatus(undefined);
        setRoadmapInitiativeError(`Roadmap initiative unavailable · ${message}`);
      });
  }

  function handleAttachTaskContextPack() {
    if (!selectedTaskOverview) return;
    const taskId = selectedTaskOverview.id;
    const contextPackId = taskContextPackDraft || undefined;
    setTaskContextStatus(undefined);
    setTaskContextError(undefined);
    setTaskState((state) => updateTaskContextPack(state, taskId, contextPackId));
    void updateNativeTaskContext({ taskId, contextPackId })
      .then((nativeTask) => {
        const task = nativeTaskToHaneulchiTask(nativeTask);
        setTaskState((state) => ({
          tasks: {
            ...state.tasks,
            [task.id]: {
              ...state.tasks[task.id],
              ...task,
            },
          },
        }));
        setTaskContextStatus(`Attached context pack ${task.contextPackId ?? "default"}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskContextError(`Task context attach failed · ${message}`);
      });
  }

  function handleDispatchTask(taskId: string) {
    const task = taskState.tasks[taskId];
    const taskContextPackId = task?.contextPackId;
    const agentProfileId = task?.assignee ?? localAgentProfileId;
    setTaskDispatchError(undefined);
    void dispatchNativeRun({
      taskId,
      agentProfileId,
      contextPackId: taskContextPackId ?? defaultContextPackId,
    })
      .then(applyNativeRunMutation)
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run API unavailable";
        setTaskDispatchError(`Dispatch blocked · ${message}`);
      });
  }

  function handleRunLifecycle(runId: string, lifecycle: RunLifecycle, statusDetail?: string) {
    setRunLifecycleError(undefined);
    const mutation = statusDetail
      ? updateNativeRunLifecycle(runId, lifecycle, statusDetail)
      : updateNativeRunLifecycle(runId, lifecycle);
    void mutation
      .then(applyNativeRunMutation)
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run API unavailable";
        setRunLifecycleError(`Run lifecycle unavailable · ${message}`);
      });
  }

  function handleCancelRun(runId: string) {
    setRunLifecycleError(undefined);
    void cancelNativeRun(runId)
      .then(applyNativeRunMutation)
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run API unavailable";
        setRunLifecycleError(`Run lifecycle unavailable · ${message}`);
      });
  }

  function handleRetryRun(runId: string) {
    setRunLifecycleError(undefined);
    void retryNativeRun(runId)
      .then(applyNativeRunMutation)
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run API unavailable";
        setRunLifecycleError(`Run lifecycle unavailable · ${message}`);
      });
  }

  function handleGenerateEvidenceForRun(runId: string) {
    const evidencePackId = `ev_${runId}`;
    setEvidenceGenerationError(undefined);
    void generateNativeEvidencePackForRun({ runId, evidencePackId })
      .then((evidence) => {
        setGeneratedEvidencePacksByRun((packs) => ({ ...packs, [runId]: evidence }));
        setStateSnapshot((snapshot) => {
          const review: StateReview = {
            id: `review_${evidence.id}`,
            state: "pending",
            evidence_pack_id: evidence.id,
            task_id: evidence.task_id,
            run_id: evidence.run_id,
            completeness_state: evidence.completeness_state,
          };
          const reviews = snapshot.reviews.some((item) => item.evidence_pack_id === evidence.id)
            ? snapshot.reviews.map((item) => (item.evidence_pack_id === evidence.id ? { ...item, ...review } : item))
            : [...snapshot.reviews, review];
          return { ...snapshot, reviews };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native evidence API unavailable";
        setEvidenceGenerationError(`Evidence generation unavailable · ${message}`);
      });
  }

  function handleLoadRunReplayMetadata(runId: string) {
    setRunReplayError(undefined);
    void getRunReplayMetadata(runId)
      .then((replay) => {
        if (!replay) {
          setRunReplayMetadataByRun((items) => {
            const next = { ...items };
            delete next[runId];
            return next;
          });
          setRunReplayError(`Run replay unavailable · no replay metadata for ${runId}`);
          return;
        }
        setRunReplayMetadataByRun((items) => ({ ...items, [runId]: replay }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run replay API unavailable";
        setRunReplayError(`Run replay unavailable · ${message}`);
      });
  }

  function handleLoadRuns() {
    setRunLifecycleError(undefined);
    void listNativeRuns(activeProjectId)
      .then((runs) => {
        const loadedRuns = runs.map(nativeRunToStateRunSummary);
        setStateSnapshot((snapshot) => {
          const items = [
            ...snapshot.runs.items.filter((run) => run.project_id !== activeProjectId),
            ...loadedRuns,
          ];
          const counts = items.reduce<Record<string, number>>((nextCounts, item) => {
            nextCounts[item.lifecycle] = (nextCounts[item.lifecycle] ?? 0) + 1;
            return nextCounts;
          }, {});
          return {
            ...snapshot,
            runs: {
              items,
              counts_by_lifecycle: counts,
            },
          };
        });
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run API unavailable";
        setRunLifecycleError(`Run list unavailable · ${message}`);
      });
  }

  function handleRunDetailDraft(runId: string, value: string) {
    setRunStatusDetailDrafts((drafts) => ({
      ...drafts,
      [runId]: value,
    }));
  }

  function handleRunStatusUpdateDraft(runId: string, value: string) {
    setRunStatusUpdateDrafts((drafts) => ({
      ...drafts,
      [runId]: value,
    }));
  }

  function handlePostRunStatusUpdate(runId: string) {
    const bodyMd = runStatusUpdateDrafts[runId]?.trim();
    if (!bodyMd) return;
    setRunLifecycleError(undefined);
    setRunStatusUpdateStatus(undefined);
    void recordNativeRunStatusUpdate({ runId, bodyMd })
      .then((comment) => {
        setRunStatusUpdateStatus(`Status update posted · ${comment.id}`);
        setRunStatusUpdateDrafts((drafts) => ({
          ...drafts,
          [runId]: "",
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native run status API unavailable";
        setRunLifecycleError(`Run status update unavailable · ${message}`);
      });
  }

  function applyWorkflowMutation(workflow: PersistedWorkflowVersion) {
    setStateSnapshot((snapshot) => ({
      ...snapshot,
      workflow: {
        ...snapshot.workflow,
        valid: workflow.valid,
        invalid_projects: workflow.valid ? [] : [localProjectId],
        current_version_id: workflow.id,
        last_known_good_version_id: workflow.valid
          ? workflow.id
          : snapshot.workflow.last_known_good_version_id,
        diagnostics: workflow.diagnostics_json,
      },
    }));
  }

  function applyWorkflowRuntimeState(runtime: WorkflowRuntimeState) {
    setStateSnapshot((snapshot) => ({
      ...snapshot,
      workflow: {
        ...snapshot.workflow,
        valid: runtime.valid,
        invalid_projects: runtime.valid ? [] : [localProjectId],
        current_version_id: runtime.current_version_id,
        last_known_good_version_id: runtime.last_known_good_version_id,
        diagnostics: runtime.diagnostics,
      },
    }));
  }

  function handleValidateSampleWorkflow() {
    setWorkflowControlError(undefined);
    setWorkflowValidationResult(undefined);
    void validateWorkflow({
      projectId: localProjectId,
      sourcePath: "WORKFLOW.md",
      content: sampleWorkflowDocument,
    })
      .then((result) => {
        setWorkflowValidationResult(result);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          workflow: {
            ...snapshot.workflow,
            valid: result.valid,
            invalid_projects: result.valid ? [] : [localProjectId],
            diagnostics: result.diagnostics_json,
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "workflow validation unavailable";
        setWorkflowValidationResult(undefined);
        setWorkflowControlError(`Workflow validation unavailable · ${message}`);
      });
  }

  function handleRefreshWorkflowRuntime() {
    setWorkflowControlError(undefined);
    setWorkflowRuntimeResult(undefined);
    void getWorkflowRuntimeState(localProjectId)
      .then((runtime) => {
        setWorkflowRuntimeResult(runtime);
        applyWorkflowRuntimeState(runtime);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "workflow runtime unavailable";
        setWorkflowRuntimeResult(undefined);
        setWorkflowControlError(`Workflow runtime unavailable · ${message}`);
      });
  }

  function handleRunWorkflowHook() {
    const runId = workflowHookRunId.trim();
    const hookName = workflowHookName.trim();
    const repoRoot = workflowHookRepoRoot.trim();
    const workspacePath = workflowHookWorkspacePath.trim();
    if (!runId || !hookName || !repoRoot) {
      setWorkflowHookResult(undefined);
      setWorkflowControlError("Workflow hook run id hook name and repo root are required");
      return;
    }
    setWorkflowControlError(undefined);
    setWorkflowHookResult(undefined);
    void runWorkflowHook({
      runId,
      hookName,
      repoRoot,
      workspacePath: workspacePath || undefined,
    })
      .then(setWorkflowHookResult)
      .catch((error) => {
        const message = error instanceof Error ? error.message : "workflow hook unavailable";
        setWorkflowHookResult(undefined);
        setWorkflowControlError(`Workflow hook unavailable · ${message}`);
      });
  }

  function handleReloadSampleWorkflow() {
    setWorkflowMarketplaceStatus(undefined);
    setWorkflowMarketplaceError(undefined);
    void reloadWorkflow({
      projectId: localProjectId,
      sourcePath: "WORKFLOW.md",
      content: sampleWorkflowDocument,
    })
      .then(applyWorkflowMutation)
      .catch((error) => {
        const message = error instanceof Error ? error.message : "workflow reload unavailable";
        setWorkflowMarketplaceError(`Workflow reload unavailable · ${message}`);
      });
  }

  function handleImportWorkflowPreset(preset: (typeof workflowMarketplacePresets)[number]) {
    setWorkflowMarketplaceStatus(undefined);
    setWorkflowMarketplaceError(undefined);
    void reloadWorkflow({
      projectId: localProjectId,
      sourcePath: preset.sourcePath,
      content: preset.content,
    })
      .then((workflow) => {
        applyWorkflowMutation(workflow);
        setWorkflowMarketplaceStatus(`Imported ${preset.name} workflow ${workflow.id}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "workflow import unavailable";
        setWorkflowMarketplaceError(`Import failed · ${message}`);
      });
  }

  function handleReleaseChannelChange(channel: ReleaseChannel) {
    setReleaseChannel(channel);
    saveReleaseChannel(channel);
    setUpdateFeedCheck(undefined);
    setUpdateFeedError(undefined);
  }

  function handleCheckUpdateFeed() {
    setUpdateFeedError(undefined);
    setUpdateFeedCheck(undefined);
    void fetch(`/update-feed/${releaseChannel}.json`, { cache: "no-store" })
      .then(async (response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return response.json() as Promise<{
          version?: string;
          platforms?: Record<string, { signature?: string; url?: string }>;
        }>;
      })
      .then((feed) => {
        const platform = "darwin-aarch64";
        const entry = feed.platforms?.[platform];
        setUpdateFeedCheck({
          channel: releaseChannel,
          version: feed.version ?? "unknown",
          platform,
          url: entry?.url,
          signatureState: classifyUpdateFeedSignature(entry?.signature),
        });
      })
      .catch((error) => {
        setUpdateFeedError(`Update feed unavailable · ${error instanceof Error ? error.message : String(error)}`);
      });
  }

  function handleRefreshReleaseWorkflows() {
    setReleaseWorkflowStatus(undefined);
    setReleaseWorkflowError(undefined);
    void getNativeReleaseWorkflowStatus()
      .then((status) => setReleaseWorkflowStatus(status))
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native release workflow API unavailable";
        setReleaseWorkflowStatus(undefined);
        setReleaseWorkflowError(`Release workflow diagnostics unavailable · ${message}`);
      });
  }

  function handleImportTerminalTheme() {
    setTerminalThemeStatus(undefined);
    setTerminalThemeError(undefined);
    try {
      const theme = parseTerminalTheme(terminalThemeJson);
      setTerminalTheme(theme);
      window.localStorage?.setItem(terminalThemeStorageKey, JSON.stringify(theme));
      setTerminalThemeJson(terminalThemeToJson(theme));
      setTerminalThemeStatus(`Imported terminal theme ${theme.name}`);
    } catch (error) {
      setTerminalThemeError(`Terminal theme import failed · ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  function handleExportTerminalTheme() {
    try {
      const theme = terminalTheme ?? parseTerminalTheme(terminalThemeJson);
      setTerminalThemeJson(terminalThemeToJson(theme));
      setTerminalThemeStatus(`Exported terminal theme ${theme.name}`);
      setTerminalThemeError(undefined);
    } catch (error) {
      setTerminalThemeStatus(undefined);
      setTerminalThemeError(`Terminal theme export failed · ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function handleSaveProjectTerminalTheme() {
    setTerminalThemeStatus(undefined);
    setTerminalThemeError(undefined);
    try {
      const theme = terminalTheme ?? parseTerminalTheme(terminalThemeJson);
      try {
        const savedTheme = nativeTerminalThemeToTerminalTheme(
          await upsertNativeTerminalThemeSettings({
            projectId: activeProjectId,
            name: theme.name,
            background: theme.background,
            foreground: theme.foreground,
            accent: theme.accent,
          }),
        );
        window.localStorage?.setItem(projectTerminalThemeStorageKey(activeProjectId), JSON.stringify(savedTheme));
        setActiveProjectTerminalTheme(savedTheme);
        setTerminalThemeStatus(`Saved project terminal theme ${savedTheme.name} for ${activeProjectName}`);
      } catch {
        window.localStorage?.setItem(projectTerminalThemeStorageKey(activeProjectId), JSON.stringify(theme));
        setActiveProjectTerminalTheme(theme);
        setTerminalThemeStatus(`Saved project terminal theme ${theme.name} for ${activeProjectName}`);
      }
    } catch (error) {
      setTerminalThemeError(`Project terminal theme save failed · ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  function applyProviderModelDefaults(settings: ProviderModelSettings) {
    setProviderModelSettings(settings);
    setProviderModelProvider(settings.provider);
    setProviderModelName(settings.model);
    setProviderModelAgent(settings.agentProfileId);
    saveProviderModelSettings(settings);
    setTokenUsageAdapter(providerTokenUsageAdapter(settings.provider));
    setTokenUsageAdapterAgent(settings.agentProfileId);
    setTokenUsageAdapterPayload(providerModelPayload(settings));
  }

  async function handleLoadProviderModel() {
    setProviderModelStatus(undefined);
    setProviderModelError(undefined);
    try {
      const loaded = await getNativeProviderModelSettings();
      const settings = {
        provider: loaded.provider,
        model: loaded.model,
        agentProfileId: loaded.agent_profile_id,
      };
      applyProviderModelDefaults(settings);
      setProviderModelStatus(`Loaded ${settings.provider} · ${settings.model}`);
    } catch (error) {
      setProviderModelError(`Provider model native load failed · ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function handleSaveProviderModel() {
    const settings: ProviderModelSettings = {
      provider: providerModelProvider.trim(),
      model: providerModelName.trim(),
      agentProfileId: providerModelAgent.trim(),
    };
    if (!settings.provider || !settings.model || !settings.agentProfileId) {
      setProviderModelStatus(undefined);
      setProviderModelError("Provider model settings require provider model and agent");
      return;
    }
    try {
      await upsertNativeProviderModelSettings(settings);
      applyProviderModelDefaults(settings);
      setProviderModelStatus(`Saved ${settings.provider} · ${settings.model}`);
      setProviderModelError(undefined);
    } catch (error) {
      applyProviderModelDefaults(settings);
      setProviderModelStatus(`Saved local fallback ${settings.provider} · ${settings.model}`);
      setProviderModelError(`Provider model native save failed · ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  function handleReviewDecision(review: StateReview, decision: "approved" | "changes_requested" | "reopened" | "blocked") {
    const bodyByDecision = {
      approved: "Approved from Review Queue.",
      changes_requested: "Changes requested from Review Queue.",
      reopened: "Reopened from Review Queue.",
      blocked: "Marked blocked from Review Queue.",
    } satisfies Record<typeof decision, string>;
    const taskStatusByDecision = {
      approved: "done",
      changes_requested: "blocked",
      reopened: "ready",
      blocked: "blocked",
    } satisfies Record<typeof decision, TaskStatus>;
    setReviewActionError(undefined);
    setReviewActionStatus(undefined);
    void recordNativeEvidenceReviewDecision({
      evidencePackId: review.evidence_pack_id,
      decision,
      reviewerId: "human",
      bodyMd: bodyByDecision[decision],
    })
      .then(() => {
        setReviewActionStatus(`Review decision recorded · ${decision} ${review.id}`);
        setReviewActionError(undefined);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          reviews: snapshot.reviews.map((item) => (item.id === review.id ? { ...item, state: decision } : item)),
        }));
        if (review.task_id) {
          setTaskState((state) => moveTask(state, review.task_id ?? "", taskStatusByDecision[decision]));
        }
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native evidence API unavailable";
        setReviewActionError(`Review decision unavailable · ${message}`);
      });
  }

  function findReviewSession(review: StateReview) {
    if (!review.run_id) return undefined;
    return sessionItems.find((session) => session.run_id === review.run_id);
  }

  function handleOpenReviewTerminal(review: StateReview) {
    const session = findReviewSession(review);
    if (!session) return;
    handleSessionFocus(session);
  }

  function handleOpenReviewWorktree() {
    handleOpenProjectDiff();
  }

  function reviewProjectId(review: StateReview) {
    return runItems.find((run) => run.id === review.run_id)?.project_id ?? activeSnapshotProject?.id ?? activeProjectId;
  }

  function handleCopyReviewPatch(review: StateReview) {
    const projectId = reviewProjectId(review);
    setReviewActionError(undefined);
    setReviewActionStatus(undefined);
    void exportNativeProjectPatch(projectId, undefined)
      .then(async (patch) => {
        await window.navigator.clipboard.writeText(patch.body);
        setReviewActionStatus(`Copied patch ${patch.patch_id} for ${review.id}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : String(error);
        setReviewActionError(`Patch copy unavailable · ${message}`);
      });
  }

  function handlePlanReviewPrLanding(review: StateReview) {
    const title = `PR: ${review.id}`;
    setReviewActionError(undefined);
    setReviewActionStatus(undefined);
    void planNativeReviewPrLanding({
      reviewId: review.id,
      title,
      draft: true,
    })
      .then((receipt) => {
        setPrLandingPlan(receipt.plan);
        setReviewActionStatus(`PR landing planned for ${review.id}`);
      })
      .catch((error) => {
        setPrLandingPlan(undefined);
        const message = error instanceof Error ? error.message : String(error);
        setReviewActionError(`PR landing unavailable · ${message}`);
      });
  }

  function handleCreateReviewFollowUp(review: StateReview) {
    const title = `Follow-up: ${review.id}`;
    const projectId = reviewProjectId(review);
    const result = addTask(taskState, {
      title,
      projectId,
      priority: "high",
    });
    if (!result.createdTask) return;

    const optimisticTask = result.createdTask;
    setTaskState(result.state);
    setReviewActionError(undefined);
    setReviewActionStatus(`Follow-up task created for ${review.id}`);
    void createNativeReviewFollowUpTask({
      reviewId: review.id,
      title,
      priority: "high",
    })
      .then((receipt) => {
        const task = nativeTaskToHaneulchiTask(receipt.task);
        setTaskState((state) => {
          const nextTasks = { ...state.tasks };
          delete nextTasks[optimisticTask.id];
          return {
            tasks: {
              ...nextTasks,
              [task.id]: task,
            },
          };
        });
      })
      .catch((error) => {
        setReviewActionError(`Follow-up saved locally; native sync unavailable: ${String(error)}`);
      });
  }

  function handlePolicyDecision(item: StateAttentionItem, decision: "approved" | "denied") {
    setPolicyApprovalError(undefined);
    void decideNativePolicyApproval({
      approvalId: item.id,
      decision,
      decisionBy: "human",
      decisionNote: decision === "approved" ? "Approved from Policy Approvals." : "Denied from Policy Approvals.",
    })
      .then((approval) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          attention: snapshot.attention.filter((candidate) => candidate.id !== item.id),
          runs: {
            ...snapshot.runs,
            items: snapshot.runs.items.map((run) =>
              approval.run_id && run.id === approval.run_id
                ? { ...run, lifecycle: decision === "approved" ? "running" : "blocked" }
                : run,
            ),
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native policy approval API unavailable";
        setPolicyApprovalError(`Policy approval decision unavailable · ${message}`);
      });
  }

  function handleLoadPolicyApprovals() {
    setPolicyApprovalError(undefined);
    void listNativePolicyApprovals(activeProjectId, "pending")
      .then((approvals) => {
        const pendingApprovals = approvals
          .filter((approval) => approval.state === "pending")
          .map(nativePolicyApprovalToAttention);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          attention: [
            ...snapshot.attention.filter((item) => !isPolicyApprovalAttention(item)),
            ...pendingApprovals,
          ],
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native policy approval API unavailable";
        setPolicyApprovalError(`Policy approvals unavailable · ${message}`);
      });
  }

  function handleSavePolicyPack() {
    const name = policyPackName.trim();
    if (!name) {
      setPolicyPackResult(undefined);
      setPolicyPackError("Policy pack name is required");
      return;
    }
    setPolicyPackError(undefined);
    setPolicyPackResult(undefined);
    void upsertNativePolicyPack({
      projectId: localProjectId,
      name,
      sandboxMode: policyPackSandboxMode,
      network: policyPackNetwork,
      networkProfile: policyPackNetworkProfile,
      fileWrite: policyPackFileWrite,
      tools: "ask",
      approvalRequired: splitCommaList(policyPackApprovals),
      forbiddenOperations: splitCommaList(policyPackForbidden),
      setActive: true,
    })
      .then((pack) => {
        setPolicyPackResult(pack);
        setPolicyPacks((packs) => [
          pack,
          ...packs
            .filter((candidate) => candidate.id !== pack.id)
            .map((candidate) => (pack.active ? { ...candidate, active: false } : candidate)),
        ]);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          security: {
            keychain: snapshot.security?.keychain ?? "unknown",
            secret_count: snapshot.security?.secret_count ?? 0,
            redaction: snapshot.security?.redaction,
            permission_audit: snapshot.security?.permission_audit,
            diagnostics: snapshot.security?.diagnostics,
            policy_pack: {
              id: pack.id,
              name: pack.name,
              sandbox_mode: pack.sandbox_mode,
              network: pack.network,
              network_profile: pack.network_profile,
              file_write: pack.file_write,
              tools: pack.tools,
              approval_required_count: pack.approval_required.length,
              forbidden_count: pack.forbidden_operations.length,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native policy pack API unavailable";
        setPolicyPackResult(undefined);
        setPolicyPackError(`Policy pack unavailable · ${message}`);
      });
  }

  function handleLoadPolicyPacks() {
    setPolicyPackError(undefined);
    void listNativePolicyPacks(localProjectId)
      .then((packs) => {
        setPolicyPacks(packs);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native policy pack API unavailable";
        setPolicyPacks([]);
        setPolicyPackError(`Policy packs unavailable · ${message}`);
      });
  }

  function handleEvaluatePolicyAction() {
    const actionKind = policyActionKind.trim();
    if (!actionKind) {
      setPolicyEvaluation(undefined);
      setPolicyPackError("Policy action kind is required");
      return;
    }
    setPolicyPackError(undefined);
    setPolicyEvaluation(undefined);
    void evaluateNativePolicyAction({
      projectId: activeProjectId,
      actionKind,
      command: policyActionCommand.trim() || undefined,
      requestedBy: "human",
    })
      .then((evaluation) => {
        setPolicyEvaluation(evaluation);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          security: {
            keychain: snapshot.security?.keychain ?? "unknown",
            secret_count: snapshot.security?.secret_count ?? 0,
            redaction: snapshot.security?.redaction,
            policy_pack: snapshot.security?.policy_pack,
            diagnostics: snapshot.security?.diagnostics,
            permission_audit: {
              ...snapshot.security?.permission_audit,
              recent_count: (snapshot.security?.permission_audit?.recent_count ?? 0) + 1,
              forbidden_count: evaluation.decision === "forbidden"
                ? (snapshot.security?.permission_audit?.forbidden_count ?? 0) + 1
                : snapshot.security?.permission_audit?.forbidden_count ?? 0,
              latest_decision: evaluation.decision,
              latest_action_kind: evaluation.action_kind,
            },
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native policy evaluation API unavailable";
        setPolicyEvaluation(undefined);
        setPolicyPackError(`Policy evaluation unavailable · ${message}`);
      });
  }

  function handleCreatePolicyApproval() {
    const actionKind = policyActionKind.trim();
    if (!actionKind) {
      setPolicyApprovalError("Policy action kind is required");
      return;
    }
    setPolicyApprovalError(undefined);
    void createNativePolicyApproval({
      projectId: activeProjectId,
      actionKind,
      command: policyActionCommand.trim() || undefined,
      riskLevel: policyApprovalRiskLevel,
      requestedBy: "human",
    })
      .then((approval) => {
        const attentionItem = nativePolicyApprovalToAttention(approval);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          attention: [
            ...snapshot.attention.filter((item) => item.id !== attentionItem.id),
            attentionItem,
          ],
          runs: {
            ...snapshot.runs,
            items: snapshot.runs.items.map((run) =>
              approval.run_id && run.id === approval.run_id
                ? {
                    ...run,
                    lifecycle: "permission_requested",
                    status_detail: `Permission requested: ${approval.action_kind}${approval.command ? ` (${approval.command})` : ""}`,
                  }
                : run,
            ),
          },
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native policy approval API unavailable";
        setPolicyApprovalError(`Policy approval unavailable · ${message}`);
      });
  }

  function handleLoadPermissionAudit() {
    const decision = permissionAuditDecision.trim();
    setPermissionAuditError(undefined);
    setPermissionAuditEvents([]);
    void listNativePermissionAudits(activeProjectId, {
      decision: decision
        ? (decision as "allowed" | "approval_required" | "forbidden")
        : undefined,
      actionKind: permissionAuditActionKind.trim() || undefined,
      runId: permissionAuditRunId.trim() || undefined,
      taskId: permissionAuditTaskId.trim() || undefined,
    })
      .then((items) => {
        setPermissionAuditEvents(items);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native permission audit API unavailable";
        setPermissionAuditEvents([]);
        setPermissionAuditError(`Permission audit unavailable · ${message}`);
      });
  }

  function handleLoadAgents() {
    setAgentDirectoryError(undefined);
    void listNativeAgentProfiles()
      .then((agents) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          agents: agents.map(nativeAgentProfileToStateAgent),
        }));
      })
      .catch(handleAgentDirectoryError);
  }

  function handleScanAgents() {
    setAgentDirectoryError(undefined);
    void scanNativeAgentProfiles()
      .then((agents) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          agents: agents.map(nativeAgentProfileToStateAgent),
        }));
      })
      .catch(handleAgentDirectoryError);
  }

  function handleRegisterAgentAdapter() {
    setAgentAdapterStatus(undefined);
    setAgentAdapterError(undefined);
    let argsJson: unknown;
    let envPolicyJson: unknown;
    let skillsJson: unknown;
    try {
      argsJson = JSON.parse(agentAdapterArgsJson || "[]");
      envPolicyJson = JSON.parse(agentAdapterEnvPolicyJson || "{}");
      skillsJson = JSON.parse(agentAdapterSkillsJson || "[]");
    } catch (error) {
      const message = error instanceof Error ? error.message : "invalid adapter JSON";
      setAgentAdapterError(`Adapter JSON invalid · ${message}`);
      return;
    }
    void upsertNativeAgentProfile({
      id: agentAdapterId.trim(),
      name: agentAdapterName.trim(),
      runtime: agentAdapterRuntime.trim(),
      command: agentAdapterCommand.trim(),
      argsJson,
      envPolicyJson,
      skillsJson,
      status: "available",
    })
      .then((agent) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          agents: [
            ...snapshot.agents.filter((item) => item.id !== agent.id),
            nativeAgentProfileToStateAgent(agent),
          ],
        }));
        setAgentAdapterStatus(`Registered ${agent.name} · ${agent.runtime}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native agent API unavailable";
        setAgentAdapterError(`Adapter registration unavailable · ${message}`);
      });
  }

  function handleLoadSkillPacks() {
    setSkillPackStatus(undefined);
    setSkillPackError(undefined);
    void listNativeSkillPacks(activeProjectId)
      .then((packs) => {
        setNativeSkillPacks(packs);
        setSkillPackStatus(`Loaded ${packs.length} skill pack${packs.length === 1 ? "" : "s"}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native skill pack API unavailable";
        setNativeSkillPacks([]);
        setSkillPackError(`Skill packs unavailable · ${message}`);
      });
  }

  function handleLoadRuntimePool() {
    setRuntimePoolStatus(undefined);
    setRuntimePoolError(undefined);
    void listNativeRuntimePool(activeProjectId)
      .then((items) => {
        setNativeRuntimePoolItems(items);
        setRuntimePoolStatus(`Loaded ${items.length} runtime${items.length === 1 ? "" : "s"}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native runtime pool API unavailable";
        setNativeRuntimePoolItems([]);
        setRuntimePoolError(`Runtime pool unavailable · ${message}`);
      });
  }

  function handleCreateSkillPack() {
    const name = skillPackName.trim();
    if (!name) {
      setSkillPackError("Skill pack name is required");
      return;
    }
    let skillsJson: string[];
    try {
      skillsJson = parseSkillPackSkillsJson(skillPackSkillsJson);
    } catch (error) {
      const message = error instanceof Error ? error.message : "invalid skills JSON";
      setSkillPackError(`Skill pack skills invalid · ${message}`);
      return;
    }
    setSkillPackStatus(undefined);
    setSkillPackError(undefined);
    void upsertNativeSkillPack({
      projectId: activeProjectId,
      name,
      description: skillPackDescription.trim() || undefined,
      skillsJson,
      sourceContextPackId: skillPackContextPackId.trim() || undefined,
    })
      .then((pack) => {
        setNativeSkillPacks((items) => [
          ...items.filter((item) => item.id !== pack.id && !(item.project_id === pack.project_id && item.name === pack.name)),
          pack,
        ]);
        setSkillPackStatus(`Saved skill pack ${pack.name}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native skill pack API unavailable";
        setSkillPackError(`Skill pack save unavailable · ${message}`);
      });
  }

  function handleAgentAvailability(agent: StateAgent, available: boolean) {
    setAgentDirectoryError(undefined);
    void updateNativeAgentProfileStatus(agent.id, available ? "available" : "paused")
      .then((updated) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          agents: snapshot.agents.map((item) =>
            item.id === agent.id
              ? {
                  ...item,
                  label: updated.name ?? item.label,
                  available: updated.status === "available",
                }
              : item,
          ),
        }));
      })
      .catch(handleAgentDirectoryError);
  }

  function handleAgentHeartbeat(agent: StateAgent) {
    setAgentDirectoryError(undefined);
    void heartbeatNativeAgentProfile(agent.id)
      .then((updated) => {
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          agents: snapshot.agents.map((item) =>
            item.id === agent.id
              ? {
                  ...item,
                  label: updated.name ?? item.label,
                  available: updated.status === "available",
                  last_heartbeat_at: updated.last_heartbeat_at,
                }
              : item,
          ),
        }));
      })
      .catch(handleAgentDirectoryError);
  }

  function handleAgentDirectoryError(error: unknown) {
    const message = error instanceof Error ? error.message : "native agent API unavailable";
    setAgentDirectoryError(`Agent directory unavailable · ${message}`);
  }

  function handleLaunchAgentTerminal(agent: StateAgent) {
    const title = `${agent.label} raw agent`;
    setAgentLaunchStatus(undefined);
    setAgentLaunchError(undefined);
    void launchNativeAgentTerminal({
      projectId: activeProjectId,
      agentProfileId: agent.id,
      title,
      cols: 100,
      rows: 30,
    })
      .then((launch) => {
        const nextSession = nativeSessionToStateSession(launch.session);
        ptySessionOwnerRef.current = {
          ...ptySessionOwnerRef.current,
          [launch.pty_session.id]: nextSession.id,
        };
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          sessions: snapshot.sessions.some((item) => item.id === nextSession.id)
            ? snapshot.sessions.map((item) => (item.id === nextSession.id ? nextSession : item))
            : [...snapshot.sessions, nextSession],
        }));
        setPtySnapshot((snapshot) => ({
          total: snapshot.sessions.some((session) => session.id === launch.pty_session.id) ? snapshot.total : snapshot.total + 1,
          sessions: snapshot.sessions.some((session) => session.id === launch.pty_session.id)
            ? snapshot.sessions
            : [...snapshot.sessions, launch.pty_session],
        }));
        setFocusedTerminalSessionId(nextSession.id);
        setAgentLaunchStatus(`Launched ${agent.label} raw terminal · ${launch.pty_session.id}`);
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native agent launch API unavailable";
        setAgentLaunchError(`Agent launch unavailable · ${message}`);
      });
  }

  function handleIngestAgentEvents() {
    const agentProfileId = agentEventProfile.trim();
    if (!agentProfileId) {
      setAgentEventResult(undefined);
      setAgentEventError("Agent event profile is required");
      return;
    }
    setAgentEventError(undefined);
    setAgentEventResult(undefined);
    void ingestNativeAgentEvents({
      projectId: activeProjectId,
      sessionId: agentEventSession.trim() || undefined,
      runId: agentEventRun.trim() || undefined,
      agentProfileId,
      adapter: agentEventAdapter,
      payload: { raw: agentEventPayload },
    })
      .then((event) => {
        setAgentEventResult(event);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          agents: snapshot.agents.map((agent) =>
            agent.id === event.agent_profile_id
              ? {
                  ...agent,
                  latest_event_kind: event.kind,
                  latest_event_detail: event.detail,
                }
              : agent,
          ),
          attention:
            event.severity === "info"
              ? snapshot.attention
              : [
                  ...snapshot.attention.filter((item) => item.id !== `agent_event_${event.agent_profile_id}`),
                  {
                    id: `agent_event_${event.agent_profile_id}`,
                    label: `Agent ${event.agent_profile_id} ${event.kind}`,
                    severity: event.severity,
                    detail: event.detail,
                  },
                ],
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native agent event API unavailable";
        setAgentEventError(`Agent event ingestion unavailable · ${message}`);
      });
  }

  function handleSessionFocus(session: StateSession) {
    const previousFocusedSessionId = activeTerminalSessionId;
    const previousMaximizedSessionId = maximizedTerminalSessionId;
    setSessionControlError(undefined);
    setFocusedTerminalSessionId(session.id);
    setMaximizedTerminalSessionId(undefined);
    void focusNativeSession(session.id)
      .then(applyNativeSessionMutation)
      .catch((error) => {
        setFocusedTerminalSessionId(previousFocusedSessionId);
        setMaximizedTerminalSessionId(previousMaximizedSessionId);
        handleSessionControlError(error);
      });
  }

  function handleSessionTakeover(session: StateSession) {
    setSessionControlError(undefined);
    void takeoverNativeSession(session.id)
      .then(applyNativeSessionMutation)
      .catch(handleSessionControlError);
  }

  function handleSessionRelease(session: StateSession) {
    setSessionControlError(undefined);
    void releaseNativeSession(session.id)
      .then(applyNativeSessionMutation)
      .catch(handleSessionControlError);
  }

  function handleSessionAttachSelectedTask(session: StateSession) {
    if (!selectedTaskOverview) return;
    setSessionControlError(undefined);
    void attachNativeSessionTask(session.id, selectedTaskOverview.id)
      .then(applyNativeSessionMutation)
      .catch(handleSessionControlError);
  }

  function handleSessionDetachTask(session: StateSession) {
    setSessionControlError(undefined);
    void detachNativeSessionTask(session.id)
      .then(applyNativeSessionMutation)
      .catch(handleSessionControlError);
  }

  function handleSessionKill(session: StateSession) {
    setSessionControlError(undefined);
    void killNativeSession(session.id)
      .then(applyNativeSessionMutation)
      .catch(handleSessionControlError);
  }

  function handleLoadSessions() {
    setSessionControlError(undefined);
    void listNativeSessions(activeProjectId)
      .then((sessions) => {
        const loadedSessions = sessions.map(nativeSessionToStateSession);
        setStateSnapshot((snapshot) => ({
          ...snapshot,
          sessions: [
            ...snapshot.sessions.filter((session) => session.project_id !== activeProjectId),
            ...loadedSessions,
          ],
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native session API unavailable";
        setSessionControlError(`Session list unavailable · ${message}`);
      });
  }

  function handleSessionTranscriptLoad(session: StateSession) {
    setSessionControlError(undefined);
    void listNativeTerminalStreamChunks(session.id, 10)
      .then((chunks) => {
        setTerminalTranscriptChunksBySession((current) => ({
          ...current,
          [session.id]: chunks,
        }));
      })
      .catch((error) => {
        const message = error instanceof Error ? error.message : "native terminal transcript API unavailable";
        setSessionControlError(`Terminal transcript unavailable · ${message}`);
      });
  }

  function handleSessionControlError(error: unknown) {
    const message = error instanceof Error ? error.message : "native session API unavailable";
    setSessionControlError(`Session control unavailable · ${message}`);
  }

  function applyNativeSessionMutation(session: NativeSession) {
    setStateSnapshot((snapshot) => ({
      ...snapshot,
      sessions: snapshot.sessions.map((item) =>
        item.id === session.id
          ? {
              ...item,
              title: session.title ?? item.title,
              mode: session.mode ?? item.mode,
              state: session.state ?? item.state,
              attention_state: session.attention_state ?? item.attention_state,
              token_budget_state: session.token_budget_state ?? item.token_budget_state,
              project_id: session.project_id === undefined ? item.project_id : session.project_id,
              pane_id: session.pane_id === undefined ? item.pane_id : session.pane_id ?? undefined,
              cwd: session.cwd === undefined ? item.cwd : session.cwd ?? undefined,
              branch: session.branch === undefined ? item.branch : session.branch ?? undefined,
              agent_profile_id:
                session.agent_profile_id === undefined ? item.agent_profile_id : session.agent_profile_id,
              task_id: session.task_id === undefined ? item.task_id : session.task_id,
              run_id: session.run_id === undefined ? item.run_id : session.run_id,
              created_at: session.created_at === undefined ? item.created_at : session.created_at,
              updated_at: session.updated_at === undefined ? item.updated_at : session.updated_at,
            }
          : item,
      ),
    }));
  }

  function applyNativeRunMutation(run: NativeRun) {
    const summary = nativeRunToStateRunSummary(run);
    setStateSnapshot((snapshot) => {
      const withRun = upsertRunSummary(snapshot, summary);
      if (!run.session_id || withRun.sessions.some((session) => session.id === run.session_id)) {
        return withRun;
      }
      return {
        ...withRun,
        sessions: [
          ...withRun.sessions,
          {
            id: run.session_id,
            project_id: run.project_id,
            pane_id: `pane_${run.session_id}`,
            mode: "agent",
            title: "Agent session",
            cwd: run.workspace_path ?? "",
            branch: "",
            state: "running",
            attention_state: "none",
            token_budget_state: "unknown",
          },
        ],
      };
    });
    const nextTaskStatus = taskStatusForRunLifecycle(summary.lifecycle);
    if (nextTaskStatus) {
      setTaskState((state) => moveTask(state, summary.task_id, nextTaskStatus));
    }
  }

  useEffect(() => {
    setTaskWorkpadDraft(selectedTaskOverview?.workpad ?? "");
    setTaskWorkpadMode("edit");
    setTaskWorkpadError(undefined);
    setNewTaskComment("");
    setTaskCommentError(undefined);
    setNewTaskSubtaskTitle("");
    setTaskSubtaskError(undefined);
    setTaskCycleDraft(selectedTaskOverview?.cycle ?? "");
    setTaskModuleDraft(selectedTaskOverview?.module ?? "");
    setTaskInitiativeDraft(selectedTaskOverview?.initiative ?? "");
    setTaskLabelsDraft((selectedTaskOverview?.labels ?? []).join(", "));
    setTaskDueDateDraft(selectedTaskOverview?.dueDate ?? "");
    setTaskEstimateDraft(selectedTaskOverview?.estimate ?? "");
    setTaskAssigneeDraft(selectedTaskOverview?.assignee ?? "");
    setTaskContextPackDraft(selectedTaskOverview?.contextPackId ?? "");
    setTaskContextStatus(undefined);
    setTaskContextError(undefined);
    setTaskPlanningError(undefined);
    setTaskPlanningStatus(undefined);
  }, [
    selectedTaskOverview?.id,
    selectedTaskOverview?.workpad,
    selectedTaskOverview?.cycle,
    selectedTaskOverview?.module,
    selectedTaskOverview?.initiative,
    selectedTaskOverview?.labels?.join(", "),
    selectedTaskOverview?.dueDate,
    selectedTaskOverview?.estimate,
    selectedTaskOverview?.assignee,
    selectedTaskOverview?.contextPackId,
  ]);

  useEffect(() => {
    saveEvidencePack(evidencePack);
  }, [evidencePack]);

  useEffect(() => {
    let active = true;
    const projectTaskState = loadTaskState(activeProjectId);
    setTaskStateProjectId(activeProjectId);
    setTaskState(projectTaskState);
    setTaskSavedViews(loadTaskSavedViews(activeProjectId));
    setTaskLoadError(undefined);

    loadNativeTaskState(activeProjectId)
      .then((state) => {
        if (!active) return;
        setTaskLoadError(undefined);
        if (Object.keys(state.tasks).length > 0) {
          setTaskStateProjectId(activeProjectId);
          setTaskState(state);
        }
      })
      .catch((error) => {
        if (!active) return;
        const message = error instanceof Error ? error.message : "native task API unavailable";
        setTaskLoadError(`Task board loaded from local cache · ${message}`);
      });

    return () => {
      active = false;
    };
  }, [activeProjectId]);

  useEffect(() => {
    saveTaskState(taskStateProjectId, taskState);
  }, [taskState, taskStateProjectId]);

  useEffect(() => {
    let active = true;
    setTaskCycles([]);
    setTaskModules([]);

    listNativeTaskCycles(activeProjectId)
      .then((cycles) => {
        if (active) setTaskCycles(cycles);
      })
      .catch(() => undefined);

    listNativeTaskModules(activeProjectId)
      .then((modules) => {
        if (active) setTaskModules(modules);
      })
      .catch(() => undefined);

    return () => {
      active = false;
    };
  }, [activeProjectId]);

  useEffect(() => {
    if (!missingRoadmapInitiativeKey) return;
    let active = true;
    listNativeInitiatives(activeProjectId)
      .then((initiatives) => {
        if (!active) return;
        setStateSnapshot((snapshot) => mergeInitiativesIntoStateSnapshot(snapshot, initiatives));
      })
      .catch(() => undefined);

    return () => {
      active = false;
    };
  }, [activeProjectId, missingRoadmapInitiativeKey]);

  useEffect(() => {
    const query = commandBlockQuery.trim();
    if (!query) {
      setCommandBlockSearchError(undefined);
      return;
    }

    let active = true;
    searchNativeCommandBlocks(query, 50)
      .then((blocks) => {
        if (!active) return;
        setCommandBlockSearchError(undefined);
        setCommandBlockState((state) =>
          mergeCommandBlockSummaries(
            state,
            blocks.map(nativeCommandBlockToCommandBlock).map((block) => ({
              id: block.id,
              sessionId: block.sessionId,
              command: block.command,
              status: block.status,
              seqStart: block.seqStart,
              seqEnd: block.seqEnd,
              cwd: block.cwd,
              branch: block.branch,
              outputExcerpt: block.outputExcerpt,
            })),
          ),
        );
      })
      .catch((error) => {
        if (!active) return;
        const message = error instanceof Error ? error.message : "native command block search unavailable";
        setCommandBlockSearchError(`Command block search degraded · ${message}`);
      });

    return () => {
      active = false;
    };
  }, [commandBlockQuery]);

  useEffect(() => {
    let active = true;

    getStateSnapshot()
      .then((snapshot) => {
        if (active) {
          applyNativeStateSnapshot(snapshot);
        }
      })
      .catch(() => {
        if (active) {
          setStateSnapshot(fallbackStateSnapshot);
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!activeSnapshotProject) {
      setProjectFileList(undefined);
      setProjectFileSearch(undefined);
      setProjectFileSearchQuery("");
      setProjectFilePreview(undefined);
      setProjectFileDraft("");
      setProjectFileSaveStatus(undefined);
      setProjectDiff(undefined);
      setProjectFileError(undefined);
      setProjectFileSearchError(undefined);
      setProjectFilePreviewError(undefined);
      setProjectFileSaveError(undefined);
      setProjectDiffError(undefined);
      return;
    }
    let active = true;
    setProjectFileError(undefined);
    setProjectFileSearch(undefined);
    setProjectFileSearchError(undefined);
    setProjectFilePreview(undefined);
    setProjectFileDraft("");
    setProjectFileSaveStatus(undefined);
    setProjectFilePreviewError(undefined);
    setProjectFileSaveError(undefined);
    setProjectDiff(undefined);
    setProjectDiffError(undefined);

    listNativeProjectFiles(activeSnapshotProject.id)
      .then((files) => {
        if (active) {
          setProjectFileList(files);
        }
      })
      .catch((error) => {
        if (active) {
          setProjectFileList(undefined);
          setProjectFileError(String(error));
        }
      });

    return () => {
      active = false;
    };
  }, [activeSnapshotProject?.id]);

  useEffect(() => {
    const cachedProjectTheme = activeProjectId ? loadProjectTerminalTheme(activeProjectId) : undefined;
    setActiveProjectTerminalTheme(cachedProjectTheme);
    if (!activeProjectId) return;

    let active = true;
    getNativeTerminalThemeSettings(activeProjectId)
      .then((theme) => {
        if (active && theme.project_id === activeProjectId) {
          const projectTheme = nativeTerminalThemeToTerminalTheme(theme);
          window.localStorage?.setItem(projectTerminalThemeStorageKey(activeProjectId), JSON.stringify(projectTheme));
          setActiveProjectTerminalTheme(projectTheme);
        }
      })
      .catch(() => undefined);

    return () => {
      active = false;
    };
  }, [activeProjectId]);

  useEffect(() => {
    const query = projectFileSearchQuery.trim();
    if (!activeSnapshotProject || !query) {
      setProjectFileSearch(undefined);
      setProjectFileSearchError(undefined);
      return;
    }
    let active = true;
    setProjectFileSearch(undefined);
    setProjectFileSearchError(undefined);

    searchNativeProjectFiles(activeSnapshotProject.id, query)
      .then((results) => {
        if (active) {
          setProjectFileSearch(results);
        }
      })
      .catch((error) => {
        if (active) {
          setProjectFileSearch(undefined);
          setProjectFileSearchError(String(error));
        }
      });

    return () => {
      active = false;
    };
  }, [activeSnapshotProject?.id, projectFileSearchQuery]);

  useEffect(() => {
    const layoutJson = activeProjectTab?.layout_json;
    if (!layoutJson) return;
    const paneIds = new Set(renderedTerminalDeckSessions.map((session) => session.id));
    const focusedSessionId = layoutJson.focusedSessionId && paneIds.has(layoutJson.focusedSessionId) ? layoutJson.focusedSessionId : undefined;
    const maximizedSessionId =
      layoutJson.mode === "maximized" && layoutJson.maximizedSessionId && paneIds.has(layoutJson.maximizedSessionId)
        ? layoutJson.maximizedSessionId
        : undefined;
    setFocusedTerminalSessionId(focusedSessionId);
    setMaximizedTerminalSessionId(maximizedSessionId);
  }, [activeProjectTab?.layout_json, terminalDeckLayoutPaneKey]);

  useEffect(() => {
    let active = true;

    getTerminalPtySnapshot()
      .then((snapshot) => {
        if (active) {
          setPtySnapshot(snapshot);
        }
      })
      .catch(() => {
        if (active) {
          setPtySnapshot(fallbackTerminalPtySnapshot);
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && /^[1-9]$/.test(event.key)) {
        const shortcutTab = workspaceTabs[Number(event.key) - 1];
        if (shortcutTab?.projectId) {
          event.preventDefault();
          handleFocusWorkspaceTab(shortcutTab);
        }
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "p") {
        event.preventDefault();
        setCommandPaletteOpen(true);
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "t") {
        event.preventDefault();
        handleCreateTerminalSession();
      }
      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "u") {
        event.preventDefault();
        handleJumpToUnread();
      }
      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "y") {
        event.preventDefault();
        setCompactRightRailOpen(true);
        knowledgeSourceInputRef.current?.focus();
      }
      if ((event.metaKey || event.ctrlKey) && !event.shiftKey && event.key.toLowerCase() === "b") {
        event.preventDefault();
        setCompactRightRailOpen((open) => !open);
      }
      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "b") {
        event.preventDefault();
        quickTaskInputRef.current?.focus();
      }
      if (event.key === "Escape") {
        setCommandPaletteOpen(false);
        setNotificationDrawerOpen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activeProjectId, handleJumpToUnread, sessionItems.length, workspaceTabShortcutKey]);

  return (
    <div className="hc-app-shell" data-testid="haneulchi-shell" data-visual-qa-screen={visualQaScreen?.slug}>
      <header className="hc-titlebar hc-hairline-bottom" aria-label="Haneulchi titlebar">
        <div className="hc-window-controls" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <div className="hc-titlebar-brand">
          <Terminal size={16} />
          <strong>Haneulchi</strong>
          <span>Terminal-first Agent Workspace</span>
        </div>
        <div className="hc-titlebar-actions">
          <button
            type="button"
            className="hc-right-rail-toggle"
            aria-label="Toggle right rail"
            aria-expanded={compactRightRailOpen}
            onClick={() => setCompactRightRailOpen((open) => !open)}
          >
            <LayoutGrid size={15} />
          </button>
          <button type="button" aria-label="Open command search" onClick={() => setCommandPaletteOpen(true)}>
            <Search size={15} />
          </button>
          <button type="button" aria-label="Show notifications" onClick={() => setNotificationDrawerOpen(true)}>
            <Bell size={15} />
            {globalAttentionItems.length > 0 ? <span>{globalAttentionItems.length}</span> : null}
          </button>
        </div>
      </header>

      {notificationDrawerOpen ? (
        <section
          aria-labelledby="hc-notifications-title"
          className="hc-notification-drawer"
          role="dialog"
        >
          <div className="hc-notification-panel">
            <header>
              <div>
                <strong id="hc-notifications-title">Notifications</strong>
                <span>Unread sessions {unreadSessionItems.length}</span>
              </div>
              <button type="button" aria-label="Close notifications" onClick={() => setNotificationDrawerOpen(false)}>
                <XCircle size={16} />
              </button>
            </header>
            <button
              className="hc-notification-jump"
              type="button"
              onClick={handleJumpToUnread}
              disabled={!nextUnreadSession}
              aria-label={nextUnreadSession ? `Jump to unread ${nextUnreadSession.title}` : "Jump to unread"}
            >
              <LocateFixed size={14} />
              <span>{nextUnreadSession ? `Jump to ${nextUnreadSession.title}` : "No unread session"}</span>
            </button>
            {notificationHealthDegraded ? (
              <div className="hc-notification-health">
                <AlertTriangle size={15} />
                <span>Notifications degraded · api {controlPlaneSnapshot.health.api}</span>
              </div>
            ) : null}
            <div className="hc-notification-list">
              {globalAttentionItems.length === 0 ? (
                <article>
                  <CheckCircle2 size={15} />
                  <span>No unread notifications</span>
                </article>
              ) : (
                globalAttentionItems.slice(0, 6).map((item) => (
                  <article key={item.id}>
                    <span className={`hc-readiness-icon ${attentionIconStatus(item.severity)}`} />
                    <span>
                      <strong>{item.label}</strong>
                      <small>{item.detail}</small>
                    </span>
                  </article>
                ))
              )}
            </div>
          </div>
        </section>
      ) : null}

      {commandPaletteOpen ? (
        <section
          aria-labelledby="hc-command-palette-title"
          className="hc-command-palette"
          role="dialog"
        >
          <div className="hc-command-palette-panel">
            <header>
              <div>
                <strong id="hc-command-palette-title">Command Palette</strong>
                <span>Projects, commands, tasks, knowledge</span>
              </div>
              <button type="button" aria-label="Close command palette" onClick={() => setCommandPaletteOpen(false)}>
                <XCircle size={16} />
              </button>
            </header>
            <label className="hc-command-palette-search">
              <Search size={15} />
              <input
                aria-label="Search commands and projects"
                autoFocus
                value={commandPaletteQuery}
                onChange={(event) => setCommandPaletteQuery(event.currentTarget.value)}
                placeholder="Search projects, commands, tasks, knowledge"
              />
            </label>
            <div className="hc-command-palette-results">
              {filteredCommandPaletteItems.length === 0 && commandPaletteKnowledgeError ? (
                <p>{commandPaletteKnowledgeError}</p>
              ) : filteredCommandPaletteItems.length === 0 ? (
                <p>No commands found</p>
              ) : (
                filteredCommandPaletteItems.map((item) => (
                  <button
                    aria-label={`Palette ${item.kind} ${item.label}`}
                    key={item.id}
                    type="button"
                    onClick={() => runCommandPaletteItem(item)}
                  >
                    <span>{item.kind}</span>
                    <strong>{item.label}</strong>
                    <small>{item.detail}</small>
                  </button>
                ))
              )}
            </div>
          </div>
        </section>
      ) : null}

      <nav className="hc-sidebar hc-hairline-right" aria-label="Project workspace">
        <section className="hc-sidebar-section">
          <div className="hc-sidebar-heading">
            <span>Projects</span>
            <button type="button" aria-label="Load projects" onClick={handleLoadProjects}>
              <RotateCcw size={14} />
            </button>
            <button type="button" aria-label="Add project" onClick={handleAddProject}>
              <Blocks size={14} />
            </button>
          </div>
          <div className="hc-project-registry-form" aria-label="Project registry">
            <label>
              <input
                aria-label="New project key"
                value={newProjectKey}
                onChange={(event) => setNewProjectKey(event.currentTarget.value)}
                placeholder="KEY"
              />
            </label>
            <label>
              <input
                aria-label="New project name"
                value={newProjectName}
                onChange={(event) => setNewProjectName(event.currentTarget.value)}
                placeholder="Project name"
              />
            </label>
            <label>
              <input
                aria-label="New project path"
                value={newProjectPath}
                onChange={(event) => setNewProjectPath(event.currentTarget.value)}
                placeholder="/path/to/workspace"
              />
            </label>
            <label>
              <input
                aria-label="New project color"
                value={newProjectColor}
                onChange={(event) => setNewProjectColor(event.currentTarget.value)}
                placeholder="Project color"
              />
            </label>
          </div>
          {projectControlError ? <div className="hc-task-dispatch-error">{projectControlError}</div> : null}
          <div className="hc-project-list">
            {sidebarProjects.map((project) => (
              <button
                aria-label={`Focus project ${project.name}`}
                className="hc-project-row"
                type="button"
                key={project.id ?? project.name}
                onClick={() => handleFocusProject(project)}
              >
                <span className={`hc-status-dot ${project.state.toLowerCase()}`} />
                <span>
                  <strong>{project.name}</strong>
                  <small>
                    <GitBranch size={11} /> {project.branch}
                  </small>
                </span>
                <em>{project.sessions}</em>
              </button>
            ))}
          </div>
        </section>

        <section className="hc-sidebar-section hc-control-projects" aria-label="Control Tower project cards">
          <div className="hc-sidebar-heading">
            <span>Control Tower</span>
            <LayoutGrid size={14} />
          </div>
          <div className="hc-control-project-card-list">
            {controlTowerProjectCards.map((project) => (
              <article className="hc-control-project-card" key={project.id}>
                <header>
                  <strong>{project.name}</strong>
                  <em>{project.state}</em>
                </header>
                <div>
                  <span>{project.sessions} {project.sessions === 1 ? "session" : "sessions"}</span>
                  <span>{project.runs} {project.runs === 1 ? "run" : "runs"}</span>
                  <span>{project.alerts} {project.alerts === 1 ? "alert" : "alerts"}</span>
                  {project.usageLine ? <span>{project.usageLine}</span> : null}
                </div>
              </article>
            ))}
          </div>
        </section>

        <section className="hc-sidebar-section hc-readiness-card" aria-label="Readiness diagnostics">
          <div className="hc-sidebar-heading">
            <span>Readiness</span>
            <CheckCircle2 size={14} />
          </div>
          <div className="hc-readiness-summary">
            <span>{readinessSummary.ready} ready</span>
            <span>{readinessSummary.warning} warnings</span>
            <span>{readinessSummary.missing} missing</span>
          </div>
          <div className="hc-readiness-list">
            {readinessSnapshot.checks.map((check) => (
              <div className="hc-readiness-row" key={check.id}>
                <span className={`hc-readiness-icon ${check.status}`} />
                <span>
                  <strong>{check.label}</strong>
                  <small>{check.detail}</small>
                </span>
                <em>{statusLabel(check.status)}</em>
              </div>
            ))}
          </div>
        </section>
      </nav>

      <main
        className={visualQaScreen ? `hc-workspace hc-visual-qa-workspace hc-visual-qa-workspace-${visualQaScreen.slug}` : "hc-workspace hc-terminal-deck"}
        aria-label={visualQaScreen ? `${visualQaScreen.label} visual QA` : "Terminal Deck"}
        data-testid={visualQaScreen ? "visual-qa-workspace" : undefined}
        data-terminal-theme={effectiveTerminalTheme?.name ?? "Haneulchi Default"}
        style={terminalThemeStyle(effectiveTerminalTheme)}
      >
        <div className="hc-workspace-tabs" role="tablist" aria-label="Workspace surfaces">
          {workspaceTabs.map((tab) => (
            <button
              className={tab.active ? "active" : ""}
              type="button"
              role="tab"
              aria-selected={tab.active}
              key={tab.id}
              onClick={() => handleFocusWorkspaceTab(tab)}
            >
              {tab.label}
            </button>
          ))}
          <div className="hc-terminal-focus-controls" aria-label="Terminal focus controls">
            <button type="button" aria-label="Focus previous terminal" onClick={() => focusRelativeTerminalSession(-1)}>
              <ChevronLeft size={14} />
            </button>
            <button type="button" aria-label="Focus next terminal" onClick={() => focusRelativeTerminalSession(1)}>
              <ChevronRight size={14} />
            </button>
            <button
              type="button"
              aria-label={maximizedTerminalSessionId ? "Restore terminal grid" : "Maximize focused terminal"}
              onClick={toggleMaximizedTerminalSession}
            >
              {maximizedTerminalSessionId ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
            </button>
          </div>
        </div>

        {visualQaScreen ? <VisualQaWorkspaceContent screen={visualQaScreen} /> : null}

        <section className="hc-ops-strip" aria-label="Control Tower ops strip">
          <span>Poll API {controlPlaneSnapshot.health.api}</span>
          <span>Blocked {runCounts.blocked ?? 0}</span>
          <span>Review {runCounts.review_ready ?? 0}</span>
          <span>{formatOpsBudgetSummary(primaryBudget)}</span>
          <span>Workflow {controlPlaneSnapshot.workflow.valid ? "valid" : `invalid ${opsWorkflowIssueCount}`}</span>
          <span>Knowledge {controlPlaneSnapshot.knowledge.stale_count} stale · {controlPlaneSnapshot.knowledge.gap_count} gaps</span>
        </section>

        <section className="hc-ssh-terminal-create" aria-label="Remote SSH terminal">
          <label>
            <span>Title</span>
            <input
              aria-label="SSH session title"
              value={sshSessionTitle}
              onChange={(event) => setSshSessionTitle(event.currentTarget.value)}
            />
          </label>
          <label>
            <span>Target</span>
            <input
              aria-label="SSH target"
              value={sshTarget}
              onChange={(event) => setSshTarget(event.currentTarget.value)}
              placeholder="deploy@staging.example.com"
            />
          </label>
          <label>
            <span>Path</span>
            <input
              aria-label="SSH remote path"
              value={sshRemotePath}
              onChange={(event) => setSshRemotePath(event.currentTarget.value)}
            />
          </label>
          <label>
            <span>Branch</span>
            <input
              aria-label="SSH branch label"
              value={sshBranchLabel}
              onChange={(event) => setSshBranchLabel(event.currentTarget.value)}
              placeholder="remote/main"
            />
          </label>
          <button type="button" onClick={handleCreateSshTerminalSession} aria-label="Create SSH terminal">
            <Terminal size={13} />
            SSH
          </button>
          {sshSessionStatus ? <span className="hc-ssh-terminal-status">{sshSessionStatus}</span> : null}
          {sshSessionError ? <span className="hc-ssh-terminal-error">{sshSessionError}</span> : null}
        </section>

        <section className="hc-terminal-command-capture" aria-label="Terminal command capture">
          <label>
            <span>Command</span>
            <input
              aria-label="Capture PTY command"
              value={terminalCaptureCommand}
              onChange={(event) => setTerminalCaptureCommand(event.currentTarget.value)}
              placeholder="npm"
            />
          </label>
          <label>
            <span>Args</span>
            <input
              aria-label="Capture PTY args"
              value={terminalCaptureArgs}
              onChange={(event) => setTerminalCaptureArgs(event.currentTarget.value)}
              placeholder="test -- --runInBand"
            />
          </label>
          <button type="button" onClick={handleCaptureTerminalCommand} aria-label="Capture PTY command output">
            <Terminal size={13} />
            Capture
          </button>
          {terminalCaptureResult ? (
            <div className="hc-terminal-capture-result">
              <strong>{formatPtyCommandCapture(terminalCaptureResult)}</strong>
              <pre>{terminalCaptureResult.output.trim() || "(no output)"}</pre>
            </div>
          ) : null}
          {terminalCaptureError ? <span className="hc-ssh-terminal-error">{terminalCaptureError}</span> : null}
        </section>

        <section className="hc-terminal-grid" aria-label="Terminal pane grid">
          {visibleTerminalDeckSessions.map((session) => (
            <TerminalPane
              session={session}
              highlighted={
                [session.id, session.ptyId].includes(activeTerminalSessionId) ||
                (focusedCommandBlock ? [session.id, session.ptyId].includes(focusedCommandBlock.sessionId) : false)
              }
              onRun={handleRunTerminalSession}
              onSplit={handleSplitTerminalSession}
              onRendererDegraded={handleTerminalRendererDegraded}
              onOpenLink={handleOpenTerminalLink}
              onInput={handleTerminalInput}
              onResize={handleTerminalResize}
              onClose={handleTerminalClose}
              key={session.id}
            />
          ))}
        </section>
        {terminalSessionCreateError ? <div className="hc-task-dispatch-error">{terminalSessionCreateError}</div> : null}
        {terminalInputRecordError ? <div className="hc-task-dispatch-error">{terminalInputRecordError}</div> : null}
        {terminalOutputListenerError ? <div className="hc-task-dispatch-error">{terminalOutputListenerError}</div> : null}

        {pendingDangerousInput ? (
          <section className="hc-dangerous-action-approval" aria-label="Dangerous action approval">
            <header>
              <ShieldAlert size={15} />
              <strong>Dangerous action approval required</strong>
            </header>
            <code>{pendingDangerousInput.command}</code>
            <div>
              <button
                type="button"
                onClick={handleAllowDangerousInput}
                aria-label={`Allow dangerous input ${pendingDangerousInput.session.id}`}
              >
                Allow once
              </button>
              <button
                type="button"
                onClick={() => {
                  setPendingDangerousInput(undefined);
                  setDangerousInputError(undefined);
                }}
                aria-label={`Cancel dangerous input ${pendingDangerousInput.session.id}`}
              >
                Cancel
              </button>
            </div>
            {dangerousInputError ? <small>{dangerousInputError}</small> : null}
          </section>
        ) : null}

        <section className="hc-bottom-panel" aria-label="Command block log">
          <header>
            <div className="hc-log-tabs">
              <button className="active" type="button">Command Blocks {commandBlocks.length}</button>
              <button type="button">Problems 2</button>
              <button type="button">Output</button>
              <button type="button">Tasks</button>
            </div>
            <label className="hc-command-search">
              <Search size={13} />
              <input
                aria-label="Search command blocks"
                value={commandBlockQuery}
                onChange={(event) => setCommandBlockQuery(event.currentTarget.value)}
                placeholder="Search commands"
              />
            </label>
          </header>
          {commandBlockPersistError ? (
            <div className="hc-command-block-degraded" role="status">
              <span>{commandBlockPersistError}</span>
              <small>Local command block log remains available</small>
              <button type="button" onClick={retryCommandBlockPersistence} aria-label="Retry command block persistence">
                <RotateCcw size={12} />
                Retry
              </button>
            </div>
          ) : commandBlockPersistStatus ? (
            <div className="hc-command-block-recovered" role="status">
              <span>{commandBlockPersistStatus}</span>
            </div>
          ) : null}
          {commandBlockEvidenceError ? (
            <div className="hc-command-block-degraded" role="status">
              <span>{commandBlockEvidenceError}</span>
            </div>
          ) : null}
          {commandBlockSearchError ? (
            <div className="hc-command-block-degraded" role="status">
              <span>{commandBlockSearchError}</span>
            </div>
          ) : null}
          <div className="hc-log-rows">
            {commandBlocks.length === 0 ? (
              <>
                <span><Clock3 size={12} /> 12:35 readiness snapshot refreshed from local shell probes</span>
                <span><Code2 size={12} /> command block indexer waiting for submitted terminal input</span>
                <span><FileText size={12} /> evidence pack notes linked to Sprint 1 release gate</span>
              </>
            ) : visibleCommandBlocks.length === 0 ? (
              <span><Search size={12} /> No command blocks match "{commandBlockQuery}"</span>
            ) : (
              visibleCommandBlocks.slice(-5).map((block, blockIndex) => (
                <div className={`hc-command-block-row ${focusedCommandBlockId === block.id ? "is-focused" : ""}`} key={block.id}>
                  <span>
                    <Code2 size={12} />
                    {block.command} · {block.status} · {block.branch} · seq {block.seqStart ?? "-"}-{block.seqEnd ?? "-"}
                  </span>
                  <div className="hc-command-block-actions">
                    <button type="button" onClick={() => handleJumpToCommandBlock(block.id)} aria-label={`Jump to ${block.command}`}>
                      <LocateFixed size={12} />
                      Jump
                    </button>
                    <button type="button" onClick={() => handleCopyCommandBlock(block.id)} aria-label={`Copy ${block.command}`}>
                      <Clipboard size={12} />
                      {copiedCommandBlockId === block.id ? "Copied" : "Copy"}
                    </button>
                    <button type="button" onClick={() => handleExplainCommandBlock(block.id)} aria-label={`Explain ${block.command}`}>
                      <Activity size={12} />
                      Explain
                    </button>
                    <button type="button" onClick={() => handleMarkCommandBlock(block.id, "completed")} aria-label={`Mark ${block.command} passed`}>
                      <CheckCircle2 size={12} />
                      Pass
                    </button>
                    <button type="button" onClick={() => handleMarkCommandBlock(block.id, "failed")} aria-label={`Mark ${block.command} failed`}>
                      <XCircle size={12} />
                      Fail
                    </button>
                    <button
                      type="button"
                      onClick={() => handleMergeCommandBlockWithPrevious(block.id)}
                      aria-label={`Merge ${block.command} with previous block`}
                      disabled={blockIndex === 0}
                    >
                      <Blocks size={12} />
                      Merge
                    </button>
                    <button type="button" onClick={() => handleSplitCommandBlock(block.id)} aria-label={`Split ${block.command}`}>
                      <StepForward size={12} />
                      Split
                    </button>
                    <button type="button" onClick={() => handleExportCommandBlockBundle(block.id)} aria-label={`Export ${block.command} bundle`}>
                      <FileText size={12} />
                      {exportedCommandBlockId === block.id ? "Exported" : "Export"}
                    </button>
                    <button type="button" onClick={() => handleAttachCommandBlock(block.id)} aria-label={`Attach ${block.command} to evidence pack`}>
                      <SquarePlus size={12} />
                      Attach
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
          {commandBlockExplanation ? (
            <section className="hc-command-block-explanation" aria-label="Command block explanation">
              <header>
                <strong>{commandBlockExplanation.command}</strong>
                <small>{commandBlockExplanation.id}</small>
              </header>
              {commandBlockExplanation.provider ? (
                <div className="hc-command-block-ai-route">
                  AI explanation draft · {commandBlockExplanation.provider}/{commandBlockExplanation.model} · agent {commandBlockExplanation.agentProfileId}
                </div>
              ) : null}
              <p>{commandBlockExplanation.summary}</p>
              <ul>
                {commandBlockExplanation.evidence.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
              {commandBlockExplanation.diagnostics?.map((item) => (
                <small className="hc-command-block-diagnostic" key={item}>
                  {item}
                </small>
              ))}
              {exportedCommandBlockId === commandBlockExplanation.commandBlockId ? (
                <span>Exported bundle {commandBlockExplanation.commandBlockId}</span>
              ) : null}
            </section>
          ) : null}
        </section>
      </main>

      <aside className={`hc-right-rail hc-hairline-left ${compactRightRailOpen ? "is-open" : ""}`} aria-label="Attention and review rail">
        {visualQaScreen ? <VisualQaRightRail /> : null}
        <section className="hc-rail-panel" aria-label="Global attention queue">
          <header>
            <span>Attention Center</span>
            <AlertTriangle size={14} />
          </header>
          <div className="hc-attention-summary" aria-label="Attention severity summary">
            <span>Critical {criticalAttentionCount}</span>
            <span>Warning {warningAttentionCount}</span>
            <span>Total {globalAttentionItems.length}</span>
          </div>
          {globalAttentionItems.length === 0 ? (
            <article className="hc-attention-row">
              <span className="hc-readiness-icon ready" />
              <div>
                <strong>No attention required</strong>
                <small>All monitored sessions and runs are clear</small>
              </div>
            </article>
          ) : (
            globalAttentionItems.slice(0, 5).map((item) => (
              <article className="hc-attention-row" key={item.id}>
                <span className={`hc-readiness-icon ${attentionIconStatus(item.severity)}`} />
                <div>
                  <strong>{item.label} · {item.severity}</strong>
                  <small>{item.detail} · attention queue</small>
                </div>
              </article>
            ))
          )}
        </section>

        <section className="hc-rail-panel" aria-label="Session Stack">
          <header>
            <span>Session Stack</span>
            <Terminal size={14} />
          </header>
          <button type="button" className="hc-task-drawer-action" onClick={handleLoadSessions} aria-label="Load sessions">
            Load sessions
          </button>
          {sessionControlError ? <div className="hc-task-dispatch-error">{sessionControlError}</div> : null}
          {projectSessionItems.length === 0 ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>No persisted sessions</span>
            </div>
          ) : (
            projectSessionItems.map((session) => {
              const sessionMetadata = formatSessionStackMetadata(session);
              const transcriptChunks = terminalTranscriptChunksBySession[session.id] ?? [];
              return (
                <div className="hc-review-row" key={session.id}>
                  <Terminal size={15} />
                  <span>
                    <span>{session.title} · {session.mode} · {session.attention_state ?? "none"}{formatSessionTokenUsage(session)}</span>
                    {sessionMetadata ? <small>{sessionMetadata}</small> : null}
                    {transcriptChunks.slice(0, 2).map((chunk) => (
                      <small key={chunk.id}>{formatTerminalTranscriptChunk(chunk)}</small>
                    ))}
                  </span>
                  <button type="button" onClick={() => handleSessionFocus(session)} aria-label={`Focus ${session.id}`}>
                    <LocateFixed size={13} />
                  </button>
                  <button type="button" onClick={() => handleSessionTakeover(session)} aria-label={`Take over ${session.id}`}>
                    <StepForward size={13} />
                  </button>
                  <button type="button" onClick={() => handleSessionRelease(session)} aria-label={`Release ${session.id}`}>
                    <Minimize2 size={13} />
                  </button>
                  {selectedTaskOverview && session.task_id !== selectedTaskOverview.id ? (
                    <button
                      type="button"
                      onClick={() => handleSessionAttachSelectedTask(session)}
                      aria-label={`Attach ${selectedTaskOverview.id} to ${session.id}`}
                    >
                      <SquarePlus size={13} />
                    </button>
                  ) : null}
                  {session.task_id ? (
                    <button type="button" onClick={() => handleSessionDetachTask(session)} aria-label={`Detach task from ${session.id}`}>
                      <XCircle size={13} />
                    </button>
                  ) : null}
                  <button type="button" onClick={() => handleSessionTranscriptLoad(session)} aria-label={`Load transcript ${session.id}`}>
                    <Clipboard size={13} />
                  </button>
                  <button type="button" onClick={() => handleSessionKill(session)} aria-label={`Kill ${session.id}`}>
                    <XCircle size={13} />
                  </button>
                </div>
              );
            })
          )}
        </section>

        <section className="hc-rail-panel hc-file-explorer" aria-label="File Explorer">
          <header>
            <span>File Explorer</span>
            <FileText size={14} />
          </header>
          <div className="hc-file-explorer-meta">
            {projectFileList
              ? `${projectFileSearchQuery.trim() ? `search ${projectFileSearchQuery.trim()}` : projectFileList.relative_path || "."} · ${projectFileEntryCount} entries`
              : activeSnapshotProject?.name ?? "No active project"}
          </div>
          <label className="hc-file-search">
            <Search size={12} />
            <input
              aria-label="Search project files"
              value={projectFileSearchQuery}
              onChange={(event) => setProjectFileSearchQuery(event.currentTarget.value)}
              placeholder="Search files"
            />
          </label>
          {projectFileError ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>Explorer unavailable</span>
            </div>
          ) : projectFileSearchError ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>File search unavailable</span>
            </div>
          ) : projectFileList?.degraded_reason ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>{projectFileList.degraded_reason}</span>
            </div>
          ) : null}
          <div className="hc-file-tree" role="list">
            {visibleProjectFileEntries ? (
              visibleProjectFileEntries.slice(0, 8).map((entry) => {
                const status = entry.git_status ?? undefined;
                const canOpenDiff = entry.kind === "file" && Boolean(status && diffableGitStatuses.has(status));
                return (
                  <div className="hc-file-row" role="listitem" key={entry.path}>
                    <button
                      className="hc-file-open"
                      type="button"
                      onClick={() => {
                        if (entry.kind === "file") {
                          handleOpenProjectFile(entry.path);
                        }
                      }}
                      disabled={entry.kind !== "file"}
                      aria-label={entry.kind === "file" ? `Open file ${entry.name}` : `Directory ${entry.name}`}
                    >
                      <span className={`hc-file-icon ${entry.kind}`} aria-hidden="true">
                        {entry.kind === "directory" ? <Folder size={12} /> : <File size={12} />}
                      </span>
                      <span className="hc-file-name">{entry.name}</span>
                    </button>
                    {status ? (
                      <span className={`hc-git-status ${status}`} aria-label={`Git status ${status} for ${entry.name}`}>
                        {gitStatusBadgeLabels[status] ?? "C"}
                      </span>
                    ) : null}
                    {canOpenDiff ? (
                      <button type="button" className="hc-file-diff" onClick={() => handleOpenProjectDiff(entry.path)} aria-label={`Open diff ${entry.name}`}>
                        <GitBranch size={12} />
                      </button>
                    ) : null}
                  </div>
                );
              })
            ) : (
              <div className="hc-review-row">
                <FileText size={15} />
                <span>No project files loaded</span>
              </div>
            )}
          </div>
          <section className="hc-code-preview" aria-label="Code preview">
            <header>
              <span>{projectFilePreview?.path ?? "No file selected"}</span>
              <small>{projectFilePreview?.language ?? "text"}</small>
            </header>
            {projectFilePreviewError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>Preview unavailable</span>
              </div>
            ) : projectFilePreview ? (
              shouldRenderMonacoEditor(projectFilePreview) ? (
                <>
                  <MonacoCodeEditor
                    language={projectFilePreview.language}
                    path={projectFilePreview.path}
                    value={projectFileDraft}
                    onChange={setProjectFileDraft}
                  />
                  <div className="hc-source-editor-actions">
                    <button type="button" onClick={handleSaveProjectFile}>
                      Save source file
                    </button>
                    {projectFileSaveStatus ? <span>{projectFileSaveStatus}</span> : null}
                    {projectFileSaveError ? (
                      <span className="hc-source-editor-error">{projectFileSaveError}</span>
                    ) : null}
                  </div>
                </>
              ) : (
                <pre>{projectFilePreview.body}</pre>
              )
            ) : (
              <p>Select a file to preview</p>
            )}
          </section>
          <section className="hc-code-preview hc-lsp-diagnostics" aria-label="LSP diagnostics">
            <header>
              <span>LSP Diagnostics</span>
              <small>{projectLspDiagnostics ? `${projectLspDiagnostics.diagnostics.length} findings` : "not run"}</small>
            </header>
            <button type="button" onClick={handleRunLspDiagnostics} aria-label="Run LSP diagnostics">
              Run LSP diagnostics
            </button>
            {projectLspError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>{projectLspError}</span>
              </div>
            ) : projectLspDiagnostics && projectLspDiagnostics.diagnostics.length > 0 ? (
              <>
                <div className="hc-lsp-diagnostic-list">
                  {projectLspDiagnostics.diagnostics.slice(0, 6).map((diagnostic) => (
                    <article className="hc-lsp-diagnostic-row" key={`${diagnostic.path}:${diagnostic.line}:${diagnostic.message}`}>
                      <strong>{diagnostic.path}:{diagnostic.line} · {diagnostic.severity}</strong>
                      <span>{diagnostic.message}</span>
                    </article>
                  ))}
                </div>
                {projectLspDiagnostics.symbols?.length ? (
                  <div className="hc-lsp-symbol-list" aria-label="LSP symbol outline">
                    {projectLspDiagnostics.symbols.slice(0, 8).map((symbol) => (
                      <button
                        type="button"
                        key={`${symbol.path}:${symbol.line}:${symbol.name}`}
                        aria-label={`Open symbol ${symbol.name}`}
                        onClick={() => handleOpenProjectFile(symbol.path)}
                      >
                        {symbol.kind} · {symbol.name} · {symbol.path}:{symbol.line}
                      </button>
                    ))}
                  </div>
                ) : null}
              </>
            ) : projectLspDiagnostics ? (
              <p>No LSP diagnostics found</p>
            ) : (
              <p>Run local diagnostics for TODO markers and weak TypeScript typing</p>
            )}
          </section>
          <section className="hc-code-preview hc-quick-preview" aria-label="Quick preview">
            <header>
              <span>{projectFilePreview?.path ?? "No file selected"}</span>
              <small>{projectFilePreview ? projectQuickPreviewKind(projectFilePreview) : "preview"}</small>
            </header>
            {projectFilePreviewError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>Quick preview unavailable</span>
              </div>
            ) : projectFilePreview ? (
              renderProjectQuickPreview(projectFilePreview)
            ) : (
              <p>Select markdown, HTML, log, PDF, or image files</p>
            )}
          </section>
          <section className="hc-code-preview hc-diff-preview" aria-label="Review diff">
            <header>
              <span>{projectDiff?.path ?? "Workspace diff"}</span>
              <small>{projectDiff ? `${projectDiff.file_count} files` : "git diff"}</small>
            </header>
            <div className="hc-patch-workflow-actions">
              <button type="button" onClick={handleExportProjectPatch} aria-label="Export project patch">
                Export patch
              </button>
              <button type="button" onClick={handleImportProjectPatch} aria-label="Import project patch">
                Import patch
              </button>
              <button type="button" onClick={handlePlanPrLanding} aria-label="Plan draft PR landing">
                Plan PR
              </button>
            </div>
            <label className="hc-patch-workflow-field">
              <span>Patch body</span>
              <textarea
                aria-label="Patch import body"
                value={patchImportBody}
                onChange={(event) => setPatchImportBody(event.currentTarget.value)}
              />
            </label>
            <label className="hc-patch-workflow-field">
              <span>PR title</span>
              <input
                aria-label="PR landing title"
                value={prLandingTitle}
                onChange={(event) => setPrLandingTitle(event.currentTarget.value)}
                placeholder="Draft PR title"
              />
            </label>
            {patchWorkflowError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>{patchWorkflowError}</span>
              </div>
            ) : null}
            <div className="hc-patch-workflow-results">
              {exportedPatch ? <span>{exportedPatch.status} · {exportedPatch.patch_id}</span> : null}
              {importedPatch ? <span>{importedPatch.status} · {importedPatch.patch_id}</span> : null}
              {prLandingPlan ? (
                <article>
                  <strong>{prLandingPlan.provider} · {prLandingPlan.draft ? "draft" : "ready"}</strong>
                  {prLandingPlan.checklist.slice(0, 4).map((item) => <span key={item}>{item}</span>)}
                </article>
              ) : null}
            </div>
            {projectDiff?.files?.length ? (
              <div className="hc-diff-file-summary" aria-label="Diff file summary">
                {projectDiff.files.map((file) => (
                  <span key={`${file.path}:${file.status}`}>
                    {file.path} · {file.status} · +{file.additions} -{file.deletions}
                  </span>
                ))}
              </div>
            ) : null}
            {projectDiffError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>Diff unavailable</span>
              </div>
            ) : projectDiff?.degraded_reason ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>{projectDiff.degraded_reason}</span>
              </div>
            ) : projectDiff ? (
              <MonacoDiffEditor path={projectDiff.path} body={projectDiff.body} />
            ) : (
              <p>Select a changed file to review</p>
            )}
          </section>
          <section className="hc-localhost-preview" aria-label="Localhost browser">
            <header>
              <span>Localhost Browser</span>
              <small>{localhostPreviewUrl ?? "not loaded"}</small>
            </header>
            <label>
              <input
                aria-label="Localhost preview URL"
                value={localhostPreviewDraft}
                onChange={(event) => setLocalhostPreviewDraft(event.currentTarget.value)}
                placeholder="http://localhost:3000"
              />
              <button type="button" onClick={handleLoadLocalhostPreview} aria-label="Load localhost preview">
                Load
              </button>
              <button type="button" onClick={handlePlanBrowserAutomation} aria-label="Plan browser automation">
                Automate
              </button>
            </label>
            {localhostPreviewError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>{localhostPreviewError}</span>
              </div>
            ) : null}
            {browserAutomationError ? (
              <div className="hc-review-row">
                <AlertTriangle size={15} />
                <span>{browserAutomationError}</span>
              </div>
            ) : null}
            {browserAutomationPlan ? (
              <div className="hc-browser-automation-plan">
                <strong>{browserAutomationPlan.status} · {browserAutomationPlan.scenario}</strong>
                {browserAutomationPlan.steps.slice(0, 4).map((step) => <span key={step}>{step}</span>)}
              </div>
            ) : null}
            {!localhostPreviewError && localhostPreviewUrl ? (
              <iframe title={`Localhost preview ${localhostPreviewUrl}`} src={localhostPreviewUrl} sandbox="allow-forms allow-scripts allow-same-origin" />
            ) : (
              <p>Load a local app URL to inspect it beside terminals and review artifacts</p>
            )}
          </section>
        </section>

        <section className="hc-rail-panel" aria-label="Task Board">
          <header>
            <span>Task Board</span>
            <ListTodo size={14} />
          </header>
          <label className="hc-quick-task">
            <input
              ref={quickTaskInputRef}
              aria-label="Quick task title"
              value={quickTaskTitle}
              onChange={(event) => setQuickTaskTitle(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  handleQuickTaskCreate();
                }
              }}
              placeholder="New inbox task"
            />
            <button type="button" onClick={handleQuickTaskCreate} aria-label="Create task">
              <Plus size={13} />
            </button>
          </label>
          {quickTaskError ? <div className="hc-task-context-error">{quickTaskError}</div> : null}
          {taskLoadError ? <div className="hc-task-context-error">{taskLoadError}</div> : null}
          <div className="hc-task-count-grid">
            {taskBoardStatuses.map((status) => (
              <div className={`hc-task-count ${status}`} key={status}>
                <span>{taskStatusLabels[status]} {taskCounts[status]}</span>
              </div>
            ))}
          </div>
          <label className="hc-task-filter">
            <Search size={12} />
            <input
              aria-label="Filter tasks"
              value={taskFilterQuery}
              onChange={(event) => setTaskFilterQuery(event.currentTarget.value)}
              placeholder="Filter tasks"
            />
          </label>
          <div className="hc-task-view-controls">
            <button type="button" onClick={handleSaveTaskView} aria-label="Save task view" disabled={!taskFilterQuery.trim()}>
              Save
            </button>
            {taskSavedViews.map((view) => (
              <span className="hc-task-view-chip" key={view.id}>
                <button type="button" onClick={() => setTaskFilterQuery(view.query)} aria-label={`Apply task view ${view.label}`}>
                  {view.label}
                </button>
                <button type="button" onClick={() => handleDeleteTaskView(view.id)} aria-label={`Delete task view ${view.label}`}>
                  <XCircle size={12} />
                </button>
              </span>
            ))}
          </div>
          {taskViewError ? <div className="hc-task-context-error">{taskViewError}</div> : null}
          <div className="hc-task-filter-count">{taskFilterLabel}</div>
          {taskDispatchError ? <div className="hc-task-dispatch-error">{taskDispatchError}</div> : null}
          <div className="hc-task-mini-list" aria-label="Task list by status">
            {taskRowsByStatus.map(({ status, tasks }) => (
              <div className="hc-task-lane" role="group" aria-label={`${taskStatusLabels[status]} lane`} key={status}>
                <header>
                  <span>{taskStatusLabels[status]}</span>
                  <small>{tasks.length}</small>
                </header>
                {tasks.length === 0 ? (
                  <span className="hc-task-lane-empty">No matching tasks</span>
                ) : (
                  tasks.map((task) => {
                    const dispatchBlockedLabel = taskDispatchBlockedLabels.get(task.id);
                    return (
                      <div className="hc-task-mini-row" key={task.id}>
                        <button className="hc-task-open" type="button" onClick={() => handleOpenTaskDrawer(task.id)} aria-label={`Open task ${task.title}`}>
                          <span className="hc-task-open-title">{task.title}</span>
                          <small className="hc-task-open-meta">{formatTaskRowMetadata(task)}</small>
                          {dispatchBlockedLabel ? <small className="hc-task-dispatch-reason">{dispatchBlockedLabel}</small> : null}
                        </button>
                        <button
                          type="button"
                          onClick={() => handleDispatchTask(task.id)}
                          aria-label={`Dispatch ${task.title}`}
                          disabled={task.status !== "ready" || Boolean(dispatchBlockedLabel)}
                        >
                          <Activity size={12} />
                        </button>
                        <button type="button" onClick={() => handleAdvanceTask(task.id)} aria-label={`Advance ${task.title}`}>
                          <StepForward size={12} />
                        </button>
                      </div>
                    );
                  })
                )}
              </div>
            ))}
          </div>
        </section>

        <section className="hc-rail-panel" aria-label="Run Queue">
          <header>
            <span>Run Queue</span>
            <Activity size={14} />
          </header>
          <button type="button" className="hc-task-drawer-action" onClick={handleLoadRuns} aria-label="Load runs">
            Load runs
          </button>
          <div className="hc-run-count-grid">
            {runLifecycleSummaryStatuses.map((lifecycle) => (
              <span className="hc-run-count" key={lifecycle}>
                {lifecycle.replace(/_/g, " ")} {runCounts[lifecycle] ?? 0}
              </span>
            ))}
          </div>
          {runLifecycleError ? <div className="hc-run-lifecycle-error">{runLifecycleError}</div> : null}
          {evidenceGenerationError ? <div className="hc-run-lifecycle-error">{evidenceGenerationError}</div> : null}
          {runReplayError ? <div className="hc-run-lifecycle-error">{runReplayError}</div> : null}
          {runStatusUpdateStatus ? <div className="hc-run-lifecycle-status">{runStatusUpdateStatus}</div> : null}
          <div className="hc-run-list">
            {runItems.length === 0 ? (
              <div className="hc-review-row">
                <Activity size={15} />
                <span>No native runs queued</span>
              </div>
            ) : (
              runItems.slice(-4).map((run) => (
                <div className="hc-run-row" key={run.id}>
                  <span>{run.id} · {run.lifecycle} · retries {run.retry_count}</span>
                  {run.session_id ? <span className="hc-run-state-detail">session {run.session_id}</span> : null}
                  {run.workspace_path ? <span className="hc-run-state-detail">worktree {run.workspace_path}</span> : null}
                  {run.status_detail ? <span className="hc-run-state-detail">state detail {run.status_detail}</span> : null}
                  {run.next_retry_at ? <span className="hc-run-next-retry">next retry {run.next_retry_at}</span> : null}
                  {generatedEvidencePacksByRun[run.id] ? (
                    <span className="hc-run-state-detail">{formatNativeEvidencePackResult(generatedEvidencePacksByRun[run.id])}</span>
                  ) : null}
                  {runReplayMetadataByRun[run.id] ? (
                    <span className="hc-run-state-detail">{formatRunReplayMetadataResult(runReplayMetadataByRun[run.id])}</span>
                  ) : null}
                  <input
                    type="text"
                    value={runStatusDetailDrafts[run.id] ?? ""}
                    onChange={(event) => handleRunDetailDraft(run.id, event.currentTarget.value)}
                    placeholder="Status detail"
                    aria-label={`Status detail for ${run.id}`}
                  />
                  <input
                    type="text"
                    value={runStatusUpdateDrafts[run.id] ?? ""}
                    onChange={(event) => handleRunStatusUpdateDraft(run.id, event.currentTarget.value)}
                    placeholder="Agent status update"
                    aria-label={`Status update for ${run.id}`}
                  />
                  <div className="hc-run-actions">
                    <button type="button" onClick={() => handlePostRunStatusUpdate(run.id)} aria-label={`Post status for ${run.id}`}>
                      <FileText size={12} />
                      Post
                    </button>
                    <button type="button" onClick={() => handleGenerateEvidenceForRun(run.id)} aria-label={`Generate evidence for ${run.id}`}>
                      <FileText size={12} />
                      Evidence
                    </button>
                    <button type="button" onClick={() => handleLoadRunReplayMetadata(run.id)} aria-label={`Load replay for ${run.id}`}>
                      <RotateCcw size={12} />
                      Replay
                    </button>
                    <button type="button" onClick={() => handleRunLifecycle(run.id, "review_ready")} aria-label={`Move ${run.id} to review`}>
                      <CheckCircle2 size={12} />
                      Review
                    </button>
                    <button
                      type="button"
                      onClick={() => handleRunLifecycle(run.id, "blocked", runStatusDetailDrafts[run.id]?.trim())}
                      aria-label={`Block ${run.id}`}
                    >
                      <AlertTriangle size={12} />
                      Block
                    </button>
                    <button
                      type="button"
                      onClick={() => handleRunLifecycle(run.id, "waiting_input", runStatusDetailDrafts[run.id]?.trim())}
                      aria-label={`Wait ${run.id}`}
                    >
                      <Clock3 size={12} />
                      Wait
                    </button>
                    <button type="button" onClick={() => handleCancelRun(run.id)} aria-label={`Cancel ${run.id}`}>
                      <XCircle size={12} />
                      Cancel
                    </button>
                    <button type="button" onClick={() => handleRetryRun(run.id)} aria-label={`Retry ${run.id}`}>
                      <RotateCcw size={12} />
                      Retry
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </section>

        {selectedTaskOverview ? (
          <section className="hc-rail-panel hc-task-drawer">
            <header>
              <span>Task Drawer</span>
              <FileText size={14} />
            </header>
            <h3>{selectedTaskOverview.title}</h3>
            <p className="hc-task-drawer-meta">
              Status {selectedTaskOverview.status} · Priority {selectedTaskOverview.priority}
            </p>
            <p className="hc-task-drawer-meta">
              Cycle {selectedTaskOverview.cycle ?? "None"} · Module {selectedTaskOverview.module ?? "None"}
            </p>
            <p className="hc-task-drawer-meta">Initiative {selectedTaskOverview.initiative ?? "None"}</p>
            <p className="hc-task-drawer-meta">Labels {(selectedTaskOverview.labels ?? []).join(", ") || "None"}</p>
            <p className="hc-task-drawer-meta">Due {selectedTaskOverview.dueDate ?? "None"}</p>
            <p className="hc-task-drawer-meta">Estimate {selectedTaskOverview.estimate ?? "None"}</p>
            <p className="hc-task-drawer-meta">Assignee {selectedTaskOverview.assignee ?? "Unassigned"}</p>
            {selectedTaskOverview.description ? <p>{selectedTaskOverview.description}</p> : null}
            <div className="hc-task-planning-grid">
              <label>
                <span>Cycle</span>
                <input
                  aria-label="Task cycle"
                  list="task-cycle-options"
                  value={taskCycleDraft}
                  onChange={(event) => setTaskCycleDraft(event.currentTarget.value)}
                />
                <datalist id="task-cycle-options">
                  {taskCycles.map((cycle) => (
                    <option value={cycle.name} key={cycle.id}>
                      {cycle.id} · {cycle.status}
                    </option>
                  ))}
                </datalist>
                <button className="hc-task-drawer-action" type="button" onClick={handleCreateTaskCycle}>
                  Create task cycle
                </button>
              </label>
              <label>
                <span>Module</span>
                <input
                  aria-label="Task module"
                  list="task-module-options"
                  value={taskModuleDraft}
                  onChange={(event) => setTaskModuleDraft(event.currentTarget.value)}
                />
                <datalist id="task-module-options">
                  {taskModules.map((taskModule) => (
                    <option value={taskModule.name} key={taskModule.id}>
                      {taskModule.id} · {taskModule.status}
                    </option>
                  ))}
                </datalist>
                <button className="hc-task-drawer-action" type="button" onClick={handleCreateTaskModule}>
                  Create task module
                </button>
              </label>
              <label>
                <span>Initiative</span>
                <input
                  aria-label="Task initiative"
                  value={taskInitiativeDraft}
                  onChange={(event) => setTaskInitiativeDraft(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Labels</span>
                <input
                  aria-label="Task labels"
                  value={taskLabelsDraft}
                  onChange={(event) => setTaskLabelsDraft(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Due</span>
                <input
                  aria-label="Task due date"
                  value={taskDueDateDraft}
                  onChange={(event) => setTaskDueDateDraft(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Estimate</span>
                <input
                  aria-label="Task estimate"
                  value={taskEstimateDraft}
                  onChange={(event) => setTaskEstimateDraft(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Assignee</span>
                <select aria-label="Task assignee" value={taskAssigneeDraft} onChange={(event) => setTaskAssigneeDraft(event.currentTarget.value)}>
                  <option value="">Unassigned</option>
                  {taskAssigneeOptions.map((agent) => (
                    <option value={agent.id} key={agent.id}>
                      {agent.label}
                    </option>
                  ))}
                </select>
              </label>
            </div>
            <button className="hc-task-drawer-action" type="button" onClick={handleSaveTaskPlanningProperties}>
              Save task planning properties
            </button>
            {taskPlanningStatus ? <div className="hc-task-context-status">{taskPlanningStatus}</div> : null}
            {taskPlanningError ? <div className="hc-task-context-error">{taskPlanningError}</div> : null}
            <div className="hc-task-context-grid">
              <label>
                <span>Context pack</span>
                <select
                  aria-label="Task context pack"
                  value={taskContextPackDraft}
                  onChange={(event) => setTaskContextPackDraft(event.currentTarget.value)}
                >
                  <option value="">Default context</option>
                  {contextPacks.map((pack) => (
                    <option value={pack.id} key={pack.id}>
                      {pack.name} ({pack.id})
                    </option>
                  ))}
                </select>
              </label>
              <button className="hc-task-drawer-action" type="button" onClick={handleAttachTaskContextPack}>
                Attach context pack to task
              </button>
              <button className="hc-task-drawer-action" type="button" onClick={() => handleDispatchTask(selectedTaskOverview.id)}>
                Dispatch selected task with context
              </button>
            </div>
            {taskContextStatus ? <div className="hc-task-context-status">{taskContextStatus}</div> : null}
            {taskContextError ? <div className="hc-task-context-error">{taskContextError}</div> : null}
            {taskDispatchError ? <div className="hc-task-dispatch-error">{taskDispatchError}</div> : null}
            <div className="hc-task-drawer-comments">
              Subtasks {selectedTaskOverview.openSubtaskCount} open / {selectedTaskOverview.subtaskCount} total
            </div>
            <div className="hc-task-drawer-comment-list">
              {selectedTaskOverview.subtasks.map((subtask) => (
                <article className="hc-task-drawer-comment-row" key={subtask.id}>
                  <strong>{subtask.status}</strong>
                  <p>{subtask.title}</p>
                  <button
                    className="hc-task-drawer-action"
                    type="button"
                    onClick={() => handleUpdateTaskSubtaskStatus(subtask.id, subtask.status === "done" ? "open" : "done")}
                    aria-label={`Mark ${subtask.title} ${subtask.status === "done" ? "open" : "done"}`}
                  >
                    {subtask.status === "done" ? "Reopen" : "Done"}
                  </button>
                </article>
              ))}
            </div>
            <label className="hc-task-drawer-comment">
              <span>New subtask</span>
              <input
                aria-label="New task subtask"
                value={newTaskSubtaskTitle}
                onChange={(event) => setNewTaskSubtaskTitle(event.currentTarget.value)}
              />
            </label>
            <button className="hc-task-drawer-action" type="button" onClick={handleAddTaskSubtask}>
              Add task subtask
            </button>
            {taskSubtaskError ? <div className="hc-task-context-error">{taskSubtaskError}</div> : null}
            <label className="hc-task-drawer-workpad">
              <div className="hc-workpad-header">
                <strong>Workpad</strong>
                <div className="hc-workpad-mode" role="group" aria-label="Task workpad mode">
                  <button
                    type="button"
                    className={taskWorkpadMode === "edit" ? "active" : ""}
                    onClick={() => setTaskWorkpadMode("edit")}
                    aria-label="Edit task workpad"
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    className={taskWorkpadMode === "preview" ? "active" : ""}
                    onClick={() => setTaskWorkpadMode("preview")}
                    aria-label="Preview task workpad"
                  >
                    Preview
                  </button>
                </div>
              </div>
              {taskWorkpadMode === "preview" ? (
                <section className="hc-workpad-preview" aria-label="Task workpad markdown preview">
                  {renderMarkdownPreview(taskWorkpadDraft)}
                </section>
              ) : (
                <CodeMirrorMarkdownEditor
                  label="Task workpad"
                  value={taskWorkpadDraft}
                  onChange={setTaskWorkpadDraft}
                />
              )}
            </label>
            <button className="hc-task-drawer-action" type="button" onClick={handleSaveTaskWorkpad}>
              Save task workpad
            </button>
            {taskWorkpadError ? <p className="hc-task-drawer-error">{taskWorkpadError}</p> : null}
            <label className="hc-task-drawer-comment">
              <span>New comment</span>
              <textarea
                aria-label="New task comment"
                value={newTaskComment}
                onChange={(event) => setNewTaskComment(event.currentTarget.value)}
              />
            </label>
            <button className="hc-task-drawer-action" type="button" onClick={handleAddTaskComment}>
              Add task comment
            </button>
            {taskCommentError ? <p className="hc-task-drawer-error">{taskCommentError}</p> : null}
            <div className="hc-task-drawer-comments">Comments {selectedTaskOverview.commentCount}</div>
            <div className="hc-task-drawer-comment-list">
              {selectedTaskOverview.comments.map((comment) => (
                <article className="hc-task-drawer-comment-row" key={comment.id}>
                  <strong>{comment.author}</strong>
                  <p>{comment.body}</p>
                </article>
              ))}
            </div>
          </section>
        ) : null}

        <section className="hc-rail-panel hc-project-window-tabs" aria-label="Project window tabs">
          <header>
            <span>Project Windows</span>
            <LayoutGrid size={14} />
          </header>
          {projectWindowDegraded ? (
            <div className="hc-project-window-degraded">
              <AlertTriangle size={15} />
              <span>Project window degraded · api {controlPlaneSnapshot.health.api} · db {controlPlaneSnapshot.health.db}</span>
            </div>
          ) : null}
          <div className="hc-project-window-summary">
            <span>Active project {activeProjectName}</span>
            {activeProjectDetachPlan ? <span>{activeProjectDetachPlan.windowId} · {activeProjectDetachPlan.status}</span> : <span>single window</span>}
          </div>
          <div className="hc-project-tab-group-list" aria-label="Project tab groups">
            {workspaceTabs.map((tab) => {
              const group = tab.projectId
                ? projectTabGroups.find((assignment) => assignment.projectId === tab.projectId)?.group ??
                  controlPlaneSnapshot.project_tabs.find((stateTab) => stateTab.project_id === tab.projectId)?.group_name
                : undefined;
              return (
                <div className="hc-project-tab-group-row" key={`project-tab-group-${tab.id}`}>
                  <span className={`hc-status-dot ${tab.active ? "active" : "idle"}`} />
                  <span>{tab.label} · {tab.active ? "active" : "idle"} · {group ?? "ungrouped"}</span>
                </div>
              );
            })}
          </div>
          <div className="hc-project-window-actions">
            <label>
              <span>Group</span>
              <input
                aria-label="Project tab group name"
                value={projectTabGroupDraft}
                onChange={(event) => setProjectTabGroupDraft(event.currentTarget.value)}
                placeholder={activeProjectTabGroup ?? "ungrouped"}
              />
            </label>
            <button type="button" onClick={handleSaveProjectTabGroup}>
              Save project tab group
            </button>
            <button type="button" onClick={handleDetachActiveProject}>
              Detach active project
            </button>
            <label>
              <span>Layout preset</span>
              <input
                aria-label="Layout preset name"
                value={projectLayoutPresetName}
                onChange={(event) => setProjectLayoutPresetName(event.currentTarget.value)}
                placeholder="Review grid"
              />
            </label>
            <button type="button" onClick={handleSaveProjectLayoutPreset}>
              Save layout preset
            </button>
            <button type="button" onClick={handleLoadProjectLayoutPresets}>
              Load layout presets
            </button>
            <div className="hc-project-layout-preset-list" aria-label="Saved layout presets">
              {activeProjectLayoutPresets.length === 0 ? (
                <span>No saved layout presets</span>
              ) : (
                activeProjectLayoutPresets.map((preset) => (
                  <button
                    type="button"
                    key={preset.id}
                    aria-label={`Apply layout preset ${preset.name}`}
                    onClick={() => handleApplyProjectLayoutPreset(preset)}
                  >
                    {preset.name} · {preset.layoutJson.mode}
                  </button>
                ))
              )}
            </div>
          </div>
          {projectWindowStatus ? <div className="hc-project-window-status">{projectWindowStatus}</div> : null}
        </section>

        <section className="hc-rail-panel" aria-label="Control Plane">
          <header>
            <span>Control Plane</span>
            <CircleDot size={14} />
          </header>
          <div className="hc-review-row">
            <FileText size={15} />
            <span>State Snapshot {controlPlaneSnapshot.snapshot_id}</span>
          </div>
          <div className="hc-review-row">
            <Activity size={15} />
            <span>DB {controlPlaneSnapshot.health.db} · PTY {controlPlaneSnapshot.health.pty} · API {controlPlaneSnapshot.health.api}</span>
          </div>
          <div className="hc-review-row">
            <Code2 size={15} />
            <span>State command blocks {controlPlaneSnapshot.command_blocks.unread_count} unread</span>
          </div>
          <div className="hc-review-row">
            <ListTodo size={15} />
            <span>State tasks {trackedTaskCount} tracked</span>
          </div>
          <div className="hc-review-row">
            <Clock3 size={15} />
            <span>{formatBudgetSummary(primaryBudget)}</span>
          </div>
          <div className="hc-review-row">
            <FileText size={15} />
            <span>{formatKnowledgeSummary(controlPlaneSnapshot)}</span>
          </div>
          <div className="hc-review-row">
            <ShieldAlert size={15} />
            <span>
              OSC {oscEventState.allowedEvents} allowed · {oscEventState.ignoredEvents} ignored · {oscEventState.rejectedEvents} rejected
            </span>
          </div>
          {controlPlaneSnapshot.attention.slice(0, 2).map((item) => (
            <div className="hc-review-row" key={item.id}>
              <AlertTriangle size={15} />
              <span>{item.label}</span>
            </div>
          ))}
        </section>

        <section className="hc-rail-panel hc-terminal-theme" aria-label="Terminal theme import export">
          <header>
            <span>Terminal Theme</span>
            <Terminal size={14} />
          </header>
          <textarea
            aria-label="Terminal theme JSON"
            value={terminalThemeJson}
            onChange={(event) => setTerminalThemeJson(event.currentTarget.value)}
          />
          <div className="hc-terminal-theme-actions">
            <button type="button" onClick={handleImportTerminalTheme}>
              Import terminal theme
            </button>
            <button type="button" onClick={handleExportTerminalTheme}>
              Export terminal theme
            </button>
            <button type="button" onClick={handleSaveProjectTerminalTheme}>
              Save project terminal theme
            </button>
          </div>
          {terminalThemeStatus ? <div className="hc-terminal-theme-status">{terminalThemeStatus}</div> : null}
          {terminalThemeError ? <div className="hc-terminal-theme-error">{terminalThemeError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-provider-model-switcher" aria-label="Provider model switcher">
          <header>
            <span>Provider Model</span>
            <Activity size={14} />
          </header>
          {controlPlaneSnapshot.health.api !== "ok" || controlPlaneSnapshot.health.db !== "ok" ? (
            <div className="hc-provider-model-degraded">
              <AlertTriangle size={15} />
              <span>Provider model degraded · api {controlPlaneSnapshot.health.api} · db {controlPlaneSnapshot.health.db}</span>
            </div>
          ) : null}
          <div className="hc-provider-model-current">
            Active {providerModelSettings.provider} · {providerModelSettings.model} · agent {providerModelSettings.agentProfileId}
          </div>
          <div className="hc-provider-model-form">
            <label>
              <span>Provider</span>
              <select
                aria-label="Default provider"
                value={providerModelProvider}
                onChange={(event) => setProviderModelProvider(event.currentTarget.value)}
              >
                <option value="openai">openai</option>
                <option value="anthropic">anthropic</option>
                <option value="google">google</option>
                <option value="local">local</option>
              </select>
            </label>
            <label>
              <span>Model</span>
              <input
                aria-label="Default model"
                value={providerModelName}
                onChange={(event) => setProviderModelName(event.currentTarget.value)}
              />
            </label>
            <label>
              <span>Agent</span>
              <input
                aria-label="Provider model agent"
                value={providerModelAgent}
                onChange={(event) => setProviderModelAgent(event.currentTarget.value)}
                list="provider-model-agents"
              />
            </label>
            <datalist id="provider-model-agents">
              {taskAssigneeOptions.map((agent) => (
                <option value={agent.id} key={`provider-model-agent-${agent.id}`}>
                  {agent.label}
                </option>
              ))}
            </datalist>
            <button type="button" onClick={handleLoadProviderModel}>
              Load provider model
            </button>
            <button type="button" onClick={handleSaveProviderModel}>
              Save provider model
            </button>
          </div>
          {providerModelStatus ? <div className="hc-provider-model-status">{providerModelStatus}</div> : null}
          {providerModelError ? <div className="hc-provider-model-error">{providerModelError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-release-channel" aria-label="Release channel settings">
          <header>
            <span>Release Channel</span>
            <Activity size={14} />
          </header>
          <label className="hc-release-channel-picker">
            <span>Release channel</span>
            <select
              aria-label="Release channel"
              value={releaseChannel}
              onChange={(event) => handleReleaseChannelChange(event.currentTarget.value === "beta" ? "beta" : "stable")}
            >
              <option value="stable">stable</option>
              <option value="beta">beta</option>
            </select>
          </label>
          <button type="button" className="hc-release-channel-action" onClick={handleCheckUpdateFeed}>
            Check update feed
          </button>
          {updateFeedCheck ? (
            <div className="hc-release-channel-result">
              <strong>{updateFeedCheck.channel} feed {updateFeedCheck.version}</strong>
              <span>{updateFeedCheck.platform} · {updateFeedCheck.url ?? "no artifact URL"}</span>
              <small>{formatUpdateFeedSignatureState(updateFeedCheck.signatureState)}</small>
            </div>
          ) : (
            <div className="hc-review-row">
              <Clock3 size={15} />
              <span>Static update feed channel {releaseChannel}</span>
            </div>
          )}
          {updateFeedError ? <div className="hc-release-channel-error">{updateFeedError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-release-channel" aria-label="Release publishing workflows">
          <header>
            <span>Release Workflows</span>
            <PackageCheck size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>Status {releaseWorkflowStatus?.status ?? "not checked"}</span>
          </div>
          <button type="button" className="hc-release-channel-action" onClick={handleRefreshReleaseWorkflows} aria-label="Refresh release workflows">
            Refresh release workflows
          </button>
          {releaseWorkflowStatus ? (
            <div className="hc-quality-runner-result">
              <strong>Release workflows {releaseWorkflowStatus.status}</strong>
              {releaseWorkflowStatus.workflows.map((workflow) => (
                <span key={`release-workflow-${workflow.id}`}>{formatReleaseWorkflowRow(workflow)}</span>
              ))}
              {releaseWorkflowStatus.workflows.map((workflow) => (
                <small key={`release-workflow-env-${workflow.id}`}>{formatReleaseWorkflowMissingEnv(workflow)}</small>
              ))}
            </div>
          ) : (
            <div className="hc-review-row">
              <Clock3 size={15} />
              <span>Signed macOS DMG, notarization, Homebrew cask, and crash symbol workflow diagnostics not checked</span>
            </div>
          )}
          {releaseWorkflowError ? <div className="hc-release-channel-error">{releaseWorkflowError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="Release gate runner">
          <header>
            <span>Release Gates</span>
            <ShieldAlert size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              {controlPlaneSnapshot.release_gates?.last_run_id ?? "No release gate run"} · {controlPlaneSnapshot.release_gates?.last_status ?? "not_run"}
              {" · "}
              {controlPlaneSnapshot.release_gates?.last_pass_count ?? 0} pass · {controlPlaneSnapshot.release_gates?.last_fail_count ?? 0} fail
            </span>
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunReleaseGates} aria-label="Run release gates">
            Run release gates
          </button>
          <button
            type="button"
            className="hc-quality-runner-action"
            onClick={handleLoadReleaseGateHistory}
            aria-label="Load release gate history"
          >
            Load gate history
          </button>
          {releaseGateRun ? (
            <div className="hc-quality-runner-result">
              <strong>Run {formatReleaseGateRun(releaseGateRun)}</strong>
              {releaseGateRun.scenarios.slice(0, 3).map((scenario) => (
                <span key={`release-gate-scenario-${scenario.gate_id}`}>{formatReleaseGateScenario(scenario)}</span>
              ))}
            </div>
          ) : null}
          {releaseGateHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {releaseGateHistory.slice(0, 3).map((run) => (
                <span key={`release-gate-history-${run.id}`}>History {formatReleaseGateRun(run)}</span>
              ))}
            </div>
          ) : null}
          {releaseGateError ? <div className="hc-quality-runner-error">{releaseGateError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="Terminal fidelity smoke tests">
          <header>
            <span>Terminal Fidelity</span>
            <Terminal size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              Latest {controlPlaneSnapshot.terminal_fidelity?.last_run_id ?? "No terminal smoke run"} · {controlPlaneSnapshot.terminal_fidelity?.last_status ?? "not_run"}
              {" · "}
              {controlPlaneSnapshot.terminal_fidelity?.last_pass_count ?? 0} pass · {controlPlaneSnapshot.terminal_fidelity?.last_fail_count ?? 0} fail · {controlPlaneSnapshot.terminal_fidelity?.last_warning_count ?? 0} warning
            </span>
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunTerminalFidelitySmoke} aria-label="Run terminal fidelity smoke">
            Run terminal smoke
          </button>
          <button type="button" className="hc-quality-runner-action" onClick={handleLoadTerminalSmokeHistory} aria-label="Load terminal smoke history">
            Load terminal history
          </button>
          {terminalSmokeRun ? (
            <div className="hc-quality-runner-result">
              <strong>{formatTerminalSmokeRun(terminalSmokeRun)}</strong>
              {terminalSmokeRun.cases.slice(0, 8).map((testCase) => (
                <span key={`terminal-smoke-case-${testCase.case_id}`}>{formatTerminalSmokeCase(testCase)}</span>
              ))}
            </div>
          ) : null}
          {terminalSmokeHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {terminalSmokeHistory.slice(0, 3).map((run) => (
                <span key={`terminal-smoke-history-${run.id}`}>History {formatTerminalSmokeRun(run)}</span>
              ))}
            </div>
          ) : null}
          {terminalSmokeError ? <div className="hc-quality-runner-error">{terminalSmokeError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="Task lifecycle E2E">
          <header>
            <span>Task Lifecycle</span>
            <ListTodo size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              {controlPlaneSnapshot.task_lifecycle?.last_run_id ?? "No lifecycle run"} · {controlPlaneSnapshot.task_lifecycle?.last_status ?? "not_run"}
            </span>
            {controlPlaneSnapshot.task_lifecycle?.last_evidence_pack_id ? (
              <span>Evidence {controlPlaneSnapshot.task_lifecycle.last_evidence_pack_id}</span>
            ) : null}
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunTaskLifecycleE2E} aria-label="Run task lifecycle E2E">
            Run lifecycle E2E
          </button>
          <button type="button" className="hc-quality-runner-action" onClick={handleLoadTaskLifecycleHistory} aria-label="Load task lifecycle history">
            Load lifecycle history
          </button>
          {taskLifecycleRun ? (
            <div className="hc-quality-runner-result">
              <strong>{formatTaskLifecycleRun(taskLifecycleRun)}</strong>
              <span>Evidence pack {taskLifecycleRun.evidence_pack_id}</span>
              {taskLifecycleRun.transitions.slice(-3).map((transition) => (
                <span key={`task-lifecycle-transition-${transition.step}`}>{formatTaskLifecycleTransition(transition)}</span>
              ))}
            </div>
          ) : null}
          {taskLifecycleHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {taskLifecycleHistory.slice(0, 3).map((run) => (
                <span key={`task-lifecycle-history-${run.id}`}>History {formatTaskLifecycleRun(run)}</span>
              ))}
            </div>
          ) : null}
          {taskLifecycleError ? <div className="hc-quality-runner-error">{taskLifecycleError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="Workflow negative tests">
          <header>
            <span>Workflow Negative</span>
            <ShieldAlert size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              {controlPlaneSnapshot.workflow_negative?.last_run_id ?? "No workflow negative run"} · {controlPlaneSnapshot.workflow_negative?.last_status ?? "not_run"}
            </span>
            {controlPlaneSnapshot.workflow_negative?.last_known_good_workflow_id ? (
              <span>LKG {controlPlaneSnapshot.workflow_negative.last_known_good_workflow_id}</span>
            ) : null}
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunWorkflowNegativeTests} aria-label="Run workflow negative tests">
            Run workflow negative
          </button>
          <button type="button" className="hc-quality-runner-action" onClick={handleLoadWorkflowNegativeHistory} aria-label="Load workflow negative history">
            Load workflow history
          </button>
          {workflowNegativeRun ? (
            <div className="hc-quality-runner-result">
              <strong>{formatWorkflowNegativeRun(workflowNegativeRun)}</strong>
              {workflowNegativeRun.cases.slice(0, 3).map((testCase) => (
                <span key={`workflow-negative-case-${testCase.case_id}`}>{formatWorkflowNegativeCase(testCase)}</span>
              ))}
            </div>
          ) : null}
          {workflowNegativeHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {workflowNegativeHistory.slice(0, 3).map((run) => (
                <span key={`workflow-negative-history-${run.id}`}>History {formatWorkflowNegativeRun(run)}</span>
              ))}
            </div>
          ) : null}
          {workflowNegativeError ? <div className="hc-quality-runner-error">{workflowNegativeError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="DMG install smoke test">
          <header>
            <span>DMG Smoke</span>
            <PackageCheck size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              {controlPlaneSnapshot.distribution?.last_dmg_smoke_run_id ?? "No DMG smoke run"} · {controlPlaneSnapshot.distribution?.last_status ?? "not_run"}
              {" · "}
              {controlPlaneSnapshot.distribution?.last_pass_count ?? 0} pass · {controlPlaneSnapshot.distribution?.last_fail_count ?? 0} fail
            </span>
            <span>Blocker {String(controlPlaneSnapshot.distribution?.explicit_blocker ?? false)}</span>
          </div>
          <div className="hc-budget-dashboard-actions">
            <input
              aria-label="DMG artifact path"
              value={dmgSmokePath}
              onChange={(event) => setDmgSmokePath(event.currentTarget.value)}
              placeholder="/tmp/Haneulchi.dmg"
            />
            <input
              aria-label="App bundle path"
              value={dmgSmokeAppBundlePath}
              onChange={(event) => setDmgSmokeAppBundlePath(event.currentTarget.value)}
              placeholder="/Applications/Haneulchi.app"
            />
            <button type="button" className="hc-quality-runner-action" onClick={handleRunDmgSmokeTest} aria-label="Run DMG smoke test">
              Run DMG smoke
            </button>
            <button type="button" className="hc-quality-runner-action" onClick={handleLoadDmgSmokeHistory} aria-label="Load DMG smoke history">
              Load DMG history
            </button>
          </div>
          {dmgSmokeRun ? (
            <div className="hc-quality-runner-result">
              <strong>{formatDmgSmokeRun(dmgSmokeRun)}</strong>
              {dmgSmokeRun.cases.slice(0, 3).map((testCase) => (
                <span key={`dmg-smoke-case-${testCase.case_id}`}>{formatDmgSmokeCase(testCase)}</span>
              ))}
            </div>
          ) : null}
          {dmgSmokeHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {dmgSmokeHistory.slice(0, 3).map((run) => (
                <span key={`dmg-smoke-history-${run.id}`}>History {formatDmgSmokeRun(run)}</span>
              ))}
            </div>
          ) : null}
          {dmgSmokeError ? <div className="hc-quality-runner-error">{dmgSmokeError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="Recovery drills">
          <header>
            <span>Recovery Drills</span>
            <Activity size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              Latest {controlPlaneSnapshot.recovery?.last_run_id ?? "No recovery drill"} · {controlPlaneSnapshot.recovery?.last_status ?? "not_run"}
              {" · "}
              {controlPlaneSnapshot.recovery?.last_pass_count ?? 0} pass · {controlPlaneSnapshot.recovery?.last_fail_count ?? 0} fail
            </span>
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunRecoveryDrills} aria-label="Run recovery drills">
            Run recovery drills
          </button>
          <button type="button" className="hc-quality-runner-action" onClick={handleLoadRecoveryDrillHistory} aria-label="Load recovery drill history">
            Load recovery history
          </button>
          {recoveryDrillRun ? (
            <div className="hc-quality-runner-result">
              <strong>{formatRecoveryDrillRun(recoveryDrillRun)}</strong>
              {recoveryDrillRun.drills.slice(0, 3).map((drill) => (
                <span key={`recovery-drill-${drill.drill_id}`}>{formatRecoveryDrill(drill)}</span>
              ))}
            </div>
          ) : null}
          {recoveryDrillHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {recoveryDrillHistory.slice(0, 3).map((run) => (
                <span key={`recovery-drill-history-${run.id}`}>History {formatRecoveryDrillRun(run)}</span>
              ))}
            </div>
          ) : null}
          {recoveryDrillError ? <div className="hc-quality-runner-error">{recoveryDrillError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-tracker-boundary" aria-label="External tracker boundary">
          <header>
            <span>Tracker Boundary</span>
            <GitBranch size={14} />
          </header>
          <div className="hc-tracker-boundary-summary">
            <span>Bindings {controlPlaneSnapshot.tracker?.binding_count ?? 0}</span>
            <span>Status {controlPlaneSnapshot.tracker?.diagnostics?.status ?? "unbound"}</span>
          </div>
          <div className="hc-tracker-boundary-form">
            <select
              aria-label="Tracker provider"
              value={trackerProvider}
              onChange={(event) => setTrackerProvider(event.currentTarget.value as NativeTrackerProvider)}
            >
              <option value="linear">linear</option>
              <option value="github">github</option>
              <option value="plane">plane</option>
              <option value="custom">custom</option>
              <option value="manual">manual</option>
            </select>
            <select
              aria-label="Tracker local kind"
              value={trackerLocalKind}
              onChange={(event) => setTrackerLocalKind(event.currentTarget.value as NativeTrackerLocalKind)}
            >
              <option value="task">task</option>
              <option value="project">project</option>
            </select>
            <input
              aria-label="Tracker local id"
              value={trackerLocalId}
              onChange={(event) => setTrackerLocalId(event.currentTarget.value)}
              placeholder="task_1"
            />
            <input
              aria-label="Tracker external id"
              value={trackerExternalId}
              onChange={(event) => setTrackerExternalId(event.currentTarget.value)}
              placeholder="LIN-42"
            />
            <input
              aria-label="Tracker external URL"
              value={trackerExternalUrl}
              onChange={(event) => setTrackerExternalUrl(event.currentTarget.value)}
              placeholder="https://linear.app/acme/issue/LIN-42"
            />
            <select
              aria-label="Tracker sync mode"
              value={trackerSyncMode}
              onChange={(event) => setTrackerSyncMode(event.currentTarget.value as NativeTrackerSyncMode)}
            >
              <option value="manual">manual</option>
              <option value="mirror">mirror</option>
              <option value="import">import</option>
              <option value="export">export</option>
            </select>
            <label className="hc-tracker-boundary-toggle">
              <input
                type="checkbox"
                checked={trackerDryRun}
                onChange={(event) => setTrackerDryRun(event.currentTarget.checked)}
              />
              <span>Dry run</span>
            </label>
            <button type="button" onClick={handleBindExternalTracker} aria-label="Bind external tracker">
              Bind tracker
            </button>
            <button type="button" onClick={handleLoadTrackerBindings} aria-label="Load tracker bindings">
              Load bindings
            </button>
            <button type="button" onClick={handleRunTrackerSync} aria-label="Run tracker sync">
              Run sync
            </button>
          </div>
          {trackerBinding ? (
            <div className="hc-tracker-boundary-result">
              <strong>{formatTrackerBinding(trackerBinding)}</strong>
              <span>{trackerBinding.local_kind} {trackerBinding.local_id} · {trackerBinding.sync_mode}</span>
            </div>
          ) : null}
          {trackerSyncRun ? (
            <div className="hc-tracker-boundary-result">
              <strong>{formatTrackerSyncRun(trackerSyncRun)}</strong>
              {trackerSyncRun.degraded_reason ? <span>{trackerSyncRun.degraded_reason}</span> : null}
            </div>
          ) : null}
          {(controlPlaneSnapshot.tracker?.bindings ?? [])
            .filter((binding) => binding.id !== trackerBinding?.id)
            .slice(0, 3)
            .map((binding) => (
            <div className="hc-tracker-boundary-result" key={`tracker-binding-${binding.id}`}>
              <span>{binding.external_id} · {binding.provider} · {binding.sync_status}</span>
            </div>
          ))}
          {trackerError ? <div className="hc-tracker-boundary-error">{trackerError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-roadmap-timeline" aria-label="Roadmap timeline">
          <header>
            <span>Roadmap Timeline</span>
            <ListTodo size={14} />
          </header>
          <div className="hc-task-planning-grid">
            <label>
              <span>Initiative</span>
              <input
                aria-label="Roadmap initiative name"
                value={roadmapInitiativeNameDraft}
                onChange={(event) => setRoadmapInitiativeNameDraft(event.target.value)}
              />
            </label>
            <label>
              <span>Status</span>
              <input
                aria-label="Roadmap initiative status"
                value={roadmapInitiativeStatusDraft}
                onChange={(event) => setRoadmapInitiativeStatusDraft(event.target.value)}
              />
            </label>
          </div>
          <button className="hc-task-drawer-action" type="button" onClick={handleCreateRoadmapInitiative}>
            Create roadmap initiative
          </button>
          {roadmapInitiativeStatus ? <div className="hc-task-context-status">{roadmapInitiativeStatus}</div> : null}
          {roadmapInitiativeError ? <div className="hc-task-context-error">{roadmapInitiativeError}</div> : null}
          {roadmapTimelineItems.length === 0 ? (
            <div className="hc-review-row">
              <ListTodo size={15} />
              <span>No planned tasks</span>
            </div>
          ) : (
            roadmapTimelineItems.slice(0, 5).map((item) => (
              <article className="hc-planning-row" key={item.id}>
                <header>
                  <strong>{item.label}</strong>
                  <small>Urgent {item.urgentCount}</small>
                </header>
                <span>{item.taskCount} {item.taskCount === 1 ? "task" : "tasks"} · {item.statusLine}</span>
              </article>
            ))
          )}
          {(controlPlaneSnapshot.initiatives ?? []).slice(0, 4).map((initiative) => (
            <article className="hc-planning-row" key={`roadmap-initiative-${initiative.id}`}>
              <header>
                <strong>{initiative.name} · {initiative.status}</strong>
                <small>{initiative.budget_id ?? "No budget"}</small>
              </header>
              {initiative.description ? <span>{initiative.description}</span> : null}
            </article>
          ))}
        </section>

        <section className="hc-rail-panel hc-calendar-view" aria-label="Calendar view">
          <header>
            <span>Calendar View</span>
            <CalendarDays size={14} />
          </header>
          {calendarTaskBuckets.length === 0 ? (
            <div className="hc-review-row">
              <CalendarDays size={15} />
              <span>No scheduled task buckets</span>
            </div>
          ) : (
            calendarTaskBuckets.slice(0, 4).map((bucket) => (
              <article className="hc-calendar-bucket" key={bucket.id}>
                <strong>{bucket.label}</strong>
                <div>
                  {bucket.tasks.slice(0, 4).map((task) => (
                    <span key={task.id}>{formatCalendarTaskLabel(task)}</span>
                  ))}
                </div>
              </article>
            ))
          )}
        </section>

        <section className="hc-rail-panel hc-skill-pack-registry" aria-label="Skill pack registry">
          <header>
            <span>Skill Pack Registry</span>
            <Blocks size={14} />
          </header>
          <div className="hc-skill-pack-controls">
            <button type="button" onClick={handleLoadSkillPacks}>
              <RotateCcw size={13} />
              <span>Load skill packs</span>
            </button>
            <input
              aria-label="Skill pack name"
              value={skillPackName}
              onChange={(event) => setSkillPackName(event.target.value)}
              placeholder="Skill pack name"
            />
            <input
              aria-label="Skill pack description"
              value={skillPackDescription}
              onChange={(event) => setSkillPackDescription(event.target.value)}
              placeholder="Description"
            />
            <input
              aria-label="Skill pack skills JSON"
              value={skillPackSkillsJson}
              onChange={(event) => setSkillPackSkillsJson(event.target.value)}
              placeholder="[&quot;code-review&quot;]"
            />
            <input
              aria-label="Skill pack context pack"
              value={skillPackContextPackId}
              onChange={(event) => setSkillPackContextPackId(event.target.value)}
              placeholder="Context pack"
            />
            <button type="button" onClick={handleCreateSkillPack}>
              <Plus size={13} />
              <span>Create skill pack</span>
            </button>
          </div>
          {skillPackStatus ? <div className="hc-skill-pack-status">{skillPackStatus}</div> : null}
          {skillPackError ? <div className="hc-skill-pack-error">{skillPackError}</div> : null}
          {skillPackRegistryItems.length === 0 ? (
            <div className="hc-review-row">
              <Blocks size={15} />
              <span>No reusable skill packs</span>
            </div>
          ) : (
            skillPackRegistryItems.slice(0, 4).map((pack) => (
              <article className="hc-skill-pack-row" key={pack.id}>
                <header>
                  <strong>{pack.name}</strong>
                  <small>{pack.id} · {pack.budgetLabel}</small>
                </header>
                <span>Used by {pack.activeWorkloadCount} active workloads</span>
                <small>{pack.sourceLine}</small>
              </article>
            ))
          )}
        </section>

        <section className="hc-rail-panel hc-runtime-pool" aria-label="Runtime pool">
          <header>
            <span>Runtime Pool</span>
            <Terminal size={14} />
          </header>
          <button className="hc-runtime-pool-action" type="button" onClick={handleLoadRuntimePool}>
            <RotateCcw size={13} />
            <span>Load runtime pool</span>
          </button>
          {runtimePoolStatus ? <div className="hc-runtime-pool-status">{runtimePoolStatus}</div> : null}
          {runtimePoolError ? <div className="hc-runtime-pool-error">{runtimePoolError}</div> : null}
          <div className="hc-runtime-pool-summary">
            {runtimePoolItems.length === 0 ? (
              <span>No runtimes</span>
            ) : (
              runtimePoolItems.map((item) => <span key={`runtime-summary-${item.id}`}>{item.label} {item.sessionCount}</span>)
            )}
          </div>
          {runtimePoolItems.map((item) => (
            <div className="hc-runtime-pool-row" key={item.id}>
              <span className={`hc-status-dot ${item.blockedCount > 0 ? "warning" : "ready"}`} />
              <span>
                {item.id} · {item.sessionCount} {item.sessionCount === 1 ? "session" : "sessions"} · {item.runCount}{" "}
                {item.runCount === 1 ? "run" : "runs"} · {item.blockedCount} blocked
              </span>
            </div>
          ))}
        </section>

        <section className="hc-rail-panel hc-budget-dashboard" aria-label="Budget dashboard">
          <header>
            <span>Budget Dashboard</span>
            <Activity size={14} />
          </header>
          {budgetDashboardDegraded ? (
            <div className="hc-budget-dashboard-degraded">
              <AlertTriangle size={15} />
              <span>Budget dashboard degraded · db {controlPlaneSnapshot.health.db}</span>
            </div>
          ) : null}
          <div className="hc-budget-dashboard-summary">
            <span>{formatBudgetDashboardRow(controlPlaneSnapshot.budgets.workspace, "Workspace")}</span>
            <span>{formatProviderPriceTable(controlPlaneSnapshot.budgets.price_table)}</span>
          </div>
          <div className="hc-budget-dashboard-actions">
            <button type="button" onClick={handleLoadBudgetSummary} aria-label="Load budget summary">
              Load summary
            </button>
            <button type="button" onClick={handleRefreshBudgetForecast} aria-label="Refresh budget forecast">
              Refresh forecast
            </button>
            <button type="button" onClick={handleLoadProviderPrices} aria-label="Load provider prices">
              Load prices
            </button>
            <input
              aria-label="Provider price update source"
              value={providerPriceUpdateSource}
              onChange={(event) => setProviderPriceUpdateSource(event.currentTarget.value)}
              placeholder="local-fixture"
            />
            <textarea
              aria-label="Provider price update payload"
              value={providerPriceUpdatePayload}
              onChange={(event) => setProviderPriceUpdatePayload(event.currentTarget.value)}
              placeholder='[{"provider":"openai","model":"gpt-5.4","inputUsdPerMillion":5,"outputUsdPerMillion":15}]'
            />
            <button type="button" onClick={handleUpdateProviderPrices} aria-label="Update provider prices">
              Update prices
            </button>
          </div>
          <div className="hc-budget-dashboard-actions">
            <select
              aria-label="Budget scope type"
              value={budgetScopeType}
              onChange={(event) => setBudgetScopeType(event.currentTarget.value as BudgetScopeType)}
            >
              <option value="workspace">workspace</option>
              <option value="project">project</option>
              <option value="goal">goal</option>
              <option value="task">task</option>
              <option value="run">run</option>
              <option value="agent">agent</option>
            </select>
            <input
              aria-label="Budget scope id"
              value={budgetScopeId}
              onChange={(event) => setBudgetScopeId(event.currentTarget.value)}
              placeholder={budgetScopeType === "workspace" ? "workspace" : localProjectId}
              disabled={budgetScopeType === "workspace"}
            />
            <input
              aria-label="Budget max USD"
              value={budgetMaxUsd}
              onChange={(event) => setBudgetMaxUsd(event.currentTarget.value)}
              inputMode="decimal"
              placeholder="20"
            />
            <input
              aria-label="Budget warning percent"
              value={budgetWarnPct}
              onChange={(event) => setBudgetWarnPct(event.currentTarget.value)}
              inputMode="decimal"
              placeholder="0.8"
            />
            <label className="hc-tracker-boundary-toggle">
              <input
                type="checkbox"
                aria-label="Budget hard limit"
                checked={budgetHardLimit}
                onChange={(event) => setBudgetHardLimit(event.currentTarget.checked)}
              />
              <span>Hard limit</span>
            </label>
            <button type="button" onClick={handleSetBudget} aria-label="Set budget">
              Set budget
            </button>
          </div>
          {budgetForecastResult ? (
            <div className="hc-budget-dashboard-status">
              Forecast refreshed ·{" "}
              {budgetForecastResult.projects.length +
                (budgetForecastResult.goals?.length ?? 0) +
                (budgetForecastResult.tasks?.length ?? 0) +
                (budgetForecastResult.runs?.length ?? 0) +
                budgetForecastResult.agents.length}{" "}
              scoped budgets
            </div>
          ) : null}
          {budgetSummaryResult ? (
            <div className="hc-budget-dashboard-status">
              Budget summary loaded · {countBudgetSummaryScopes(budgetSummaryResult)} scoped budgets
            </div>
          ) : null}
          {providerPriceUpdateResult ? (
            <div className="hc-budget-dashboard-status">
              Updated provider prices · {providerPriceUpdateResult.updated} {providerPriceUpdateResult.updated === 1 ? "model" : "models"}
            </div>
          ) : null}
          {budgetSetResult ? (
            <div className="hc-budget-dashboard-status">
              Saved budget {formatNativeBudgetResult(budgetSetResult)}
            </div>
          ) : null}
          {budgetWorkflowError ? (
            <div className="hc-budget-dashboard-error">{budgetWorkflowError}</div>
          ) : null}
          <div className="hc-budget-dashboard-group">
            <strong>Provider Prices {providerPrices.length}</strong>
            {providerPrices.length === 0 ? (
              <span>No provider prices loaded</span>
            ) : (
              providerPrices.slice(0, 4).map((price) => (
                <span className="hc-provider-price-row" key={`${price.provider}-${price.model}-${price.source}`}>
                  <strong>{formatProviderPriceRow(price)}</strong>
                  <small>{formatProviderPriceMetadata(price)}</small>
                </span>
              ))
            )}
          </div>
          <div className="hc-budget-dashboard-group">
            <strong>Projects {controlPlaneSnapshot.budgets.projects.length}</strong>
            {controlPlaneSnapshot.budgets.projects.length === 0 ? (
              <span>No project budgets</span>
            ) : (
              controlPlaneSnapshot.budgets.projects.slice(0, 4).map((budget) => (
                <span key={`project-budget-${budget.scope_id ?? budget.id ?? "workspace"}`}>
                  {formatBudgetDashboardRow(budget, budget.scope_id ?? "project")}
                </span>
              ))
            )}
          </div>
          <div className="hc-budget-dashboard-group">
            <strong>Goals {controlPlaneSnapshot.budgets.goals?.length ?? 0}</strong>
            {!controlPlaneSnapshot.budgets.goals?.length ? (
              <span>No goal budgets</span>
            ) : (
              controlPlaneSnapshot.budgets.goals.slice(0, 4).map((budget) => (
                <span key={`goal-budget-${budget.scope_id ?? budget.id ?? "goal"}`}>
                  {formatBudgetDashboardRow(budget, budget.scope_id ?? "goal")}
                </span>
              ))
            )}
          </div>
          <div className="hc-budget-dashboard-group">
            <strong>Tasks {controlPlaneSnapshot.budgets.tasks?.length ?? 0}</strong>
            {!controlPlaneSnapshot.budgets.tasks?.length ? (
              <span>No task budgets</span>
            ) : (
              controlPlaneSnapshot.budgets.tasks.slice(0, 4).map((budget) => (
                <span key={`task-budget-${budget.scope_id ?? budget.id ?? "task"}`}>
                  {formatBudgetDashboardRow(budget, budget.scope_id ?? "task")}
                </span>
              ))
            )}
          </div>
          <div className="hc-budget-dashboard-group">
            <strong>Runs {controlPlaneSnapshot.budgets.runs?.length ?? 0}</strong>
            {!controlPlaneSnapshot.budgets.runs?.length ? (
              <span>No run budgets</span>
            ) : (
              controlPlaneSnapshot.budgets.runs.slice(0, 4).map((budget) => (
                <span key={`run-budget-${budget.scope_id ?? budget.id ?? "run"}`}>
                  {formatBudgetDashboardRow(budget, budget.scope_id ?? "run")}
                </span>
              ))
            )}
          </div>
          <div className="hc-budget-dashboard-group">
            <strong>Agents {controlPlaneSnapshot.budgets.agents.length}</strong>
            {controlPlaneSnapshot.budgets.agents.length === 0 ? (
              <span>No agent budgets</span>
            ) : (
              controlPlaneSnapshot.budgets.agents.slice(0, 4).map((budget) => (
                <span key={`agent-budget-${budget.scope_id ?? budget.id ?? "agent"}`}>
                  {formatBudgetDashboardRow(budget, budget.scope_id ?? "agent")}
                </span>
              ))
            )}
          </div>
          {controlPlaneSnapshot.budgets.forecasts?.projects?.length ||
          controlPlaneSnapshot.budgets.forecasts?.goals?.length ||
          controlPlaneSnapshot.budgets.forecasts?.tasks?.length ||
          controlPlaneSnapshot.budgets.forecasts?.runs?.length ||
          controlPlaneSnapshot.budgets.forecasts?.agents?.length ? (
            <div className="hc-budget-dashboard-group">
              <strong>Forecasts</strong>
              {[
                ...(controlPlaneSnapshot.budgets.forecasts.projects ?? []),
                ...(controlPlaneSnapshot.budgets.forecasts.goals ?? []),
                ...(controlPlaneSnapshot.budgets.forecasts.tasks ?? []),
                ...(controlPlaneSnapshot.budgets.forecasts.runs ?? []),
                ...(controlPlaneSnapshot.budgets.forecasts.agents ?? []),
              ].slice(0, 4).map((forecast) => (
                <span key={`budget-forecast-${forecast.scope_type ?? "scope"}-${forecast.scope_id ?? "workspace"}`}>
                  {formatBudgetForecastRow(forecast)}
                </span>
              ))}
            </div>
          ) : null}
        </section>

        <section className="hc-rail-panel hc-benchmark-dashboard" aria-label="Benchmark suite dashboard">
          <header>
            <span>Benchmark Suite</span>
            <Activity size={14} />
          </header>
          <div className="hc-benchmark-dashboard-summary">
            <span>{formatBenchmarkSummary(controlPlaneSnapshot)}</span>
            <span>
              Suites {controlPlaneSnapshot.benchmarks?.diagnostics?.suite_count ?? controlPlaneSnapshot.benchmarks?.suites.length ?? 0}
              {" · "}
              Duration {controlPlaneSnapshot.benchmarks?.diagnostics?.duration_ms ?? 0} ms
            </span>
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunBenchmarks} aria-label="Run benchmark suite">
            Run benchmark
          </button>
          <button type="button" className="hc-quality-runner-action" onClick={handleLoadBenchmarkHistory} aria-label="Load benchmark history">
            Load benchmark history
          </button>
          <div className="hc-benchmark-dashboard-group">
            {controlPlaneSnapshot.benchmarks?.suites.length ? (
              controlPlaneSnapshot.benchmarks.suites.slice(0, 4).map((suite) => (
                <span key={`benchmark-suite-${suite.suite_id}`}>
                  {formatBenchmarkSuiteRow(suite)}
                </span>
              ))
            ) : (
              <span>No benchmark suites recorded</span>
            )}
          </div>
          {benchmarkRun ? (
            <div className="hc-quality-runner-result">
              <strong>Run {formatBenchmarkRun(benchmarkRun)}</strong>
            </div>
          ) : null}
          {benchmarkHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {benchmarkHistory.slice(0, 3).map((run) => (
                <span key={`benchmark-history-${run.id}`}>History {formatBenchmarkRun(run)}</span>
              ))}
            </div>
          ) : null}
          {benchmarkError ? <div className="hc-quality-runner-error">{benchmarkError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-quality-runner" aria-label="Dogfood telemetry review">
          <header>
            <span>Dogfood Telemetry</span>
            <Activity size={14} />
          </header>
          <div className="hc-quality-runner-summary">
            <span>
              {controlPlaneSnapshot.dogfood?.last_review_id ?? "No dogfood review"} · {controlPlaneSnapshot.dogfood?.last_status ?? "not_run"}
              {" · "}
              {controlPlaneSnapshot.dogfood?.last_pass_count ?? 0} pass · {controlPlaneSnapshot.dogfood?.last_warning_count ?? 0} warning · {controlPlaneSnapshot.dogfood?.last_fail_count ?? 0} fail
            </span>
            {controlPlaneSnapshot.dogfood?.last_evidence_pack_id ? (
              <span>Evidence {controlPlaneSnapshot.dogfood.last_evidence_pack_id}</span>
            ) : null}
          </div>
          <button type="button" className="hc-quality-runner-action" onClick={handleRunDogfoodTelemetryReview} aria-label="Run dogfood telemetry review">
            Run telemetry review
          </button>
          <button type="button" className="hc-quality-runner-action" onClick={handleLoadDogfoodTelemetryHistory} aria-label="Load dogfood telemetry history">
            Load telemetry history
          </button>
          {dogfoodReview ? (
            <div className="hc-quality-runner-result">
              <strong>Run {formatDogfoodReview(dogfoodReview)}</strong>
              <span>Evidence pack {dogfoodReview.evidence_pack_id}</span>
              {dogfoodReview.findings.slice(0, 3).map((finding) => (
                <span key={`dogfood-finding-${finding.finding_id}`}>{formatDogfoodFinding(finding)}</span>
              ))}
            </div>
          ) : null}
          {dogfoodReviewHistory.length > 0 ? (
            <div className="hc-quality-runner-result">
              {dogfoodReviewHistory.slice(0, 3).map((review) => (
                <span key={`dogfood-history-${review.id}`}>History {formatDogfoodReview(review)}</span>
              ))}
            </div>
          ) : null}
          {dogfoodReviewError ? <div className="hc-quality-runner-error">{dogfoodReviewError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-visual-harness" aria-label="Visual harness canvas">
          <header>
            <span>Visual Harness</span>
            <LayoutGrid size={14} />
          </header>
          <div className="hc-visual-harness-summary">
            <span>Nodes {controlPlaneSnapshot.visual_harness?.diagnostics?.node_count ?? controlPlaneSnapshot.visual_harness?.nodes.length ?? 0}</span>
            <span>Edges {controlPlaneSnapshot.visual_harness?.diagnostics?.edge_count ?? controlPlaneSnapshot.visual_harness?.edges.length ?? 0}</span>
          </div>
          <VisualHarnessCanvas
            graph={controlPlaneSnapshot.visual_harness}
            onConnectNodes={handleCreateVisualHarnessLinkFromCanvas}
          />
          <div className="hc-visual-harness-link-form">
            <input
              aria-label="Visual harness source"
              value={visualHarnessSource}
              onChange={(event) => setVisualHarnessSource(event.currentTarget.value)}
              placeholder="ctx_default"
            />
            <input
              aria-label="Visual harness target"
              value={visualHarnessTarget}
              onChange={(event) => setVisualHarnessTarget(event.currentTarget.value)}
              placeholder="task_1"
            />
            <select
              aria-label="Visual harness link kind"
              value={visualHarnessKind}
              onChange={(event) => setVisualHarnessKind(event.currentTarget.value)}
            >
              <option value="context">context</option>
              <option value="tool">tool</option>
              <option value="task">task</option>
              <option value="workflow">workflow</option>
              <option value="dependency">dependency</option>
            </select>
            <button type="button" onClick={handleCreateVisualHarnessLink} aria-label="Create visual harness link">
              Create link
            </button>
            <button type="button" onClick={handleLoadVisualHarnessLinks} aria-label="Load visual harness links">
              Load links
            </button>
          </div>
          {visualHarnessLink ? (
            <div className="hc-quality-runner-result">
              <strong>Created visual link {visualHarnessLink.id}</strong>
              <span>Manual edge {visualHarnessLink.source_id} -&gt; {visualHarnessLink.target_id} · {visualHarnessLink.kind}</span>
            </div>
          ) : null}
          {visualHarnessError ? <div className="hc-quality-runner-error">{visualHarnessError}</div> : null}
          <div className="hc-visual-harness-grid">
            <div>
              <strong>Nodes</strong>
              {(controlPlaneSnapshot.visual_harness?.nodes ?? []).slice(0, 5).map((node) => (
                <span key={`visual-node-${node.kind}-${node.id}`}>
                  {node.label} · {node.kind} · {node.status}
                </span>
              ))}
              {controlPlaneSnapshot.visual_harness?.nodes.length ? null : <span>No graph nodes</span>}
            </div>
            <div>
              <strong>Edges</strong>
              {(controlPlaneSnapshot.visual_harness?.edges ?? []).slice(0, 5).map((edge) => (
                <span key={`visual-edge-${edge.id}`}>
                  {edge.source_id} -&gt; {edge.target_id} · {edge.kind}
                </span>
              ))}
              {controlPlaneSnapshot.visual_harness?.edges.length ? null : <span>No graph edges</span>}
            </div>
          </div>
        </section>

        <section className="hc-rail-panel hc-token-usage-adapters" aria-label="Token usage ingestion adapters">
          <header>
            <span>Token Usage Adapters</span>
            <Activity size={14} />
          </header>
          <div className="hc-token-usage-adapter-form">
            <select
              aria-label="Token usage adapter"
              value={tokenUsageAdapter}
              onChange={(event) => setTokenUsageAdapter(event.currentTarget.value)}
            >
              <option value="openai.responses">openai.responses</option>
              <option value="codex.log">codex.log</option>
              <option value="local.usage-json">local.usage-json</option>
            </select>
            <input
              aria-label="Token usage adapter agent"
              value={tokenUsageAdapterAgent}
              onChange={(event) => setTokenUsageAdapterAgent(event.currentTarget.value)}
              placeholder="agent_codex"
            />
            <textarea
              aria-label="Token usage adapter payload"
              value={tokenUsageAdapterPayload}
              onChange={(event) => setTokenUsageAdapterPayload(event.currentTarget.value)}
              placeholder='{"model":"gpt-5.4","usage":{"input_tokens":1200,"output_tokens":800},"cost_usd":8.5}'
            />
            <button type="button" onClick={handleIngestTokenUsageAdapter} aria-label="Ingest token usage adapter">
              Ingest usage
            </button>
          </div>
          {tokenUsageAdapterResult ? (
            <div className="hc-token-usage-adapter-result">
              <strong>Ingested {tokenUsageAdapter} · {tokenUsageAdapterResult.model}</strong>
              <span>
                {tokenUsageAdapterResult.input_tokens + tokenUsageAdapterResult.output_tokens} tokens · {formatUsd(tokenUsageAdapterResult.cost_usd)}
              </span>
              <small>{tokenUsageAdapterResult.source}</small>
            </div>
          ) : (
            <div className="hc-review-row">
              <Activity size={15} />
              <span>Normalize CLI logs structured adapter payloads and local usage files</span>
            </div>
          )}
          <div className="hc-token-usage-adapter-form">
            <input
              aria-label="Manual token usage provider"
              value={manualTokenUsageProvider}
              onChange={(event) => setManualTokenUsageProvider(event.currentTarget.value)}
              placeholder="openai"
            />
            <input
              aria-label="Manual token usage model"
              value={manualTokenUsageModel}
              onChange={(event) => setManualTokenUsageModel(event.currentTarget.value)}
              placeholder="gpt-5.4"
            />
            <input
              aria-label="Manual token usage agent"
              value={manualTokenUsageAgent}
              onChange={(event) => setManualTokenUsageAgent(event.currentTarget.value)}
              placeholder="agent_codex"
            />
            <input
              aria-label="Manual token usage session"
              value={manualTokenUsageSession}
              onChange={(event) => setManualTokenUsageSession(event.currentTarget.value)}
              placeholder="session_1"
            />
            <input
              aria-label="Manual token usage task"
              value={manualTokenUsageTask}
              onChange={(event) => setManualTokenUsageTask(event.currentTarget.value)}
              placeholder="task_1"
            />
            <input
              aria-label="Manual token usage run"
              value={manualTokenUsageRun}
              onChange={(event) => setManualTokenUsageRun(event.currentTarget.value)}
              placeholder="run_1"
            />
            <input
              aria-label="Manual token usage input tokens"
              value={manualTokenUsageInputTokens}
              onChange={(event) => setManualTokenUsageInputTokens(event.currentTarget.value)}
              inputMode="numeric"
              placeholder="1200"
            />
            <input
              aria-label="Manual token usage output tokens"
              value={manualTokenUsageOutputTokens}
              onChange={(event) => setManualTokenUsageOutputTokens(event.currentTarget.value)}
              inputMode="numeric"
              placeholder="800"
            />
            <input
              aria-label="Manual token usage cost USD"
              value={manualTokenUsageCostUsd}
              onChange={(event) => setManualTokenUsageCostUsd(event.currentTarget.value)}
              inputMode="decimal"
              placeholder="8.5"
            />
            <input
              aria-label="Manual token usage source"
              value={manualTokenUsageSource}
              onChange={(event) => setManualTokenUsageSource(event.currentTarget.value)}
              placeholder="manual"
            />
            <button type="button" onClick={handleRecordManualTokenUsage} aria-label="Record manual token usage">
              Record manual token usage
            </button>
          </div>
          {tokenUsageRecordResult ? (
            <div className="hc-token-usage-adapter-result">
              <strong>Recorded manual usage · {tokenUsageRecordResult.model}</strong>
              <span>
                {tokenUsageRecordResult.input_tokens + tokenUsageRecordResult.output_tokens} tokens · {formatUsd(tokenUsageRecordResult.cost_usd)}
              </span>
              <small>{tokenUsageRecordResult.source}</small>
            </div>
          ) : null}
          {tokenUsageRecordError ? (
            <div className="hc-token-usage-adapter-error">{tokenUsageRecordError}</div>
          ) : null}
          {tokenUsageAdapterError ? (
            <div className="hc-token-usage-adapter-error">{tokenUsageAdapterError}</div>
          ) : null}
        </section>

        <section className="hc-rail-panel hc-secret-storage" aria-label="Keychain secret storage">
          <header>
            <span>Keychain Secret Storage</span>
            <KeyRound size={14} />
          </header>
          <div className="hc-secret-storage-summary">
            <span>Keychain {securityState.keychain} · {securityState.secret_count} {securityState.secret_count === 1 ? "secret" : "secrets"}</span>
            <span>Redaction {redactionStatus} · {protectedSecretCount} protected {protectedSecretCount === 1 ? "value" : "values"}</span>
          </div>
          <div className="hc-secret-storage-form">
            <input
              aria-label="Secret name"
              value={secretName}
              onChange={(event) => setSecretName(event.currentTarget.value)}
              placeholder="OPENAI_API_KEY"
            />
            <input
              aria-label="Secret value"
              value={secretValue}
              onChange={(event) => setSecretValue(event.currentTarget.value)}
              placeholder="Paste secret"
              type="password"
            />
            <button type="button" onClick={handleSaveSecret} aria-label="Save Keychain secret">
              Save secret
            </button>
            <button type="button" onClick={handleLoadSecrets} aria-label="Load Keychain secrets">
              Load metadata
            </button>
          </div>
          {secretResult ? (
            <div className="hc-secret-storage-result">
              <strong>Saved {secretResult.name} · {secretResult.redacted ? "redacted" : "stored"}</strong>
              <small>{secretResult.keychain_ref}</small>
            </div>
          ) : (
            <div className="hc-review-row">
              <ShieldAlert size={15} />
              <span>Secrets are stored by reference and redacted from state snapshots</span>
            </div>
          )}
          {secretInventory.length > 0 ? (
            <div className="hc-secret-storage-inventory" aria-label="Loaded Keychain secrets">
              {secretInventory.map((secret) => (
                <span key={secret.id}>
                  {secret.name} · {secret.keychain_ref} · {secret.redacted ? "redacted" : "stored"}
                </span>
              ))}
            </div>
          ) : null}
          {secretError ? (
            <div className="hc-secret-storage-error">{secretError}</div>
          ) : null}
        </section>

        {hasKnowledgeWarnings ? (
          <section className="hc-rail-panel hc-knowledge-warning" aria-label="Knowledge freshness warnings">
            <header>
              <span>Knowledge context needs attention</span>
              <AlertTriangle size={14} />
            </header>
            <div className="hc-knowledge-warning-grid">
              <strong>{controlPlaneSnapshot.knowledge.stale_count} stale sources</strong>
              <strong>{controlPlaneSnapshot.knowledge.gap_count} coverage gaps</strong>
              <span>Recent page {recentKnowledgePage}</span>
            </div>
            <p>Re-index stale sources or update the context pack before dispatch</p>
          </section>
        ) : null}

        <section className="hc-rail-panel hc-knowledge-source-index" aria-label="Knowledge source index">
          <header>
            <span>Knowledge Source Index</span>
            <FileText size={14} />
          </header>
          <label className="hc-knowledge-source-form">
            <input
              ref={knowledgeSourceInputRef}
              aria-label="Knowledge source path"
              value={knowledgeSourcePath}
              onChange={(event) => setKnowledgeSourcePath(event.currentTarget.value)}
              placeholder="docs/overview.md"
            />
            <button type="button" onClick={handleIndexKnowledgeSource} aria-label="Index knowledge source">
              <Plus size={13} />
            </button>
          </label>
          {knowledgeSourceStatus ? (
            <div className="hc-knowledge-source-status">{knowledgeSourceStatus}</div>
          ) : null}
          {knowledgeSourceError ? (
            <div className="hc-knowledge-source-error">{knowledgeSourceError}</div>
          ) : null}
          <div className="hc-knowledge-source-list">
            {knowledgeSources.length === 0 ? (
              <div className="hc-review-row">
                <FileText size={15} />
                <span>No knowledge sources indexed</span>
              </div>
            ) : (
              knowledgeSources.slice(0, 5).map((source) => (
                <article className="hc-knowledge-source-row" key={source.id}>
                  <strong>{source.path_or_ref}</strong>
                  <span>{source.kind} · {source.status}</span>
                  <small>{source.fingerprint}</small>
                </article>
              ))
            )}
          </div>
        </section>

        <section className="hc-rail-panel hc-knowledge-pages" aria-label="Markdown knowledge pages">
          <header>
            <span>Markdown Knowledge Pages</span>
            <FileText size={14} />
          </header>
          <div className="hc-knowledge-page-editor">
            <input
              aria-label="Knowledge page slug"
              value={knowledgePageSlug}
              onChange={(event) => setKnowledgePageSlug(event.currentTarget.value)}
              placeholder="auth-flow"
            />
            <input
              aria-label="Knowledge page title"
              value={knowledgePageTitle}
              onChange={(event) => setKnowledgePageTitle(event.currentTarget.value)}
              placeholder="Auth Flow"
            />
            <textarea
              aria-label="Knowledge page markdown"
              value={knowledgePageBody}
              onChange={(event) => setKnowledgePageBody(event.currentTarget.value)}
              placeholder="# Notes"
            />
            <button type="button" onClick={handleSaveKnowledgePage} aria-label="Save markdown knowledge page">
              Save page
            </button>
          </div>
          {knowledgePageStatus ? (
            <div className="hc-knowledge-page-status">{knowledgePageStatus}</div>
          ) : null}
          {knowledgePageError ? (
            <div className="hc-knowledge-page-error">{knowledgePageError}</div>
          ) : null}
          <div className="hc-knowledge-page-list">
            {knowledgePages.length === 0 ? (
              <div className="hc-review-row">
                <FileText size={15} />
                <span>No markdown knowledge pages</span>
              </div>
            ) : (
              knowledgePages.slice(0, 4).map((page) => (
                <article className="hc-knowledge-page-row" key={page.id}>
                  <header>
                    <strong>{page.title}</strong>
                    <small>{page.slug} · {page.freshness_state}</small>
                  </header>
                  <div className="hc-knowledge-page-preview">{renderMarkdownPreview(page.body_md)}</div>
                </article>
              ))
            )}
          </div>
        </section>

        <section className="hc-rail-panel hc-knowledge-concepts" aria-label="Knowledge concepts">
          <header>
            <span>Concepts</span>
            <GitBranch size={14} />
          </header>
          <button
            type="button"
            className="hc-knowledge-export-button"
            onClick={handleExportKnowledgeObsidianMarkdown}
            aria-label="Export Obsidian markdown"
          >
            Export Obsidian markdown
          </button>
          {knowledgeObsidianExport ? (
            <div className="hc-knowledge-export-result">
              <strong>{knowledgeObsidianExport.status} · {knowledgeObsidianExport.file_count} files</strong>
              <span>{knowledgeObsidianExport.export_root}</span>
              <small>{knowledgeObsidianExport.files.slice(0, 4).join(", ")}</small>
            </div>
          ) : null}
          {knowledgeConceptError ? (
            <div className="hc-knowledge-concept-error">{knowledgeConceptError}</div>
          ) : null}
          {knowledgeObsidianExportError ? (
            <div className="hc-knowledge-concept-error">{knowledgeObsidianExportError}</div>
          ) : null}
          <div className="hc-knowledge-concept-list">
            {knowledgeConcepts.length === 0 ? (
              <div className="hc-review-row">
                <GitBranch size={15} />
                <span>No concept links indexed</span>
              </div>
            ) : (
              knowledgeConcepts.slice(0, 6).map((concept) => (
                <article className="hc-knowledge-concept-row" key={concept.slug}>
                  <header>
                    <strong>{concept.title}</strong>
                    <small>{concept.slug} · {concept.page_id ?? "unresolved page"}</small>
                  </header>
                  <span>Links {concept.outbound_slugs.join(", ") || "none"}</span>
                  <span>Backlinks {concept.inbound_page_ids.join(", ") || "none"}</span>
                </article>
              ))
            )}
          </div>
        </section>

        <section className="hc-rail-panel hc-knowledge-chat" aria-label="Knowledge chat">
          <header>
            <span>Knowledge Chat</span>
            <Search size={14} />
          </header>
          <div className="hc-knowledge-chat-form">
            <textarea
              aria-label="Knowledge question"
              value={knowledgeQuestion}
              onChange={(event) => setKnowledgeQuestion(event.currentTarget.value)}
              placeholder="Ask a local project knowledge question"
            />
            <select
              aria-label="Knowledge chat context pack"
              value={knowledgeChatContextPackId}
              onChange={(event) => setKnowledgeChatContextPackId(event.currentTarget.value)}
            >
              <option value="">All knowledge</option>
              {contextPacks.map((pack) => (
                <option value={pack.id} key={pack.id}>
                  {pack.name} ({pack.id})
                </option>
              ))}
            </select>
            <button type="button" onClick={handleAnswerKnowledgeQuestion} aria-label="Ask Knowledge Vault">
              Ask Knowledge Vault
            </button>
          </div>
          {knowledgeChatAnswer ? (
            <div className="hc-knowledge-chat-result">
              <strong>{knowledgeChatAnswer.source_count} local citations</strong>
              <span>{knowledgeChatAnswer.cited_page_ids.join(", ") || "no citations"}</span>
              <div className="hc-knowledge-chat-preview">{renderMarkdownPreview(knowledgeChatAnswer.answer_md)}</div>
            </div>
          ) : (
            <div className="hc-review-row">
              <Search size={15} />
              <span>Ask against local knowledge pages and saved context</span>
            </div>
          )}
          {knowledgeChatError ? (
            <div className="hc-knowledge-chat-error">{knowledgeChatError}</div>
          ) : null}
        </section>

        <section className="hc-rail-panel hc-knowledge-explorations" aria-label="Saved knowledge explorations">
          <header>
            <span>Saved Explorations</span>
            <Search size={14} />
          </header>
          <div className="hc-knowledge-exploration-editor">
            <input
              aria-label="Knowledge exploration title"
              value={knowledgeExplorationTitle}
              onChange={(event) => setKnowledgeExplorationTitle(event.currentTarget.value)}
              placeholder="Deploy rollback answer"
            />
            <input
              aria-label="Knowledge exploration question"
              value={knowledgeExplorationQuestion}
              onChange={(event) => setKnowledgeExplorationQuestion(event.currentTarget.value)}
              placeholder="Which context should release checks use?"
            />
            <textarea
              aria-label="Knowledge exploration answer"
              value={knowledgeExplorationAnswer}
              onChange={(event) => setKnowledgeExplorationAnswer(event.currentTarget.value)}
              placeholder="Saved answer markdown"
            />
            <select
              aria-label="Knowledge exploration page"
              value={knowledgeExplorationPageId}
              onChange={(event) => setKnowledgeExplorationPageId(event.currentTarget.value)}
            >
              <option value="">No cited page</option>
              {knowledgePages.map((page) => (
                <option value={page.id} key={page.id}>
                  {page.title} ({page.id})
                </option>
              ))}
            </select>
            <select
              aria-label="Knowledge exploration context pack"
              value={knowledgeExplorationContextPackId}
              onChange={(event) => setKnowledgeExplorationContextPackId(event.currentTarget.value)}
            >
              <option value="">No context pack</option>
              {contextPacks.map((pack) => (
                <option value={pack.id} key={pack.id}>
                  {pack.name} ({pack.id})
                </option>
              ))}
            </select>
            <button type="button" onClick={handleSaveKnowledgeExploration} aria-label="Save knowledge exploration">
              Save exploration
            </button>
          </div>
          {knowledgeExplorationStatus ? (
            <div className="hc-knowledge-exploration-status">{knowledgeExplorationStatus}</div>
          ) : null}
          {knowledgeExplorationError ? (
            <div className="hc-knowledge-exploration-error">{knowledgeExplorationError}</div>
          ) : null}
          <div className="hc-knowledge-exploration-list">
            {knowledgeExplorations.length === 0 ? (
              <div className="hc-review-row">
                <Search size={15} />
                <span>No saved explorations</span>
              </div>
            ) : (
              knowledgeExplorations.slice(0, 4).map((exploration) => (
                <article className="hc-knowledge-exploration-row" key={exploration.id}>
                  <header>
                    <strong>{exploration.title}</strong>
                    <small>
                      {exploration.context_pack_id ?? "no context pack"} · {exploration.page_ids.join(", ") || "no pages"}
                    </small>
                  </header>
                  <span>{exploration.question}</span>
                  <div className="hc-knowledge-exploration-preview">{renderMarkdownPreview(exploration.answer_md)}</div>
                </article>
              ))
            )}
          </div>
        </section>

        <section className="hc-rail-panel hc-knowledge-automation" aria-label="Knowledge automation">
          <header>
            <span>Knowledge Automation</span>
            <RotateCcw size={14} />
          </header>
          <label className="hc-knowledge-automation-toggle">
            <input
              type="checkbox"
              aria-label="Watch knowledge changes"
              checked={knowledgeAutomationWatch}
              onChange={(event) => setKnowledgeAutomationWatch(event.currentTarget.checked)}
            />
            <span>Watch knowledge changes</span>
          </label>
          <button type="button" onClick={handleRunKnowledgeAutomation} aria-label="Compile knowledge vault">
            Compile knowledge vault
          </button>
          <div className="hc-knowledge-lint-form">
            <input
              aria-label="Knowledge lint stale count"
              value={knowledgeLintStaleCount}
              onChange={(event) => setKnowledgeLintStaleCount(event.currentTarget.value)}
              inputMode="numeric"
            />
            <input
              aria-label="Knowledge lint gap count"
              value={knowledgeLintGapCount}
              onChange={(event) => setKnowledgeLintGapCount(event.currentTarget.value)}
              inputMode="numeric"
            />
            <input
              aria-label="Knowledge lint contradiction count"
              value={knowledgeLintContradictionCount}
              onChange={(event) => setKnowledgeLintContradictionCount(event.currentTarget.value)}
              inputMode="numeric"
            />
            <textarea
              aria-label="Knowledge lint body"
              value={knowledgeLintBody}
              onChange={(event) => setKnowledgeLintBody(event.currentTarget.value)}
              placeholder="Gap: missing rollback evidence"
            />
            <button type="button" onClick={handleRecordKnowledgeLintReport} aria-label="Record knowledge lint report">
              Record lint
            </button>
          </div>
          {knowledgeAutomationRun ? (
            <div className="hc-knowledge-automation-result">
              <strong>{knowledgeAutomationRun.status} · watch {knowledgeAutomationRun.watch_enabled ? "on" : "off"}</strong>
              <span>{knowledgeAutomationRun.source_count} sources · {knowledgeAutomationRun.page_count} pages</span>
              <span>{knowledgeAutomationRun.stale_count} stale · {knowledgeAutomationRun.gap_count} gaps</span>
              <small>lint {knowledgeAutomationRun.lint_report_id}</small>
            </div>
          ) : (
            <div className="hc-review-row">
              <RotateCcw size={15} />
              <span>Compile source index pages and lint into a local report</span>
            </div>
          )}
          {knowledgeLintReport ? (
            <div className="hc-knowledge-automation-result">
              <strong>{formatKnowledgeLintReport(knowledgeLintReport)}</strong>
              <small>{knowledgeLintReport.artifact_path}</small>
            </div>
          ) : null}
          {knowledgeLintError ? (
            <div className="hc-knowledge-automation-error">{knowledgeLintError}</div>
          ) : null}
          {knowledgeAutomationError ? (
            <div className="hc-knowledge-automation-error">{knowledgeAutomationError}</div>
          ) : null}
        </section>

        <section className="hc-rail-panel hc-knowledge-ingestion" aria-label="Long document and multimodal ingestion">
          <header>
            <span>Long & Multimodal Ingestion</span>
            <FileText size={14} />
          </header>
          <div className="hc-knowledge-ingestion-form">
            <input
              aria-label="Ingestion artifact path"
              value={knowledgeIngestionPath}
              onChange={(event) => setKnowledgeIngestionPath(event.currentTarget.value)}
              placeholder="docs/runbook.pdf"
            />
            <select
              aria-label="Ingestion artifact kind"
              value={knowledgeIngestionKind}
              onChange={(event) => setKnowledgeIngestionKind(event.currentTarget.value)}
            >
              {["markdown", "pdf", "image", "html", "log", "json", "yaml"].map((kind) => (
                <option value={kind} key={kind}>
                  {kind}
                </option>
              ))}
            </select>
            <input
              aria-label="Ingestion artifact title"
              value={knowledgeIngestionTitle}
              onChange={(event) => setKnowledgeIngestionTitle(event.currentTarget.value)}
              placeholder="Release Runbook"
            />
            <textarea
              aria-label="Ingestion artifact body"
              value={knowledgeIngestionBody}
              onChange={(event) => setKnowledgeIngestionBody(event.currentTarget.value)}
              placeholder="Paste extracted text or multimodal notes"
            />
            <button type="button" onClick={handleIngestKnowledgeArtifact} aria-label="Ingest knowledge artifact">
              Ingest artifact
            </button>
          </div>
          {knowledgeIngestionResult ? (
            <div className="hc-knowledge-ingestion-result">
              <strong>Ingested {knowledgeIngestionLastPath}</strong>
              <span>{knowledgeIngestionResult.modality} · {knowledgeIngestionResult.chunk_count} chunks</span>
              <small>page {knowledgeIngestionResult.page_id} · source {knowledgeIngestionResult.source_id}</small>
            </div>
          ) : (
            <div className="hc-review-row">
              <FileText size={15} />
              <span>Index extracted long documents and visual notes into knowledge pages</span>
            </div>
          )}
          {knowledgeIngestionError ? (
            <div className="hc-knowledge-ingestion-error">{knowledgeIngestionError}</div>
          ) : null}
        </section>

        <section className="hc-rail-panel hc-context-pack-builder" aria-label="Context pack builder">
          <header>
            <span>Context Pack Builder</span>
            <Blocks size={14} />
          </header>
          <div className="hc-context-pack-form">
            <input
              aria-label="Context pack name"
              value={contextPackName}
              onChange={(event) => setContextPackName(event.currentTarget.value)}
              placeholder="run-default"
            />
            <input
              aria-label="Context pack description"
              value={contextPackDescription}
              onChange={(event) => setContextPackDescription(event.currentTarget.value)}
              placeholder="Run docs"
            />
            <select
              aria-label="Context pack source"
              value={contextPackSourceId}
              onChange={(event) => setContextPackSourceId(event.currentTarget.value)}
            >
              <option value="">Select knowledge page</option>
              {knowledgePages.map((page) => (
                <option value={page.id} key={page.id}>
                  {page.title} ({page.id})
                </option>
              ))}
            </select>
            <input
              aria-label="Context pack max tokens"
              value={contextPackMaxTokens}
              onChange={(event) => setContextPackMaxTokens(event.currentTarget.value)}
              inputMode="numeric"
              placeholder="24000"
            />
            <button type="button" onClick={handleSaveContextPack} aria-label="Save context pack">
              Save pack
            </button>
          </div>
          {contextPackStatus ? (
            <div className="hc-context-pack-status">{contextPackStatus}</div>
          ) : null}
          {contextPackError ? (
            <div className="hc-context-pack-error">{contextPackError}</div>
          ) : null}
          <div className="hc-context-pack-list">
            {contextPacks.length === 0 ? (
              <div className="hc-review-row">
                <Blocks size={15} />
                <span>No context packs</span>
              </div>
            ) : (
              contextPacks.slice(0, 4).map((pack) => (
                <article className="hc-context-pack-row" key={pack.id}>
                  <header>
                    <strong>{pack.name}</strong>
                    <small>{pack.id} · {contextPackBudgetLabel(pack)}</small>
                  </header>
                  {pack.description ? <p>{pack.description}</p> : null}
                  <span>
                    {contextPackSources(pack)
                      .map((source) => source.id ?? source.path ?? source.type ?? "source")
                      .join(", ") || "no sources"}
                  </span>
                </article>
              ))
            )}
          </div>
        </section>

        <section className="hc-rail-panel" aria-label="Workflow Contract">
          <header>
            <span>Workflow Contract</span>
            <FileText size={14} />
          </header>
          <div className={`hc-workflow-state ${controlPlaneSnapshot.workflow.valid ? "valid" : "invalid"}`}>
            {workflowStatusLine}
          </div>
          <div className="hc-workflow-diagnostics">
            {workflowDiagnostics.length === 0 ? (
              <span>No workflow diagnostics</span>
            ) : (
              workflowDiagnostics.slice(0, 3).map((diagnostic) => (
                <span key={`${diagnostic.code}-${diagnostic.message}`}>{diagnostic.message}</span>
              ))
            )}
          </div>
          <div className="hc-project-window-actions">
            <button className="hc-task-drawer-action" type="button" onClick={handleValidateSampleWorkflow} aria-label="Validate sample workflow">
              Validate sample workflow
            </button>
            <button className="hc-task-drawer-action" type="button" onClick={handleRefreshWorkflowRuntime} aria-label="Refresh workflow runtime">
              Refresh workflow runtime
            </button>
          </div>
          {workflowValidationResult ? (
            <div className="hc-workflow-marketplace-status">{formatWorkflowValidationResult(workflowValidationResult)}</div>
          ) : null}
          {workflowRuntimeResult ? (
            <div className="hc-workflow-marketplace-status">{formatWorkflowRuntimeResult(workflowRuntimeResult)}</div>
          ) : null}
          {workflowControlError ? (
            <div className="hc-workflow-marketplace-error">{workflowControlError}</div>
          ) : null}
          <section className="hc-workflow-debugger" aria-label="Visual workflow debugger">
            <div className="hc-workflow-debugger-summary">
              <span>Current workflow {controlPlaneSnapshot.workflow.current_version_id ?? "none"} {controlPlaneSnapshot.workflow.valid ? "valid" : "invalid"}</span>
              <span>Last known good {controlPlaneSnapshot.workflow.last_known_good_version_id ?? "none"}</span>
            </div>
            <div className="hc-workflow-stepper">
              {workflowDebuggerSteps.map((step, index) => {
                const blocked = !controlPlaneSnapshot.workflow.valid && index > 0;
                return (
                  <div className={`hc-workflow-step ${blocked ? "blocked" : "ready"}`} key={step}>
                    <span>{index + 1}</span>
                    <strong>{step}</strong>
                  </div>
                );
              })}
            </div>
            <div className="hc-workflow-debugger-diagnostics">
              {workflowDiagnostics.length === 0 ? (
                <span>No debugger diagnostics</span>
              ) : (
                workflowDiagnostics.slice(0, 3).map((diagnostic) => (
                  <span key={`debug-${diagnostic.code}-${diagnostic.message}`}>
                    {diagnostic.code} · {diagnostic.message}
                  </span>
                ))
              )}
            </div>
          </section>
          <section className="hc-workflow-marketplace" aria-label="Workflow marketplace">
            <header>
              <span>Workflow Marketplace</span>
              <Blocks size={14} />
            </header>
            {workflowMarketplaceStatus ? (
              <div className="hc-workflow-marketplace-status">{workflowMarketplaceStatus}</div>
            ) : null}
            {workflowMarketplaceError ? (
              <div className="hc-workflow-marketplace-error">{workflowMarketplaceError}</div>
            ) : null}
            {workflowMarketplacePresets.map((preset) => (
              <article key={preset.id}>
                <span>
                  <strong>{preset.name}</strong>
                  <small>{preset.description}</small>
                </span>
                <button
                  type="button"
                  onClick={() => handleImportWorkflowPreset(preset)}
                  aria-label={`Import ${preset.name} workflow preset`}
                >
                  Import
                </button>
              </article>
            ))}
          </section>
          <section className="hc-workflow-marketplace" aria-label="Workflow hook runner">
            <header>
              <span>Workflow Hook Runner</span>
              <Activity size={14} />
            </header>
            <div className="hc-tracker-boundary-form">
              <input
                aria-label="Workflow hook run id"
                value={workflowHookRunId}
                onChange={(event) => setWorkflowHookRunId(event.currentTarget.value)}
                placeholder="run_1"
              />
              <input
                aria-label="Workflow hook name"
                value={workflowHookName}
                onChange={(event) => setWorkflowHookName(event.currentTarget.value)}
                placeholder="before_run"
              />
              <input
                aria-label="Workflow hook repo root"
                value={workflowHookRepoRoot}
                onChange={(event) => setWorkflowHookRepoRoot(event.currentTarget.value)}
                placeholder="/repo"
              />
              <input
                aria-label="Workflow hook workspace path"
                value={workflowHookWorkspacePath}
                onChange={(event) => setWorkflowHookWorkspacePath(event.currentTarget.value)}
                placeholder="/repo/.haneulchi/worktrees/run_1"
              />
              <button type="button" onClick={handleRunWorkflowHook} aria-label="Run workflow hook">
                Run workflow hook
              </button>
            </div>
            {workflowHookResult ? (
              <div className="hc-quality-runner-result">
                <strong>{formatWorkflowHookRunResult(workflowHookResult)}</strong>
                <span>{workflowHookResult.workspace_path}</span>
                {workflowHookResult.mirrored_path ? <small>{workflowHookResult.mirrored_path}</small> : null}
              </div>
            ) : null}
          </section>
          <button className="hc-task-drawer-action" type="button" onClick={handleReloadSampleWorkflow} aria-label="Reload sample workflow">
            Reload sample workflow
          </button>
        </section>

        <section className="hc-rail-panel" aria-label="Dashboard widgets">
          <header>
            <span>Dashboard Widgets</span>
            <LayoutGrid size={14} />
          </header>
          <div className="hc-widget-customizer">
            <label>
              <input
                type="checkbox"
                checked={dashboardWidgets.agentTeam}
                onChange={(event) => setDashboardWidgetVisible("agentTeam", event.currentTarget.checked)}
                aria-label="Show Agent Team widget"
              />
              <span>Agent Team {dashboardWidgets.agentTeam ? "visible" : "hidden"}</span>
            </label>
            <label>
              <input
                type="checkbox"
                checked={dashboardWidgets.historicalAnalytics}
                onChange={(event) => setDashboardWidgetVisible("historicalAnalytics", event.currentTarget.checked)}
                aria-label="Show Historical Analytics widget"
              />
              <span>Historical Analytics {dashboardWidgets.historicalAnalytics ? "visible" : "hidden"}</span>
            </label>
            <label>
              <input
                type="checkbox"
                checked={dashboardWidgets.recentEvidence}
                onChange={(event) => setDashboardWidgetVisible("recentEvidence", event.currentTarget.checked)}
                aria-label="Show Recent Evidence widget"
              />
              <span>Recent Evidence {dashboardWidgets.recentEvidence ? "visible" : "hidden"}</span>
            </label>
            {allOptionalDashboardWidgetsHidden ? (
              <div className="hc-widget-empty">
                <AlertTriangle size={15} />
                <span>All optional dashboard widgets hidden</span>
              </div>
            ) : null}
          </div>
        </section>

        {dashboardWidgets.agentTeam ? (
        <section className="hc-rail-panel" aria-label="Agent Team mini dashboard">
          <header>
            <span>Agent Team</span>
            <Terminal size={14} />
          </header>
          <div className="hc-agent-team-summary">
            <span>Available {availableAgentCount}</span>
            <span>Paused {pausedAgentCount}</span>
            <span>Active runs {activeAgentRunCount}</span>
            <span>Blocked {blockedAgentRunCount}</span>
          </div>
          {agentTeamItems.length === 0 ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>No agent profiles scanned</span>
            </div>
          ) : (
            agentTeamItems.slice(0, 4).map((agent) => (
              <div className="hc-agent-team-row" key={agent.id}>
                <span className={`hc-status-dot ${agent.available ? "ready" : "warning"}`} />
                <span className="hc-agent-team-copy">
                  <span>
                    {agent.label} · {agent.available ? "available" : "paused"} · {agent.runCount} {agent.runCount === 1 ? "run" : "runs"} · budget{" "}
                    {agent.budgetState}
                    {agent.usageLine ? ` · ${agent.usageLine}` : ""}
                  </span>
                  {agent.currentTaskLabel ? (
                    <small>
                      Task {agent.currentTaskLabel}
                      {agent.currentRunId ? ` · run ${agent.currentRunId}` : ""}
                    </small>
                  ) : null}
                  {agent.recentBlockerRunId ? (
                    <small>
                      Blocker {agent.recentBlockerRunId}
                      {agent.recentBlockerLifecycle ? ` · ${agent.recentBlockerLifecycle}` : ""}
                    </small>
                  ) : null}
                </span>
                <em>{agent.blockedCount > 0 ? `${agent.blockedCount} blocked` : "clear"}</em>
              </div>
            ))
          )}
        </section>
        ) : null}

        {dashboardWidgets.historicalAnalytics ? (
        <section className="hc-rail-panel" aria-label="Historical analytics charts">
          <header>
            <span>Historical Analytics</span>
            <Activity size={14} />
          </header>
          <div className="hc-analytics-summary">
            <span>Samples {historicalAnalytics.sampleCount}</span>
            <span>{historicalAnalytics.budgetLine}</span>
          </div>
          {analyticsDegraded ? (
            <div className="hc-analytics-health">
              <AlertTriangle size={15} />
              <span>Analytics degraded · api {controlPlaneSnapshot.health.api} · db {controlPlaneSnapshot.health.db}</span>
            </div>
          ) : null}
          <div className="hc-analytics-chart" aria-label="Run lifecycle chart">
            <h4>Run health {historicalAnalytics.runTotal} total</h4>
            {historicalAnalytics.runBars.length === 0 ? (
              <span className="hc-analytics-empty">No run samples</span>
            ) : (
              historicalAnalytics.runBars.map((item) => (
                <div className="hc-analytics-bar" key={`run-${item.label}`}>
                  <span>{item.label} {item.count}</span>
                  <div><i style={{ width: `${item.percent}%` }} /></div>
                </div>
              ))
            )}
          </div>
          <div className="hc-analytics-chart" aria-label="Evidence completeness chart">
            <h4>Evidence {historicalAnalytics.reviewTotal} reviews</h4>
            {historicalAnalytics.reviewBars.length === 0 ? (
              <span className="hc-analytics-empty">No review samples</span>
            ) : (
              historicalAnalytics.reviewBars.map((item) => (
                <div className="hc-analytics-bar" key={`review-${item.label}`}>
                  <span>{item.label} {item.count}</span>
                  <div><i style={{ width: `${item.percent}%` }} /></div>
                </div>
              ))
            )}
          </div>
        </section>
        ) : null}

        {dashboardWidgets.recentEvidence ? (
        <section className="hc-rail-panel" aria-label="Recent evidence activity">
          <header>
            <span>Recent Evidence</span>
            <FileText size={14} />
          </header>
          {evidenceActivityDegraded ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>Evidence activity degraded · api {controlPlaneSnapshot.health.api} · db {controlPlaneSnapshot.health.db}</span>
            </div>
          ) : null}
          <table className="hc-evidence-activity-table" aria-label="Recent evidence activity">
            <thead>
              <tr>
                <th scope="col">Evidence</th>
                <th scope="col">Task</th>
                <th scope="col">State</th>
                <th scope="col">Run</th>
              </tr>
            </thead>
            <tbody>
              {recentEvidenceItems.length === 0 ? (
                <tr>
                  <td colSpan={4}>No recent evidence activity</td>
                </tr>
              ) : (
                recentEvidenceItems.slice(0, 4).map((item) => (
                  <tr key={item.id}>
                    <td>
                      <strong>{item.evidencePackId}</strong>
                      <small>{item.id}</small>
                    </td>
                    <td>
                      <strong>{item.taskLabel}</strong>
                      <small>{item.projectLabel}</small>
                    </td>
                    <td>{item.reviewState} · {item.completenessState}</td>
                    <td>
                      {item.runLifecycle}
                      {item.usageLine ? <small>{item.usageLine}</small> : null}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </section>
        ) : null}

        <section className="hc-rail-panel hc-agent-adapter-sdk" aria-label="Agent adapter SDK">
          <header>
            <span>Agent Adapter SDK</span>
            <Blocks size={14} />
          </header>
          <div className="hc-agent-adapter-form">
            <label>
              <span>ID</span>
              <input aria-label="Adapter profile id" value={agentAdapterId} onChange={(event) => setAgentAdapterId(event.currentTarget.value)} />
            </label>
            <label>
              <span>Name</span>
              <input aria-label="Adapter profile name" value={agentAdapterName} onChange={(event) => setAgentAdapterName(event.currentTarget.value)} />
            </label>
            <label>
              <span>Runtime</span>
              <input aria-label="Adapter runtime" value={agentAdapterRuntime} onChange={(event) => setAgentAdapterRuntime(event.currentTarget.value)} />
            </label>
            <label>
              <span>Command</span>
              <input aria-label="Adapter command" value={agentAdapterCommand} onChange={(event) => setAgentAdapterCommand(event.currentTarget.value)} />
            </label>
            <label>
              <span>Args</span>
              <textarea aria-label="Adapter args JSON" value={agentAdapterArgsJson} onChange={(event) => setAgentAdapterArgsJson(event.currentTarget.value)} />
            </label>
            <label>
              <span>Env Policy</span>
              <textarea aria-label="Adapter env policy JSON" value={agentAdapterEnvPolicyJson} onChange={(event) => setAgentAdapterEnvPolicyJson(event.currentTarget.value)} />
            </label>
            <label>
              <span>Skills</span>
              <textarea aria-label="Adapter skills JSON" value={agentAdapterSkillsJson} onChange={(event) => setAgentAdapterSkillsJson(event.currentTarget.value)} />
            </label>
            <button type="button" onClick={handleRegisterAgentAdapter}>
              Register adapter profile
            </button>
          </div>
          {controlPlaneSnapshot.health.api !== "ok" || controlPlaneSnapshot.health.db !== "ok" ? (
            <div className="hc-agent-adapter-error">
              <AlertTriangle size={15} />
              <span>Adapter registration degraded · api {controlPlaneSnapshot.health.api} · db {controlPlaneSnapshot.health.db}</span>
            </div>
          ) : null}
          {agentAdapterStatus ? <div className="hc-agent-adapter-status">{agentAdapterStatus}</div> : null}
          {agentAdapterError ? <div className="hc-agent-adapter-error">{agentAdapterError}</div> : null}
        </section>

        <section className="hc-rail-panel" aria-label="Agent Directory">
          <header>
            <span>Agent Directory</span>
            <Terminal size={14} />
          </header>
          <button className="hc-task-drawer-action" type="button" onClick={handleLoadAgents} aria-label="Load agent profiles">
            Load agents
          </button>
          <button className="hc-task-drawer-action" type="button" onClick={handleScanAgents} aria-label="Scan agent profiles">
            Scan agents
          </button>
          {agentItems.length === 0 ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>No agent profiles scanned</span>
            </div>
          ) : (
            agentItems.map((agent) => (
              <div className="hc-review-row" key={agent.id}>
                <Terminal size={15} />
                {agent.attention_severity ? (
                  <span
                    aria-label={`Agent ${agent.id} attention ${agent.attention_severity}`}
                    className={`hc-agent-attention-ring ${attentionIconStatus(normalizeAttentionSeverity(agent.attention_severity))}`}
                  />
                ) : null}
                <span>{agent.label} · {agent.available ? "available" : "paused"}{agent.attention_state ? ` · ${agent.attention_state}` : ""}</span>
                {agent.latest_event_kind ? (
                  <small>{agent.notification_count ? `${agent.notification_count} notifications · ` : ""}{agent.latest_event_kind} · {agent.latest_event_detail}</small>
                ) : null}
                {agent.last_heartbeat_at ? (
                  <small>last heartbeat · {agent.last_heartbeat_at}</small>
                ) : null}
                <button
                  type="button"
                  onClick={() => handleAgentAvailability(agent, !agent.available)}
                  aria-label={`${agent.available ? "Pause" : "Resume"} ${agent.id}`}
                >
                  {agent.available ? <XCircle size={13} /> : <StepForward size={13} />}
                </button>
                <button
                  type="button"
                  onClick={() => handleAgentHeartbeat(agent)}
                  aria-label={`Heartbeat ${agent.id}`}
                >
                  <Activity size={13} />
                </button>
                <button
                  type="button"
                  onClick={() => handleLaunchAgentTerminal(agent)}
                  aria-label={`Launch ${agent.id} raw terminal`}
                  disabled={!agent.available}
                >
                  <Terminal size={13} />
                </button>
              </div>
            ))
          )}
          {agentLaunchStatus ? <div className="hc-agent-adapter-status">{agentLaunchStatus}</div> : null}
          {agentLaunchError ? <div className="hc-agent-adapter-error">{agentLaunchError}</div> : null}
          {agentDirectoryError ? <div className="hc-agent-adapter-error">{agentDirectoryError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-agent-event-normalizer" aria-label="Agent event normalizer">
          <header>
            <span>Agent Events</span>
            <Activity size={14} />
          </header>
          <div className="hc-agent-adapter-form">
            <label>
              <span>Adapter</span>
              <select aria-label="Agent event adapter" value={agentEventAdapter} onChange={(event) => setAgentEventAdapter(event.currentTarget.value)}>
                <option value="raw-jsonl">raw-jsonl</option>
                <option value="generic.agent-json">generic.agent-json</option>
              </select>
            </label>
            <label>
              <span>Agent</span>
              <input aria-label="Agent event profile" value={agentEventProfile} onChange={(event) => setAgentEventProfile(event.currentTarget.value)} />
            </label>
            <label>
              <span>Session</span>
              <input aria-label="Agent event session" value={agentEventSession} onChange={(event) => setAgentEventSession(event.currentTarget.value)} />
            </label>
            <label>
              <span>Run</span>
              <input aria-label="Agent event run" value={agentEventRun} onChange={(event) => setAgentEventRun(event.currentTarget.value)} />
            </label>
            <label>
              <span>Payload</span>
              <textarea aria-label="Agent event payload" value={agentEventPayload} onChange={(event) => setAgentEventPayload(event.currentTarget.value)} />
            </label>
            <button type="button" onClick={handleIngestAgentEvents}>
              Ingest agent events
            </button>
          </div>
          {agentEventResult ? (
            <div className="hc-agent-adapter-status">
              <strong>{agentEventResult.kind} · {agentEventResult.severity}</strong>
              <span>{agentEventResult.detail}</span>
              <small>{agentEventResult.source}</small>
            </div>
          ) : (
            <div className="hc-review-row">
              <Activity size={15} />
              <span>Normalize agent JSONL status, messages, tool calls, and errors</span>
            </div>
          )}
          {agentEventError ? <div className="hc-agent-adapter-error">{agentEventError}</div> : null}
        </section>

        <section className="hc-rail-panel hc-security-diagnostics" aria-label="Security diagnostics">
          <header>
            <span>Security Diagnostics</span>
            <ShieldAlert size={14} />
          </header>
          <div className="hc-security-diagnostics-summary">
            <span>Security diagnostics · {securityDiagnostics.status ?? "unknown"}</span>
            <small>{securityDiagnostics.pending_policy_approvals ?? 0} pending approvals</small>
          </div>
          <div className="hc-security-diagnostics-list">
            {(securityDiagnostics.checks ?? []).map((check) => (
              <div className="hc-security-diagnostics-row" key={check.id ?? check.label}>
                <span>{check.label ?? "Security check"} · {check.status ?? "unknown"}</span>
                <small>{check.detail ?? "No diagnostic detail"}</small>
              </div>
            ))}
          </div>
          <div className="hc-security-diagnostics-summary">
            <span>
              Audit {permissionAudit.recent_count ?? 0} recent · {permissionAudit.forbidden_count ?? 0} blocked · latest {permissionAudit.latest_action_kind ?? "none"} {permissionAudit.latest_decision ?? "none"}
            </span>
          </div>
          <div className="hc-policy-evaluation-form">
            <select
              aria-label="Permission audit decision"
              value={permissionAuditDecision}
              onChange={(event) => setPermissionAuditDecision(event.currentTarget.value)}
            >
              <option value="">any decision</option>
              <option value="allowed">allowed</option>
              <option value="approval_required">approval_required</option>
              <option value="forbidden">forbidden</option>
            </select>
            <input
              aria-label="Permission audit action"
              value={permissionAuditActionKind}
              onChange={(event) => setPermissionAuditActionKind(event.currentTarget.value)}
              placeholder="network"
            />
            <input
              aria-label="Permission audit run"
              value={permissionAuditRunId}
              onChange={(event) => setPermissionAuditRunId(event.currentTarget.value)}
              placeholder="run_1"
            />
            <input
              aria-label="Permission audit task"
              value={permissionAuditTaskId}
              onChange={(event) => setPermissionAuditTaskId(event.currentTarget.value)}
              placeholder="task_1"
            />
            <button type="button" onClick={handleLoadPermissionAudit} aria-label="Load permission audit">
              Load audit
            </button>
          </div>
          {permissionAuditEvents.length ? (
            <div className="hc-policy-pack-result">
              {permissionAuditEvents.slice(0, 4).map((audit) => (
                <span key={audit.id}>
                  {audit.id} · {audit.action_kind} · {audit.decision} · run {audit.run_id ?? "none"} · task {audit.task_id ?? "none"} · {audit.reason}
                </span>
              ))}
            </div>
          ) : null}
          {permissionAuditError ? (
            <div className="hc-policy-pack-error">{permissionAuditError}</div>
          ) : null}
        </section>

        <section className="hc-rail-panel">
          <header>
            <span>Policy Pack Model</span>
            <ShieldAlert size={14} />
          </header>
          <div className="hc-policy-pack-summary">
            <span>Policy pack {activePolicyPack.name ?? "Default ask-before-write"} · {activePolicyPack.sandbox_mode ?? "ask-before-write"}</span>
            <small>
              network {activePolicyPack.network_profile ?? "internet"} · approvals {activePolicyPack.approval_required_count ?? 0} · forbidden {activePolicyPack.forbidden_count ?? 0}
            </small>
          </div>
          <div className="hc-policy-pack-form">
            <input
              aria-label="Policy pack name"
              value={policyPackName}
              onChange={(event) => setPolicyPackName(event.currentTarget.value)}
              placeholder={activePolicyPack.name ?? "Ask before write"}
            />
            <select
              aria-label="Policy sandbox mode"
              value={policyPackSandboxMode}
              onChange={(event) => setPolicyPackSandboxMode(event.currentTarget.value)}
            >
              <option value="normal">normal</option>
              <option value="ask-before-write">ask-before-write</option>
              <option value="sandboxed">sandboxed</option>
            </select>
            <select
              aria-label="Policy network permission"
              value={policyPackNetwork}
              onChange={(event) => setPolicyPackNetwork(event.currentTarget.value)}
            >
              <option value="allowed">allowed</option>
              <option value="ask">ask</option>
              <option value="blocked">blocked</option>
            </select>
            <select
              aria-label="Policy network sandbox profile"
              value={policyPackNetworkProfile}
              onChange={(event) => setPolicyPackNetworkProfile(event.currentTarget.value)}
            >
              <option value="internet">internet</option>
              <option value="local-only">local-only</option>
              <option value="offline">offline</option>
            </select>
            <select
              aria-label="Policy file write permission"
              value={policyPackFileWrite}
              onChange={(event) => setPolicyPackFileWrite(event.currentTarget.value)}
            >
              <option value="allowed">allowed</option>
              <option value="ask">ask</option>
              <option value="blocked">blocked</option>
            </select>
            <input
              aria-label="Policy approval required actions"
              value={policyPackApprovals}
              onChange={(event) => setPolicyPackApprovals(event.currentTarget.value)}
              placeholder="shell_command,file_write"
            />
            <input
              aria-label="Policy forbidden operations"
              value={policyPackForbidden}
              onChange={(event) => setPolicyPackForbidden(event.currentTarget.value)}
              placeholder="network"
            />
            <button type="button" onClick={handleSavePolicyPack} aria-label="Save policy pack">
              Save policy pack
            </button>
            <button type="button" onClick={handleLoadPolicyPacks} aria-label="Load policy packs">
              Load policy packs
            </button>
          </div>
          <div className="hc-policy-evaluation-form">
            <select
              aria-label="Policy action kind"
              value={policyActionKind}
              onChange={(event) => setPolicyActionKind(event.currentTarget.value)}
            >
              <option value="network">network</option>
              <option value="shell_command">shell_command</option>
              <option value="file_write">file_write</option>
              <option value="tool_use">tool_use</option>
            </select>
            <input
              aria-label="Policy action command"
              value={policyActionCommand}
              onChange={(event) => setPolicyActionCommand(event.currentTarget.value)}
              placeholder="curl http://127.0.0.1:3000"
            />
            <select
              aria-label="Policy approval risk level"
              value={policyApprovalRiskLevel}
              onChange={(event) => setPolicyApprovalRiskLevel(event.currentTarget.value)}
            >
              <option value="low">low</option>
              <option value="medium">medium</option>
              <option value="high">high</option>
              <option value="critical">critical</option>
            </select>
            <button type="button" onClick={handleEvaluatePolicyAction} aria-label="Evaluate policy action">
              Evaluate action
            </button>
            <button type="button" onClick={handleCreatePolicyApproval} aria-label="Create policy approval">
              Create approval
            </button>
          </div>
          {policyPackResult ? (
            <div className="hc-policy-pack-result">
              <strong>Saved policy pack {policyPackResult.name} · {policyPackResult.sandbox_mode}</strong>
              <small>{policyPackResult.network_profile} network · {policyPackResult.file_write} file write</small>
            </div>
          ) : null}
          {policyPacks.length > 0 ? (
            <div className="hc-policy-pack-list" aria-label="Loaded policy packs">
              {policyPacks.map((pack) => (
                <span key={pack.id}>
                  {pack.name} · {pack.sandbox_mode} · {pack.active ? "active" : "inactive"}
                </span>
              ))}
            </div>
          ) : null}
          {policyEvaluation ? (
            <div className="hc-policy-pack-result">
              <strong>Policy evaluation {policyEvaluation.action_kind} · {policyEvaluation.decision}</strong>
              <small>{policyEvaluation.reason}</small>
            </div>
          ) : null}
          {policyPackError ? (
            <div className="hc-policy-pack-error">{policyPackError}</div>
          ) : null}
        </section>

        <section className="hc-rail-panel" aria-label="Policy approvals">
          <header>
            <span>Policy Approvals</span>
            <button type="button" onClick={handleLoadPolicyApprovals} aria-label="Load policy approvals">
              <RotateCcw size={13} />
            </button>
            <ShieldAlert size={14} />
          </header>
          {policyApprovalItems.length === 0 ? (
            <div className="hc-review-row">
              <CheckCircle2 size={15} />
              <span>No pending dangerous actions</span>
            </div>
          ) : (
            policyApprovalItems.slice(0, 4).map((item) => (
              <div className="hc-review-row" key={item.id}>
                <ShieldAlert size={15} />
                <span>{item.label} · {item.severity}</span>
                <small>{item.detail}</small>
                <button type="button" onClick={() => handlePolicyDecision(item, "approved")} aria-label={`Approve ${item.id}`}>
                  <CheckCircle2 size={13} />
                </button>
                <button type="button" onClick={() => handlePolicyDecision(item, "denied")} aria-label={`Deny ${item.id}`}>
                  <XCircle size={13} />
                </button>
              </div>
            ))
          )}
          {policyApprovalError ? <div className="hc-policy-pack-error">{policyApprovalError}</div> : null}
        </section>

        <section className="hc-rail-panel">
          <header>
            <span>Review Queue</span>
            <Activity size={14} />
          </header>
          {reviewItems.length > 0 ? (
            <div className="hc-review-filter-grid">
              <label>
                <span>State</span>
                <select aria-label="Review state filter" value={reviewStateFilter} onChange={(event) => setReviewStateFilter(event.currentTarget.value)}>
                  <option value="all">All states</option>
                  <option value="pending">Pending</option>
                  <option value="approved">Approved</option>
                  <option value="changes_requested">Changes requested</option>
                  <option value="reopened">Reopened</option>
                  <option value="blocked">Blocked</option>
                </select>
              </label>
              <label>
                <span>Evidence</span>
                <select
                  aria-label="Review completeness filter"
                  value={reviewCompletenessFilter}
                  onChange={(event) => setReviewCompletenessFilter(event.currentTarget.value)}
                >
                  <option value="all">All evidence</option>
                  <option value="complete">Complete</option>
                  <option value="incomplete">Incomplete</option>
                </select>
              </label>
            </div>
          ) : null}
          {reviewActionError ? (
            <div className="hc-review-row">
              <AlertTriangle size={15} />
              <span>{reviewActionError}</span>
            </div>
          ) : reviewActionStatus ? (
            <div className="hc-review-row">
              <CheckCircle2 size={15} />
              <span>{reviewActionStatus}</span>
            </div>
          ) : null}
          {reviewItems.length === 0 ? (
            <>
              <div className="hc-review-row">
                <ShieldAlert size={15} />
                <span>{formatTerminalFidelityProof(controlPlaneSnapshot.terminal_fidelity)}</span>
              </div>
              <div className="hc-review-row">
                <FileText size={15} />
                <span>Evidence Pack {evidencePack.commandBlocks.length} command blocks attached</span>
              </div>
            </>
          ) : filteredReviewItems.length === 0 ? (
            <div className="hc-review-row">
              <FileText size={15} />
              <span>No reviews match filters</span>
            </div>
          ) : (
            filteredReviewItems.slice(0, 4).map((review) => {
              const reviewSession = findReviewSession(review);
              return (
                <div className="hc-review-row" key={review.id}>
                  <FileText size={15} />
                  <span>
                    {review.id} · {review.state} · {review.completeness_state}
                    {review.diff_summary?.summary ? <small>Diff {review.diff_summary.summary}</small> : null}
                  </span>
                  <button
                    type="button"
                    onClick={() => handleOpenReviewTerminal(review)}
                    aria-label={`Open terminal ${review.id}`}
                    disabled={!reviewSession}
                  >
                    <Terminal size={13} />
                  </button>
                  <button
                    type="button"
                    onClick={handleOpenReviewWorktree}
                    aria-label={`Open worktree ${review.id}`}
                    disabled={!activeSnapshotProject}
                  >
                    <GitBranch size={13} />
                  </button>
                  <button type="button" onClick={() => handleCopyReviewPatch(review)} aria-label={`Copy patch ${review.id}`}>
                    <Clipboard size={13} />
                  </button>
                  <button type="button" onClick={() => handlePlanReviewPrLanding(review)} aria-label={`Plan PR ${review.id}`}>
                    <GitPullRequest size={13} />
                  </button>
                  <button type="button" onClick={() => handleCreateReviewFollowUp(review)} aria-label={`Create follow-up ${review.id}`}>
                    <Plus size={13} />
                  </button>
                  <button type="button" onClick={() => handleReviewDecision(review, "approved")} aria-label={`Approve ${review.id}`}>
                    <CheckCircle2 size={13} />
                  </button>
                  <button
                    type="button"
                    onClick={() => handleReviewDecision(review, "changes_requested")}
                    aria-label={`Request changes ${review.id}`}
                  >
                    <XCircle size={13} />
                  </button>
                  <button type="button" onClick={() => handleReviewDecision(review, "reopened")} aria-label={`Reopen ${review.id}`}>
                    <RotateCcw size={13} />
                  </button>
                  <button type="button" onClick={() => handleReviewDecision(review, "blocked")} aria-label={`Block ${review.id}`}>
                    <ShieldAlert size={13} />
                  </button>
                </div>
              );
            })
          )}
        </section>
      </aside>

      <footer className="hc-statusbar hc-hairline-top" aria-label="Workspace status">
        <span><CircleDot size={12} /> Local-first workspace</span>
        <span>{readinessSummary.total} readiness checks</span>
        <span>{ptySnapshot.total} PTY sessions</span>
        {notificationJumpStatus ? <span>{notificationJumpStatus}</span> : null}
        <span><LayoutGrid size={12} /> 2x2 terminal layout</span>
      </footer>
    </div>
  );
}

export default App;
