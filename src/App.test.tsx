import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { TerminalSession } from "./domain/terminal";
import type { StateSnapshot } from "./services/stateSnapshotClient";
import type { TerminalPtyOutputEvent } from "./services/terminalPtyClient";

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((innerResolve) => {
    resolve = innerResolve;
  });
  return { promise, resolve };
}

async function resolveDeferred<T>(deferred: ReturnType<typeof createDeferred<T>>, value: T) {
  await act(async () => {
    deferred.resolve(value);
    await deferred.promise;
  });
}

function immediateResolved<T>(value: T): Promise<T> {
  const thenable = {
    then(onFulfilled?: ((value: T) => unknown) | null) {
      if (!onFulfilled) return immediateResolved(value);
      try {
        return immediateResolved(onFulfilled(value));
      } catch (error) {
        return immediateRejected(error);
      }
    },
    catch() {
      return thenable;
    },
    finally(onFinally?: (() => void) | null) {
      onFinally?.();
      return thenable;
    },
    [Symbol.toStringTag]: "Promise",
  };
  return thenable as unknown as Promise<T>;
}

function immediateRejected<T = never>(reason: unknown): Promise<T> {
  const thenable = {
    then(_onFulfilled?: ((value: T) => unknown) | null, onRejected?: ((reason: unknown) => unknown) | null) {
      if (!onRejected) return immediateRejected(reason);
      try {
        return immediateResolved(onRejected(reason));
      } catch (error) {
        return immediateRejected(error);
      }
    },
    catch(onRejected?: ((reason: unknown) => unknown) | null) {
      return thenable.then(undefined, onRejected);
    },
    finally(onFinally?: (() => void) | null) {
      onFinally?.();
      return thenable;
    },
    [Symbol.toStringTag]: "Promise",
  };
  return thenable as unknown as Promise<T>;
}

async function renderApp() {
  const result = render(<App />);
  await flushAppEffects();
  return result;
}

async function flushAppEffects() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

function resolveStateSnapshotMocksImmediately() {
  const snapshotMock = stateSnapshot.getStateSnapshot as typeof stateSnapshot.getStateSnapshot & {
    mockResolvedValueOnce: (value: StateSnapshot) => typeof stateSnapshot.getStateSnapshot;
  };

  snapshotMock.mockResolvedValueOnce = (value: StateSnapshot) => snapshotMock.mockReturnValueOnce(immediateResolved(value));
}

function defaultStateSnapshot(): StateSnapshot {
  return {
    snapshot_id: "snap_test",
    generated_at: "2026-04-30T01:00:00Z",
    app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
    projects: [],
    project_tabs: [],
    sessions: [],
    command_blocks: { recent: [], unread_count: 0 },
    tasks: { items: [], counts_by_status: {} },
    runs: { items: [], counts_by_lifecycle: {} },
    agents: [],
    reviews: [],
    attention: [{ id: "db-not-configured", label: "Database persistence not configured", severity: "warning", detail: "pending" }],
    provider_model: { provider: "openai", model: "gpt-5.4", agent_profile_id: "agent_codex" },
    budgets: { workspace: {}, projects: [], agents: [] },
    workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
    knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
    health: { db: "degraded", pty: "ok", api: "ok" },
  };
}

const terminalPty = vi.hoisted(() => ({
  outputHandler: undefined as ((event: TerminalPtyOutputEvent) => void) | undefined,
  getTerminalPtySnapshot: vi.fn(async () => ({ total: 0, sessions: [] })),
  captureTerminalPtyCommand: vi.fn(),
  listenToTerminalPtyOutput: vi.fn(async (handler: (event: TerminalPtyOutputEvent) => void) => {
    terminalPty.outputHandler = handler;
    return vi.fn();
  }),
  spawnTerminalPtySession: vi.fn(async () => ({
    id: "pty_1",
    title: "1. haneulchi (zsh)",
    command: "sh",
    cols: 100,
    rows: 30,
  })),
  writeTerminalPtyInput: vi.fn(async () => undefined),
  closeTerminalPtySession: vi.fn(async () => undefined),
}));

const stateSnapshot = vi.hoisted(() => ({
  getStateSnapshot: vi.fn(async (): Promise<StateSnapshot> => ({
    snapshot_id: "snap_test",
    generated_at: "2026-04-30T01:00:00Z",
    app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
    projects: [],
    project_tabs: [],
    sessions: [],
    command_blocks: { recent: [], unread_count: 0 },
    tasks: { items: [], counts_by_status: {} },
    runs: { items: [], counts_by_lifecycle: {} },
    agents: [],
    reviews: [],
    attention: [{ id: "db-not-configured", label: "Database persistence not configured", severity: "warning", detail: "pending" }],
    provider_model: { provider: "openai", model: "gpt-5.4", agent_profile_id: "agent_codex" },
    budgets: { workspace: {}, projects: [], agents: [] },
    workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
    knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
    health: { db: "degraded", pty: "ok", api: "ok" },
  })),
}));

const taskApi = vi.hoisted(() => ({
  loadNativeTaskState: vi.fn(),
  createNativeTask: vi.fn(),
  createNativeReviewFollowUpTask: vi.fn(),
  moveNativeTask: vi.fn(),
  addNativeTaskComment: vi.fn(),
  addNativeTaskSubtask: vi.fn(),
  createNativeTaskCycle: vi.fn(),
  createNativeTaskModule: vi.fn(),
  listNativeTaskComments: vi.fn(),
  listNativeTaskCycles: vi.fn(),
  listNativeTaskModules: vi.fn(),
  listNativeTaskSubtasks: vi.fn(),
  saveNativeTaskWorkpad: vi.fn(),
  updateNativeTaskSubtaskStatus: vi.fn(),
  updateNativeTaskPlanning: vi.fn(),
  updateNativeTaskContext: vi.fn(),
  nativeTaskToHaneulchiTask: vi.fn((task: {
    id: string;
    title: string;
    status: string;
    priority: string;
    project_id: string;
    assignee_id?: string | null;
    cycle_id?: string | null;
    module_id?: string | null;
    initiative_id?: string | null;
    due_at?: string | null;
    estimate?: string | null;
    workpad_md?: string | null;
    context_pack_id?: string | null;
  }) => ({
    id: task.id,
    title: task.title,
    status: task.status,
    priority: task.priority,
    projectId: task.project_id,
    assignee: task.assignee_id ?? undefined,
    workpad: task.workpad_md ?? undefined,
    cycle: task.cycle_id ?? undefined,
    module: task.module_id ?? undefined,
    initiative: task.initiative_id ?? undefined,
    dueDate: task.due_at ?? undefined,
    estimate: task.estimate ?? undefined,
    contextPackId: task.context_pack_id ?? undefined,
  })),
}));

const initiativeApi = vi.hoisted(() => ({
  listNativeInitiatives: vi.fn(),
  createNativeInitiative: vi.fn(),
}));

const commandBlockApi = vi.hoisted(() => ({
  upsertNativeCommandBlock: vi.fn(),
  searchNativeCommandBlocks: vi.fn(),
  markNativeCommandBlock: vi.fn(),
  mergeNativeCommandBlocks: vi.fn(),
  splitNativeCommandBlock: vi.fn(),
  attachNativeCommandBlockToEvidence: vi.fn(),
  explainNativeCommandBlock: vi.fn(),
  exportNativeCommandBlockBundle: vi.fn(),
  generateNativeEvidencePackForRun: vi.fn(),
  recordNativeEvidenceReviewDecision: vi.fn(),
}));

const runApi = vi.hoisted(() => ({
  dispatchNativeRun: vi.fn(),
  listNativeRuns: vi.fn(),
  updateNativeRunLifecycle: vi.fn(),
  cancelNativeRun: vi.fn(),
  retryNativeRun: vi.fn(),
  recordNativeRunStatusUpdate: vi.fn(),
  nativeRunToStateRunSummary: vi.fn((run: {
    id: string;
    task_id: string;
    project_id: string;
    agent_profile_id?: string | null;
    session_id?: string | null;
    lifecycle: string;
    retry_count: number;
    next_retry_at?: string | null;
    status_detail?: string | null;
    context_pack_id?: string | null;
    workspace_path?: string | null;
  }) => ({
    id: run.id,
    task_id: run.task_id,
    project_id: run.project_id,
    agent_profile_id: run.agent_profile_id ?? undefined,
    session_id: run.session_id ?? undefined,
    lifecycle: run.lifecycle,
    retry_count: run.retry_count,
    next_retry_at: run.next_retry_at ?? undefined,
    status_detail: run.status_detail ?? undefined,
    context_pack_id: run.context_pack_id ?? undefined,
    workspace_path: run.workspace_path ?? undefined,
  })),
}));

const browserAutomationApi = vi.hoisted(() => ({
  planNativeBrowserAutomation: vi.fn(),
}));

const releaseWorkflowApi = vi.hoisted(() => ({
  getNativeReleaseWorkflowStatus: vi.fn(),
}));

const qualityApi = vi.hoisted(() => ({
  listNativeBenchmarkRuns: vi.fn(),
  listNativeDmgSmokeRuns: vi.fn(),
  listNativeDogfoodTelemetryReviews: vi.fn(),
  listNativeRecoveryDrillRuns: vi.fn(),
  listNativeReleaseGateRuns: vi.fn(),
  listNativeTaskLifecycleE2ERuns: vi.fn(),
  listNativeTerminalFidelitySmokeRuns: vi.fn(),
  listNativeWorkflowNegativeTestRuns: vi.fn(),
  runNativeBenchmarks: vi.fn(),
  runNativeDmgSmokeTest: vi.fn(),
  runNativeDogfoodTelemetryReview: vi.fn(),
  runNativeRecoveryDrills: vi.fn(),
  runNativeReleaseGates: vi.fn(),
  runNativeTaskLifecycleE2E: vi.fn(),
  runNativeTerminalFidelitySmoke: vi.fn(),
  runNativeWorkflowNegativeTests: vi.fn(),
}));

const visualHarnessApi = vi.hoisted(() => ({
  createNativeVisualHarnessLink: vi.fn(),
  listNativeVisualHarnessLinks: vi.fn(),
}));

const trackerApi = vi.hoisted(() => ({
  listNativeExternalTrackerBindings: vi.fn(),
  upsertNativeExternalTrackerBinding: vi.fn(),
  runNativeTrackerSync: vi.fn(),
}));

const workflowApi = vi.hoisted(() => ({
  reloadWorkflow: vi.fn(),
  getWorkflowRuntimeState: vi.fn(),
  getRunReplayMetadata: vi.fn(),
  validateWorkflow: vi.fn(),
  runWorkflowHook: vi.fn(),
}));

const knowledgeApi = vi.hoisted(() => ({
  answerNativeKnowledgeQuestion: vi.fn(),
  exportNativeKnowledgeObsidianMarkdown: vi.fn(),
  listNativeKnowledgeSources: vi.fn(),
  upsertNativeKnowledgeSource: vi.fn(),
  saveNativeKnowledgePage: vi.fn(),
  searchNativeKnowledgePages: vi.fn(),
  listNativeKnowledgeConcepts: vi.fn(),
  saveNativeKnowledgeExploration: vi.fn(),
  listNativeKnowledgeExplorations: vi.fn(),
  listNativeContextPacks: vi.fn(),
  upsertNativeContextPack: vi.fn(),
  recordNativeKnowledgeLintReport: vi.fn(),
  runNativeKnowledgeAutomation: vi.fn(),
  ingestNativeKnowledgeArtifact: vi.fn(),
}));

const budgetApi = vi.hoisted(() => ({
  ingestNativeTokenUsageAdapter: vi.fn(),
  getNativeBudgetSummary: vi.fn(),
  getNativeBudgetForecast: vi.fn(),
  listNativeProviderPrices: vi.fn(),
  updateNativeProviderPriceTable: vi.fn(),
  upsertNativeBudget: vi.fn(),
  recordNativeTokenUsage: vi.fn(),
}));

const secretApi = vi.hoisted(() => ({
  listNativeSecrets: vi.fn(),
  upsertNativeSecret: vi.fn(),
}));

const policyApi = vi.hoisted(() => ({
  createNativePolicyApproval: vi.fn(),
  listNativePolicyApprovals: vi.fn(),
  listNativePolicyPacks: vi.fn(),
  listNativePermissionAudits: vi.fn(),
  decideNativePolicyApproval: vi.fn(),
  upsertNativePolicyPack: vi.fn(),
  evaluateNativePolicyAction: vi.fn(),
}));

const agentApi = vi.hoisted(() => ({
  heartbeatNativeAgentProfile: vi.fn(),
  ingestNativeAgentEvents: vi.fn(),
  listNativeAgentProfiles: vi.fn(),
  listNativeRuntimePool: vi.fn(),
  listNativeSkillPacks: vi.fn(),
  scanNativeAgentProfiles: vi.fn(),
  upsertNativeAgentProfile: vi.fn(),
  upsertNativeSkillPack: vi.fn(),
  updateNativeAgentProfileStatus: vi.fn(),
}));

const providerModelApi = vi.hoisted(() => ({
  getNativeProviderModelSettings: vi.fn(),
  upsertNativeProviderModelSettings: vi.fn(),
}));

const terminalThemeApi = vi.hoisted(() => ({
  getNativeTerminalThemeSettings: vi.fn(),
  upsertNativeTerminalThemeSettings: vi.fn(),
}));

const sessionApi = vi.hoisted(() => ({
  attachNativeSessionTask: vi.fn(),
  createNativeSession: vi.fn(),
  detachNativeSessionTask: vi.fn(),
  launchNativeAgentTerminal: vi.fn(),
  listNativeSessions: vi.fn(),
  focusNativeSession: vi.fn(),
  takeoverNativeSession: vi.fn(),
  releaseNativeSession: vi.fn(),
  killNativeSession: vi.fn(),
  listNativeTerminalStreamChunks: vi.fn(),
  recordNativeSessionInput: vi.fn(),
  recordNativeTerminalStreamChunk: vi.fn(),
}));

const projectApi = vi.hoisted(() => ({
  addNativeProject: vi.fn(),
  listNativeProjects: vi.fn(),
  focusNativeProject: vi.fn(),
  updateNativeProjectLayout: vi.fn(),
  saveNativeProjectLayoutPreset: vi.fn(),
  listNativeProjectLayoutPresets: vi.fn(),
  listNativeProjectFiles: vi.fn(),
  readNativeProjectDiff: vi.fn(),
  readNativeProjectFile: vi.fn(),
  collectNativeProjectLspDiagnostics: vi.fn(),
  exportNativeProjectPatch: vi.fn(),
  importNativeProjectPatch: vi.fn(),
  planNativeProjectDetach: vi.fn(),
  planNativePrLanding: vi.fn(),
  planNativeReviewPrLanding: vi.fn(),
  searchNativeProjectFiles: vi.fn(),
  saveNativeProjectFile: vi.fn(),
  upsertNativeProjectTabGroup: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => {
    throw new Error("Tauri unavailable in tests");
  }),
}));

vi.mock("./services/terminalPtyClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/terminalPtyClient")>()),
  getTerminalPtySnapshot: terminalPty.getTerminalPtySnapshot,
  captureTerminalPtyCommand: terminalPty.captureTerminalPtyCommand,
  listenToTerminalPtyOutput: terminalPty.listenToTerminalPtyOutput,
  spawnTerminalPtySession: terminalPty.spawnTerminalPtySession,
  writeTerminalPtyInput: terminalPty.writeTerminalPtyInput,
  closeTerminalPtySession: terminalPty.closeTerminalPtySession,
}));

vi.mock("./services/stateSnapshotClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/stateSnapshotClient")>()),
  getStateSnapshot: stateSnapshot.getStateSnapshot,
}));

vi.mock("./services/taskApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/taskApiClient")>()),
  loadNativeTaskState: taskApi.loadNativeTaskState,
  createNativeTask: taskApi.createNativeTask,
  createNativeReviewFollowUpTask: taskApi.createNativeReviewFollowUpTask,
  moveNativeTask: taskApi.moveNativeTask,
  addNativeTaskComment: taskApi.addNativeTaskComment,
  addNativeTaskSubtask: taskApi.addNativeTaskSubtask,
  createNativeTaskCycle: taskApi.createNativeTaskCycle,
  createNativeTaskModule: taskApi.createNativeTaskModule,
  listNativeTaskComments: taskApi.listNativeTaskComments,
  listNativeTaskCycles: taskApi.listNativeTaskCycles,
  listNativeTaskModules: taskApi.listNativeTaskModules,
  listNativeTaskSubtasks: taskApi.listNativeTaskSubtasks,
  saveNativeTaskWorkpad: taskApi.saveNativeTaskWorkpad,
  updateNativeTaskSubtaskStatus: taskApi.updateNativeTaskSubtaskStatus,
  updateNativeTaskPlanning: taskApi.updateNativeTaskPlanning,
  updateNativeTaskContext: taskApi.updateNativeTaskContext,
  nativeTaskToHaneulchiTask: taskApi.nativeTaskToHaneulchiTask,
}));

vi.mock("./services/initiativeApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/initiativeApiClient")>()),
  listNativeInitiatives: initiativeApi.listNativeInitiatives,
  createNativeInitiative: initiativeApi.createNativeInitiative,
}));

vi.mock("./services/commandBlockApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/commandBlockApiClient")>()),
  upsertNativeCommandBlock: commandBlockApi.upsertNativeCommandBlock,
  searchNativeCommandBlocks: commandBlockApi.searchNativeCommandBlocks,
  markNativeCommandBlock: commandBlockApi.markNativeCommandBlock,
  mergeNativeCommandBlocks: commandBlockApi.mergeNativeCommandBlocks,
  splitNativeCommandBlock: commandBlockApi.splitNativeCommandBlock,
  attachNativeCommandBlockToEvidence: commandBlockApi.attachNativeCommandBlockToEvidence,
  explainNativeCommandBlock: commandBlockApi.explainNativeCommandBlock,
  exportNativeCommandBlockBundle: commandBlockApi.exportNativeCommandBlockBundle,
  generateNativeEvidencePackForRun: commandBlockApi.generateNativeEvidencePackForRun,
  recordNativeEvidenceReviewDecision: commandBlockApi.recordNativeEvidenceReviewDecision,
}));

vi.mock("./services/runApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/runApiClient")>()),
  dispatchNativeRun: runApi.dispatchNativeRun,
  listNativeRuns: runApi.listNativeRuns,
  updateNativeRunLifecycle: runApi.updateNativeRunLifecycle,
  cancelNativeRun: runApi.cancelNativeRun,
  retryNativeRun: runApi.retryNativeRun,
  recordNativeRunStatusUpdate: runApi.recordNativeRunStatusUpdate,
  nativeRunToStateRunSummary: runApi.nativeRunToStateRunSummary,
}));

vi.mock("./services/browserAutomationApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/browserAutomationApiClient")>()),
  planNativeBrowserAutomation: browserAutomationApi.planNativeBrowserAutomation,
}));

vi.mock("./services/releaseWorkflowApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/releaseWorkflowApiClient")>()),
  getNativeReleaseWorkflowStatus: releaseWorkflowApi.getNativeReleaseWorkflowStatus,
}));

vi.mock("./services/qualityApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/qualityApiClient")>()),
  listNativeBenchmarkRuns: qualityApi.listNativeBenchmarkRuns,
  listNativeDmgSmokeRuns: qualityApi.listNativeDmgSmokeRuns,
  listNativeDogfoodTelemetryReviews: qualityApi.listNativeDogfoodTelemetryReviews,
  listNativeRecoveryDrillRuns: qualityApi.listNativeRecoveryDrillRuns,
  listNativeReleaseGateRuns: qualityApi.listNativeReleaseGateRuns,
  listNativeTaskLifecycleE2ERuns: qualityApi.listNativeTaskLifecycleE2ERuns,
  listNativeTerminalFidelitySmokeRuns: qualityApi.listNativeTerminalFidelitySmokeRuns,
  listNativeWorkflowNegativeTestRuns: qualityApi.listNativeWorkflowNegativeTestRuns,
  runNativeBenchmarks: qualityApi.runNativeBenchmarks,
  runNativeDmgSmokeTest: qualityApi.runNativeDmgSmokeTest,
  runNativeDogfoodTelemetryReview: qualityApi.runNativeDogfoodTelemetryReview,
  runNativeRecoveryDrills: qualityApi.runNativeRecoveryDrills,
  runNativeReleaseGates: qualityApi.runNativeReleaseGates,
  runNativeTaskLifecycleE2E: qualityApi.runNativeTaskLifecycleE2E,
  runNativeTerminalFidelitySmoke: qualityApi.runNativeTerminalFidelitySmoke,
  runNativeWorkflowNegativeTests: qualityApi.runNativeWorkflowNegativeTests,
}));

vi.mock("./services/visualHarnessApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/visualHarnessApiClient")>()),
  createNativeVisualHarnessLink: visualHarnessApi.createNativeVisualHarnessLink,
  listNativeVisualHarnessLinks: visualHarnessApi.listNativeVisualHarnessLinks,
}));

vi.mock("./services/trackerApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/trackerApiClient")>()),
  listNativeExternalTrackerBindings: trackerApi.listNativeExternalTrackerBindings,
  upsertNativeExternalTrackerBinding: trackerApi.upsertNativeExternalTrackerBinding,
  runNativeTrackerSync: trackerApi.runNativeTrackerSync,
}));

vi.mock("./services/workflowApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/workflowApiClient")>()),
  reloadWorkflow: workflowApi.reloadWorkflow,
  getWorkflowRuntimeState: workflowApi.getWorkflowRuntimeState,
  getRunReplayMetadata: workflowApi.getRunReplayMetadata,
  validateWorkflow: workflowApi.validateWorkflow,
  runWorkflowHook: workflowApi.runWorkflowHook,
}));

vi.mock("./services/knowledgeApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/knowledgeApiClient")>()),
  answerNativeKnowledgeQuestion: knowledgeApi.answerNativeKnowledgeQuestion,
  exportNativeKnowledgeObsidianMarkdown: knowledgeApi.exportNativeKnowledgeObsidianMarkdown,
  listNativeKnowledgeSources: knowledgeApi.listNativeKnowledgeSources,
  upsertNativeKnowledgeSource: knowledgeApi.upsertNativeKnowledgeSource,
  saveNativeKnowledgePage: knowledgeApi.saveNativeKnowledgePage,
  searchNativeKnowledgePages: knowledgeApi.searchNativeKnowledgePages,
  listNativeKnowledgeConcepts: knowledgeApi.listNativeKnowledgeConcepts,
  saveNativeKnowledgeExploration: knowledgeApi.saveNativeKnowledgeExploration,
  listNativeKnowledgeExplorations: knowledgeApi.listNativeKnowledgeExplorations,
  listNativeContextPacks: knowledgeApi.listNativeContextPacks,
  upsertNativeContextPack: knowledgeApi.upsertNativeContextPack,
  recordNativeKnowledgeLintReport: knowledgeApi.recordNativeKnowledgeLintReport,
  runNativeKnowledgeAutomation: knowledgeApi.runNativeKnowledgeAutomation,
  ingestNativeKnowledgeArtifact: knowledgeApi.ingestNativeKnowledgeArtifact,
}));

vi.mock("./services/budgetApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/budgetApiClient")>()),
  ingestNativeTokenUsageAdapter: budgetApi.ingestNativeTokenUsageAdapter,
  getNativeBudgetSummary: budgetApi.getNativeBudgetSummary,
  getNativeBudgetForecast: budgetApi.getNativeBudgetForecast,
  listNativeProviderPrices: budgetApi.listNativeProviderPrices,
  updateNativeProviderPriceTable: budgetApi.updateNativeProviderPriceTable,
  upsertNativeBudget: budgetApi.upsertNativeBudget,
  recordNativeTokenUsage: budgetApi.recordNativeTokenUsage,
}));

vi.mock("./services/secretApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/secretApiClient")>()),
  listNativeSecrets: secretApi.listNativeSecrets,
  upsertNativeSecret: secretApi.upsertNativeSecret,
}));

vi.mock("./services/policyApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/policyApiClient")>()),
  createNativePolicyApproval: policyApi.createNativePolicyApproval,
  listNativePolicyApprovals: policyApi.listNativePolicyApprovals,
  listNativePolicyPacks: policyApi.listNativePolicyPacks,
  listNativePermissionAudits: policyApi.listNativePermissionAudits,
  decideNativePolicyApproval: policyApi.decideNativePolicyApproval,
  upsertNativePolicyPack: policyApi.upsertNativePolicyPack,
  evaluateNativePolicyAction: policyApi.evaluateNativePolicyAction,
}));

vi.mock("./services/agentApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/agentApiClient")>()),
  heartbeatNativeAgentProfile: agentApi.heartbeatNativeAgentProfile,
  ingestNativeAgentEvents: agentApi.ingestNativeAgentEvents,
  listNativeAgentProfiles: agentApi.listNativeAgentProfiles,
  listNativeRuntimePool: agentApi.listNativeRuntimePool,
  listNativeSkillPacks: agentApi.listNativeSkillPacks,
  scanNativeAgentProfiles: agentApi.scanNativeAgentProfiles,
  upsertNativeAgentProfile: agentApi.upsertNativeAgentProfile,
  upsertNativeSkillPack: agentApi.upsertNativeSkillPack,
  updateNativeAgentProfileStatus: agentApi.updateNativeAgentProfileStatus,
}));

vi.mock("./services/providerModelApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/providerModelApiClient")>()),
  getNativeProviderModelSettings: providerModelApi.getNativeProviderModelSettings,
  upsertNativeProviderModelSettings: providerModelApi.upsertNativeProviderModelSettings,
}));

vi.mock("./services/terminalThemeApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/terminalThemeApiClient")>()),
  getNativeTerminalThemeSettings: terminalThemeApi.getNativeTerminalThemeSettings,
  upsertNativeTerminalThemeSettings: terminalThemeApi.upsertNativeTerminalThemeSettings,
}));

vi.mock("./services/sessionApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/sessionApiClient")>()),
  attachNativeSessionTask: sessionApi.attachNativeSessionTask,
  createNativeSession: sessionApi.createNativeSession,
  detachNativeSessionTask: sessionApi.detachNativeSessionTask,
  launchNativeAgentTerminal: sessionApi.launchNativeAgentTerminal,
  listNativeSessions: sessionApi.listNativeSessions,
  focusNativeSession: sessionApi.focusNativeSession,
  takeoverNativeSession: sessionApi.takeoverNativeSession,
  releaseNativeSession: sessionApi.releaseNativeSession,
  killNativeSession: sessionApi.killNativeSession,
  listNativeTerminalStreamChunks: sessionApi.listNativeTerminalStreamChunks,
  recordNativeSessionInput: sessionApi.recordNativeSessionInput,
  recordNativeTerminalStreamChunk: sessionApi.recordNativeTerminalStreamChunk,
}));

vi.mock("./services/projectApiClient", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./services/projectApiClient")>()),
  addNativeProject: projectApi.addNativeProject,
  listNativeProjects: projectApi.listNativeProjects,
  focusNativeProject: projectApi.focusNativeProject,
  updateNativeProjectLayout: projectApi.updateNativeProjectLayout,
  saveNativeProjectLayoutPreset: projectApi.saveNativeProjectLayoutPreset,
  listNativeProjectLayoutPresets: projectApi.listNativeProjectLayoutPresets,
  listNativeProjectFiles: projectApi.listNativeProjectFiles,
  readNativeProjectDiff: projectApi.readNativeProjectDiff,
  readNativeProjectFile: projectApi.readNativeProjectFile,
  collectNativeProjectLspDiagnostics: projectApi.collectNativeProjectLspDiagnostics,
  exportNativeProjectPatch: projectApi.exportNativeProjectPatch,
  importNativeProjectPatch: projectApi.importNativeProjectPatch,
  planNativeProjectDetach: projectApi.planNativeProjectDetach,
  planNativePrLanding: projectApi.planNativePrLanding,
  planNativeReviewPrLanding: projectApi.planNativeReviewPrLanding,
  searchNativeProjectFiles: projectApi.searchNativeProjectFiles,
  saveNativeProjectFile: projectApi.saveNativeProjectFile,
  upsertNativeProjectTabGroup: projectApi.upsertNativeProjectTabGroup,
}));

vi.mock("./components/TerminalPane", () => ({
  TerminalPane: ({
    session,
    highlighted,
    onRun,
    onInput,
    onClose,
    onSplit,
    onRendererDegraded,
    onOpenLink,
  }: {
    session: TerminalSession;
    highlighted?: boolean;
    onRun?: (session: TerminalSession) => void;
    onInput?: (session: TerminalSession, input: string) => void;
    onClose?: (session: TerminalSession) => void;
    onSplit?: (session: TerminalSession) => void;
    onRendererDegraded?: (session: TerminalSession, reason: string) => void;
    onOpenLink?: (url: string) => void;
  }) => (
    <article aria-label={`Terminal pane ${session.title}`} data-highlighted={highlighted ? "true" : "false"}>
      <span>{session.renderer.kind === "webgl" ? "WebGL" : session.renderer.kind === "canvas" ? "Fallback" : "DOM"}</span>
      {session.renderer.reason ? <span>{session.renderer.reason}</span> : null}
      {session.lines.some((line) => line.includes("http://localhost:3000/docs")) ? (
        <button type="button" aria-label={`Open terminal link ${session.title}`} onClick={() => onOpenLink?.("http://localhost:3000/docs")}>
          Open terminal link
        </button>
      ) : null}
      <button type="button" aria-label={`Split ${session.title}`} onClick={() => onSplit?.(session)}>
        Split
      </button>
      <button type="button" aria-label={`Degrade ${session.title}`} onClick={() => onRendererDegraded?.(session, "WebGL context lost")}>
        Degrade
      </button>
      <button type="button" aria-label={`Run ${session.title}`} onClick={() => onRun?.(session)}>
        Run
      </button>
      <button type="button" aria-label={`Submit ${session.title}`} onClick={() => onInput?.(session, "npm test\r")}>
        Submit
      </button>
      <button type="button" aria-label={`Dangerous ${session.title}`} onClick={() => onInput?.(session, "rm -rf /tmp/build\r")}>
        Dangerous
      </button>
      <button type="button" aria-label={`Close ${session.title}`} onClick={() => onClose?.(session)}>
        Close
      </button>
    </article>
  ),
}));

describe("Haneulchi app shell", () => {
  const storage = new Map<string, string>();

  beforeEach(() => {
    vi.resetAllMocks();
    resolveStateSnapshotMocksImmediately();
    terminalPty.outputHandler = undefined;
    vi.mocked(invoke).mockReturnValue(immediateRejected(new Error("Tauri unavailable in tests")));
    terminalPty.getTerminalPtySnapshot.mockReturnValue(immediateResolved({ total: 0, sessions: [] }));
    terminalPty.captureTerminalPtyCommand.mockReturnValue(immediateRejected(new Error("native terminal capture API unavailable")));
    terminalPty.listenToTerminalPtyOutput.mockImplementation((handler: (event: TerminalPtyOutputEvent) => void) => {
      terminalPty.outputHandler = handler;
      return immediateResolved(vi.fn());
    });
    terminalPty.spawnTerminalPtySession.mockReturnValue(immediateResolved({
      id: "pty_1",
      title: "1. haneulchi (zsh)",
      command: "sh",
      cols: 100,
      rows: 30,
    }));
    terminalPty.writeTerminalPtyInput.mockReturnValue(immediateResolved(undefined));
    terminalPty.closeTerminalPtySession.mockReturnValue(immediateResolved(undefined));
    stateSnapshot.getStateSnapshot.mockReturnValue(immediateResolved(defaultStateSnapshot()));
    taskApi.loadNativeTaskState.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.createNativeTask.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.createNativeReviewFollowUpTask.mockReturnValue(immediateRejected(new Error("native review follow-up API unavailable")));
    taskApi.moveNativeTask.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.addNativeTaskComment.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.addNativeTaskSubtask.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.createNativeTaskCycle.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.createNativeTaskModule.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.listNativeTaskComments.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.listNativeTaskCycles.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.listNativeTaskModules.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.listNativeTaskSubtasks.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.saveNativeTaskWorkpad.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.updateNativeTaskSubtaskStatus.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.updateNativeTaskPlanning.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    taskApi.updateNativeTaskContext.mockReturnValue(immediateRejected(new Error("native task API unavailable")));
    initiativeApi.listNativeInitiatives.mockReturnValue(immediateRejected(new Error("native initiative API unavailable")));
    initiativeApi.createNativeInitiative.mockReturnValue(immediateRejected(new Error("native initiative API unavailable")));
    commandBlockApi.upsertNativeCommandBlock.mockReturnValue(immediateRejected(new Error("native command block API unavailable")));
    commandBlockApi.searchNativeCommandBlocks.mockReturnValue(immediateRejected(new Error("native command block search unavailable")));
    commandBlockApi.markNativeCommandBlock.mockReturnValue(immediateRejected(new Error("native command block mark unavailable")));
    commandBlockApi.mergeNativeCommandBlocks.mockReturnValue(immediateRejected(new Error("native command block merge unavailable")));
    commandBlockApi.splitNativeCommandBlock.mockReturnValue(immediateRejected(new Error("native command block split unavailable")));
    commandBlockApi.attachNativeCommandBlockToEvidence.mockReturnValue(immediateRejected(new Error("native evidence API unavailable")));
    commandBlockApi.explainNativeCommandBlock.mockReturnValue(immediateRejected(new Error("native command block explain API unavailable")));
    commandBlockApi.exportNativeCommandBlockBundle.mockReturnValue(immediateRejected(new Error("native command block bundle API unavailable")));
    commandBlockApi.generateNativeEvidencePackForRun.mockReturnValue(immediateRejected(new Error("native evidence API unavailable")));
    commandBlockApi.recordNativeEvidenceReviewDecision.mockReturnValue(immediateRejected(new Error("native evidence API unavailable")));
    runApi.dispatchNativeRun.mockReturnValue(immediateRejected(new Error("native run API unavailable")));
    runApi.listNativeRuns.mockReturnValue(immediateRejected(new Error("native run API unavailable")));
    runApi.updateNativeRunLifecycle.mockReturnValue(immediateRejected(new Error("native run API unavailable")));
    runApi.cancelNativeRun.mockReturnValue(immediateRejected(new Error("native run API unavailable")));
    runApi.retryNativeRun.mockReturnValue(immediateRejected(new Error("native run API unavailable")));
    runApi.recordNativeRunStatusUpdate.mockReturnValue(immediateRejected(new Error("native run status API unavailable")));
    workflowApi.reloadWorkflow.mockReturnValue(immediateRejected(new Error("native workflow API unavailable")));
    workflowApi.getWorkflowRuntimeState.mockReturnValue(immediateRejected(new Error("native workflow API unavailable")));
    workflowApi.getRunReplayMetadata.mockReturnValue(immediateRejected(new Error("native run replay API unavailable")));
    workflowApi.validateWorkflow.mockReturnValue(immediateRejected(new Error("native workflow validation unavailable")));
    workflowApi.runWorkflowHook.mockReturnValue(immediateRejected(new Error("native workflow hook unavailable")));
    knowledgeApi.answerNativeKnowledgeQuestion.mockReturnValue(immediateRejected(new Error("native knowledge chat API unavailable")));
    knowledgeApi.exportNativeKnowledgeObsidianMarkdown.mockReturnValue(immediateRejected(new Error("native knowledge export API unavailable")));
    knowledgeApi.listNativeKnowledgeSources.mockReturnValue(immediateResolved([]));
    knowledgeApi.upsertNativeKnowledgeSource.mockReturnValue(immediateRejected(new Error("native knowledge API unavailable")));
    knowledgeApi.saveNativeKnowledgePage.mockReturnValue(immediateRejected(new Error("native knowledge API unavailable")));
    knowledgeApi.searchNativeKnowledgePages.mockReturnValue(immediateResolved([]));
    knowledgeApi.listNativeKnowledgeConcepts.mockReturnValue(immediateResolved([]));
    knowledgeApi.saveNativeKnowledgeExploration.mockReturnValue(immediateRejected(new Error("native knowledge exploration API unavailable")));
    knowledgeApi.listNativeKnowledgeExplorations.mockReturnValue(immediateResolved([]));
    knowledgeApi.listNativeContextPacks.mockReturnValue(immediateResolved([]));
    knowledgeApi.upsertNativeContextPack.mockReturnValue(immediateRejected(new Error("native context API unavailable")));
    knowledgeApi.recordNativeKnowledgeLintReport.mockReturnValue(immediateRejected(new Error("native knowledge lint unavailable")));
    knowledgeApi.runNativeKnowledgeAutomation.mockReturnValue(immediateRejected(new Error("native knowledge automation unavailable")));
    knowledgeApi.ingestNativeKnowledgeArtifact.mockReturnValue(immediateRejected(new Error("native knowledge ingestion unavailable")));
    budgetApi.ingestNativeTokenUsageAdapter.mockReturnValue(immediateRejected(new Error("native token usage adapter unavailable")));
    budgetApi.getNativeBudgetSummary.mockReturnValue(immediateRejected(new Error("native budget summary unavailable")));
    budgetApi.getNativeBudgetForecast.mockReturnValue(immediateRejected(new Error("native budget forecast unavailable")));
    budgetApi.listNativeProviderPrices.mockReturnValue(immediateRejected(new Error("native provider prices unavailable")));
    budgetApi.updateNativeProviderPriceTable.mockReturnValue(immediateRejected(new Error("native provider price update unavailable")));
    budgetApi.upsertNativeBudget.mockReturnValue(immediateRejected(new Error("native budget API unavailable")));
    budgetApi.recordNativeTokenUsage.mockReturnValue(immediateRejected(new Error("native token usage API unavailable")));
    secretApi.listNativeSecrets.mockReturnValue(immediateRejected(new Error("native secret API unavailable")));
    secretApi.upsertNativeSecret.mockReturnValue(immediateRejected(new Error("native secret API unavailable")));
    policyApi.createNativePolicyApproval.mockReturnValue(immediateRejected(new Error("native policy API unavailable")));
    policyApi.listNativePolicyApprovals.mockReturnValue(immediateRejected(new Error("native policy API unavailable")));
    policyApi.listNativePolicyPacks.mockReturnValue(immediateRejected(new Error("native policy pack API unavailable")));
    policyApi.listNativePermissionAudits.mockReturnValue(immediateRejected(new Error("native permission audit API unavailable")));
    policyApi.decideNativePolicyApproval.mockReturnValue(immediateRejected(new Error("native policy API unavailable")));
    policyApi.upsertNativePolicyPack.mockReturnValue(immediateRejected(new Error("native policy pack API unavailable")));
    policyApi.evaluateNativePolicyAction.mockReturnValue(immediateRejected(new Error("native policy evaluation API unavailable")));
    agentApi.listNativeAgentProfiles.mockReturnValue(immediateRejected(new Error("native agent API unavailable")));
    agentApi.listNativeRuntimePool.mockReturnValue(immediateRejected(new Error("native runtime pool API unavailable")));
    agentApi.listNativeSkillPacks.mockReturnValue(immediateRejected(new Error("native skill pack API unavailable")));
    agentApi.heartbeatNativeAgentProfile.mockReturnValue(immediateRejected(new Error("native agent heartbeat API unavailable")));
    agentApi.ingestNativeAgentEvents.mockReturnValue(immediateRejected(new Error("native agent event API unavailable")));
    agentApi.scanNativeAgentProfiles.mockReturnValue(immediateRejected(new Error("native agent API unavailable")));
    agentApi.upsertNativeAgentProfile.mockReturnValue(immediateRejected(new Error("native agent API unavailable")));
    agentApi.upsertNativeSkillPack.mockReturnValue(immediateRejected(new Error("native skill pack API unavailable")));
    agentApi.updateNativeAgentProfileStatus.mockReturnValue(immediateRejected(new Error("native agent API unavailable")));
    terminalThemeApi.getNativeTerminalThemeSettings.mockReturnValue(immediateRejected(new Error("native terminal theme API unavailable")));
    terminalThemeApi.upsertNativeTerminalThemeSettings.mockReturnValue(immediateRejected(new Error("native terminal theme API unavailable")));
    sessionApi.createNativeSession.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.launchNativeAgentTerminal.mockReturnValue(immediateRejected(new Error("native agent launch API unavailable")));
    sessionApi.listNativeSessions.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.focusNativeSession.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.takeoverNativeSession.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.releaseNativeSession.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.killNativeSession.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.attachNativeSessionTask.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.detachNativeSessionTask.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.listNativeTerminalStreamChunks.mockReturnValue(immediateRejected(new Error("native terminal transcript API unavailable")));
    sessionApi.recordNativeSessionInput.mockReturnValue(immediateRejected(new Error("native session API unavailable")));
    sessionApi.recordNativeTerminalStreamChunk.mockReturnValue(immediateRejected(new Error("native terminal transcript API unavailable")));
    projectApi.addNativeProject.mockReturnValue(immediateRejected(new Error("native project API unavailable")));
    projectApi.listNativeProjects.mockReturnValue(immediateRejected(new Error("native project API unavailable")));
    projectApi.focusNativeProject.mockReturnValue(immediateRejected(new Error("native project API unavailable")));
    projectApi.updateNativeProjectLayout.mockReturnValue(immediateResolved(undefined));
    projectApi.saveNativeProjectLayoutPreset.mockReturnValue(immediateRejected(new Error("native project layout preset API unavailable")));
    projectApi.listNativeProjectLayoutPresets.mockReturnValue(immediateRejected(new Error("native project layout preset API unavailable")));
    projectApi.listNativeProjectFiles.mockReturnValue(immediateRejected(new Error("native project files API unavailable")));
    projectApi.readNativeProjectDiff.mockReturnValue(immediateRejected(new Error("native project diff API unavailable")));
    projectApi.readNativeProjectFile.mockReturnValue(immediateRejected(new Error("native project file preview API unavailable")));
    projectApi.saveNativeProjectFile.mockReturnValue(immediateRejected(new Error("native project file save API unavailable")));
    projectApi.collectNativeProjectLspDiagnostics.mockReturnValue(immediateRejected(new Error("native LSP API unavailable")));
    projectApi.exportNativeProjectPatch.mockReturnValue(immediateRejected(new Error("native patch export API unavailable")));
    projectApi.importNativeProjectPatch.mockReturnValue(immediateRejected(new Error("native patch import API unavailable")));
    projectApi.planNativeProjectDetach.mockReturnValue(immediateRejected(new Error("native project detach API unavailable")));
    projectApi.planNativePrLanding.mockReturnValue(immediateRejected(new Error("native PR planning API unavailable")));
    projectApi.planNativeReviewPrLanding.mockReturnValue(immediateRejected(new Error("native review PR planning API unavailable")));
    projectApi.searchNativeProjectFiles.mockReturnValue(immediateRejected(new Error("native project file search API unavailable")));
    projectApi.upsertNativeProjectTabGroup.mockReturnValue(immediateRejected(new Error("native project tab group API unavailable")));
    browserAutomationApi.planNativeBrowserAutomation.mockReturnValue(immediateRejected(new Error("native browser automation API unavailable")));
    releaseWorkflowApi.getNativeReleaseWorkflowStatus.mockReturnValue(immediateRejected(new Error("native release workflow API unavailable")));
    qualityApi.listNativeBenchmarkRuns.mockReturnValue(immediateRejected(new Error("native benchmark API unavailable")));
    qualityApi.listNativeDmgSmokeRuns.mockReturnValue(immediateRejected(new Error("native DMG smoke API unavailable")));
    qualityApi.listNativeDogfoodTelemetryReviews.mockReturnValue(immediateRejected(new Error("native dogfood telemetry API unavailable")));
    qualityApi.listNativeRecoveryDrillRuns.mockReturnValue(immediateRejected(new Error("native recovery drill API unavailable")));
    qualityApi.listNativeReleaseGateRuns.mockReturnValue(immediateRejected(new Error("native release gate API unavailable")));
    qualityApi.listNativeTaskLifecycleE2ERuns.mockReturnValue(immediateRejected(new Error("native task lifecycle API unavailable")));
    qualityApi.listNativeTerminalFidelitySmokeRuns.mockReturnValue(immediateRejected(new Error("native terminal smoke API unavailable")));
    qualityApi.listNativeWorkflowNegativeTestRuns.mockReturnValue(immediateRejected(new Error("native workflow negative API unavailable")));
    qualityApi.runNativeBenchmarks.mockReturnValue(immediateRejected(new Error("native benchmark API unavailable")));
    qualityApi.runNativeDmgSmokeTest.mockReturnValue(immediateRejected(new Error("native DMG smoke API unavailable")));
    qualityApi.runNativeDogfoodTelemetryReview.mockReturnValue(immediateRejected(new Error("native dogfood telemetry API unavailable")));
    qualityApi.runNativeRecoveryDrills.mockReturnValue(immediateRejected(new Error("native recovery drill API unavailable")));
    qualityApi.runNativeReleaseGates.mockReturnValue(immediateRejected(new Error("native release gate API unavailable")));
    qualityApi.runNativeTaskLifecycleE2E.mockReturnValue(immediateRejected(new Error("native task lifecycle API unavailable")));
    qualityApi.runNativeTerminalFidelitySmoke.mockReturnValue(immediateRejected(new Error("native terminal smoke API unavailable")));
    qualityApi.runNativeWorkflowNegativeTests.mockReturnValue(immediateRejected(new Error("native workflow negative API unavailable")));
    visualHarnessApi.createNativeVisualHarnessLink.mockReturnValue(immediateRejected(new Error("native visual harness API unavailable")));
    visualHarnessApi.listNativeVisualHarnessLinks.mockReturnValue(immediateRejected(new Error("native visual harness API unavailable")));
    trackerApi.listNativeExternalTrackerBindings.mockReturnValue(immediateRejected(new Error("native tracker binding API unavailable")));
    trackerApi.upsertNativeExternalTrackerBinding.mockReturnValue(immediateRejected(new Error("native tracker binding API unavailable")));
    trackerApi.runNativeTrackerSync.mockReturnValue(immediateRejected(new Error("native tracker sync API unavailable")));
    storage.clear();
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: {
        getItem: (key: string) => storage.get(key) ?? null,
        setItem: (key: string, value: string) => storage.set(key, value),
        clear: () => storage.clear(),
      },
    });
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: vi.fn(async () => undefined),
      },
    });
    window.history.replaceState({}, "", "/");
  });

  afterEach(async () => {
    await flushAppEffects();
    cleanup();
    await flushAppEffects();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("renders the persistent shell regions from Sprint 1", () => {
    render(<App />);

    expect(screen.getByRole("banner", { name: /haneulchi titlebar/i })).toBeInTheDocument();
    expect(screen.getByRole("navigation", { name: /project workspace/i })).toBeInTheDocument();
    expect(screen.getByRole("main", { name: /terminal deck/i })).toBeInTheDocument();
    expect(screen.getByRole("complementary", { name: /attention and review rail/i })).toBeInTheDocument();
    expect(screen.getByRole("contentinfo", { name: /workspace status/i })).toBeInTheDocument();
  });

  it.each([
    ["control-tower", "Control Tower", ["KPI Strip", "Session Map", "Active Projects", "Orchestration Timeline"]],
    ["terminal-deck", "Terminal Deck", ["Terminal 1", "Terminal 2", "Terminal 3", "Terminal 4", "Bottom Logs"]],
    ["grid-split", "Grid Split", ["Terminal Logs", "Explorer", "Git", "Preview"]],
    ["explorer", "Explorer", ["Project Selector", "Branch Selector", "File Tree", "Summary", "Git Blame", "History"]],
    ["git", "Git", ["Fetch", "Compare", "Pull", "Commit Graph", "Changes", "Stash", "Pull Requests"]],
    ["inspector", "Inspector", ["Session", "Task", "Worktree", "Agent", "Environment", "Recent Commands", "Recent Events", "Artifacts"]],
    ["preview", "Preview", ["Routes", "Address", "Web Preview", "Markdown", "Diff Preview", "Preview Events", "Network"]],
    ["board", "Board", ["Board Selector", "New Task", "Backlog", "In Progress", "In Review", "Done"]],
  ])("renders the documented visual QA route /dev/%s", (slug, label, anchors) => {
    window.history.pushState({}, "", `/dev/${slug}`);

    render(<App />);

    const shell = screen.getByTestId("haneulchi-shell");
    expect(shell).toHaveAttribute("data-visual-qa-screen", slug);
    expect(screen.getByRole("tab", { name: label })).toHaveAttribute("aria-selected", "true");

    const workspace = within(screen.getByTestId("visual-qa-workspace")).getByTestId(`visual-qa-${slug}`);
    expect(workspace).toHaveAttribute("aria-label", `${label} reference screen`);
    anchors.forEach((anchor) => expect(workspace).toHaveTextContent(anchor));

    const rail = screen.getByRole("complementary", { name: /attention and review rail/i });
    const visualRailPanels = within(rail).getAllByTestId("visual-qa-right-rail-panel");
    expect(visualRailPanels.map((panel) => panel.getAttribute("aria-label"))).toEqual([
      "Attention Center",
      "Review Queue",
      "Recent Sessions",
    ]);
  });

  it("marks dynamic visual QA regions for screenshot masking", () => {
    window.history.pushState({}, "", "/dev/terminal-deck");
    const terminalRoute = render(<App />);

    expect(terminalRoute.container.querySelectorAll('[data-dynamic="terminal-output"]').length).toBeGreaterThan(0);
    expect(terminalRoute.container.querySelectorAll('[data-dynamic="timestamp"]').length).toBeGreaterThan(0);

    terminalRoute.unmount();
    window.history.pushState({}, "", "/dev/git");
    const gitRoute = render(<App />);

    expect(gitRoute.container.querySelectorAll('[data-dynamic="avatar"]').length).toBeGreaterThan(0);
  });

  it("maps concept asset filename aliases to visual QA routes", () => {
    window.history.pushState({}, "", "/dev/control_towel");

    render(<App />);

    expect(screen.getByTestId("haneulchi-shell")).toHaveAttribute("data-visual-qa-screen", "control-tower");
    expect(screen.getByRole("tab", { name: "Control Tower" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByTestId("visual-qa-control-tower")).toHaveTextContent("Session Map");
  });

  it("activates fallback workspace surface tabs without requiring a project tab", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));

    expect(screen.getByRole("tab", { name: "Preview" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByTestId("haneulchi-shell")).toHaveAttribute("data-visual-qa-screen", "preview");
    const workspace = within(screen.getByTestId("visual-qa-workspace")).getByTestId("visual-qa-preview");
    expect(workspace).toHaveTextContent("Web Preview");
    expect(workspace).toHaveTextContent("Preview Events");
    expect(workspace).toHaveTextContent("Network");
  });

  it("toggles the compact right rail drawer state", () => {
    render(<App />);

    const rail = screen.getByRole("complementary", { name: /attention and review rail/i });
    expect(rail).not.toHaveClass("is-open");

    fireEvent.click(screen.getByRole("button", { name: /toggle right rail/i }));
    expect(rail).toHaveClass("is-open");

    fireEvent.click(screen.getByRole("button", { name: /toggle right rail/i }));
    expect(rail).not.toHaveClass("is-open");
  });

  it("toggles the compact session stack rail with the command B shortcut", () => {
    render(<App />);

    const rail = screen.getByRole("complementary", { name: /attention and review rail/i });
    expect(rail).not.toHaveClass("is-open");

    fireEvent.keyDown(window, { key: "b", metaKey: true });
    expect(rail).toHaveClass("is-open");

    fireEvent.keyDown(window, { key: "b", metaKey: true });
    expect(rail).not.toHaveClass("is-open");
  });

  it("opens the Knowledge Vault rail with the command shift Y shortcut", () => {
    render(<App />);

    const rail = screen.getByRole("complementary", { name: /attention and review rail/i });
    expect(rail).not.toHaveClass("is-open");

    fireEvent.keyDown(window, { key: "Y", metaKey: true, shiftKey: true });

    expect(rail).toHaveClass("is-open");
    expect(screen.getByLabelText("Knowledge source path")).toHaveFocus();
  });

  it("surfaces readiness diagnostics and terminal panes", () => {
    render(<App />);

    expect(screen.getByText("Readiness")).toBeInTheDocument();
    expect(screen.getByText("Terminal Deck")).toBeInTheDocument();
    expect(screen.getAllByLabelText(/terminal pane \d/i)).toHaveLength(4);
    expect(screen.getByLabelText("Search command blocks")).toBeInTheDocument();
    expect(screen.getByText(/Evidence Pack/)).toBeInTheDocument();
  });

  it("surfaces terminal output listener registration failures", async () => {
    terminalPty.listenToTerminalPtyOutput.mockRejectedValueOnce(new Error("pty listener offline"));

    render(<App />);

    await waitFor(() => expect(terminalPty.listenToTerminalPtyOutput).toHaveBeenCalled());
    expect(await screen.findByText("Terminal output listener unavailable · pty listener offline")).toBeInTheDocument();
  });

  it("imports and exports a custom terminal theme", async () => {
    render(<App />);

    const themeJson = '{"name":"Nocturne Ops","background":"#101820","foreground":"#f2f7ff","accent":"#42e355"}';
    fireEvent.change(screen.getByLabelText("Terminal theme JSON"), { target: { value: themeJson } });
    fireEvent.click(screen.getByRole("button", { name: /import terminal theme/i }));

    expect(await screen.findByText(/Imported terminal theme Nocturne Ops/i)).toBeInTheDocument();
    expect(screen.getByRole("main", { name: /terminal deck/i })).toHaveAttribute("data-terminal-theme", "Nocturne Ops");
    expect(storage.get("haneulchi:terminal-theme")).toContain("Nocturne Ops");

    fireEvent.click(screen.getByRole("button", { name: /export terminal theme/i }));

    expect((screen.getByLabelText("Terminal theme JSON") as HTMLTextAreaElement).value).toContain("Nocturne Ops");
  });

  it("loads a project-specific terminal theme for the active project", async () => {
    window.localStorage.setItem(
      "haneulchi:terminal-theme:proj_auth",
      '{"name":"Auth Focus","background":"#09111f","foreground":"#eaf6ff","accent":"#19c37d"}',
    );
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_theme",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    await waitFor(() => expect(screen.getByRole("main", { name: /terminal deck/i })).toHaveAttribute("data-terminal-theme", "Auth Focus"));
  });

  it("loads a native project terminal theme for the active project", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_theme_native",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    terminalThemeApi.getNativeTerminalThemeSettings.mockImplementation(async (projectId?: string) =>
      projectId === "proj_auth"
        ? {
            project_id: "proj_auth",
            name: "Native Auth Focus",
            background: "#08111c",
            foreground: "#eef8ff",
            accent: "#42e355",
          }
        : {
            project_id: null,
            name: "Haneulchi Default",
            background: "#050607",
            foreground: "#d7ffe1",
            accent: "#42e355",
          },
    );

    render(<App />);

    await waitFor(() => expect(terminalThemeApi.getNativeTerminalThemeSettings).toHaveBeenCalledWith("proj_auth"));
    await waitFor(() =>
      expect(screen.getByRole("main", { name: /terminal deck/i })).toHaveAttribute("data-terminal-theme", "Native Auth Focus"),
    );
    expect(window.localStorage.getItem("haneulchi:terminal-theme:proj_auth")).toContain("Native Auth Focus");
  });

  it("saves the current terminal theme for the active project", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_theme_save",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    terminalThemeApi.upsertNativeTerminalThemeSettings.mockResolvedValueOnce({
      project_id: "proj_auth",
      name: "Auth Focus",
      background: "#09111f",
      foreground: "#eaf6ff",
      accent: "#19c37d",
    });

    render(<App />);

    const themeJson = '{"name":"Auth Focus","background":"#09111f","foreground":"#eaf6ff","accent":"#19c37d"}';
    fireEvent.change(screen.getByLabelText("Terminal theme JSON"), { target: { value: themeJson } });
    fireEvent.click(screen.getByRole("button", { name: /import terminal theme/i }));
    fireEvent.click(await screen.findByRole("button", { name: /save project terminal theme/i }));

    await waitFor(() =>
      expect(terminalThemeApi.upsertNativeTerminalThemeSettings).toHaveBeenCalledWith({
        projectId: "proj_auth",
        name: "Auth Focus",
        background: "#09111f",
        foreground: "#eaf6ff",
        accent: "#19c37d",
      }),
    );
    expect(await screen.findByText("Saved project terminal theme Auth Focus for Auth Service")).toBeInTheDocument();
    expect(window.localStorage.getItem("haneulchi:terminal-theme:proj_auth")).toContain("Auth Focus");
    expect(screen.getByRole("main", { name: /terminal deck/i })).toHaveAttribute("data-terminal-theme", "Auth Focus");
  });

  it("persists release channel settings and checks the selected update feed", async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        version: "0.1.0",
        pub_date: "2026-05-02T00:00:00Z",
        platforms: {
          "darwin-aarch64": {
            signature: "SIGNATURE_REQUIRED_FOR_RELEASE",
            url: "https://github.com/haneulchi/haneulchi/releases/download/v0.1.0-beta/Haneulchi_0.1.0_aarch64.dmg",
          },
        },
      }),
    })) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);

    render(<App />);

    fireEvent.change(screen.getByLabelText("Release channel"), { target: { value: "beta" } });
    fireEvent.click(screen.getByRole("button", { name: /check update feed/i }));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/update-feed/beta.json", { cache: "no-store" }));
    expect(await screen.findByText(/beta feed 0.1.0/i)).toBeInTheDocument();
    expect(screen.getByText(/signature placeholder/i)).toBeInTheDocument();
    expect(storage.get("haneulchi:release-channel")).toBe("beta");
  });

  it("marks update feeds without platform signatures as blocked for release publishing", async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        version: "0.1.0",
        platforms: {
          "darwin-aarch64": {
            url: "https://github.com/haneulchi/haneulchi/releases/download/v0.1.0/Haneulchi_0.1.0_aarch64.dmg",
          },
        },
      }),
    })) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /check update feed/i }));

    expect(await screen.findByText(/stable feed 0.1.0/i)).toBeInTheDocument();
    expect(screen.getByText("signature missing blocks release publishing")).toBeInTheDocument();
  });

  it("surfaces signed macOS, Homebrew cask, and crash symbol release workflow diagnostics", async () => {
    releaseWorkflowApi.getNativeReleaseWorkflowStatus.mockResolvedValueOnce({
      status: "warning",
      workflows: [
        {
          id: "macos_dmg",
          label: "Signed macOS DMG",
          script: "release:macos:dmg",
          configured: true,
          required_env: ["APPLE_CERTIFICATE", "APPLE_CERTIFICATE_PASSWORD", "APPLE_SIGNING_IDENTITY", "KEYCHAIN_PASSWORD"],
          missing_env: ["APPLE_CERTIFICATE", "APPLE_SIGNING_IDENTITY"],
          detail: ".github/workflows/release-macos.yml configured",
        },
        {
          id: "macos_notarization",
          label: "Apple notarization",
          script: "release:macos:notarize",
          configured: true,
          required_env: ["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"],
          missing_env: ["APPLE_ID"],
          detail: "scripts/release/notarize-macos.sh configured",
        },
        {
          id: "macos_artifact_verification",
          label: "macOS artifact verification",
          script: "release:macos:verify",
          configured: true,
          required_env: [],
          missing_env: [],
          detail: "scripts/release/verify-macos-artifacts.sh configured",
        },
        {
          id: "homebrew_cask",
          label: "Homebrew cask",
          script: "release:homebrew:cask",
          configured: true,
          required_env: ["DMG_URL", "HOMEBREW_TAP_REPOSITORY"],
          missing_env: ["DMG_URL"],
          detail: "scripts/release/render-homebrew-cask.sh configured",
        },
        {
          id: "crash_symbols",
          label: "Crash symbols",
          script: "release:symbols:upload",
          configured: true,
          required_env: ["SENTRY_AUTH_TOKEN", "SENTRY_ORG", "SENTRY_PROJECT"],
          missing_env: ["SENTRY_AUTH_TOKEN", "SENTRY_ORG"],
          detail: "scripts/release/upload-symbols.sh configured",
        },
      ],
    });

    render(<App />);

    const workflows = await screen.findByRole("region", { name: /release publishing workflows/i });
    fireEvent.click(within(workflows).getByRole("button", { name: /refresh release workflows/i }));

    await waitFor(() => expect(releaseWorkflowApi.getNativeReleaseWorkflowStatus).toHaveBeenCalled());
    expect(await within(workflows).findByText("Signed macOS DMG · release:macos:dmg · configured")).toBeInTheDocument();
    expect(within(workflows).getByText("Missing APPLE_CERTIFICATE, APPLE_SIGNING_IDENTITY")).toBeInTheDocument();
    expect(within(workflows).getByText("Apple notarization · release:macos:notarize · configured")).toBeInTheDocument();
    expect(within(workflows).getByText("Missing APPLE_ID")).toBeInTheDocument();
    expect(within(workflows).getByText("macOS artifact verification · release:macos:verify · configured")).toBeInTheDocument();
    expect(within(workflows).getByText("Required environment present")).toBeInTheDocument();
    expect(await within(workflows).findByText("Homebrew cask · release:homebrew:cask · configured")).toBeInTheDocument();
    expect(within(workflows).getByText("Missing DMG_URL")).toBeInTheDocument();
    expect(within(workflows).getByText("Crash symbols · release:symbols:upload · configured")).toBeInTheDocument();
    expect(within(workflows).getByText("Missing SENTRY_AUTH_TOKEN, SENTRY_ORG")).toBeInTheDocument();
    expect(within(workflows).getByText("Release workflows warning")).toBeInTheDocument();
  });

  it("frames unchecked release workflow diagnostics around signed macOS distribution", async () => {
    render(<App />);

    const workflows = await screen.findByRole("region", { name: /release publishing workflows/i });

    expect(
      within(workflows).getByText("Signed macOS DMG, notarization, Homebrew cask, and crash symbol workflow diagnostics not checked"),
    ).toBeInTheDocument();
  });

  it("clears stale release workflow diagnostics after failures and recovers on retry", async () => {
    releaseWorkflowApi.getNativeReleaseWorkflowStatus
      .mockResolvedValueOnce({
        status: "warning",
        workflows: [
          {
            id: "homebrew_cask",
            label: "Homebrew cask",
            script: "release:homebrew:cask",
            configured: true,
            required_env: ["DMG_URL"],
            missing_env: ["DMG_URL"],
            detail: "cask configured",
          },
        ],
      })
      .mockRejectedValueOnce(new Error("release workflow service down"))
      .mockResolvedValueOnce({
        status: "ok",
        workflows: [
          {
            id: "crash_symbols",
            label: "Crash symbols",
            script: "release:symbols:upload",
            configured: true,
            required_env: ["SENTRY_AUTH_TOKEN"],
            missing_env: [],
            detail: "symbols configured",
          },
        ],
      });

    render(<App />);

    const workflows = await screen.findByRole("region", { name: /release publishing workflows/i });
    fireEvent.click(within(workflows).getByRole("button", { name: /refresh release workflows/i }));
    expect(await within(workflows).findByText("Release workflows warning")).toBeInTheDocument();
    expect(workflows).toHaveTextContent("Missing DMG_URL");

    fireEvent.click(within(workflows).getByRole("button", { name: /refresh release workflows/i }));
    await waitFor(() => expect(workflows).toHaveTextContent("Release workflow diagnostics unavailable · release workflow service down"));
    expect(within(workflows).queryByText("Release workflows warning")).not.toBeInTheDocument();
    expect(workflows).not.toHaveTextContent("Missing DMG_URL");

    fireEvent.click(within(workflows).getByRole("button", { name: /refresh release workflows/i }));
    expect(await within(workflows).findByText("Release workflows ok")).toBeInTheDocument();
    expect(workflows).toHaveTextContent("Crash symbols · release:symbols:upload · configured");
    expect(workflows).not.toHaveTextContent("Release workflow diagnostics unavailable · release workflow service down");
  });

  it("renders persisted state sessions as navigable terminal panes", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_sessions_grid",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: Array.from({ length: 20 }, (_, index) => ({
        id: `session_${index + 1}`,
        project_id: "proj_auth",
        pane_id: `pane_session_${index + 1}`,
        mode: index % 2 === 0 ? "agent" : "shell",
        title: `Agent session ${index + 1}`,
        cwd: `/repo/auth/${index + 1}`,
        branch: "main",
        state: "running",
        attention_state: "none",
        token_budget_state: "ok",
      })),
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByLabelText("Terminal pane Agent session 20")).toBeInTheDocument();
    expect(screen.getAllByLabelText(/terminal pane agent session/i)).toHaveLength(20);
    expect(screen.getByRole("button", { name: /focus session_20/i })).toBeInTheDocument();
  });

  it("creates a persisted split session from a terminal pane", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_split",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_auth",
          pane_id: "pane_session_1",
          mode: "shell",
          title: "Auth shell",
          cwd: "/repo/auth",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.createNativeSession.mockResolvedValueOnce({
      id: "session_2",
      project_id: "proj_auth",
      pane_id: "pane_session_2",
      mode: "shell",
      title: "Auth shell split",
      cwd: "/repo/auth",
      branch: "main",
      agent_profile_id: null,
      task_id: null,
      run_id: null,
      state: "running",
      attention_state: "none",
      token_budget_state: "ok",
      created_at: "2026-04-30T01:01:00Z",
      updated_at: "2026-04-30T01:01:00Z",
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /split auth shell/i }));

    await waitFor(() =>
      expect(sessionApi.createNativeSession).toHaveBeenCalledWith({
        projectId: "proj_auth",
        mode: "shell",
        title: "Auth shell split",
        cwd: "/repo/auth",
        branch: "main",
      }),
    );
    expect(await screen.findByLabelText("Terminal pane Auth shell split")).toBeInTheDocument();
  });

  it("keeps a pane usable and visible when renderer health degrades", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_renderer_degraded",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_auth",
          pane_id: "pane_session_1",
          mode: "shell",
          title: "Auth shell",
          cwd: "/repo/auth",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const pane = await screen.findByLabelText("Terminal pane Auth shell");
    expect(within(pane).getByText("WebGL")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /degrade auth shell/i }));

    expect(await within(pane).findByText("Fallback")).toBeInTheDocument();
    expect(within(pane).getByText("Renderer degraded: WebGL context lost")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /run auth shell/i })).toBeInTheDocument();
  });

  it("creates a persisted terminal session with the command shortcut", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_new_terminal",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.createNativeSession.mockResolvedValueOnce({
      id: "session_1",
      project_id: "proj_auth",
      pane_id: "pane_session_1",
      mode: "shell",
      title: "Terminal session 1",
      cwd: null,
      branch: null,
      agent_profile_id: null,
      task_id: null,
      run_id: null,
      state: "running",
      attention_state: "none",
      token_budget_state: "ok",
      created_at: "2026-04-30T01:01:00Z",
      updated_at: "2026-04-30T01:01:00Z",
    });

    render(<App />);

    expect(await screen.findByRole("tab", { name: /auth service/i })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(window, { key: "t", metaKey: true });

    await waitFor(() =>
      expect(sessionApi.createNativeSession).toHaveBeenCalledWith({
        projectId: "proj_auth",
        mode: "shell",
        title: "Terminal session 1",
      }),
    );
    expect(await screen.findByLabelText("Terminal pane Terminal session 1")).toBeInTheDocument();
  });

  it("surfaces terminal session creation failures from the command shortcut", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_new_terminal_error",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.createNativeSession.mockRejectedValueOnce(new Error("session daemon offline"));

    render(<App />);

    expect(await screen.findByRole("tab", { name: /auth service/i })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(window, { key: "t", metaKey: true });

    await waitFor(() =>
      expect(sessionApi.createNativeSession).toHaveBeenCalledWith({
        projectId: "proj_auth",
        mode: "shell",
        title: "Terminal session 1",
      }),
    );
    expect(await screen.findByText("Terminal session unavailable · session daemon offline")).toBeInTheDocument();
  });

  it("creates a remote SSH terminal session from the terminal deck", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_ssh_create",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [{ id: "session_1", project_id: "proj_auth", mode: "agent", title: "Codex AUTH-104", state: "running" }],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.createNativeSession.mockResolvedValueOnce({
      id: "session_ssh",
      project_id: "proj_auth",
      pane_id: "pane_session_ssh",
      mode: "ssh",
      title: "SSH staging",
      cwd: "ssh://deploy@staging.example.com/srv/app",
      branch: "remote/main",
      agent_profile_id: null,
      task_id: null,
      run_id: null,
      state: "running",
      attention_state: "none",
      token_budget_state: "unknown",
      created_at: "2026-04-30T01:01:00Z",
      updated_at: "2026-04-30T01:01:00Z",
    });

    render(<App />);

    const sshPanel = await screen.findByRole("region", { name: /remote ssh terminal/i });
    fireEvent.change(within(sshPanel).getByLabelText("SSH session title"), { target: { value: "SSH staging" } });
    fireEvent.change(within(sshPanel).getByLabelText("SSH target"), { target: { value: "deploy@staging.example.com" } });
    fireEvent.change(within(sshPanel).getByLabelText("SSH remote path"), { target: { value: "/srv/app" } });
    fireEvent.change(within(sshPanel).getByLabelText("SSH branch label"), { target: { value: "remote/main" } });
    fireEvent.click(within(sshPanel).getByRole("button", { name: /create ssh terminal/i }));

    await waitFor(() =>
      expect(sessionApi.createNativeSession).toHaveBeenCalledWith({
        projectId: "proj_auth",
        mode: "ssh",
        title: "SSH staging",
        cwd: "ssh://deploy@staging.example.com/srv/app",
        branch: "remote/main",
      }),
    );
    expect(await screen.findByLabelText("Terminal pane SSH staging")).toBeInTheDocument();
    expect(sshPanel).toHaveTextContent("SSH session SSH staging ready");
  });

  it("runs remote SSH terminal panes through an ssh PTY command", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_ssh_run",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_ssh",
          project_id: "proj_auth",
          pane_id: "pane_session_ssh",
          mode: "ssh",
          title: "SSH staging",
          cwd: "ssh://deploy@staging.example.com/srv/app",
          branch: "remote/main",
          state: "running",
          attention_state: "none",
          token_budget_state: "unknown",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const sshPane = await screen.findByLabelText("Terminal pane SSH staging");
    fireEvent.click(within(sshPane).getByRole("button", { name: /run ssh staging/i }));

    await waitFor(() =>
      expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalledWith({
        title: "SSH staging",
        command: "ssh",
        args: ["-t", "deploy@staging.example.com", "cd '/srv/app' && exec ${SHELL:-sh} -l"],
        cols: 100,
        rows: 30,
      }),
    );
  });

  it("captures one-shot PTY commands from the Terminal Deck", async () => {
    terminalPty.captureTerminalPtyCommand.mockResolvedValueOnce({
      output: "Haneulchi capture ok\n",
      exitCode: 0,
      exitSuccess: true,
    });

    render(<App />);

    const capturePanel = await screen.findByRole("region", { name: /terminal command capture/i });
    fireEvent.change(within(capturePanel).getByLabelText("Capture PTY command"), { target: { value: "printf" } });
    fireEvent.change(within(capturePanel).getByLabelText("Capture PTY args"), { target: { value: "Haneulchi capture ok" } });
    fireEvent.click(within(capturePanel).getByRole("button", { name: "Capture PTY command output" }));

    await waitFor(() =>
      expect(terminalPty.captureTerminalPtyCommand).toHaveBeenCalledWith({
        command: "printf",
        args: ["Haneulchi", "capture", "ok"],
        cols: 120,
        rows: 30,
      }),
    );
    expect(await within(capturePanel).findByText("Capture exit 0 · success")).toBeInTheDocument();
    expect(within(capturePanel).getByText("Haneulchi capture ok")).toBeInTheDocument();
  });

  it("focuses next and previous terminal panes and maximizes the focused pane", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_terminal_focus",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [1, 2, 3].map((index) => ({
        id: `session_${index}`,
        project_id: "proj_auth",
        pane_id: `pane_session_${index}`,
        mode: "shell",
        title: `Shell ${index}`,
        cwd: "/repo/auth",
        branch: "main",
        state: "running",
        attention_state: "none",
        token_budget_state: "ok",
      })),
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByLabelText("Terminal pane Shell 1")).toHaveAttribute("data-highlighted", "true");
    fireEvent.click(screen.getByRole("button", { name: /focus next terminal/i }));
    expect(screen.getByLabelText("Terminal pane Shell 2")).toHaveAttribute("data-highlighted", "true");
    await waitFor(() =>
      expect(projectApi.updateNativeProjectLayout).toHaveBeenCalledWith("proj_auth", {
        mode: "grid",
        focusedSessionId: "session_2",
        maximizedSessionId: null,
        panes: ["session_1", "session_2", "session_3"],
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: /maximize focused terminal/i }));
    expect(screen.getByLabelText("Terminal pane Shell 2")).toBeInTheDocument();
    expect(screen.queryByLabelText("Terminal pane Shell 1")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Terminal pane Shell 3")).not.toBeInTheDocument();
    await waitFor(() =>
      expect(projectApi.updateNativeProjectLayout).toHaveBeenCalledWith("proj_auth", {
        mode: "maximized",
        focusedSessionId: "session_2",
        maximizedSessionId: "session_2",
        panes: ["session_1", "session_2", "session_3"],
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: /restore terminal grid/i }));
    fireEvent.click(screen.getByRole("button", { name: /focus previous terminal/i }));
    expect(screen.getByLabelText("Terminal pane Shell 1")).toHaveAttribute("data-highlighted", "true");
  });

  it("surfaces project terminal layout persistence failures while keeping the local layout", async () => {
    projectApi.updateNativeProjectLayout.mockRejectedValueOnce(new Error("layout store offline"));
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_terminal_layout_failure",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [1, 2, 3].map((index) => ({
        id: `session_${index}`,
        project_id: "proj_auth",
        pane_id: `pane_session_${index}`,
        mode: "shell",
        title: `Shell ${index}`,
        cwd: "/repo/auth",
        branch: "main",
        state: "running",
        attention_state: "none",
        token_budget_state: "ok",
      })),
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByLabelText("Terminal pane Shell 1")).toHaveAttribute("data-highlighted", "true");
    fireEvent.click(screen.getByRole("button", { name: /focus next terminal/i }));

    expect(screen.getByLabelText("Terminal pane Shell 2")).toHaveAttribute("data-highlighted", "true");
    await waitFor(() =>
      expect(projectApi.updateNativeProjectLayout).toHaveBeenCalledWith("proj_auth", {
        mode: "grid",
        focusedSessionId: "session_2",
        maximizedSessionId: null,
        panes: ["session_1", "session_2", "session_3"],
      }),
    );
    expect(await screen.findByText("Project layout saved locally · layout store offline")).toBeInTheDocument();
  });

  it("restores the saved terminal layout for the active project tab", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_saved_layout",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [
        {
          id: "tab_proj_auth",
          project_id: "proj_auth",
          label: "Auth Service",
          active: true,
          layout_json: {
            mode: "maximized",
            focusedSessionId: "session_2",
            maximizedSessionId: "session_2",
            panes: ["session_1", "session_2"],
          },
        },
      ],
      sessions: [1, 2].map((index) => ({
        id: `session_${index}`,
        project_id: "proj_auth",
        pane_id: `pane_session_${index}`,
        mode: "shell",
        title: `Shell ${index}`,
        cwd: "/repo/auth",
        branch: "main",
        state: "running",
        attention_state: "none",
        token_budget_state: "ok",
      })),
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    await waitFor(() => expect(screen.getByLabelText("Terminal pane Shell 2")).toHaveAttribute("data-highlighted", "true"));
    await waitFor(() => expect(screen.queryByLabelText("Terminal pane Shell 1")).not.toBeInTheDocument());
  });

  it("saves and reapplies named project layout presets", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_layout_presets",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [1, 2].map((index) => ({
        id: `session_${index}`,
        project_id: "proj_auth",
        pane_id: `pane_session_${index}`,
        mode: "shell",
        title: `Shell ${index}`,
        cwd: "/repo/auth",
        branch: "main",
        state: "running",
        attention_state: "none",
        token_budget_state: "ok",
      })),
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    fireEvent.click(screen.getByRole("button", { name: /focus next terminal/i }));
    await waitFor(() =>
      expect(projectApi.updateNativeProjectLayout).toHaveBeenCalledWith("proj_auth", {
        mode: "grid",
        focusedSessionId: "session_2",
        maximizedSessionId: null,
        panes: ["session_1", "session_2"],
      }),
    );

    fireEvent.change(within(projectWindowTabs).getByLabelText("Layout preset name"), { target: { value: "Review grid" } });
    projectApi.saveNativeProjectLayoutPreset.mockResolvedValueOnce({
      id: "layout_preset_1",
      project_id: "proj_auth",
      name: "Review grid",
      layout_json: {
        mode: "grid",
        focusedSessionId: "session_2",
        maximizedSessionId: null,
        panes: ["session_1", "session_2"],
      },
    });
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /save layout preset/i }));
    await waitFor(() =>
      expect(projectApi.saveNativeProjectLayoutPreset).toHaveBeenCalledWith("proj_auth", "Review grid", {
        mode: "grid",
        focusedSessionId: "session_2",
        maximizedSessionId: null,
        panes: ["session_1", "session_2"],
      }),
    );
    expect(await within(projectWindowTabs).findByText("Saved layout preset Review grid · native")).toBeInTheDocument();
    expect(within(projectWindowTabs).getByRole("button", { name: /apply layout preset review grid/i })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /maximize focused terminal/i }));
    await waitFor(() =>
      expect(projectApi.updateNativeProjectLayout).toHaveBeenCalledWith("proj_auth", {
        mode: "maximized",
        focusedSessionId: "session_2",
        maximizedSessionId: "session_2",
        panes: ["session_1", "session_2"],
      }),
    );
    expect(screen.queryByLabelText("Terminal pane Shell 1")).not.toBeInTheDocument();

    projectApi.updateNativeProjectLayout.mockClear();
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /apply layout preset review grid/i }));

    expect(screen.getByLabelText("Terminal pane Shell 1")).toBeInTheDocument();
    expect(screen.getByLabelText("Terminal pane Shell 2")).toHaveAttribute("data-highlighted", "true");
    await waitFor(() =>
      expect(projectApi.updateNativeProjectLayout).toHaveBeenCalledWith("proj_auth", {
        mode: "grid",
        focusedSessionId: "session_2",
        maximizedSessionId: null,
        panes: ["session_1", "session_2"],
      }),
    );
  });

  it("loads persisted native project layout presets into the project window", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_layout_preset_load",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_auth",
          pane_id: "pane_session_1",
          mode: "shell",
          title: "Shell 1",
          cwd: "/repo/auth",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectLayoutPresets.mockResolvedValueOnce([
      {
        id: "layout_preset_1",
        project_id: "proj_auth",
        name: "Review grid",
        layout_json: {
          mode: "grid",
          focusedSessionId: "session_1",
          maximizedSessionId: null,
          panes: ["session_1"],
        },
      },
    ]);

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /load layout presets/i }));

    await waitFor(() => expect(projectApi.listNativeProjectLayoutPresets).toHaveBeenCalledWith("proj_auth"));
    expect(await within(projectWindowTabs).findByRole("button", { name: /apply layout preset review grid/i })).toBeInTheDocument();
    expect(within(projectWindowTabs).getByText("Loaded 1 layout preset")).toBeInTheDocument();
  });

  it("surfaces state snapshot health from the control-plane command", async () => {
    render(<App />);

    expect(await screen.findByText(/State Snapshot snap_test/i)).toBeInTheDocument();
    expect(screen.getByText(/DB degraded · PTY ok · API ok/i)).toBeInTheDocument();
    expect(screen.getByText("Database persistence not configured")).toBeInTheDocument();
  });

  it("renders a global attention queue from snapshot, session, and run signals", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_attention_queue",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_attention",
          project_id: "proj_auth",
          mode: "agent",
          title: "Auth agent",
          state: "running",
          attention_state: "needs_input",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_waiting",
            task_id: "task_auth",
            project_id: "proj_auth",
            lifecycle: "waiting_input",
            retry_count: 0,
            status_detail: "Waiting on harness confirmation",
          },
          {
            id: "run_blocked",
            task_id: "task_auth",
            project_id: "proj_auth",
            lifecycle: "blocked",
            retry_count: 0,
            status_detail: "Blocked by release gate",
          },
        ],
        counts_by_lifecycle: { waiting_input: 1, blocked: 1 },
      },
      agents: [],
      reviews: [],
      attention: [
        {
          id: "policy_approval_1",
          label: "Policy approval required: shell_command",
          severity: "critical",
          detail: "rm -rf build/cache",
        },
      ],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const queue = await screen.findByRole("region", { name: /global attention queue/i });
    expect(queue).toHaveTextContent("Critical 2");
    expect(queue).toHaveTextContent("Warning 2");
    expect(queue).toHaveTextContent("Policy approval required: shell_command");
    expect(queue).toHaveTextContent("Session Auth agent needs_input");
    expect(queue).toHaveTextContent("Run run_waiting waiting input");
    expect(queue).toHaveTextContent("Waiting on harness confirmation");
    expect(queue).toHaveTextContent("Run run_blocked blocked");
  });

  it("opens a notification drawer and jumps to the unread session", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_notifications",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_idle",
          project_id: "proj_auth",
          mode: "shell",
          title: "Idle Shell",
          state: "running",
          attention_state: "none",
        },
        {
          id: "session_unread",
          project_id: "proj_auth",
          mode: "agent",
          title: "Codex AUTH-104",
          state: "running",
          attention_state: "unread",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [
        {
          id: "policy_approval_1",
          label: "Policy approval required: shell_command",
          severity: "critical",
          detail: "rm -rf build/cache",
        },
      ],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByRole("article", { name: "Terminal pane Codex AUTH-104" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Show notifications" }));

    const drawer = await screen.findByRole("dialog", { name: /notifications/i });
    expect(drawer).toHaveTextContent("Unread sessions 1");
    expect(drawer).toHaveTextContent("Session Codex AUTH-104 unread");
    expect(drawer).toHaveTextContent("Policy approval required: shell_command");

    fireEvent.click(screen.getByRole("button", { name: "Jump to unread Codex AUTH-104" }));

    expect(screen.queryByRole("dialog", { name: /notifications/i })).not.toBeInTheDocument();
    expect(screen.getByText("Focused Codex AUTH-104")).toBeInTheDocument();
  });

  it("jumps to the most recent unread session with command shift U", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_notifications_shortcut",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_idle",
          project_id: "proj_auth",
          mode: "shell",
          title: "Idle Shell",
          state: "running",
          attention_state: "none",
        },
        {
          id: "session_unread",
          project_id: "proj_auth",
          mode: "agent",
          title: "Codex AUTH-104",
          state: "running",
          attention_state: "unread",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByRole("article", { name: "Terminal pane Codex AUTH-104" })).toBeInTheDocument();
    expect(screen.queryByRole("dialog", { name: /notifications/i })).not.toBeInTheDocument();

    fireEvent.keyDown(window, { key: "U", metaKey: true, shiftKey: true });

    expect(await screen.findByText("Focused Codex AUTH-104")).toBeInTheDocument();
  });

  it("renders agent notification rings in the drawer and directory", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_agent_attention",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [
        {
          id: "agent_codex",
          label: "Codex",
          available: true,
          latest_event_kind: "status",
          latest_event_detail: "Waiting for review",
          attention_state: "needs_input",
          attention_severity: "warning",
          notification_count: 2,
        },
      ],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("Codex · available · needs_input")).toBeInTheDocument();
    expect(screen.getByText("2 notifications · status · Waiting for review")).toBeInTheDocument();
    expect(screen.getByLabelText("Agent agent_codex attention warning")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Show notifications" }));

    const drawer = await screen.findByRole("dialog", { name: /notifications/i });
    expect(drawer).toHaveTextContent("Agent Codex needs_input");
    expect(drawer).toHaveTextContent("2 notifications · Waiting for review");
  });

  it("surfaces degraded notification health in the drawer", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_notifications_degraded",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "degraded" },
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Show notifications" }));
    const drawer = await screen.findByRole("dialog", { name: /notifications/i });
    expect(drawer).toHaveTextContent("Notifications degraded · api degraded");
    expect(drawer).toHaveTextContent("No unread notifications");
    expect(screen.getByRole("button", { name: "Jump to unread" })).toBeDisabled();
  });

  it("renders historical analytics charts from snapshot run review and budget signals", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_analytics",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          { id: "run_queued", task_id: "task_1", project_id: "proj_local", lifecycle: "queued", retry_count: 0 },
          { id: "run_blocked", task_id: "task_2", project_id: "proj_local", lifecycle: "blocked", retry_count: 0 },
          { id: "run_review", task_id: "task_3", project_id: "proj_local", lifecycle: "review_ready", retry_count: 0 },
        ],
        counts_by_lifecycle: { queued: 1, blocked: 1, review_ready: 1 },
      },
      agents: [],
      reviews: [
        { id: "review_complete", evidence_pack_id: "ev_1", state: "pending", completeness_state: "complete" },
        { id: "review_incomplete", evidence_pack_id: "ev_2", state: "changes_requested", completeness_state: "incomplete" },
      ],
      attention: [],
      budgets: {
        workspace: { scope_type: "workspace", used_usd: 8.5, max_usd: 10, state: "warn" },
        projects: [],
        agents: [],
      },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const charts = await screen.findByRole("region", { name: /historical analytics charts/i });
    expect(charts).toHaveTextContent("Samples 1");
    expect(charts).toHaveTextContent("Run health 3 total");
    expect(charts).toHaveTextContent("queued 1");
    expect(charts).toHaveTextContent("blocked 1");
    expect(charts).toHaveTextContent("review ready 1");
    expect(charts).toHaveTextContent("Evidence 2 reviews");
    expect(charts).toHaveTextContent("complete 1");
    expect(charts).toHaveTextContent("incomplete 1");
    expect(charts).toHaveTextContent("Budget warn · $8.50 / $10.00");
  });

  it("surfaces degraded state for historical analytics charts", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_analytics_degraded",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "degraded", pty: "ok", api: "degraded" },
    });

    render(<App />);

    const charts = await screen.findByRole("region", { name: /historical analytics charts/i });
    expect(charts).toHaveTextContent("Analytics degraded · api degraded · db degraded");
    expect(charts).toHaveTextContent("No run samples");
    expect(charts).toHaveTextContent("No review samples");
  });

  it("customizes Control Tower dashboard widget visibility from the right rail", async () => {
    render(<App />);

    const customizer = await screen.findByRole("region", { name: /dashboard widgets/i });
    expect(customizer).toHaveTextContent("Agent Team visible");
    expect(customizer).toHaveTextContent("Historical Analytics visible");
    expect(customizer).toHaveTextContent("Recent Evidence visible");
    expect(screen.getByRole("region", { name: /agent team mini dashboard/i })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: /historical analytics charts/i })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: /recent evidence activity/i })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("checkbox", { name: "Show Recent Evidence widget" }));

    expect(customizer).toHaveTextContent("Recent Evidence hidden");
    expect(screen.queryByRole("region", { name: /recent evidence activity/i })).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: /agent team mini dashboard/i })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: /historical analytics charts/i })).toBeInTheDocument();
  });

  it("shows an empty dashboard widget state when all optional widgets are hidden", async () => {
    render(<App />);

    const customizer = await screen.findByRole("region", { name: /dashboard widgets/i });
    fireEvent.click(screen.getByRole("checkbox", { name: "Show Agent Team widget" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Show Historical Analytics widget" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Show Recent Evidence widget" }));

    expect(customizer).toHaveTextContent("All optional dashboard widgets hidden");
    expect(screen.queryByRole("region", { name: /agent team mini dashboard/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("region", { name: /historical analytics charts/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("region", { name: /recent evidence activity/i })).not.toBeInTheDocument();
  });

  it("renders a Control Tower ops strip for workflow poll budget and knowledge", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_ops_strip",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          { id: "run_blocked_1", task_id: "task_1", project_id: "proj_auth", lifecycle: "blocked", retry_count: 0 },
          { id: "run_blocked_2", task_id: "task_2", project_id: "proj_auth", lifecycle: "blocked", retry_count: 0 },
          { id: "run_review_1", task_id: "task_3", project_id: "proj_auth", lifecycle: "review_ready", retry_count: 0 },
        ],
        counts_by_lifecycle: { blocked: 2, review_ready: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: {
        workspace: { used_usd: 2.5, state: "ok" },
        projects: [{ scope_id: "proj_auth", used_usd: 8.5, max_usd: 10, state: "warn" }],
        agents: [],
      },
      workflow: {
        valid: false,
        invalid_projects: ["proj_auth"],
        current_version_id: "workflow_7",
        last_known_good_version_id: "workflow_6",
        diagnostics: { errors: [] },
      },
      knowledge: { stale_count: 2, gap_count: 1, recent_pages: ["auth-flow"] },
      health: { db: "ok", pty: "ok", api: "degraded" },
    });

    render(<App />);

    const strip = await screen.findByRole("region", { name: /control tower ops strip/i });
    expect(strip).toHaveTextContent("Poll API degraded");
    expect(strip).toHaveTextContent("Blocked 2");
    expect(strip).toHaveTextContent("Review 1");
    expect(strip).toHaveTextContent("Budget warn · $8.50 / $10.00");
    expect(strip).toHaveTextContent("Workflow invalid 1");
    expect(strip).toHaveTextContent("Knowledge 2 stale · 1 gaps");
  });

  it("uses state snapshot projects in the workspace sidebar and focuses them", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_projects",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [{ id: "session_1", project_id: "proj_auth", mode: "agent", title: "Codex AUTH-104", state: "running" }],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.focusNativeProject.mockResolvedValueOnce({ id: "proj_docs", name: "Docs Workspace", status: "active" });

    render(<App />);

    expect(await screen.findByRole("button", { name: /focus project auth service/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /focus project docs workspace/i })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /focus project docs workspace/i }));

    await waitFor(() => expect(projectApi.focusNativeProject).toHaveBeenCalledWith("proj_docs"));
  });

  it("reloads a project-scoped state snapshot after focusing a workspace project", async () => {
    stateSnapshot.getStateSnapshot
      .mockResolvedValueOnce({
        snapshot_id: "snap_projects",
        generated_at: "2026-04-30T01:00:00Z",
        app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
        projects: [
          { id: "proj_auth", name: "Auth Service", state: "active" },
          { id: "proj_docs", name: "Docs Workspace", state: "idle" },
        ],
        project_tabs: [
          { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true },
          { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: false },
        ],
        sessions: [],
        command_blocks: { recent: [], unread_count: 0 },
        tasks: { items: [], counts_by_status: {} },
        runs: { items: [], counts_by_lifecycle: {} },
        agents: [],
        reviews: [],
        attention: [],
        budgets: { workspace: {}, projects: [], agents: [] },
        workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
        knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
        health: { db: "ok", pty: "ok", api: "ok" },
      })
      .mockResolvedValueOnce({
        snapshot_id: "snap_docs",
        generated_at: "2026-04-30T01:01:00Z",
        app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
        projects: [
          { id: "proj_auth", name: "Auth Service", state: "idle" },
          { id: "proj_docs", name: "Docs Workspace", state: "active" },
        ],
        project_tabs: [
          { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: false },
          { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: true },
        ],
        sessions: [],
        command_blocks: { recent: [], unread_count: 0 },
        tasks: {
          items: [{ id: "task_docs", title: "Refresh docs", status: "ready", priority: "high", project_id: "proj_docs" }],
          counts_by_status: { ready: 1 },
        },
        runs: { items: [], counts_by_lifecycle: {} },
        agents: [],
        reviews: [],
        attention: [],
        budgets: { workspace: {}, projects: [], agents: [] },
        workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
        knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
        health: { db: "ok", pty: "ok", api: "ok" },
      });
    projectApi.focusNativeProject.mockResolvedValueOnce({ id: "proj_docs", name: "Docs Workspace", status: "active" });

    render(<App />);

    await waitFor(() => expect(screen.getByRole("button", { name: /focus project docs workspace/i })).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: /focus project docs workspace/i }));

    await waitFor(() => expect(projectApi.focusNativeProject).toHaveBeenCalledWith("proj_docs"));
    await waitFor(() => expect(stateSnapshot.getStateSnapshot).toHaveBeenCalledWith("proj_docs"));
    expect(await screen.findByText("State Snapshot snap_docs")).toBeInTheDocument();
  });

  it("surfaces project focus failures in the workspace sidebar", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_focus_error",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [{ id: "session_1", project_id: "proj_auth", mode: "agent", title: "Codex AUTH-104", state: "running" }],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.focusNativeProject.mockRejectedValueOnce(new Error("project daemon offline"));

    render(<App />);

    expect(await screen.findByRole("button", { name: /focus project auth service/i })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /focus project docs workspace/i }));

    await waitFor(() => expect(projectApi.focusNativeProject).toHaveBeenCalledWith("proj_docs"));
    expect(await screen.findByText("Project control unavailable · project daemon offline")).toBeInTheDocument();
  });

  it("adds a project from the project registry form", async () => {
    projectApi.addNativeProject.mockResolvedValueOnce({
      id: "proj_docs",
      key: "DOCS",
      name: "Docs Workspace",
      path: "/repo/docs",
      color: "#0ea5e9",
      status: "active",
      created_at: "2026-05-05T03:00:00Z",
      updated_at: "2026-05-05T03:00:00Z",
    });

    render(<App />);

    fireEvent.change(screen.getByLabelText("New project key"), { target: { value: "DOCS" } });
    fireEvent.change(screen.getByLabelText("New project name"), { target: { value: "Docs Workspace" } });
    fireEvent.change(screen.getByLabelText("New project path"), { target: { value: "/repo/docs" } });
    fireEvent.change(screen.getByLabelText("New project color"), { target: { value: "#0ea5e9" } });
    fireEvent.click(screen.getByRole("button", { name: "Add project" }));

    await waitFor(() =>
      expect(projectApi.addNativeProject).toHaveBeenCalledWith({
        key: "DOCS",
        name: "Docs Workspace",
        path: "/repo/docs",
        color: "#0ea5e9",
      }),
    );
    expect(await screen.findByRole("button", { name: /focus project docs workspace/i })).toBeInTheDocument();
  });

  it("loads native projects into the project registry", async () => {
    projectApi.listNativeProjects.mockResolvedValueOnce([
      {
        id: "proj_auth",
        key: "AUTH",
        name: "Auth Service",
        path: "/repo/auth",
        color: "#2563eb",
        status: "idle",
        created_at: "2026-05-05T02:00:00Z",
        updated_at: "2026-05-05T02:00:00Z",
      },
      {
        id: "proj_docs",
        key: "DOCS",
        name: "Docs Workspace",
        path: "/repo/docs",
        color: "#0ea5e9",
        status: "active",
        created_at: "2026-05-05T03:00:00Z",
        updated_at: "2026-05-05T03:00:00Z",
      },
    ]);

    render(<App />);

    const workspace = await screen.findByRole("navigation", { name: /project workspace/i });
    fireEvent.click(within(workspace).getByRole("button", { name: /load projects/i }));

    await waitFor(() => expect(projectApi.listNativeProjects).toHaveBeenCalledWith());
    expect(await within(workspace).findByRole("button", { name: /focus project auth service/i })).toBeInTheDocument();
    expect(within(workspace).getByRole("button", { name: /focus project docs workspace/i })).toBeInTheDocument();
    expect(within(workspace).getAllByText("active").length).toBeGreaterThan(0);
  });

  it("renders Control Tower project cards with per-project ops signals", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_control_tower_projects",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        {
          id: "proj_auth",
          name: "Auth Service",
          state: "active",
          token_usage: {
            input_tokens: 900,
            output_tokens: 600,
            total_tokens: 1500,
            cost_usd: 1.25,
          },
        },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_auth_1",
          project_id: "proj_auth",
          mode: "agent",
          title: "Auth agent",
          state: "running",
          attention_state: "needs_input",
        },
        {
          id: "session_auth_2",
          project_id: "proj_auth",
          mode: "shell",
          title: "Auth shell",
          state: "running",
          attention_state: "none",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_auth_review",
            task_id: "task_auth",
            project_id: "proj_auth",
            lifecycle: "review_ready",
            retry_count: 0,
          },
          {
            id: "run_auth_failed",
            task_id: "task_auth",
            project_id: "proj_auth",
            lifecycle: "failed",
            retry_count: 1,
          },
          {
            id: "run_docs_done",
            task_id: "task_docs",
            project_id: "proj_docs",
            lifecycle: "completed",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { review_ready: 1, failed: 1, completed: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const cards = await screen.findByRole("region", { name: /control tower project cards/i });
    expect(cards).toHaveTextContent("Auth Service");
    expect(cards).toHaveTextContent("active");
    expect(cards).toHaveTextContent("2 sessions");
    expect(cards).toHaveTextContent("2 runs");
    expect(cards).toHaveTextContent("2 alerts");
    expect(cards).toHaveTextContent("$1.25 · 1500 tokens");
    expect(cards).toHaveTextContent("Docs Workspace");
    expect(cards).toHaveTextContent("0 sessions");
    expect(cards).toHaveTextContent("0 alerts");
  });

  it("uses project tabs from the state snapshot for workspace switching", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_tabs",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.focusNativeProject.mockResolvedValueOnce({ id: "proj_docs", name: "Docs Workspace", status: "active" });

    render(<App />);

    expect(await screen.findByRole("tab", { name: /auth service/i })).toHaveAttribute("aria-selected", "true");
    fireEvent.click(screen.getByRole("tab", { name: /docs workspace/i }));

    await waitFor(() => expect(projectApi.focusNativeProject).toHaveBeenCalledWith("proj_docs"));
  });

  it("switches workspace project tabs with command number shortcuts", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_tab_shortcuts",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.focusNativeProject.mockResolvedValueOnce({ id: "proj_docs", name: "Docs Workspace", status: "active" });

    render(<App />);

    expect(await screen.findByRole("tab", { name: /auth service/i })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(window, { key: "2", metaKey: true });

    await waitFor(() => expect(projectApi.focusNativeProject).toHaveBeenCalledWith("proj_docs"));
  });

  it("persists project tab group assignments for the active project", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_tab_groups",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    expect(projectWindowTabs).toHaveTextContent("Auth Service · active · ungrouped");

    fireEvent.change(within(projectWindowTabs).getByLabelText("Project tab group name"), {
      target: { value: "Backend" },
    });
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /save project tab group/i }));

    expect(projectWindowTabs).toHaveTextContent("Auth Service · active · Backend");
    expect(JSON.parse(window.localStorage.getItem("haneulchi:project-tab-groups") ?? "[]")).toContainEqual({
      projectId: "proj_auth",
      group: "Backend",
    });
  });

  it("persists project tab group assignments through the native project API when available", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_tab_groups_native",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.upsertNativeProjectTabGroup.mockResolvedValueOnce({
      project_id: "proj_auth",
      group_name: "Backend",
      updated_at: "2026-05-03T01:00:00Z",
    });

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    fireEvent.change(within(projectWindowTabs).getByLabelText("Project tab group name"), {
      target: { value: "Backend" },
    });
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /save project tab group/i }));

    await waitFor(() => expect(projectApi.upsertNativeProjectTabGroup).toHaveBeenCalledWith("proj_auth", "Backend"));
  });

  it("surfaces native project tab group persistence failures while keeping the local assignment", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_tab_groups_native_error",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.upsertNativeProjectTabGroup.mockRejectedValueOnce(new Error("project tab group store offline"));

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    fireEvent.change(within(projectWindowTabs).getByLabelText("Project tab group name"), {
      target: { value: "Backend" },
    });
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /save project tab group/i }));

    expect(projectWindowTabs).toHaveTextContent("Auth Service · active · Backend");
    await waitFor(() => expect(projectApi.upsertNativeProjectTabGroup).toHaveBeenCalledWith("proj_auth", "Backend"));
    expect(await within(projectWindowTabs).findByText("Project tab group saved locally · project tab group store offline")).toBeInTheDocument();
  });

  it("records a local detach plan for the active project when project APIs are degraded", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_detach",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "degraded" },
    });

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    expect(projectWindowTabs).toHaveTextContent("Project window degraded · api degraded · db ok");

    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /detach active project/i }));

    expect(await within(projectWindowTabs).findByText("Detached Auth Service to window win_proj_auth")).toBeInTheDocument();
    expect(JSON.parse(window.localStorage.getItem("haneulchi:project-detach-plans") ?? "[]")).toContainEqual({
      projectId: "proj_auth",
      projectName: "Auth Service",
      windowId: "win_proj_auth",
      status: "planned",
    });
  });

  it("plans project detach through the native project API when available", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_detach_native",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.planNativeProjectDetach.mockResolvedValueOnce({
      project_id: "proj_auth",
      project_name: "Auth Service",
      window_id: "win_proj_auth",
      status: "planned",
      degraded_reason: null,
    });

    render(<App />);

    const projectWindowTabs = await screen.findByRole("region", { name: /project window tabs/i });
    fireEvent.click(within(projectWindowTabs).getByRole("button", { name: /detach active project/i }));

    await waitFor(() => expect(projectApi.planNativeProjectDetach).toHaveBeenCalledWith("proj_auth"));
    expect(await within(projectWindowTabs).findByText("Detached Auth Service to window win_proj_auth")).toBeInTheDocument();
    expect(JSON.parse(window.localStorage.getItem("haneulchi:project-detach-plans") ?? "[]")).toContainEqual({
      projectId: "proj_auth",
      projectName: "Auth Service",
      windowId: "win_proj_auth",
      status: "planned",
    });
  });

  it("renders active project file explorer entries with git status badges", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_files",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [
        { name: "src", path: "src", kind: "directory", git_status: "modified" },
        { name: "README.md", path: "README.md", kind: "file", git_status: "added" },
      ],
    });

    render(<App />);

    expect(await screen.findByRole("region", { name: /file explorer/i })).toBeInTheDocument();
    await waitFor(() => expect(projectApi.listNativeProjectFiles).toHaveBeenCalledWith("proj_auth"));
    expect(await screen.findByText("README.md")).toBeInTheDocument();
    expect(screen.getByLabelText("Git status added for README.md")).toHaveTextContent("A");
  });

  it("opens a read-only project file preview from the explorer", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_file_preview",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "README.md", path: "README.md", kind: "file", git_status: "modified" }],
    });
    projectApi.readNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "README.md",
      name: "README.md",
      language: "markdown",
      body: "# Auth Service\n\nRun tests before merge.\n",
      size_bytes: 39,
      truncated: false,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file readme.md/i }));

    await waitFor(() => expect(projectApi.readNativeProjectFile).toHaveBeenCalledWith("proj_auth", "README.md"));
    expect(await screen.findByRole("region", { name: /code preview/i })).toHaveTextContent("Run tests before merge.");
  });

  it("clears stale project file previews while a new file is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_file_preview_stale",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [
        { name: "README.md", path: "README.md", kind: "file", git_status: "modified" },
        { name: "server.ts", path: "src/server.ts", kind: "file", git_status: "modified" },
      ],
    });
    const pendingPreview = createDeferred<{
      project_id: string;
      path: string;
      name: string;
      language: string;
      body: string;
      size_bytes: number;
      truncated: boolean;
    }>();
    projectApi.readNativeProjectFile
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "README.md",
        name: "README.md",
        language: "markdown",
        body: "# Auth Service\n\nRun tests before merge.\n",
        size_bytes: 39,
        truncated: false,
      })
      .mockImplementationOnce(() => pendingPreview.promise);

    render(<App />);

    const preview = await screen.findByRole("region", { name: /code preview/i });
    fireEvent.click(await screen.findByRole("button", { name: /open file readme.md/i }));
    await waitFor(() => expect(preview).toHaveTextContent("Run tests before merge."));

    fireEvent.click(await screen.findByRole("button", { name: /open file server.ts/i }));

    await waitFor(() => expect(projectApi.readNativeProjectFile).toHaveBeenCalledWith("proj_auth", "src/server.ts"));
    expect(preview).not.toHaveTextContent("Run tests before merge.");
    expect(preview).toHaveTextContent("No file selected");
    await resolveDeferred(pendingPreview, {
      project_id: "proj_auth",
      path: "src/server.ts",
      name: "server.ts",
      language: "typescript",
      body: "export const server = true;\n",
      size_bytes: 27,
      truncated: false,
    });
  });

  it("renders a Monaco code editor for source project files", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_monaco_editor",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "server.ts", path: "src/server.ts", kind: "file", git_status: "modified" }],
    });
    projectApi.readNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/server.ts",
      name: "server.ts",
      language: "typescript",
      body: "export function boot() { return 'ok'; }\n",
      size_bytes: 39,
      truncated: false,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file server.ts/i }));

    expect(await screen.findByRole("region", { name: /monaco code editor/i })).toHaveAttribute("data-language", "typescript");
    expect(screen.getByRole("region", { name: /monaco code editor/i })).toHaveTextContent("export function boot");
  });

  it("edits and saves source project files through the Monaco editor", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_monaco_save",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "server.ts", path: "src/server.ts", kind: "file", git_status: "modified" }],
    });
    projectApi.readNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/server.ts",
      name: "server.ts",
      language: "typescript",
      body: "export const value = 1;\n",
      size_bytes: 24,
      truncated: false,
    });
    projectApi.saveNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/server.ts",
      name: "server.ts",
      language: "typescript",
      body: "export const value = 2;\n",
      size_bytes: 24,
      truncated: false,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file server.ts/i }));
    fireEvent.change(await screen.findByLabelText("Monaco editor buffer"), {
      target: { value: "export const value = 2;\n" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save source file" }));

    await waitFor(() =>
      expect(projectApi.saveNativeProjectFile).toHaveBeenCalledWith(
        "proj_auth",
        "src/server.ts",
        "export const value = 2;\n",
      ),
    );
    expect(await screen.findByText("Saved src/server.ts")).toBeInTheDocument();
  });

  it("renders a quick preview for markdown project files", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_quick_preview",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [] as never[], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "README.md", path: "README.md", kind: "file", git_status: null }],
    });
    projectApi.readNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "README.md",
      name: "README.md",
      language: "markdown",
      body: "# Auth Service\n\n- Run tests\n- Attach evidence\n",
      size_bytes: 45,
      truncated: false,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file readme.md/i }));

    const preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(preview.querySelector("h1")).toHaveTextContent("Auth Service");
    expect(preview.querySelectorAll("li")).toHaveLength(2);
  });

  it("renders dedicated quick preview viewers for HTML image PDF JSON and YAML files", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_multimodal_preview",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [] as never[], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [
        { name: "preview.html", path: "preview.html", kind: "file", git_status: null },
        { name: "pixel.png", path: "pixel.png", kind: "file", git_status: null },
        { name: "brief.pdf", path: "brief.pdf", kind: "file", git_status: null },
        { name: "config.json", path: "config.json", kind: "file", git_status: null },
        { name: "workflow.yaml", path: "workflow.yaml", kind: "file", git_status: null },
        { name: "server.log", path: "server.log", kind: "file", git_status: null },
      ],
    });
    projectApi.readNativeProjectFile
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "preview.html",
        name: "preview.html",
        language: "html",
        body: "<main><h1>Preview</h1></main>",
        size_bytes: 29,
        truncated: false,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "pixel.png",
        name: "pixel.png",
        language: "image",
        body: "data:image/png;base64,iVBORw0KGgo=",
        size_bytes: 8,
        truncated: false,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "brief.pdf",
        name: "brief.pdf",
        language: "pdf",
        body: "data:application/pdf;base64,JVBERi0xLjcK",
        size_bytes: 9,
        truncated: false,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "config.json",
        name: "config.json",
        language: "json",
        body: "{\"releaseGate\":true}",
        size_bytes: 20,
        truncated: false,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "workflow.yaml",
        name: "workflow.yaml",
        language: "yaml",
        body: "workflow:\n  gate: release\n",
        size_bytes: 25,
        truncated: false,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "server.log",
        name: "server.log",
        language: "log",
        body: "INFO boot\nWARN retry\n",
        size_bytes: 21,
        truncated: false,
      });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file preview.html/i }));
    let preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(within(preview).getByTitle("HTML preview preview.html")).toHaveAttribute("sandbox");

    fireEvent.click(screen.getByRole("button", { name: /open file pixel.png/i }));
    preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(within(preview).getByRole("img", { name: /image preview pixel.png/i })).toHaveAttribute("src", "data:image/png;base64,iVBORw0KGgo=");

    fireEvent.click(screen.getByRole("button", { name: /open file brief.pdf/i }));
    preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(within(preview).getByTitle("PDF preview brief.pdf")).toHaveAttribute("src", "data:application/pdf;base64,JVBERi0xLjcK");

    fireEvent.click(screen.getByRole("button", { name: /open file config.json/i }));
    preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(preview).toHaveTextContent('"releaseGate": true');

    fireEvent.click(screen.getByRole("button", { name: /open file workflow.yaml/i }));
    preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(preview).toHaveTextContent("gate: release");

    fireEvent.click(screen.getByRole("button", { name: /open file server.log/i }));
    preview = await screen.findByRole("region", { name: /quick preview/i });
    expect(within(preview).getAllByRole("listitem")).toHaveLength(2);
    expect(preview).toHaveTextContent("WARN retry");
  });

  it("searches active project files from the explorer input", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_file_search",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "README.md", path: "README.md", kind: "file", git_status: null }],
    });
    projectApi.searchNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      query: "login",
      degraded_reason: null,
      entries: [{ name: "login.ts", path: "src/auth/login.ts", kind: "file", git_status: "untracked" }],
    });

    render(<App />);

    fireEvent.change(await screen.findByLabelText("Search project files"), { target: { value: "login" } });

    await waitFor(() => expect(projectApi.searchNativeProjectFiles).toHaveBeenCalledWith("proj_auth", "login"));
    expect(await screen.findByText("login.ts")).toBeInTheDocument();
    expect(screen.queryByText("README.md")).not.toBeInTheDocument();
  });

  it("clears stale project file search results while a new query is pending", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_search_stale",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "README.md", path: "README.md", kind: "file", git_status: null }],
    });
    const pendingSearch = createDeferred<{
      project_id: string;
      query: string;
      degraded_reason: string | null;
      entries: Array<{ name: string; path: string; kind: "file"; git_status: string | null }>;
    }>();
    projectApi.searchNativeProjectFiles
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        query: "login",
        degraded_reason: null,
        entries: [{ name: "login.ts", path: "src/auth/login.ts", kind: "file", git_status: "untracked" }],
      })
      .mockImplementationOnce(() => pendingSearch.promise);

    render(<App />);

    const search = await screen.findByLabelText("Search project files");
    fireEvent.change(search, { target: { value: "login" } });
    expect(await screen.findByText("login.ts")).toBeInTheDocument();

    fireEvent.change(search, { target: { value: "billing" } });

    await waitFor(() => expect(projectApi.searchNativeProjectFiles).toHaveBeenCalledWith("proj_auth", "billing"));
    expect(screen.queryByText("login.ts")).not.toBeInTheDocument();
    await resolveDeferred(pendingSearch, {
      project_id: "proj_auth",
      query: "billing",
      degraded_reason: null,
      entries: [{ name: "billing.ts", path: "src/billing.ts", kind: "file", git_status: null }],
    });
  });

  it("opens a review diff from a modified explorer file", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_diff",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "README.md", path: "README.md", kind: "file", git_status: "modified" }],
    });
    projectApi.readNativeProjectDiff.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "README.md",
      body: "diff --git a/README.md b/README.md\n+Run tests before merge.\n",
      file_count: 1,
      files: [{ path: "README.md", status: "modified", additions: 1, deletions: 0 }],
      truncated: false,
      degraded_reason: null,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open diff readme.md/i }));

    await waitFor(() => expect(projectApi.readNativeProjectDiff).toHaveBeenCalledWith("proj_auth", "README.md"));
    const diff = await screen.findByRole("region", { name: /review diff/i });
    expect(diff).toHaveTextContent("README.md · modified · +1 -0");
    expect(diff).toHaveTextContent("+Run tests before merge.");
    const monacoDiff = within(diff).getByRole("region", { name: /monaco diff editor/i });
    expect(monacoDiff).toHaveAttribute("data-path", "README.md");
  });

  it("clears stale review diffs while a new diff is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_diff_stale",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "README.md", path: "README.md", kind: "file", git_status: "modified" }],
    });
    const pendingDiff = createDeferred<{
      project_id: string;
      path: string;
      body: string;
      file_count: number;
      files: Array<{ path: string; status: string; additions: number; deletions: number }>;
      truncated: boolean;
      degraded_reason: string | null;
    }>();
    projectApi.readNativeProjectDiff
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "README.md",
        body: "diff --git a/README.md b/README.md\n+Run tests before merge.\n",
        file_count: 1,
        files: [{ path: "README.md", status: "modified", additions: 1, deletions: 0 }],
        truncated: false,
        degraded_reason: null,
      })
      .mockImplementationOnce(() => pendingDiff.promise);

    render(<App />);

    const diff = await screen.findByRole("region", { name: /review diff/i });
    fireEvent.click(await screen.findByRole("button", { name: /open diff readme.md/i }));
    await waitFor(() => expect(diff).toHaveTextContent("+Run tests before merge."));

    fireEvent.click(await screen.findByRole("button", { name: /open diff readme.md/i }));
    await waitFor(() => expect(projectApi.readNativeProjectDiff).toHaveBeenCalledTimes(2));

    expect(diff).not.toHaveTextContent("+Run tests before merge.");
    expect(diff).toHaveTextContent("Workspace diff");
    expect(diff).toHaveTextContent("Select a changed file to review");
    await resolveDeferred(pendingDiff, {
      project_id: "proj_auth",
      path: "README.md",
      body: "diff --git a/README.md b/README.md\n+Refresh docs.\n",
      file_count: 1,
      files: [{ path: "README.md", status: "modified", additions: 1, deletions: 0 }],
      truncated: false,
      degraded_reason: null,
    });
  });

  it("exports imports and plans PR landing from the review diff panel", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_patch_pr",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });
    projectApi.exportNativeProjectPatch.mockResolvedValueOnce({
      project_id: "proj_auth",
      patch_id: "patch_readme",
      body: "diff --git a/README.md b/README.md\n+Run tests before merge.\n",
      file_count: 1,
      status: "exported",
      degraded_reason: null,
    });
    projectApi.importNativeProjectPatch.mockResolvedValueOnce({
      project_id: "proj_auth",
      patch_id: "patch_readme",
      body: "diff --git a/README.md b/README.md\n+Run tests before merge.\n",
      file_count: 1,
      status: "validated",
      degraded_reason: null,
    });
    projectApi.planNativePrLanding.mockResolvedValueOnce({
      project_id: "proj_auth",
      provider: "github",
      title: "Ship Auth Service changes",
      draft: true,
      checklist: ["export patch and review diff summary", "open draft PR with evidence pack links"],
      degraded_reason: "network push is intentionally not executed by local planner",
    });

    render(<App />);

    const diff = await screen.findByRole("region", { name: /review diff/i });
    fireEvent.change(within(diff).getByLabelText("Patch import body"), {
      target: { value: "diff --git a/README.md b/README.md\n+Run tests before merge.\n" },
    });
    fireEvent.change(within(diff).getByLabelText("PR landing title"), { target: { value: "Ship Auth Service changes" } });
    fireEvent.click(within(diff).getByRole("button", { name: /export project patch/i }));
    fireEvent.click(within(diff).getByRole("button", { name: /import project patch/i }));
    fireEvent.click(within(diff).getByRole("button", { name: /plan draft pr landing/i }));

    await waitFor(() => expect(projectApi.exportNativeProjectPatch).toHaveBeenCalledWith("proj_auth", undefined));
    expect(projectApi.importNativeProjectPatch).toHaveBeenCalledWith("proj_auth", "diff --git a/README.md b/README.md\n+Run tests before merge.\n");
    expect(projectApi.planNativePrLanding).toHaveBeenCalledWith({ projectId: "proj_auth", title: "Ship Auth Service changes", draft: true });
    expect(diff).toHaveTextContent("exported · patch_readme");
    expect(diff).toHaveTextContent("validated · patch_readme");
    expect(diff).toHaveTextContent("github · draft");
    expect(diff).toHaveTextContent("open draft PR with evidence pack links");
  });

  it("clears stale exported patches while a new patch export is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_patch_export_refresh",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });
    const pendingExport = createDeferred<{
      project_id: string;
      patch_id: string;
      body: string;
      file_count: number;
      status: string;
      degraded_reason: string | null;
    }>();
    projectApi.exportNativeProjectPatch
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        patch_id: "patch_readme",
        body: "diff --git a/README.md b/README.md\n+Run tests before merge.\n",
        file_count: 1,
        status: "exported",
        degraded_reason: null,
      })
      .mockImplementationOnce(() => pendingExport.promise);

    render(<App />);

    const diff = await screen.findByRole("region", { name: /review diff/i });
    fireEvent.click(within(diff).getByRole("button", { name: /export project patch/i }));
    await waitFor(() => expect(diff).toHaveTextContent("exported · patch_readme"));

    fireEvent.click(within(diff).getByRole("button", { name: /export project patch/i }));
    await waitFor(() => expect(projectApi.exportNativeProjectPatch).toHaveBeenCalledTimes(2));

    expect(diff).not.toHaveTextContent("exported · patch_readme");
    await resolveDeferred(pendingExport, {
      project_id: "proj_auth",
      patch_id: "patch_next",
      body: "diff --git a/README.md b/README.md\n+Refresh docs.\n",
      file_count: 1,
      status: "exported",
      degraded_reason: null,
    });
  });

  it("clears stale imported patches while a new patch import is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_patch_import_refresh",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });
    const pendingImport = createDeferred<{
      project_id: string;
      patch_id: string;
      body: string;
      file_count: number;
      status: string;
      degraded_reason: string | null;
    }>();
    projectApi.importNativeProjectPatch
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        patch_id: "patch_readme",
        body: "diff --git a/README.md b/README.md\n+Run tests before merge.\n",
        file_count: 1,
        status: "validated",
        degraded_reason: null,
      })
      .mockImplementationOnce(() => pendingImport.promise);

    render(<App />);

    const diff = await screen.findByRole("region", { name: /review diff/i });
    fireEvent.change(within(diff).getByLabelText("Patch import body"), {
      target: { value: "diff --git a/README.md b/README.md\n+Run tests before merge.\n" },
    });
    fireEvent.click(within(diff).getByRole("button", { name: /import project patch/i }));
    await waitFor(() => expect(diff).toHaveTextContent("validated · patch_readme"));

    fireEvent.click(within(diff).getByRole("button", { name: /import project patch/i }));
    await waitFor(() => expect(projectApi.importNativeProjectPatch).toHaveBeenCalledTimes(2));

    expect(diff).not.toHaveTextContent("validated · patch_readme");
    await resolveDeferred(pendingImport, {
      project_id: "proj_auth",
      patch_id: "patch_next",
      body: "diff --git a/README.md b/README.md\n+Refresh docs.\n",
      file_count: 1,
      status: "validated",
      degraded_reason: null,
    });
  });

  it("clears stale PR landing plans while a new PR landing plan is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_pr_landing_refresh",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });
    const pendingPrPlan = createDeferred<{
      project_id: string;
      provider: string;
      title: string;
      draft: boolean;
      checklist: string[];
      degraded_reason: string | null;
    }>();
    projectApi.planNativePrLanding
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        provider: "github",
        title: "Ship Auth Service changes",
        draft: true,
        checklist: ["export patch and review diff summary", "open draft PR with evidence pack links"],
        degraded_reason: null,
      })
      .mockImplementationOnce(() => pendingPrPlan.promise);

    render(<App />);

    const diff = await screen.findByRole("region", { name: /review diff/i });
    fireEvent.change(within(diff).getByLabelText("PR landing title"), { target: { value: "Ship Auth Service changes" } });
    fireEvent.click(within(diff).getByRole("button", { name: /plan draft pr landing/i }));
    await waitFor(() => expect(diff).toHaveTextContent("github · draft"));

    fireEvent.click(within(diff).getByRole("button", { name: /plan draft pr landing/i }));
    await waitFor(() => expect(projectApi.planNativePrLanding).toHaveBeenCalledTimes(2));

    expect(diff).not.toHaveTextContent("github · draft");
    expect(diff).not.toHaveTextContent("open draft PR with evidence pack links");
    await resolveDeferred(pendingPrPlan, {
      project_id: "proj_auth",
      provider: "github",
      title: "Ship Auth Service changes",
      draft: true,
      checklist: ["confirm patch evidence", "publish draft PR"],
      degraded_reason: null,
    });
  });

  it("loads a guarded localhost browser preview pane", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_localhost_preview",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });

    render(<App />);

    fireEvent.change(await screen.findByLabelText("Localhost preview URL"), { target: { value: "http://localhost:3000/docs" } });
    fireEvent.click(screen.getByRole("button", { name: /load localhost preview/i }));

    expect(await screen.findByRole("region", { name: /localhost browser/i })).toHaveTextContent("http://localhost:3000/docs");
    expect(screen.getByTitle("Localhost preview http://localhost:3000/docs")).toBeInTheDocument();
  });

  it("opens safe terminal links in the localhost browser pane", async () => {
    await renderApp();

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    });
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());

    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "server ready at http://localhost:3000/docs\n",
      });
    });

    fireEvent.click(await screen.findByRole("button", { name: "Open terminal link 1. haneulchi (zsh)" }));

    expect(await screen.findByRole("region", { name: /localhost browser/i })).toHaveTextContent("http://localhost:3000/docs");
    expect(screen.getByTitle("Localhost preview http://localhost:3000/docs")).toBeInTheDocument();
  });

  it("plans browser automation from the localhost browser pane", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_browser_automation",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });
    browserAutomationApi.planNativeBrowserAutomation.mockResolvedValueOnce({
      project_id: "proj_auth",
      url: "http://localhost:3000/docs",
      scenario: "smoke",
      status: "planned",
      steps: ["open http://localhost:3000/docs", "capture browser screenshot for smoke"],
      degraded_reason: null,
    });

    render(<App />);

    const browser = await screen.findByRole("region", { name: /localhost browser/i });
    fireEvent.change(within(browser).getByLabelText("Localhost preview URL"), { target: { value: "http://localhost:3000/docs" } });
    fireEvent.click(within(browser).getByRole("button", { name: /load localhost preview/i }));
    fireEvent.click(within(browser).getByRole("button", { name: /plan browser automation/i }));

    await waitFor(() =>
      expect(browserAutomationApi.planNativeBrowserAutomation).toHaveBeenCalledWith({
        projectId: "proj_auth",
        url: "http://localhost:3000/docs",
        scenario: "smoke",
      }),
    );
    expect(browser).toHaveTextContent("planned · smoke");
    expect(browser).toHaveTextContent("capture browser screenshot for smoke");
  });

  it("clears stale browser automation plans while a new plan is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_browser_automation_refresh",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });
    const pendingBrowserPlan = createDeferred<{
      project_id: string;
      url: string;
      scenario: string;
      status: string;
      steps: string[];
      degraded_reason: string | null;
    }>();
    browserAutomationApi.planNativeBrowserAutomation
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        url: "http://localhost:3000/docs",
        scenario: "smoke",
        status: "planned",
        steps: ["open http://localhost:3000/docs", "capture browser screenshot for smoke"],
        degraded_reason: null,
      })
      .mockImplementationOnce(() => pendingBrowserPlan.promise);

    render(<App />);

    const browser = await screen.findByRole("region", { name: /localhost browser/i });
    fireEvent.change(within(browser).getByLabelText("Localhost preview URL"), { target: { value: "http://localhost:3000/docs" } });
    fireEvent.click(within(browser).getByRole("button", { name: /load localhost preview/i }));
    fireEvent.click(within(browser).getByRole("button", { name: /plan browser automation/i }));
    await waitFor(() => expect(browser).toHaveTextContent("capture browser screenshot for smoke"));

    fireEvent.click(within(browser).getByRole("button", { name: /plan browser automation/i }));
    await waitFor(() => expect(browserAutomationApi.planNativeBrowserAutomation).toHaveBeenCalledTimes(2));

    expect(browser).not.toHaveTextContent("planned · smoke");
    expect(browser).not.toHaveTextContent("capture browser screenshot for smoke");
    await resolveDeferred(pendingBrowserPlan, {
      project_id: "proj_auth",
      url: "http://localhost:3000/docs",
      scenario: "smoke",
      status: "planned",
      steps: ["open http://localhost:3000/docs", "capture latest screenshot"],
      degraded_reason: null,
    });
  });

  it("rejects remote URLs in the localhost browser pane", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_localhost_remote",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [],
    });

    render(<App />);

    fireEvent.change(await screen.findByLabelText("Localhost preview URL"), { target: { value: "https://example.com" } });
    fireEvent.click(screen.getByRole("button", { name: /load localhost preview/i }));

    expect(await screen.findByText("Only localhost HTTP(S) URLs are allowed")).toBeInTheDocument();
    expect(screen.queryByTitle(/localhost preview/i)).not.toBeInTheDocument();
  });

  it("runs LSP diagnostics for the selected project file", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_lsp_diagnostics",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "app.ts", path: "src/app.ts", kind: "file", git_status: "modified" }],
    });
    projectApi.readNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/app.ts",
      name: "app.ts",
      language: "typescript",
      body: "export function loadUser() {\n  return TODO as any;\n}\n",
      size_bytes: 24,
      truncated: false,
    });
    projectApi.collectNativeProjectLspDiagnostics.mockResolvedValueOnce({
      project_id: "proj_auth",
      diagnostics: [
        {
          path: "src/app.ts",
          line: 2,
          severity: "warning",
          message: "TypeScript explicit any weakens local LSP guarantees",
        },
      ],
      symbols: [{ path: "src/app.ts", name: "loadUser", kind: "function", line: 1 }],
      degraded_reason: null,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file app.ts/i }));
    const lsp = await screen.findByRole("region", { name: /lsp diagnostics/i });
    fireEvent.click(within(lsp).getByRole("button", { name: /run lsp diagnostics/i }));

    await waitFor(() => expect(projectApi.collectNativeProjectLspDiagnostics).toHaveBeenCalledWith("proj_auth", "src/app.ts"));
    expect(lsp).toHaveTextContent("src/app.ts:2 · warning");
    expect(lsp).toHaveTextContent("TypeScript explicit any weakens local LSP guarantees");
    expect(lsp).toHaveTextContent("function · loadUser · src/app.ts:1");
  });

  it("opens files from the LSP symbol outline", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_lsp_symbol_open",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [
        { name: "app.ts", path: "src/app.ts", kind: "file", git_status: "modified" },
        { name: "lib.ts", path: "src/lib.ts", kind: "file", git_status: "modified" },
      ],
    });
    projectApi.readNativeProjectFile
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "src/app.ts",
        name: "app.ts",
        language: "typescript",
        body: "export function loadUser() {\n  return TODO;\n}\n",
        size_bytes: 24,
        truncated: false,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        path: "src/lib.ts",
        name: "lib.ts",
        language: "typescript",
        body: "export function helper() {\n  return true;\n}\n",
        size_bytes: 42,
        truncated: false,
      });
    projectApi.collectNativeProjectLspDiagnostics.mockResolvedValueOnce({
      project_id: "proj_auth",
      diagnostics: [{ path: "src/app.ts", line: 2, severity: "warning", message: "TODO marker remains" }],
      symbols: [{ path: "src/lib.ts", name: "helper", kind: "function", line: 1 }],
      degraded_reason: null,
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file app.ts/i }));
    const lsp = await screen.findByRole("region", { name: /lsp diagnostics/i });
    fireEvent.click(within(lsp).getByRole("button", { name: /run lsp diagnostics/i }));
    fireEvent.click(await within(lsp).findByRole("button", { name: /open symbol helper/i }));

    await waitFor(() => expect(projectApi.readNativeProjectFile).toHaveBeenCalledWith("proj_auth", "src/lib.ts"));
    expect(await screen.findByRole("region", { name: /monaco code editor/i })).toHaveTextContent("export function helper");
  });

  it("clears stale LSP diagnostics while a new diagnostics run is loading", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_lsp_diagnostics_refresh",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.listNativeProjectFiles.mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth",
      relative_path: "",
      degraded_reason: null,
      entries: [{ name: "app.ts", path: "src/app.ts", kind: "file", git_status: "modified" }],
    });
    projectApi.readNativeProjectFile.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/app.ts",
      name: "app.ts",
      language: "typescript",
      body: "export function loadUser() {\n  return TODO as any;\n}\n",
      size_bytes: 24,
      truncated: false,
    });
    const pendingLspDiagnostics = createDeferred<{
      project_id: string;
      diagnostics: Array<{ path: string; line: number; severity: string; message: string }>;
      symbols: Array<{ path: string; name: string; kind: string; line: number }>;
      degraded_reason: string | null;
    }>();
    projectApi.collectNativeProjectLspDiagnostics
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        diagnostics: [
          {
            path: "src/app.ts",
            line: 2,
            severity: "warning",
            message: "TypeScript explicit any weakens local LSP guarantees",
          },
        ],
        symbols: [{ path: "src/app.ts", name: "loadUser", kind: "function", line: 1 }],
        degraded_reason: null,
      })
      .mockImplementationOnce(() => pendingLspDiagnostics.promise);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /open file app.ts/i }));
    const lsp = await screen.findByRole("region", { name: /lsp diagnostics/i });
    fireEvent.click(within(lsp).getByRole("button", { name: /run lsp diagnostics/i }));
    await waitFor(() => expect(lsp).toHaveTextContent("TypeScript explicit any weakens local LSP guarantees"));

    fireEvent.click(within(lsp).getByRole("button", { name: /run lsp diagnostics/i }));
    await waitFor(() => expect(projectApi.collectNativeProjectLspDiagnostics).toHaveBeenCalledTimes(2));

    expect(lsp).not.toHaveTextContent("src/app.ts:2 · warning");
    expect(lsp).not.toHaveTextContent("TypeScript explicit any weakens local LSP guarantees");
    expect(lsp).toHaveTextContent("Run local diagnostics for TODO markers and weak TypeScript typing");
    await resolveDeferred(pendingLspDiagnostics, {
      project_id: "proj_auth",
      diagnostics: [],
      symbols: [{ path: "src/app.ts", name: "loadUser", kind: "function", line: 1 }],
      degraded_reason: null,
    });
  });

  it("opens a command palette for project switching", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_palette",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: ["auth/runbook"] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.focusNativeProject.mockResolvedValueOnce({ id: "proj_docs", name: "Docs Workspace", status: "active" });

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open command search/i }));
    expect(await screen.findByRole("dialog", { name: /command palette/i })).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Search commands and projects"), { target: { value: "docs" } });
    fireEvent.click(screen.getByRole("button", { name: /palette project docs workspace/i }));

    await waitFor(() => expect(projectApi.focusNativeProject).toHaveBeenCalledWith("proj_docs"));
  });

  it("searches knowledge pages from the command palette and opens a result", async () => {
    knowledgeApi.searchNativeKnowledgePages.mockResolvedValueOnce([
      {
        id: "kp_auth",
        project_id: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        artifact_path: "artifacts/knowledge/auth-flow.md",
        source_ids: ["ks_auth"],
        freshness_state: "current",
        body_md: "# Auth Flow\n\nToken rotation and rollback notes.",
      },
    ]);

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open command search/i }));
    expect(await screen.findByRole("dialog", { name: /command palette/i })).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Search commands and projects"), { target: { value: "token" } });
    fireEvent.click(await screen.findByRole("button", { name: /palette knowledge auth flow/i }));

    expect(screen.queryByRole("dialog", { name: /command palette/i })).not.toBeInTheDocument();
    expect(screen.getByLabelText("Knowledge page slug")).toHaveValue("auth-flow");
    expect(screen.getByLabelText("Knowledge page title")).toHaveValue("Auth Flow");
    expect(screen.getByLabelText("Knowledge page markdown")).toHaveValue("# Auth Flow\n\nToken rotation and rollback notes.");
  });

  it("shows command palette knowledge search degraded state", async () => {
    knowledgeApi.searchNativeKnowledgePages.mockRejectedValueOnce(new Error("native knowledge API unavailable"));

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open command search/i }));
    expect(await screen.findByRole("dialog", { name: /command palette/i })).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Search commands and projects"), { target: { value: "auth" } });

    expect(await screen.findByText("Knowledge search unavailable · native knowledge API unavailable")).toBeInTheDocument();
  });

  it("surfaces budget warning data from the state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_budget",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: {
        workspace: { used_usd: 8.5, state: "warn" },
        projects: [{ scope_id: "proj_local", used_usd: 8.5, max_usd: 10, state: "warn" }],
        agents: [],
        price_table: { count: 1, source: "local-fixture" },
      },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("Budget proj_local · warn · $8.50 / $10.00")).toBeInTheDocument();
    expect(await screen.findByText("Provider prices local-fixture · 1 model")).toBeInTheDocument();
    const queue = screen.getByRole("region", { name: /global attention queue/i });
    expect(queue).toHaveTextContent("Budget warning: project proj_local · warning");
    expect(queue).toHaveTextContent("$8.50 of $10.00 used · 85%");
  });

  it("renders per-project and per-agent budget dashboard rows", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_budget_dashboard",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: {
        workspace: { scope_type: "workspace", used_usd: 10, max_usd: 20, state: "ok" },
        projects: [
          { scope_id: "proj_auth", used_usd: 8.5, max_usd: 10, state: "warn", hard_limit: true },
          { scope_id: "proj_docs", used_usd: 1, max_usd: 5, state: "ok", hard_limit: false },
        ],
        goals: [
          { scope_id: "init_platform", used_usd: 3.5, max_usd: 4, state: "warn", hard_limit: false },
        ],
        tasks: [
          { scope_id: "task_budget", used_usd: 2.25, max_usd: 3, state: "warn", hard_limit: true },
        ],
        runs: [
          { scope_id: "run_budget", used_usd: 1.5, max_usd: 2, state: "warn", hard_limit: false },
        ],
        agents: [
          { scope_id: "agent_codex", used_usd: 4, max_usd: 5, state: "warn" },
          { scope_id: "agent_claude", used_usd: 1, max_usd: 5, state: "ok" },
        ],
        forecasts: {
          workspace: { scope_type: "workspace", average_run_cost_usd: 5, estimated_runs_remaining: 2, remaining_usd: 10, run_sample_count: 2 },
          projects: [
            { scope_type: "project", scope_id: "proj_auth", average_run_cost_usd: 6, estimated_runs_remaining: 1, remaining_usd: 1.5, run_sample_count: 2 },
          ],
          goals: [
            { scope_type: "goal", scope_id: "init_platform", average_run_cost_usd: 1.5, estimated_runs_remaining: 0, remaining_usd: 0.5, run_sample_count: 2 },
          ],
          tasks: [
            { scope_type: "task", scope_id: "task_budget", average_run_cost_usd: 2.25, estimated_runs_remaining: 0, remaining_usd: 0.75, run_sample_count: 1 },
          ],
          runs: [
            { scope_type: "run", scope_id: "run_budget", average_run_cost_usd: 1.5, estimated_runs_remaining: 0, remaining_usd: 0.5, run_sample_count: 1 },
          ],
          agents: [],
        },
      },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    expect(dashboard).toHaveTextContent("Workspace · ok · $10.00 / $20.00 · 50%");
    expect(dashboard).toHaveTextContent("Projects 2");
    expect(dashboard).toHaveTextContent("proj_auth · warn · $8.50 / $10.00 · 85% · hard");
    expect(dashboard).toHaveTextContent("proj_docs · ok · $1.00 / $5.00 · 20%");
    expect(dashboard).toHaveTextContent("Goals 1");
    expect(dashboard).toHaveTextContent("init_platform · warn · $3.50 / $4.00 · 88%");
    expect(dashboard).toHaveTextContent("Tasks 1");
    expect(dashboard).toHaveTextContent("task_budget · warn · $2.25 / $3.00 · 75% · hard");
    expect(dashboard).toHaveTextContent("Runs 1");
    expect(dashboard).toHaveTextContent("run_budget · warn · $1.50 / $2.00 · 75%");
    expect(dashboard).toHaveTextContent("Agents 2");
    expect(dashboard).toHaveTextContent("agent_codex · warn · $4.00 / $5.00 · 80%");
    expect(dashboard).toHaveTextContent("agent_claude · ok · $1.00 / $5.00 · 20%");
    expect(dashboard).toHaveTextContent("Forecast proj_auth · avg $6.00/run · 1 run left · $1.50 remaining");
    expect(dashboard).toHaveTextContent("Forecast init_platform · avg $1.50/run · 0 runs left · $0.50 remaining");
    expect(dashboard).toHaveTextContent("Forecast task_budget · avg $2.25/run · 0 runs left · $0.75 remaining");
    expect(dashboard).toHaveTextContent("Forecast run_budget · avg $1.50/run · 0 runs left · $0.50 remaining");
  });

  it("loads native budget summary rows from the budget dashboard", async () => {
    budgetApi.getNativeBudgetSummary.mockResolvedValueOnce({
      workspace: { scope_type: "workspace", used_usd: 18, max_usd: 20, state: "warn" },
      projects: [
        { scope_id: "proj_loaded", used_usd: 7.5, max_usd: 10, state: "warn", hard_limit: true },
      ],
      goals: [
        { scope_id: "init_loaded", used_usd: 2, max_usd: 5, state: "ok", hard_limit: false },
      ],
      tasks: [
        { scope_id: "task_loaded", used_usd: 1.25, max_usd: 2, state: "warn", hard_limit: false },
      ],
      runs: [
        { scope_id: "run_loaded", used_usd: 1, max_usd: 1, state: "exceeded", hard_limit: true },
      ],
      agents: [
        { scope_id: "agent_loaded", used_usd: 4, max_usd: 8, state: "ok", hard_limit: false },
      ],
    });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    fireEvent.click(within(dashboard).getByRole("button", { name: /load budget summary/i }));

    await waitFor(() => expect(budgetApi.getNativeBudgetSummary).toHaveBeenCalledWith());
    expect(await within(dashboard).findByText("Budget summary loaded · 5 scoped budgets")).toBeInTheDocument();
    expect(dashboard).toHaveTextContent("Workspace · warn · $18.00 / $20.00 · 90%");
    expect(dashboard).toHaveTextContent("Projects 1");
    expect(dashboard).toHaveTextContent("proj_loaded · warn · $7.50 / $10.00 · 75% · hard");
    expect(dashboard).toHaveTextContent("Goals 1");
    expect(dashboard).toHaveTextContent("init_loaded · ok · $2.00 / $5.00 · 40%");
    expect(dashboard).toHaveTextContent("Tasks 1");
    expect(dashboard).toHaveTextContent("task_loaded · warn · $1.25 / $2.00 · 63%");
    expect(dashboard).toHaveTextContent("Runs 1");
    expect(dashboard).toHaveTextContent("run_loaded · exceeded · $1.00 / $1.00 · 100% · hard");
    expect(dashboard).toHaveTextContent("Agents 1");
    expect(dashboard).toHaveTextContent("agent_loaded · ok · $4.00 / $8.00 · 50%");
  });

  it("refreshes budget forecast and updates provider prices from the budget dashboard", async () => {
    budgetApi.getNativeBudgetForecast.mockResolvedValueOnce({
      workspace: {},
      projects: [
        {
          scope_type: "project",
          scope_id: "proj_local",
          average_run_cost_usd: 4,
          estimated_runs_remaining: 3,
          remaining_usd: 12,
          run_sample_count: 3,
        },
      ],
      agents: [],
    });
    budgetApi.updateNativeProviderPriceTable.mockResolvedValueOnce({ source: "auto-fixture", updated: 1 });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    fireEvent.click(within(dashboard).getByRole("button", { name: /refresh budget forecast/i }));
    expect(await within(dashboard).findByText("Forecast proj_local · avg $4.00/run · 3 runs left · $12.00 remaining")).toBeInTheDocument();
    expect(budgetApi.getNativeBudgetForecast).toHaveBeenCalledWith();

    fireEvent.change(within(dashboard).getByLabelText("Provider price update source"), { target: { value: "auto-fixture" } });
    fireEvent.change(within(dashboard).getByLabelText("Provider price update payload"), {
      target: {
        value: JSON.stringify([
          {
            provider: "openai",
            model: "gpt-5.4",
            inputUsdPerMillion: 5,
            outputUsdPerMillion: 15,
          },
        ]),
      },
    });
    fireEvent.click(within(dashboard).getByRole("button", { name: /update provider prices/i }));

    await waitFor(() => expect(budgetApi.updateNativeProviderPriceTable).toHaveBeenCalledWith({
      source: "auto-fixture",
      prices: [
        {
          provider: "openai",
          model: "gpt-5.4",
          inputUsdPerMillion: 5,
          outputUsdPerMillion: 15,
        },
      ],
    }));
    expect(await within(dashboard).findByText("Provider prices auto-fixture · 1 model")).toBeInTheDocument();
    expect(within(dashboard).getByText("Updated provider prices · 1 model")).toBeInTheDocument();
  });

  it("loads native provider price rows from the budget dashboard", async () => {
    budgetApi.listNativeProviderPrices.mockResolvedValueOnce([
      {
        provider: "openai",
        model: "gpt-5.4",
        input_usd_per_million: 5,
        output_usd_per_million: 15,
        source: "auto-fixture",
        updated_at: "2026-05-05T01:00:00Z",
      },
      {
        provider: "anthropic",
        model: "claude-4",
        input_usd_per_million: 3,
        output_usd_per_million: 12,
        source: "manual",
        updated_at: "2026-05-05T01:05:00Z",
      },
    ]);

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    fireEvent.click(within(dashboard).getByRole("button", { name: /load provider prices/i }));

    await waitFor(() => expect(budgetApi.listNativeProviderPrices).toHaveBeenCalledWith());
    expect(await within(dashboard).findByText("openai/gpt-5.4 · in $5.00/M · out $15.00/M")).toBeInTheDocument();
    expect(within(dashboard).getByText("auto-fixture · 2026-05-05T01:00:00Z")).toBeInTheDocument();
    expect(within(dashboard).getByText("anthropic/claude-4 · in $3.00/M · out $12.00/M")).toBeInTheDocument();
  });

  it("sets native budget limits from the budget dashboard", async () => {
    budgetApi.upsertNativeBudget.mockResolvedValueOnce({
      id: "budget_project_proj_local",
      scope_type: "project",
      scope_id: "proj_local",
      max_usd: 32,
      warn_pct: 0.8,
      hard_limit: true,
    });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    fireEvent.change(within(dashboard).getByLabelText("Budget scope type"), { target: { value: "project" } });
    fireEvent.change(within(dashboard).getByLabelText("Budget scope id"), { target: { value: "proj_local" } });
    fireEvent.change(within(dashboard).getByLabelText("Budget max USD"), { target: { value: "32" } });
    fireEvent.change(within(dashboard).getByLabelText("Budget warning percent"), { target: { value: "0.8" } });
    fireEvent.click(within(dashboard).getByRole("button", { name: /set budget/i }));

    await waitFor(() => expect(budgetApi.upsertNativeBudget).toHaveBeenCalledWith({
      scopeType: "project",
      scopeId: "proj_local",
      maxUsd: 32,
      warnPct: 0.8,
      hardLimit: true,
    }));
    expect(await within(dashboard).findByText("Saved budget project proj_local · $32.00 · warn 80% · hard")).toBeInTheDocument();
    expect(dashboard).toHaveTextContent("proj_local · ok · $0.00 / $32.00 · 0% · hard");
  });

  it("clears stale budget forecast status after failures and recovers on retry", async () => {
    budgetApi.getNativeBudgetForecast
      .mockResolvedValueOnce({
        workspace: {},
        projects: [
          {
            scope_type: "project",
            scope_id: "proj_local",
            average_run_cost_usd: 4,
            estimated_runs_remaining: 3,
            remaining_usd: 12,
            run_sample_count: 3,
          },
        ],
        agents: [],
      })
      .mockRejectedValueOnce(new Error("forecast store down"))
      .mockResolvedValueOnce({
        workspace: {},
        projects: [],
        agents: [
          {
            scope_type: "agent",
            scope_id: "agent_codex",
            average_run_cost_usd: 2,
            estimated_runs_remaining: 4,
            remaining_usd: 8,
            run_sample_count: 4,
          },
        ],
      });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    fireEvent.click(within(dashboard).getByRole("button", { name: /refresh budget forecast/i }));
    expect(await within(dashboard).findByText("Forecast refreshed · 1 scoped budgets")).toBeInTheDocument();
    expect(await within(dashboard).findByText("Forecast proj_local · avg $4.00/run · 3 runs left · $12.00 remaining")).toBeInTheDocument();

    fireEvent.click(within(dashboard).getByRole("button", { name: /refresh budget forecast/i }));
    await waitFor(() => expect(dashboard).toHaveTextContent("Budget forecast unavailable · forecast store down"));
    expect(dashboard).not.toHaveTextContent("Forecast refreshed · 1 scoped budgets");

    fireEvent.click(within(dashboard).getByRole("button", { name: /refresh budget forecast/i }));
    expect(await within(dashboard).findByText("Forecast refreshed · 1 scoped budgets")).toBeInTheDocument();
    expect(await within(dashboard).findByText("Forecast agent_codex · avg $2.00/run · 4 runs left · $8.00 remaining")).toBeInTheDocument();
    expect(dashboard).not.toHaveTextContent("Budget forecast unavailable · forecast store down");
  });

  it("clears stale provider price update status after failures and recovers on retry", async () => {
    budgetApi.updateNativeProviderPriceTable
      .mockResolvedValueOnce({ source: "auto-fixture", updated: 1 })
      .mockRejectedValueOnce(new Error("price feed down"))
      .mockResolvedValueOnce({ source: "manual-fixture", updated: 2 });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    const source = within(dashboard).getByLabelText("Provider price update source");
    const payload = within(dashboard).getByLabelText("Provider price update payload");
    fireEvent.change(source, { target: { value: "auto-fixture" } });
    fireEvent.change(payload, {
      target: {
        value: JSON.stringify([
          {
            provider: "openai",
            model: "gpt-5.4",
            inputUsdPerMillion: 5,
            outputUsdPerMillion: 15,
          },
        ]),
      },
    });
    fireEvent.click(within(dashboard).getByRole("button", { name: /update provider prices/i }));
    expect(await within(dashboard).findByText("Updated provider prices · 1 model")).toBeInTheDocument();

    fireEvent.click(within(dashboard).getByRole("button", { name: /update provider prices/i }));
    await waitFor(() => expect(dashboard).toHaveTextContent("Provider price update unavailable · price feed down"));
    expect(dashboard).not.toHaveTextContent("Updated provider prices · 1 model");

    fireEvent.change(source, { target: { value: "manual-fixture" } });
    fireEvent.change(payload, {
      target: {
        value: JSON.stringify([
          {
            provider: "openai",
            model: "gpt-5.5",
            inputUsdPerMillion: 6,
            outputUsdPerMillion: 16,
          },
          {
            provider: "anthropic",
            model: "claude-opus-5",
            inputUsdPerMillion: 7,
            outputUsdPerMillion: 20,
          },
        ]),
      },
    });
    fireEvent.click(within(dashboard).getByRole("button", { name: /update provider prices/i }));

    expect(await within(dashboard).findByText("Updated provider prices · 2 models")).toBeInTheDocument();
    expect(await within(dashboard).findByText("Provider prices manual-fixture · 2 models")).toBeInTheDocument();
    expect(dashboard).not.toHaveTextContent("Provider price update unavailable · price feed down");
  });

  it("shows budget dashboard degraded and empty states", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_budget_dashboard_empty",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "degraded", pty: "ok", api: "ok" },
    });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /budget dashboard/i });
    expect(dashboard).toHaveTextContent("Budget dashboard degraded · db degraded");
    expect(dashboard).toHaveTextContent("No project budgets");
    expect(dashboard).toHaveTextContent("No agent budgets");
  });

  it("renders benchmark suite dashboard from state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_benchmark_dashboard",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      benchmarks: {
        last_run_id: "benchmark_7",
        last_status: "passed",
        last_pass_count: 4,
        last_fail_count: 0,
        last_warning_count: 0,
        suites: [
          {
            suite_id: "state_snapshot_latency",
            name: "State snapshot latency",
            status: "pass",
            metric_value: 12,
            target_value: 50,
            unit: "ms",
            detail: "Snapshot assembly stayed within budget.",
          },
        ],
        diagnostics: { status: "passed", suite_count: 4, duration_ms: 37 },
      },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /benchmark suite dashboard/i });
    expect(dashboard).toHaveTextContent("benchmark_7 · passed · 4 pass · 0 fail");
    expect(dashboard).toHaveTextContent("State snapshot latency · pass · 12 ms / 50 ms");
  });

  it("runs release gates and benchmarks from quality dashboards", async () => {
    qualityApi.runNativeReleaseGates.mockResolvedValueOnce({
      id: "release_gate_1",
      project_id: "proj_local",
      status: "blocked",
      scenario_count: 2,
      pass_count: 1,
      fail_count: 1,
      warning_count: 0,
      scenarios: [{ gate_id: "RG-01", name: "Evidence schema", status: "fail", detail: "missing", evidence: [] }],
      created_at: "2026-05-02T01:00:00Z",
    });
    qualityApi.runNativeBenchmarks.mockResolvedValueOnce({
      id: "benchmark_1",
      project_id: "proj_local",
      status: "passed",
      suite_count: 1,
      pass_count: 1,
      fail_count: 0,
      warning_count: 0,
      duration_ms: 12,
      suites: [{ suite_id: "state_snapshot_latency", name: "State snapshot latency", status: "pass", metric_value: 12, target_value: 250, unit: "ms", detail: "ok" }],
      created_at: "2026-05-02T01:00:00Z",
    });

    render(<App />);

    const releaseGates = await screen.findByRole("region", { name: /release gate runner/i });
    fireEvent.click(within(releaseGates).getByRole("button", { name: /run release gates/i }));
    await waitFor(() => expect(qualityApi.runNativeReleaseGates).toHaveBeenCalledWith("proj_local"));
    expect(await within(releaseGates).findByText("release_gate_1 · blocked · 1 pass · 1 fail")).toBeInTheDocument();
    expect(within(releaseGates).getByText("RG-01 · Evidence schema · fail")).toBeInTheDocument();

    const benchmark = await screen.findByRole("region", { name: /benchmark suite dashboard/i });
    fireEvent.click(within(benchmark).getByRole("button", { name: /run benchmark suite/i }));
    await waitFor(() => expect(qualityApi.runNativeBenchmarks).toHaveBeenCalledWith("proj_local"));
    expect(await within(benchmark).findByText("benchmark_1 · passed · 1 pass · 0 fail")).toBeInTheDocument();
    expect(within(benchmark).getByText("State snapshot latency · pass · 12 ms / 250 ms")).toBeInTheDocument();
  });

  it("loads release gate run history from the quality dashboard", async () => {
    qualityApi.listNativeReleaseGateRuns.mockResolvedValueOnce([
      {
        id: "release_gate_2",
        project_id: "proj_local",
        status: "passed",
        scenario_count: 2,
        pass_count: 2,
        fail_count: 0,
        warning_count: 0,
        scenarios: [
          { gate_id: "RG-01", name: "Evidence schema", status: "pass", detail: "ok", evidence: ["ev_release"] },
        ],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "release_gate_1",
        project_id: "proj_local",
        status: "blocked",
        scenario_count: 2,
        pass_count: 1,
        fail_count: 1,
        warning_count: 0,
        scenarios: [
          { gate_id: "RG-09", name: "Transcript evidence", status: "fail", detail: "missing transcript", evidence: [] },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const releaseGates = await screen.findByRole("region", { name: /release gate runner/i });
    fireEvent.click(within(releaseGates).getByRole("button", { name: /load release gate history/i }));

    await waitFor(() => expect(qualityApi.listNativeReleaseGateRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(releaseGates).findByText("History release_gate_2 · passed · 2 pass · 0 fail")).toBeInTheDocument();
    expect(within(releaseGates).getByText("History release_gate_1 · blocked · 1 pass · 1 fail")).toBeInTheDocument();
  });

  it("loads benchmark run history from the benchmark suite dashboard", async () => {
    qualityApi.listNativeBenchmarkRuns.mockResolvedValueOnce([
      {
        id: "benchmark_2",
        project_id: "proj_local",
        status: "passed",
        suite_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        duration_ms: 18,
        suites: [
          {
            suite_id: "state_snapshot_latency",
            name: "State snapshot latency",
            status: "pass",
            metric_value: 18,
            target_value: 250,
            unit: "ms",
            detail: "ok",
          },
        ],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "benchmark_1",
        project_id: "proj_local",
        status: "warning",
        suite_count: 1,
        pass_count: 0,
        fail_count: 0,
        warning_count: 1,
        duration_ms: 320,
        suites: [
          {
            suite_id: "visual_harness_render",
            name: "Visual harness render",
            status: "warning",
            metric_value: 320,
            target_value: 250,
            unit: "ms",
            detail: "slow",
          },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const benchmark = await screen.findByRole("region", { name: /benchmark suite dashboard/i });
    fireEvent.click(within(benchmark).getByRole("button", { name: /load benchmark history/i }));

    await waitFor(() => expect(qualityApi.listNativeBenchmarkRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(benchmark).findByText("History benchmark_2 · passed · 1 pass · 0 fail")).toBeInTheDocument();
    expect(within(benchmark).getByText("History benchmark_1 · warning · 0 pass · 0 fail")).toBeInTheDocument();
  });

  it("clears stale release gate results after failures and recovers on retry", async () => {
    qualityApi.runNativeReleaseGates
      .mockResolvedValueOnce({
        id: "release_gate_1",
        project_id: "proj_local",
        status: "blocked",
        scenario_count: 2,
        pass_count: 1,
        fail_count: 1,
        warning_count: 0,
        scenarios: [{ gate_id: "RG-01", name: "Evidence schema", status: "fail", detail: "missing", evidence: [] }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("gate runner down"))
      .mockResolvedValueOnce({
        id: "release_gate_2",
        project_id: "proj_local",
        status: "passed",
        scenario_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        scenarios: [{ gate_id: "RG-02", name: "DMG evidence", status: "pass", detail: "ok", evidence: ["ev_release"] }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const releaseGates = await screen.findByRole("region", { name: /release gate runner/i });
    fireEvent.click(within(releaseGates).getByRole("button", { name: /run release gates/i }));
    expect(await within(releaseGates).findByText("release_gate_1 · blocked · 1 pass · 1 fail")).toBeInTheDocument();

    fireEvent.click(within(releaseGates).getByRole("button", { name: /run release gates/i }));
    await waitFor(() => expect(releaseGates).toHaveTextContent("Release gate runner unavailable · gate runner down"));
    expect(within(releaseGates).queryByText("RG-01 · Evidence schema · fail")).not.toBeInTheDocument();

    fireEvent.click(within(releaseGates).getByRole("button", { name: /run release gates/i }));
    expect(await within(releaseGates).findByText("release_gate_2 · passed · 1 pass · 0 fail")).toBeInTheDocument();
    expect(within(releaseGates).getByText("RG-02 · DMG evidence · pass")).toBeInTheDocument();
    expect(releaseGates).not.toHaveTextContent("Release gate runner unavailable · gate runner down");
  });

  it("clears stale benchmark results after failures and recovers on retry", async () => {
    qualityApi.runNativeBenchmarks
      .mockResolvedValueOnce({
        id: "benchmark_1",
        project_id: "proj_local",
        status: "passed",
        suite_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        duration_ms: 12,
        suites: [{ suite_id: "state_snapshot_latency", name: "State snapshot latency", status: "pass", metric_value: 12, target_value: 250, unit: "ms", detail: "ok" }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("benchmark runner down"))
      .mockResolvedValueOnce({
        id: "benchmark_2",
        project_id: "proj_local",
        status: "warning",
        suite_count: 1,
        pass_count: 0,
        fail_count: 0,
        warning_count: 1,
        duration_ms: 18,
        suites: [{ suite_id: "startup_latency", name: "Startup latency", status: "warning", metric_value: 410, target_value: 350, unit: "ms", detail: "slow" }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const benchmark = await screen.findByRole("region", { name: /benchmark suite dashboard/i });
    fireEvent.click(within(benchmark).getByRole("button", { name: /run benchmark suite/i }));
    expect(await within(benchmark).findByText("benchmark_1 · passed · 1 pass · 0 fail")).toBeInTheDocument();
    expect(within(benchmark).getByText("Run benchmark_1 · passed · 1 pass · 0 fail")).toBeInTheDocument();

    fireEvent.click(within(benchmark).getByRole("button", { name: /run benchmark suite/i }));
    await waitFor(() => expect(benchmark).toHaveTextContent("Benchmark runner unavailable · benchmark runner down"));
    expect(within(benchmark).queryByText("Run benchmark_1 · passed · 1 pass · 0 fail")).not.toBeInTheDocument();

    fireEvent.click(within(benchmark).getByRole("button", { name: /run benchmark suite/i }));
    expect(await within(benchmark).findByText("benchmark_2 · warning · 0 pass · 0 fail")).toBeInTheDocument();
    expect(within(benchmark).getByText("Run benchmark_2 · warning · 0 pass · 0 fail")).toBeInTheDocument();
    expect(benchmark).not.toHaveTextContent("Benchmark runner unavailable · benchmark runner down");
  });

  it("runs dogfood telemetry review from the quality rail", async () => {
    qualityApi.runNativeDogfoodTelemetryReview.mockResolvedValueOnce({
      id: "dogfood_review_1",
      project_id: "proj_local",
      status: "warning",
      evidence_pack_id: "ev_dogfood_review_1",
      finding_count: 2,
      pass_count: 1,
      warning_count: 1,
      fail_count: 0,
      findings: [
        { finding_id: "telemetry_command_blocks", status: "warning", detail: "empty" },
        { finding_id: "telemetry_budget", status: "pass", detail: "available" },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });

    render(<App />);

    const dogfood = await screen.findByRole("region", { name: /dogfood telemetry review/i });
    fireEvent.click(within(dogfood).getByRole("button", { name: /run dogfood telemetry review/i }));
    await waitFor(() => expect(qualityApi.runNativeDogfoodTelemetryReview).toHaveBeenCalledWith("proj_local"));
    expect(await within(dogfood).findByText("dogfood_review_1 · warning · 1 pass · 1 warning · 0 fail")).toBeInTheDocument();
    expect(within(dogfood).getByText("telemetry_command_blocks · warning")).toBeInTheDocument();
    expect(within(dogfood).getByText("Evidence ev_dogfood_review_1")).toBeInTheDocument();
  });

  it("loads dogfood telemetry review history from the quality rail", async () => {
    qualityApi.listNativeDogfoodTelemetryReviews.mockResolvedValueOnce([
      {
        id: "dogfood_review_2",
        project_id: "proj_local",
        status: "passed",
        evidence_pack_id: "ev_dogfood_review_2",
        finding_count: 1,
        pass_count: 1,
        warning_count: 0,
        fail_count: 0,
        findings: [{ finding_id: "telemetry_evidence", status: "pass", detail: "complete" }],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "dogfood_review_1",
        project_id: "proj_local",
        status: "warning",
        evidence_pack_id: "ev_dogfood_review_1",
        finding_count: 2,
        pass_count: 1,
        warning_count: 1,
        fail_count: 0,
        findings: [
          { finding_id: "telemetry_command_blocks", status: "warning", detail: "empty" },
          { finding_id: "telemetry_budget", status: "pass", detail: "available" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const dogfood = await screen.findByRole("region", { name: /dogfood telemetry review/i });
    fireEvent.click(within(dogfood).getByRole("button", { name: /load dogfood telemetry history/i }));

    await waitFor(() => expect(qualityApi.listNativeDogfoodTelemetryReviews).toHaveBeenCalledWith("proj_local"));
    expect(await within(dogfood).findByText("History dogfood_review_2 · passed · 1 pass · 0 warning · 0 fail")).toBeInTheDocument();
    expect(within(dogfood).getByText("History dogfood_review_1 · warning · 1 pass · 1 warning · 0 fail")).toBeInTheDocument();
  });

  it("clears stale dogfood telemetry review results after failures and recovers on retry", async () => {
    qualityApi.runNativeDogfoodTelemetryReview
      .mockResolvedValueOnce({
        id: "dogfood_review_1",
        project_id: "proj_local",
        status: "warning",
        evidence_pack_id: "ev_dogfood_review_1",
        finding_count: 2,
        pass_count: 1,
        warning_count: 1,
        fail_count: 0,
        findings: [
          { finding_id: "telemetry_command_blocks", status: "warning", detail: "empty" },
          { finding_id: "telemetry_budget", status: "pass", detail: "available" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("telemetry review down"))
      .mockResolvedValueOnce({
        id: "dogfood_review_2",
        project_id: "proj_local",
        status: "passed",
        evidence_pack_id: "ev_dogfood_review_2",
        finding_count: 1,
        pass_count: 1,
        warning_count: 0,
        fail_count: 0,
        findings: [{ finding_id: "telemetry_evidence", status: "pass", detail: "complete" }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const dogfood = await screen.findByRole("region", { name: /dogfood telemetry review/i });
    fireEvent.click(within(dogfood).getByRole("button", { name: /run dogfood telemetry review/i }));
    expect(await within(dogfood).findByText("dogfood_review_1 · warning · 1 pass · 1 warning · 0 fail")).toBeInTheDocument();
    expect(within(dogfood).getByText("telemetry_command_blocks · warning")).toBeInTheDocument();

    fireEvent.click(within(dogfood).getByRole("button", { name: /run dogfood telemetry review/i }));
    await waitFor(() => expect(dogfood).toHaveTextContent("Dogfood telemetry review unavailable · telemetry review down"));
    expect(within(dogfood).queryByText("telemetry_command_blocks · warning")).not.toBeInTheDocument();
    expect(within(dogfood).queryByText("Evidence pack ev_dogfood_review_1")).not.toBeInTheDocument();

    fireEvent.click(within(dogfood).getByRole("button", { name: /run dogfood telemetry review/i }));
    expect(await within(dogfood).findByText("dogfood_review_2 · passed · 1 pass · 0 warning · 0 fail")).toBeInTheDocument();
    expect(within(dogfood).getByText("telemetry_evidence · pass")).toBeInTheDocument();
    expect(within(dogfood).getByText("Evidence pack ev_dogfood_review_2")).toBeInTheDocument();
    expect(dogfood).not.toHaveTextContent("Dogfood telemetry review unavailable · telemetry review down");
  });

  it("clears stale terminal fidelity smoke results after failures and recovers on retry", async () => {
    qualityApi.runNativeTerminalFidelitySmoke
      .mockResolvedValueOnce({
        id: "terminal_smoke_1",
        project_id: "proj_local",
        status: "warning",
        case_count: 3,
        pass_count: 2,
        fail_count: 0,
        warning_count: 1,
        cases: [
          { case_id: "ansi_palette", name: "ANSI palette", status: "pass", detail: "ok" },
          { case_id: "shell_resize", name: "Shell resize", status: "warning", detail: "manual review" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("terminal smoke down"))
      .mockResolvedValueOnce({
        id: "terminal_smoke_2",
        project_id: "proj_local",
        status: "passed",
        case_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        cases: [{ case_id: "scrollback", name: "Scrollback", status: "pass", detail: "ok" }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const terminal = await screen.findByRole("region", { name: /terminal fidelity smoke tests/i });
    fireEvent.click(within(terminal).getByRole("button", { name: /run terminal fidelity smoke/i }));
    expect(await within(terminal).findByText("terminal_smoke_1 · warning · 2 pass · 0 fail · 1 warning")).toBeInTheDocument();
    expect(within(terminal).getByText("ansi_palette · ANSI palette · pass")).toBeInTheDocument();

    fireEvent.click(within(terminal).getByRole("button", { name: /run terminal fidelity smoke/i }));
    await waitFor(() => expect(terminal).toHaveTextContent("Terminal fidelity smoke unavailable · terminal smoke down"));
    expect(within(terminal).queryByText("ansi_palette · ANSI palette · pass")).not.toBeInTheDocument();

    fireEvent.click(within(terminal).getByRole("button", { name: /run terminal fidelity smoke/i }));
    expect(await within(terminal).findByText("terminal_smoke_2 · passed · 1 pass · 0 fail · 0 warning")).toBeInTheDocument();
    expect(within(terminal).getByText("scrollback · Scrollback · pass")).toBeInTheDocument();
    expect(terminal).not.toHaveTextContent("Terminal fidelity smoke unavailable · terminal smoke down");
  });

  it("loads terminal fidelity smoke history from the quality rail", async () => {
    qualityApi.listNativeTerminalFidelitySmokeRuns.mockResolvedValueOnce([
      {
        id: "terminal_smoke_2",
        project_id: "proj_local",
        status: "passed",
        case_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        cases: [{ case_id: "scrollback", name: "Scrollback", status: "pass", detail: "ok" }],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "terminal_smoke_1",
        project_id: "proj_local",
        status: "warning",
        case_count: 2,
        pass_count: 1,
        fail_count: 0,
        warning_count: 1,
        cases: [
          { case_id: "ansi_palette", name: "ANSI palette", status: "pass", detail: "ok" },
          { case_id: "shell_resize", name: "Shell resize", status: "warning", detail: "manual review" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const terminal = await screen.findByRole("region", { name: /terminal fidelity smoke tests/i });
    fireEvent.click(within(terminal).getByRole("button", { name: /load terminal smoke history/i }));

    await waitFor(() => expect(qualityApi.listNativeTerminalFidelitySmokeRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(terminal).findByText("History terminal_smoke_2 · passed · 1 pass · 0 fail · 0 warning")).toBeInTheDocument();
    expect(within(terminal).getByText("History terminal_smoke_1 · warning · 1 pass · 0 fail · 1 warning")).toBeInTheDocument();
  });

  it("surfaces safe link and OSC allowlist cases from terminal fidelity smoke", async () => {
    qualityApi.runNativeTerminalFidelitySmoke.mockResolvedValueOnce({
      id: "terminal_smoke_links",
      project_id: "proj_local",
      status: "passed",
      case_count: 5,
      pass_count: 5,
      fail_count: 0,
      warning_count: 0,
      cases: [
        { case_id: "shell_basic", name: "Shell basics", status: "pass", detail: "ok" },
        { case_id: "unicode_cjk_emoji", name: "Unicode CJK emoji", status: "pass", detail: "ok" },
        { case_id: "throughput", name: "High-throughput output", status: "pass", detail: "ok" },
        { case_id: "safe_link_sanitization", name: "Safe link sanitization", status: "pass", detail: "safe" },
        { case_id: "osc_allowlist", name: "OSC allowlist", status: "pass", detail: "allowed" },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });

    render(<App />);

    const terminal = await screen.findByRole("region", { name: /terminal fidelity smoke tests/i });
    fireEvent.click(within(terminal).getByRole("button", { name: /run terminal fidelity smoke/i }));

    expect(await within(terminal).findByText("terminal_smoke_links · passed · 5 pass · 0 fail · 0 warning")).toBeInTheDocument();
    expect(within(terminal).getByText("safe_link_sanitization · Safe link sanitization · pass")).toBeInTheDocument();
    expect(within(terminal).getByText("osc_allowlist · OSC allowlist · pass")).toBeInTheDocument();
  });

  it("clears stale task lifecycle E2E results after failures and recovers on retry", async () => {
    qualityApi.runNativeTaskLifecycleE2E
      .mockResolvedValueOnce({
        id: "task_lifecycle_e2e_1",
        project_id: "proj_local",
        status: "passed",
        task_id: "task_e2e_1",
        run_id: "run_e2e_1",
        evidence_pack_id: "ev_lifecycle_run_e2e_1",
        transitions: [
          { step: "ready", task_status: "ready", run_lifecycle: null, evidence_pack_id: null },
          { step: "done", task_status: "done", run_lifecycle: "completed", evidence_pack_id: "ev_lifecycle_run_e2e_1" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("lifecycle runner down"))
      .mockResolvedValueOnce({
        id: "task_lifecycle_e2e_2",
        project_id: "proj_local",
        status: "passed",
        task_id: "task_e2e_2",
        run_id: "run_e2e_2",
        evidence_pack_id: "ev_lifecycle_run_e2e_2",
        transitions: [
          { step: "assigned", task_status: "assigned", run_lifecycle: null, evidence_pack_id: null },
          { step: "verified", task_status: "done", run_lifecycle: "completed", evidence_pack_id: "ev_lifecycle_run_e2e_2" },
        ],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const lifecycle = await screen.findByRole("region", { name: /task lifecycle e2e/i });
    fireEvent.click(within(lifecycle).getByRole("button", { name: /run task lifecycle e2e/i }));
    expect(await within(lifecycle).findByText("task_lifecycle_e2e_1 · passed · task task_e2e_1 · run run_e2e_1")).toBeInTheDocument();
    expect(within(lifecycle).getByText("Evidence pack ev_lifecycle_run_e2e_1")).toBeInTheDocument();
    expect(within(lifecycle).getByText("done · done · completed")).toBeInTheDocument();

    fireEvent.click(within(lifecycle).getByRole("button", { name: /run task lifecycle e2e/i }));
    await waitFor(() => expect(lifecycle).toHaveTextContent("Task lifecycle E2E unavailable · lifecycle runner down"));
    expect(within(lifecycle).queryByText("Evidence pack ev_lifecycle_run_e2e_1")).not.toBeInTheDocument();
    expect(within(lifecycle).queryByText("done · done · completed")).not.toBeInTheDocument();

    fireEvent.click(within(lifecycle).getByRole("button", { name: /run task lifecycle e2e/i }));
    expect(await within(lifecycle).findByText("task_lifecycle_e2e_2 · passed · task task_e2e_2 · run run_e2e_2")).toBeInTheDocument();
    expect(within(lifecycle).getByText("Evidence pack ev_lifecycle_run_e2e_2")).toBeInTheDocument();
    expect(within(lifecycle).getByText("verified · done · completed")).toBeInTheDocument();
    expect(lifecycle).not.toHaveTextContent("Task lifecycle E2E unavailable · lifecycle runner down");
  });

  it("loads task lifecycle E2E history from the quality rail", async () => {
    qualityApi.listNativeTaskLifecycleE2ERuns.mockResolvedValueOnce([
      {
        id: "task_lifecycle_e2e_2",
        project_id: "proj_local",
        status: "passed",
        task_id: "task_e2e_2",
        run_id: "run_e2e_2",
        evidence_pack_id: "ev_lifecycle_run_e2e_2",
        transitions: [
          { step: "assigned", task_status: "assigned", run_lifecycle: null, evidence_pack_id: null },
          { step: "verified", task_status: "done", run_lifecycle: "completed", evidence_pack_id: "ev_lifecycle_run_e2e_2" },
        ],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "task_lifecycle_e2e_1",
        project_id: "proj_local",
        status: "passed",
        task_id: "task_e2e_1",
        run_id: "run_e2e_1",
        evidence_pack_id: "ev_lifecycle_run_e2e_1",
        transitions: [
          { step: "ready", task_status: "ready", run_lifecycle: null, evidence_pack_id: null },
          { step: "done", task_status: "done", run_lifecycle: "completed", evidence_pack_id: "ev_lifecycle_run_e2e_1" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const lifecycle = await screen.findByRole("region", { name: /task lifecycle e2e/i });
    fireEvent.click(within(lifecycle).getByRole("button", { name: /load task lifecycle history/i }));

    await waitFor(() => expect(qualityApi.listNativeTaskLifecycleE2ERuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(lifecycle).findByText("History task_lifecycle_e2e_2 · passed · task task_e2e_2 · run run_e2e_2")).toBeInTheDocument();
    expect(within(lifecycle).getByText("History task_lifecycle_e2e_1 · passed · task task_e2e_1 · run run_e2e_1")).toBeInTheDocument();
  });

  it("clears stale workflow negative test results after failures and recovers on retry", async () => {
    qualityApi.runNativeWorkflowNegativeTests
      .mockResolvedValueOnce({
        id: "workflow_negative_1",
        project_id: "proj_local",
        status: "passed",
        baseline_workflow_id: "wf_base",
        invalid_workflow_id: "wf_bad",
        last_known_good_workflow_id: "wf_base",
        dispatch_run_id: "run_lkg",
        cases: [
          { case_id: "invalid_reload_preserves_lkg", status: "pass", detail: "LKG kept" },
          { case_id: "unsafe_hook_rejected", status: "pass", detail: "blocked" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("workflow negative down"))
      .mockResolvedValueOnce({
        id: "workflow_negative_2",
        project_id: "proj_local",
        status: "passed",
        baseline_workflow_id: "wf_base_2",
        invalid_workflow_id: "wf_bad_2",
        last_known_good_workflow_id: "wf_base_2",
        dispatch_run_id: "run_lkg_2",
        cases: [{ case_id: "unsafe_hook_rejected", status: "pass", detail: "blocked" }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const workflow = await screen.findByRole("region", { name: /workflow negative tests/i });
    fireEvent.click(within(workflow).getByRole("button", { name: /run workflow negative tests/i }));
    expect(await within(workflow).findByText("workflow_negative_1 · passed · baseline wf_base · LKG wf_base")).toBeInTheDocument();
    expect(within(workflow).getByText("invalid_reload_preserves_lkg · pass")).toBeInTheDocument();

    fireEvent.click(within(workflow).getByRole("button", { name: /run workflow negative tests/i }));
    await waitFor(() => expect(workflow).toHaveTextContent("Workflow negative tests unavailable · workflow negative down"));
    expect(within(workflow).queryByText("invalid_reload_preserves_lkg · pass")).not.toBeInTheDocument();

    fireEvent.click(within(workflow).getByRole("button", { name: /run workflow negative tests/i }));
    expect(await within(workflow).findByText("workflow_negative_2 · passed · baseline wf_base_2 · LKG wf_base_2")).toBeInTheDocument();
    expect(within(workflow).getByText("unsafe_hook_rejected · pass")).toBeInTheDocument();
    expect(workflow).not.toHaveTextContent("Workflow negative tests unavailable · workflow negative down");
  });

  it("loads workflow negative test history from the quality rail", async () => {
    qualityApi.listNativeWorkflowNegativeTestRuns.mockResolvedValueOnce([
      {
        id: "workflow_negative_2",
        project_id: "proj_local",
        status: "passed",
        baseline_workflow_id: "wf_base_2",
        invalid_workflow_id: "wf_bad_2",
        last_known_good_workflow_id: "wf_base_2",
        dispatch_run_id: "run_lkg_2",
        cases: [{ case_id: "unsafe_hook_rejected", status: "pass", detail: "blocked" }],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "workflow_negative_1",
        project_id: "proj_local",
        status: "passed",
        baseline_workflow_id: "wf_base",
        invalid_workflow_id: "wf_bad",
        last_known_good_workflow_id: "wf_base",
        dispatch_run_id: "run_lkg",
        cases: [
          { case_id: "invalid_reload_preserves_lkg", status: "pass", detail: "LKG kept" },
          { case_id: "unsafe_hook_rejected", status: "pass", detail: "blocked" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const workflow = await screen.findByRole("region", { name: /workflow negative tests/i });
    fireEvent.click(within(workflow).getByRole("button", { name: /load workflow negative history/i }));

    await waitFor(() => expect(qualityApi.listNativeWorkflowNegativeTestRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(workflow).findByText("History workflow_negative_2 · passed · baseline wf_base_2 · LKG wf_base_2")).toBeInTheDocument();
    expect(within(workflow).getByText("History workflow_negative_1 · passed · baseline wf_base · LKG wf_base")).toBeInTheDocument();
  });

  it("clears stale DMG smoke results after failures and recovers on retry", async () => {
    qualityApi.runNativeDmgSmokeTest
      .mockResolvedValueOnce({
        id: "dmg_smoke_1",
        project_id: "proj_local",
        status: "blocked",
        explicit_blocker: true,
        dmg_path: "/tmp/Haneulchi.dmg",
        app_bundle_path: "/Applications/Haneulchi.app",
        case_count: 2,
        pass_count: 1,
        fail_count: 1,
        warning_count: 0,
        cases: [
          { case_id: "dmg_artifact", name: "DMG artifact", status: "fail", detail: "missing signature" },
          { case_id: "mount", name: "Mount", status: "pass", detail: "mounted" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("dmg runner down"))
      .mockResolvedValueOnce({
        id: "dmg_smoke_2",
        project_id: "proj_local",
        status: "passed",
        explicit_blocker: false,
        dmg_path: "/tmp/Haneulchi.dmg",
        app_bundle_path: "/Applications/Haneulchi.app",
        case_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        cases: [{ case_id: "launch", name: "Launch app", status: "pass", detail: "opened" }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const dmg = await screen.findByRole("region", { name: /dmg install smoke test/i });
    fireEvent.change(within(dmg).getByLabelText(/dmg artifact path/i), { target: { value: "/tmp/Haneulchi.dmg" } });
    fireEvent.change(within(dmg).getByLabelText(/app bundle path/i), { target: { value: "/Applications/Haneulchi.app" } });
    fireEvent.click(within(dmg).getByRole("button", { name: /run dmg smoke test/i }));
    expect(await within(dmg).findByText("dmg_smoke_1 · blocked · 1 pass · 1 fail · blocker true")).toBeInTheDocument();
    expect(within(dmg).getByText("dmg_artifact · DMG artifact · fail")).toBeInTheDocument();

    fireEvent.click(within(dmg).getByRole("button", { name: /run dmg smoke test/i }));
    await waitFor(() => expect(dmg).toHaveTextContent("DMG smoke test unavailable · dmg runner down"));
    expect(within(dmg).queryByText("dmg_artifact · DMG artifact · fail")).not.toBeInTheDocument();

    fireEvent.click(within(dmg).getByRole("button", { name: /run dmg smoke test/i }));
    expect(await within(dmg).findByText("dmg_smoke_2 · passed · 1 pass · 0 fail · blocker false")).toBeInTheDocument();
    expect(within(dmg).getByText("launch · Launch app · pass")).toBeInTheDocument();
    expect(dmg).not.toHaveTextContent("DMG smoke test unavailable · dmg runner down");
  });

  it("loads DMG smoke run history from the quality rail", async () => {
    qualityApi.listNativeDmgSmokeRuns.mockResolvedValueOnce([
      {
        id: "dmg_smoke_2",
        project_id: "proj_local",
        status: "passed",
        explicit_blocker: false,
        dmg_path: "/tmp/Haneulchi.dmg",
        app_bundle_path: "/Applications/Haneulchi.app",
        case_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        cases: [{ case_id: "launch", name: "Launch app", status: "pass", detail: "opened" }],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "dmg_smoke_1",
        project_id: "proj_local",
        status: "blocked",
        explicit_blocker: true,
        dmg_path: "/tmp/Haneulchi.dmg",
        app_bundle_path: "/Applications/Haneulchi.app",
        case_count: 2,
        pass_count: 1,
        fail_count: 1,
        warning_count: 0,
        cases: [
          { case_id: "dmg_artifact", name: "DMG artifact", status: "fail", detail: "missing signature" },
          { case_id: "mount", name: "Mount", status: "pass", detail: "mounted" },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const dmg = await screen.findByRole("region", { name: /dmg install smoke test/i });
    fireEvent.click(within(dmg).getByRole("button", { name: /load dmg smoke history/i }));

    await waitFor(() => expect(qualityApi.listNativeDmgSmokeRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(dmg).findByText("History dmg_smoke_2 · passed · 1 pass · 0 fail · blocker false")).toBeInTheDocument();
    expect(within(dmg).getByText("History dmg_smoke_1 · blocked · 1 pass · 1 fail · blocker true")).toBeInTheDocument();
  });

  it("clears stale recovery drill results after failures and recovers on retry", async () => {
    qualityApi.runNativeRecoveryDrills
      .mockResolvedValueOnce({
        id: "recovery_drill_1",
        project_id: "proj_local",
        status: "passed",
        drill_count: 2,
        pass_count: 2,
        fail_count: 0,
        warning_count: 0,
        drills: [
          { drill_id: "invalid_workflow_lkg", name: "Invalid workflow recovery", status: "pass", detail: "ok", evidence: ["wf_base"] },
          { drill_id: "db_degraded", name: "DB degraded fallback", status: "pass", detail: "ok", evidence: ["snap_1"] },
        ],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("recovery runner down"))
      .mockResolvedValueOnce({
        id: "recovery_drill_2",
        project_id: "proj_local",
        status: "passed",
        drill_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        drills: [{ drill_id: "restore_snapshot", name: "Restore snapshot", status: "pass", detail: "ok", evidence: ["snap_2"] }],
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const recovery = await screen.findByRole("region", { name: /recovery drills/i });
    fireEvent.click(within(recovery).getByRole("button", { name: /run recovery drills/i }));
    expect(await within(recovery).findByText("recovery_drill_1 · passed · 2 pass · 0 fail")).toBeInTheDocument();
    expect(within(recovery).getByText("invalid_workflow_lkg · Invalid workflow recovery · pass")).toBeInTheDocument();

    fireEvent.click(within(recovery).getByRole("button", { name: /run recovery drills/i }));
    await waitFor(() => expect(recovery).toHaveTextContent("Recovery drills unavailable · recovery runner down"));
    expect(within(recovery).queryByText("invalid_workflow_lkg · Invalid workflow recovery · pass")).not.toBeInTheDocument();

    fireEvent.click(within(recovery).getByRole("button", { name: /run recovery drills/i }));
    expect(await within(recovery).findByText("recovery_drill_2 · passed · 1 pass · 0 fail")).toBeInTheDocument();
    expect(within(recovery).getByText("restore_snapshot · Restore snapshot · pass")).toBeInTheDocument();
    expect(recovery).not.toHaveTextContent("Recovery drills unavailable · recovery runner down");
  });

  it("loads recovery drill run history from the quality rail", async () => {
    qualityApi.listNativeRecoveryDrillRuns.mockResolvedValueOnce([
      {
        id: "recovery_drill_2",
        project_id: "proj_local",
        status: "passed",
        drill_count: 1,
        pass_count: 1,
        fail_count: 0,
        warning_count: 0,
        drills: [{ drill_id: "restore_snapshot", name: "Restore snapshot", status: "pass", detail: "ok", evidence: ["snap_2"] }],
        created_at: "2026-05-02T01:05:00Z",
      },
      {
        id: "recovery_drill_1",
        project_id: "proj_local",
        status: "passed",
        drill_count: 2,
        pass_count: 2,
        fail_count: 0,
        warning_count: 0,
        drills: [
          { drill_id: "invalid_workflow_lkg", name: "Invalid workflow recovery", status: "pass", detail: "ok", evidence: ["wf_base"] },
          { drill_id: "db_degraded", name: "DB degraded fallback", status: "pass", detail: "ok", evidence: ["snap_1"] },
        ],
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const recovery = await screen.findByRole("region", { name: /recovery drills/i });
    fireEvent.click(within(recovery).getByRole("button", { name: /load recovery drill history/i }));

    await waitFor(() => expect(qualityApi.listNativeRecoveryDrillRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(recovery).findByText("History recovery_drill_2 · passed · 1 pass · 0 fail")).toBeInTheDocument();
    expect(within(recovery).getByText("History recovery_drill_1 · passed · 2 pass · 0 fail")).toBeInTheDocument();
  });

  it("runs hardening drills from the quality rail", async () => {
    qualityApi.runNativeTerminalFidelitySmoke.mockResolvedValueOnce({
      id: "terminal_smoke_1",
      project_id: "proj_local",
      status: "warning",
      case_count: 3,
      pass_count: 2,
      fail_count: 0,
      warning_count: 1,
      cases: [
        { case_id: "ansi_palette", name: "ANSI palette", status: "pass", detail: "ok" },
        { case_id: "shell_resize", name: "Shell resize", status: "warning", detail: "manual review" },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });
    qualityApi.runNativeTaskLifecycleE2E.mockResolvedValueOnce({
      id: "task_lifecycle_e2e_1",
      project_id: "proj_local",
      status: "passed",
      task_id: "task_e2e_1",
      run_id: "run_e2e_1",
      evidence_pack_id: "ev_lifecycle_run_e2e_1",
      transitions: [
        { step: "ready", task_status: "ready", run_lifecycle: null, evidence_pack_id: null },
        { step: "done", task_status: "done", run_lifecycle: "completed", evidence_pack_id: "ev_lifecycle_run_e2e_1" },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });
    qualityApi.runNativeWorkflowNegativeTests.mockResolvedValueOnce({
      id: "workflow_negative_1",
      project_id: "proj_local",
      status: "passed",
      baseline_workflow_id: "wf_base",
      invalid_workflow_id: "wf_bad",
      last_known_good_workflow_id: "wf_base",
      dispatch_run_id: "run_lkg",
      cases: [
        { case_id: "invalid_reload_preserves_lkg", status: "pass", detail: "LKG kept" },
        { case_id: "unsafe_hook_rejected", status: "pass", detail: "blocked" },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });
    qualityApi.runNativeDmgSmokeTest.mockResolvedValueOnce({
      id: "dmg_smoke_1",
      project_id: "proj_local",
      status: "blocked",
      explicit_blocker: true,
      dmg_path: "/tmp/Haneulchi.dmg",
      app_bundle_path: "/Applications/Haneulchi.app",
      case_count: 4,
      pass_count: 3,
      fail_count: 1,
      warning_count: 0,
      cases: [
        { case_id: "dmg_artifact", name: "DMG artifact", status: "fail", detail: "missing signature" },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });
    qualityApi.runNativeRecoveryDrills.mockResolvedValueOnce({
      id: "recovery_drill_1",
      project_id: "proj_local",
      status: "passed",
      drill_count: 4,
      pass_count: 4,
      fail_count: 0,
      warning_count: 0,
      drills: [
        { drill_id: "invalid_workflow_lkg", name: "Invalid workflow recovery", status: "pass", detail: "ok", evidence: ["wf_base"] },
      ],
      created_at: "2026-05-02T01:00:00Z",
    });

    render(<App />);

    const terminal = await screen.findByRole("region", { name: /terminal fidelity smoke tests/i });
    fireEvent.click(within(terminal).getByRole("button", { name: /run terminal fidelity smoke/i }));
    await waitFor(() => expect(qualityApi.runNativeTerminalFidelitySmoke).toHaveBeenCalledWith("proj_local"));
    expect(await within(terminal).findByText("terminal_smoke_1 · warning · 2 pass · 0 fail · 1 warning")).toBeInTheDocument();
    expect(within(terminal).getByText("ansi_palette · ANSI palette · pass")).toBeInTheDocument();

    const lifecycle = await screen.findByRole("region", { name: /task lifecycle e2e/i });
    fireEvent.click(within(lifecycle).getByRole("button", { name: /run task lifecycle e2e/i }));
    await waitFor(() => expect(qualityApi.runNativeTaskLifecycleE2E).toHaveBeenCalledWith("proj_local"));
    expect(await within(lifecycle).findByText("task_lifecycle_e2e_1 · passed · task task_e2e_1 · run run_e2e_1")).toBeInTheDocument();
    expect(within(lifecycle).getByText("done · done · completed")).toBeInTheDocument();

    const workflow = await screen.findByRole("region", { name: /workflow negative tests/i });
    fireEvent.click(within(workflow).getByRole("button", { name: /run workflow negative tests/i }));
    await waitFor(() => expect(qualityApi.runNativeWorkflowNegativeTests).toHaveBeenCalledWith("proj_local"));
    expect(await within(workflow).findByText("workflow_negative_1 · passed · baseline wf_base · LKG wf_base")).toBeInTheDocument();
    expect(within(workflow).getByText("invalid_reload_preserves_lkg · pass")).toBeInTheDocument();

    const dmg = await screen.findByRole("region", { name: /dmg install smoke test/i });
    fireEvent.change(within(dmg).getByLabelText(/dmg artifact path/i), { target: { value: "/tmp/Haneulchi.dmg" } });
    fireEvent.change(within(dmg).getByLabelText(/app bundle path/i), { target: { value: "/Applications/Haneulchi.app" } });
    fireEvent.click(within(dmg).getByRole("button", { name: /run dmg smoke test/i }));
    await waitFor(() => expect(qualityApi.runNativeDmgSmokeTest).toHaveBeenCalledWith("proj_local", "/tmp/Haneulchi.dmg", "/Applications/Haneulchi.app"));
    expect(await within(dmg).findByText("dmg_smoke_1 · blocked · 3 pass · 1 fail · blocker true")).toBeInTheDocument();
    expect(within(dmg).getByText("dmg_artifact · DMG artifact · fail")).toBeInTheDocument();

    const recovery = await screen.findByRole("region", { name: /recovery drills/i });
    fireEvent.click(within(recovery).getByRole("button", { name: /run recovery drills/i }));
    await waitFor(() => expect(qualityApi.runNativeRecoveryDrills).toHaveBeenCalledWith("proj_local"));
    expect(await within(recovery).findByText("recovery_drill_1 · passed · 4 pass · 0 fail")).toBeInTheDocument();
    expect(within(recovery).getByText("invalid_workflow_lkg · Invalid workflow recovery · pass")).toBeInTheDocument();
  });

  it("renders visual harness canvas nodes and edges from state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_visual_harness",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: "workflow_1", last_known_good_version_id: "workflow_1", diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      visual_harness: {
        nodes: [
          { id: "task_1", label: "Implement graph", kind: "task", status: "ready" },
          { id: "run_1", label: "run_1", kind: "run", status: "queued" },
          { id: "review_ev_run_1", label: "ev_run_1", kind: "review_gate", status: "pending" },
          { id: "tool_browser", label: "localhost preview", kind: "tool", status: "configured" },
        ],
        edges: [
          { id: "edge_task_1_run_1", source_id: "task_1", target_id: "run_1", kind: "dispatch", status: "active" },
          { id: "edge_run_1_review_ev_run_1", source_id: "run_1", target_id: "review_ev_run_1", kind: "review_gate", status: "incomplete" },
          { id: "edge_workflow_1_tool_browser", source_id: "workflow_1", target_id: "tool_browser", kind: "tool", status: "configured" },
        ],
        diagnostics: { status: "ok", node_count: 4, edge_count: 3 },
      },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const canvas = await screen.findByRole("region", { name: /visual harness canvas/i });
    expect(canvas).toHaveTextContent("Nodes 4");
    expect(canvas).toHaveTextContent("Edges 3");
    expect(within(canvas).getByRole("img", { name: /visual harness dependency graph/i })).toBeInTheDocument();
    expect(within(canvas).getByLabelText("Implement graph node")).toBeInTheDocument();
    expect(within(canvas).getByLabelText("run_1 node")).toBeInTheDocument();
    expect(within(canvas).getByLabelText("ev_run_1 node")).toBeInTheDocument();
    expect(within(canvas).getByLabelText("localhost preview node")).toBeInTheDocument();
    expect(within(canvas).getByLabelText("task_1 to run_1 dispatch edge")).toBeInTheDocument();
    expect(within(canvas).getByLabelText("run_1 to review_ev_run_1 review_gate edge")).toBeInTheDocument();
    expect(canvas).toHaveTextContent("Implement graph · task · ready");
    expect(canvas).toHaveTextContent("task_1 -> run_1 · dispatch");
  });

  it("creates manual visual harness links from the canvas panel", async () => {
    visualHarnessApi.createNativeVisualHarnessLink.mockResolvedValueOnce({
      id: "visual_link_1",
      project_id: "proj_local",
      source_id: "ctx_default",
      target_id: "task_1",
      kind: "context",
      created_at: "2026-05-02T01:00:00Z",
    });

    render(<App />);

    const canvas = await screen.findByRole("region", { name: /visual harness canvas/i });
    fireEvent.change(within(canvas).getByLabelText("Visual harness source"), { target: { value: "ctx_default" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness target"), { target: { value: "task_1" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness link kind"), { target: { value: "context" } });
    fireEvent.click(within(canvas).getByRole("button", { name: /create visual harness link/i }));

    await waitFor(() => expect(visualHarnessApi.createNativeVisualHarnessLink).toHaveBeenCalledWith({
      projectId: "proj_local",
      sourceId: "ctx_default",
      targetId: "task_1",
      kind: "context",
    }));
    expect(await within(canvas).findByText("ctx_default -> task_1 · context")).toBeInTheDocument();
    expect(within(canvas).getByText("Created visual link visual_link_1")).toBeInTheDocument();
  });

  it("loads native visual harness links from the canvas panel", async () => {
    visualHarnessApi.listNativeVisualHarnessLinks.mockResolvedValueOnce([
      {
        id: "visual_link_1",
        project_id: "proj_local",
        source_id: "ctx_default",
        target_id: "task_1",
        kind: "context",
        created_at: "2026-05-02T01:00:00Z",
      },
      {
        id: "visual_link_2",
        project_id: "proj_local",
        source_id: "tool_browser",
        target_id: "run_1",
        kind: "tool",
        created_at: "2026-05-02T01:05:00Z",
      },
    ]);

    render(<App />);

    const canvas = await screen.findByRole("region", { name: /visual harness canvas/i });
    fireEvent.click(within(canvas).getByRole("button", { name: /load visual harness links/i }));

    await waitFor(() => expect(visualHarnessApi.listNativeVisualHarnessLinks).toHaveBeenCalledWith("proj_local"));
    expect(await within(canvas).findByText("ctx_default -> task_1 · context")).toBeInTheDocument();
    expect(within(canvas).getByText("tool_browser -> run_1 · tool")).toBeInTheDocument();
    expect(canvas).toHaveTextContent("Edges 2");
  });

  it("clears stale visual harness link results after failures and recovers on retry", async () => {
    visualHarnessApi.createNativeVisualHarnessLink
      .mockResolvedValueOnce({
        id: "visual_link_1",
        project_id: "proj_local",
        source_id: "ctx_default",
        target_id: "task_1",
        kind: "context",
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("visual harness down"))
      .mockResolvedValueOnce({
        id: "visual_link_2",
        project_id: "proj_local",
        source_id: "tool_browser",
        target_id: "run_1",
        kind: "tool",
        created_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const canvas = await screen.findByRole("region", { name: /visual harness canvas/i });
    fireEvent.change(within(canvas).getByLabelText("Visual harness source"), { target: { value: "ctx_default" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness target"), { target: { value: "task_1" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness link kind"), { target: { value: "context" } });
    fireEvent.click(within(canvas).getByRole("button", { name: /create visual harness link/i }));
    expect(await within(canvas).findByText("Created visual link visual_link_1")).toBeInTheDocument();

    fireEvent.change(within(canvas).getByLabelText("Visual harness source"), { target: { value: "ctx_default" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness target"), { target: { value: "run_1" } });
    fireEvent.click(within(canvas).getByRole("button", { name: /create visual harness link/i }));
    await waitFor(() => expect(canvas).toHaveTextContent("Visual harness link unavailable · visual harness down"));
    expect(within(canvas).queryByText("Created visual link visual_link_1")).not.toBeInTheDocument();

    fireEvent.change(within(canvas).getByLabelText("Visual harness source"), { target: { value: "tool_browser" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness target"), { target: { value: "run_1" } });
    fireEvent.change(within(canvas).getByLabelText("Visual harness link kind"), { target: { value: "tool" } });
    fireEvent.click(within(canvas).getByRole("button", { name: /create visual harness link/i }));
    expect(await within(canvas).findByText("Created visual link visual_link_2")).toBeInTheDocument();
    expect(within(canvas).getByText("Manual edge tool_browser -> run_1 · tool")).toBeInTheDocument();
    expect(canvas).not.toHaveTextContent("Visual harness link unavailable · visual harness down");
  });

  it("creates visual harness links by dragging between canvas nodes", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_visual_drag",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: "workflow_1", last_known_good_version_id: "workflow_1", diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      visual_harness: {
        nodes: [
          { id: "ctx_default", label: "Context default", kind: "context", status: "linked" },
          { id: "task_1", label: "Implement graph", kind: "task", status: "ready" },
        ],
        edges: [],
        diagnostics: { status: "ok", node_count: 2, edge_count: 0 },
      },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    visualHarnessApi.createNativeVisualHarnessLink.mockResolvedValueOnce({
      id: "visual_link_1",
      project_id: "proj_local",
      source_id: "ctx_default",
      target_id: "task_1",
      kind: "context",
      created_at: "2026-05-02T01:00:00Z",
    });

    render(<App />);

    const canvas = await screen.findByRole("region", { name: /visual harness canvas/i });
    fireEvent.pointerDown(within(canvas).getByLabelText("Context default node"));
    fireEvent.pointerUp(within(canvas).getByLabelText("Implement graph node"));

    await waitFor(() => expect(visualHarnessApi.createNativeVisualHarnessLink).toHaveBeenCalledWith({
      projectId: "proj_local",
      sourceId: "ctx_default",
      targetId: "task_1",
      kind: "context",
    }));
    expect(await within(canvas).findByText("ctx_default -> task_1 · context")).toBeInTheDocument();
  });

  it("binds external trackers and runs provider sync from the tracker boundary panel", async () => {
    trackerApi.upsertNativeExternalTrackerBinding.mockResolvedValueOnce({
      id: "tracker_binding_1",
      project_id: "proj_local",
      local_kind: "task",
      local_id: "task_1",
      provider: "linear",
      external_id: "LIN-42",
      external_url: "https://linear.app/acme/issue/LIN-42",
      sync_mode: "mirror",
      sync_status: "pending",
      conflict_state: "none",
      metadata_json: {},
      created_at: "2026-05-03T01:00:00Z",
      updated_at: "2026-05-03T01:00:00Z",
    });
    trackerApi.runNativeTrackerSync.mockResolvedValueOnce({
      id: "tracker_sync_1",
      project_id: "proj_local",
      provider: "linear",
      dry_run: true,
      status: "planned",
      operation_count: 1,
      degraded_reason: null,
      operations: [{ binding_id: "tracker_binding_1", local_kind: "task", local_id: "task_1", external_id: "LIN-42", operation: "issueUpdate", payload: {} }],
      created_at: "2026-05-03T01:00:00Z",
    });

    render(<App />);

    const tracker = await screen.findByRole("region", { name: /external tracker boundary/i });
    fireEvent.change(within(tracker).getByLabelText("Tracker local id"), { target: { value: "task_1" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker external id"), { target: { value: "LIN-42" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker external URL"), { target: { value: "https://linear.app/acme/issue/LIN-42" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker sync mode"), { target: { value: "mirror" } });
    fireEvent.click(within(tracker).getByRole("button", { name: /bind external tracker/i }));

    await waitFor(() => expect(trackerApi.upsertNativeExternalTrackerBinding).toHaveBeenCalledWith({
      projectId: "proj_local",
      localKind: "task",
      localId: "task_1",
      provider: "linear",
      externalId: "LIN-42",
      externalUrl: "https://linear.app/acme/issue/LIN-42",
      syncMode: "mirror",
    }));
    expect(await within(tracker).findByText("LIN-42 · linear · pending")).toBeInTheDocument();

    fireEvent.click(within(tracker).getByRole("button", { name: /run tracker sync/i }));
    await waitFor(() => expect(trackerApi.runNativeTrackerSync).toHaveBeenCalledWith({
      projectId: "proj_local",
      provider: "linear",
      dryRun: true,
    }));
    expect(await within(tracker).findByText("tracker_sync_1 · linear · planned · 1 operation")).toBeInTheDocument();
  });

  it("loads native external tracker bindings from the tracker boundary panel", async () => {
    trackerApi.listNativeExternalTrackerBindings.mockResolvedValueOnce([
      {
        id: "tracker_binding_1",
        project_id: "proj_local",
        local_kind: "task",
        local_id: "task_1",
        provider: "linear",
        external_id: "LIN-42",
        external_url: "https://linear.app/acme/issue/LIN-42",
        sync_mode: "mirror",
        sync_status: "pending",
        conflict_state: "none",
        metadata_json: {},
        created_at: "2026-05-03T01:00:00Z",
        updated_at: "2026-05-03T01:00:00Z",
      },
      {
        id: "tracker_binding_2",
        project_id: "proj_local",
        local_kind: "task",
        local_id: "task_2",
        provider: "github",
        external_id: "octo/repo#7",
        external_url: "https://github.com/octo/repo/issues/7",
        sync_mode: "manual",
        sync_status: "synced",
        conflict_state: "none",
        metadata_json: {},
        created_at: "2026-05-03T01:05:00Z",
        updated_at: "2026-05-03T01:05:00Z",
      },
    ]);

    render(<App />);

    const tracker = await screen.findByRole("region", { name: /external tracker boundary/i });
    fireEvent.click(within(tracker).getByRole("button", { name: /load tracker bindings/i }));

    await waitFor(() => expect(trackerApi.listNativeExternalTrackerBindings).toHaveBeenCalledWith("proj_local"));
    expect(await within(tracker).findByText("Bindings 2")).toBeInTheDocument();
    expect(within(tracker).getByText("LIN-42 · linear · pending")).toBeInTheDocument();
    expect(within(tracker).getByText("octo/repo#7 · github · synced")).toBeInTheDocument();
  });

  it("clears stale external tracker bindings after failures and recovers on retry", async () => {
    trackerApi.upsertNativeExternalTrackerBinding
      .mockResolvedValueOnce({
        id: "tracker_binding_1",
        project_id: "proj_local",
        local_kind: "task",
        local_id: "task_1",
        provider: "linear",
        external_id: "LIN-42",
        external_url: "https://linear.app/acme/issue/LIN-42",
        sync_mode: "mirror",
        sync_status: "pending",
        conflict_state: "none",
        metadata_json: {},
        created_at: "2026-05-03T01:00:00Z",
        updated_at: "2026-05-03T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("tracker binding down"))
      .mockResolvedValueOnce({
        id: "tracker_binding_2",
        project_id: "proj_local",
        local_kind: "task",
        local_id: "task_2",
        provider: "linear",
        external_id: "LIN-43",
        external_url: "https://linear.app/acme/issue/LIN-43",
        sync_mode: "mirror",
        sync_status: "pending",
        conflict_state: "none",
        metadata_json: {},
        created_at: "2026-05-03T01:05:00Z",
        updated_at: "2026-05-03T01:05:00Z",
      });

    render(<App />);

    const tracker = await screen.findByRole("region", { name: /external tracker boundary/i });
    fireEvent.change(within(tracker).getByLabelText("Tracker local id"), { target: { value: "task_1" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker external id"), { target: { value: "LIN-42" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker external URL"), { target: { value: "https://linear.app/acme/issue/LIN-42" } });
    fireEvent.click(within(tracker).getByRole("button", { name: /bind external tracker/i }));
    expect(await within(tracker).findByText("LIN-42 · linear · pending")).toBeInTheDocument();

    fireEvent.change(within(tracker).getByLabelText("Tracker local id"), { target: { value: "task_2" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker external id"), { target: { value: "LIN-43" } });
    fireEvent.change(within(tracker).getByLabelText("Tracker external URL"), { target: { value: "https://linear.app/acme/issue/LIN-43" } });
    fireEvent.click(within(tracker).getByRole("button", { name: /bind external tracker/i }));
    await waitFor(() => expect(tracker).toHaveTextContent("Tracker binding unavailable · tracker binding down"));
    expect(within(tracker).queryByText("task task_1 · mirror")).not.toBeInTheDocument();

    fireEvent.click(within(tracker).getByRole("button", { name: /bind external tracker/i }));
    expect(await within(tracker).findByText("LIN-43 · linear · pending")).toBeInTheDocument();
    expect(within(tracker).getByText("task task_2 · mirror")).toBeInTheDocument();
    expect(tracker).not.toHaveTextContent("Tracker binding unavailable · tracker binding down");
  });

  it("clears stale tracker sync runs after failures and recovers on retry", async () => {
    trackerApi.runNativeTrackerSync
      .mockResolvedValueOnce({
        id: "tracker_sync_1",
        project_id: "proj_local",
        provider: "linear",
        dry_run: true,
        status: "planned",
        operation_count: 1,
        degraded_reason: null,
        operations: [{ binding_id: "tracker_binding_1", local_kind: "task", local_id: "task_1", external_id: "LIN-42", operation: "issueUpdate", payload: {} }],
        created_at: "2026-05-03T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("tracker sync down"))
      .mockResolvedValueOnce({
        id: "tracker_sync_2",
        project_id: "proj_local",
        provider: "linear",
        dry_run: true,
        status: "planned",
        operation_count: 2,
        degraded_reason: null,
        operations: [
          { binding_id: "tracker_binding_1", local_kind: "task", local_id: "task_1", external_id: "LIN-42", operation: "issueUpdate", payload: {} },
          { binding_id: "tracker_binding_2", local_kind: "task", local_id: "task_2", external_id: "LIN-43", operation: "commentMirror", payload: {} },
        ],
        created_at: "2026-05-03T01:05:00Z",
      });

    render(<App />);

    const tracker = await screen.findByRole("region", { name: /external tracker boundary/i });
    fireEvent.click(within(tracker).getByRole("button", { name: /run tracker sync/i }));
    expect(await within(tracker).findByText("tracker_sync_1 · linear · planned · 1 operation")).toBeInTheDocument();

    fireEvent.click(within(tracker).getByRole("button", { name: /run tracker sync/i }));
    await waitFor(() => expect(tracker).toHaveTextContent("Tracker sync unavailable · tracker sync down"));
    expect(within(tracker).queryByText("tracker_sync_1 · linear · planned · 1 operation")).not.toBeInTheDocument();

    fireEvent.click(within(tracker).getByRole("button", { name: /run tracker sync/i }));
    expect(await within(tracker).findByText("tracker_sync_2 · linear · planned · 2 operations")).toBeInTheDocument();
    expect(tracker).not.toHaveTextContent("Tracker sync unavailable · tracker sync down");
  });

  it("clears stale tracker sync runs when the selected provider cannot run sync", async () => {
    trackerApi.runNativeTrackerSync.mockResolvedValueOnce({
      id: "tracker_sync_1",
      project_id: "proj_local",
      provider: "linear",
      dry_run: true,
      status: "planned",
      operation_count: 1,
      degraded_reason: null,
      operations: [{ binding_id: "tracker_binding_1", local_kind: "task", local_id: "task_1", external_id: "LIN-42", operation: "issueUpdate", payload: {} }],
      created_at: "2026-05-03T01:00:00Z",
    });

    render(<App />);

    const tracker = await screen.findByRole("region", { name: /external tracker boundary/i });
    fireEvent.click(within(tracker).getByRole("button", { name: /run tracker sync/i }));
    expect(await within(tracker).findByText("tracker_sync_1 · linear · planned · 1 operation")).toBeInTheDocument();

    fireEvent.change(within(tracker).getByLabelText("Tracker provider"), { target: { value: "custom" } });
    fireEvent.click(within(tracker).getByRole("button", { name: /run tracker sync/i }));

    expect(tracker).toHaveTextContent("Tracker sync runs support linear github and plane providers");
    expect(within(tracker).queryByText("tracker_sync_1 · linear · planned · 1 operation")).not.toBeInTheDocument();
    expect(trackerApi.runNativeTrackerSync).toHaveBeenCalledTimes(1);
  });

  it("ingests token usage through adapter payloads", async () => {
    budgetApi.ingestNativeTokenUsageAdapter.mockResolvedValueOnce({
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

    render(<App />);

    const ingestion = await screen.findByRole("region", { name: /token usage ingestion adapters/i });
    fireEvent.change(screen.getByLabelText("Token usage adapter"), { target: { value: "openai.responses" } });
    fireEvent.change(screen.getByLabelText("Token usage adapter agent"), { target: { value: "agent_codex" } });
    fireEvent.change(screen.getByLabelText("Token usage adapter payload"), {
      target: {
        value: JSON.stringify({
          model: "gpt-5.4",
          usage: { input_tokens: 1200, output_tokens: 800 },
          cost_usd: 8.5,
        }),
      },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ingest token usage adapter" }));

    await waitFor(() => expect(budgetApi.ingestNativeTokenUsageAdapter).toHaveBeenCalledWith({
      projectId: "proj_local",
      agentProfileId: "agent_codex",
      adapter: "openai.responses",
      payload: {
        model: "gpt-5.4",
        usage: { input_tokens: 1200, output_tokens: 800 },
        cost_usd: 8.5,
      },
    }));
    expect(ingestion).toHaveTextContent("Ingested openai.responses · gpt-5.4");
    expect(ingestion).toHaveTextContent("2000 tokens · $8.50");
  });

  it("records manual token usage from the token usage panel", async () => {
    budgetApi.recordNativeTokenUsage.mockResolvedValueOnce({
      id: "usage_manual_1",
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
      source: "manual",
    });

    render(<App />);

    const ingestion = await screen.findByRole("region", { name: /token usage ingestion adapters/i });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage provider"), { target: { value: "openai" } });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage model"), { target: { value: "gpt-5.4" } });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage agent"), { target: { value: "agent_codex" } });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage input tokens"), { target: { value: "1200" } });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage output tokens"), { target: { value: "800" } });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage cost USD"), { target: { value: "8.5" } });
    fireEvent.change(within(ingestion).getByLabelText("Manual token usage source"), { target: { value: "manual" } });
    fireEvent.click(within(ingestion).getByRole("button", { name: /record manual token usage/i }));

    await waitFor(() => expect(budgetApi.recordNativeTokenUsage).toHaveBeenCalledWith({
      projectId: "proj_local",
      agentProfileId: "agent_codex",
      provider: "openai",
      model: "gpt-5.4",
      inputTokens: 1200,
      outputTokens: 800,
      costUsd: 8.5,
      source: "manual",
    }));
    expect(await within(ingestion).findByText("Recorded manual usage · gpt-5.4")).toBeInTheDocument();
    expect(ingestion).toHaveTextContent("2000 tokens · $8.50");
  });

  it("clears stale token usage adapter results after failures and recovers on retry", async () => {
    budgetApi.ingestNativeTokenUsageAdapter
      .mockResolvedValueOnce({
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
      })
      .mockRejectedValueOnce(new Error("native adapter down"))
      .mockResolvedValueOnce({
        id: "usage_2",
        project_id: "proj_local",
        session_id: null,
        task_id: null,
        run_id: null,
        agent_profile_id: "agent_codex",
        provider: "openai",
        model: "gpt-5.5",
        input_tokens: 300,
        output_tokens: 200,
        cost_usd: 2,
        source: "adapter:openai.responses",
      });

    render(<App />);

    const ingestion = await screen.findByRole("region", { name: /token usage ingestion adapters/i });
    const payload = within(ingestion).getByLabelText("Token usage adapter payload");
    fireEvent.change(within(ingestion).getByLabelText("Token usage adapter"), { target: { value: "openai.responses" } });
    fireEvent.change(within(ingestion).getByLabelText("Token usage adapter agent"), { target: { value: "agent_codex" } });
    fireEvent.change(payload, {
      target: {
        value: JSON.stringify({
          model: "gpt-5.4",
          usage: { input_tokens: 1200, output_tokens: 800 },
          cost_usd: 8.5,
        }),
      },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest token usage adapter" }));

    expect(await within(ingestion).findByText("Ingested openai.responses · gpt-5.4")).toBeInTheDocument();

    fireEvent.change(payload, {
      target: {
        value: JSON.stringify({
          model: "gpt-5.4",
          usage: { input_tokens: 100, output_tokens: 50 },
          cost_usd: 1,
        }),
      },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest token usage adapter" }));

    await waitFor(() => expect(ingestion).toHaveTextContent("Token usage adapter unavailable · native adapter down"));
    expect(ingestion).not.toHaveTextContent("Ingested openai.responses · gpt-5.4");
    expect(ingestion).not.toHaveTextContent("2000 tokens · $8.50");

    fireEvent.change(payload, {
      target: {
        value: JSON.stringify({
          model: "gpt-5.5",
          usage: { input_tokens: 300, output_tokens: 200 },
          cost_usd: 2,
        }),
      },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest token usage adapter" }));

    expect(await within(ingestion).findByText("Ingested openai.responses · gpt-5.5")).toBeInTheDocument();
    expect(ingestion).toHaveTextContent("500 tokens · $2.00");
    expect(ingestion).not.toHaveTextContent("Token usage adapter unavailable · native adapter down");
  });

  it("shows per-session token and cost totals in the session stack", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_session_usage",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pane_session_1",
          mode: "agent",
          title: "Codex run",
          cwd: "/repo",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
          token_usage: { input_tokens: 1200, output_tokens: 800, cost_usd: 8.5 },
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const stack = await screen.findByRole("region", { name: /session stack/i });
    expect(stack).toHaveTextContent("Codex run · agent · none · 2000 tokens · $8.50");
  });

  it("shows project session metadata in the session stack", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_session_stack_metadata",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_auth",
          pane_id: "pane_session_1",
          mode: "agent",
          title: "Codex AUTH-104",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          agent_profile_id: "agent_codex",
          task_id: "task_104",
          ports: [3000, 9229],
          state: "running",
          attention_state: "unread",
          token_budget_state: "ok",
          updated_at: "2026-05-02T01:02:00Z",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const stack = await screen.findByRole("region", { name: /session stack/i });
    await waitFor(() =>
      expect(stack).toHaveTextContent(
        "/repo/auth-service · branch fix/auth · task task_104 · agent agent_codex · ports 3000, 9229 · heartbeat 2026-05-02T01:02:00Z",
      ),
    );
  });

  it("surfaces knowledge lint counts and recent pages from the state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_knowledge",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 2, gap_count: 1, recent_pages: ["auth-flow"] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("Knowledge 2 stale · 1 gaps · auth-flow")).toBeInTheDocument();
  });

  it("shows a stale and gap warning MVP for degraded knowledge context", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_knowledge_warning",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 3, gap_count: 2, recent_pages: ["auth-flow"] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const warning = await screen.findByRole("region", { name: /knowledge freshness warnings/i });
    expect(warning).toHaveTextContent("Knowledge context needs attention");
    expect(warning).toHaveTextContent("3 stale sources");
    expect(warning).toHaveTextContent("2 coverage gaps");
    expect(warning).toHaveTextContent("Recent page auth-flow");
    expect(warning).toHaveTextContent("Re-index stale sources or update the context pack before dispatch");
  });

  it("renders the knowledge source index and can add a local source", async () => {
    knowledgeApi.listNativeKnowledgeSources.mockResolvedValueOnce([
      {
        id: "ks_1",
        project_id: "proj_local",
        kind: "file",
        path_or_ref: "docs/auth.md",
        fingerprint: "sha256:auth",
        status: "stale",
      },
    ]);
    knowledgeApi.upsertNativeKnowledgeSource.mockResolvedValueOnce({
      id: "ks_2",
      project_id: "proj_local",
      kind: "file",
      path_or_ref: "WORKFLOW.md",
      fingerprint: "local:WORKFLOW.md",
      status: "current",
    });

    render(<App />);

    const index = await screen.findByRole("region", { name: /knowledge source index/i });
    expect(index).toHaveTextContent("docs/auth.md");
    expect(index).toHaveTextContent("file");
    expect(index).toHaveTextContent("stale");

    fireEvent.change(screen.getByLabelText("Knowledge source path"), { target: { value: "WORKFLOW.md" } });
    fireEvent.click(screen.getByRole("button", { name: "Index knowledge source" }));

    await waitFor(() => expect(knowledgeApi.upsertNativeKnowledgeSource).toHaveBeenCalledWith(expect.objectContaining({
      projectId: "proj_local",
      kind: "file",
      pathOrRef: "WORKFLOW.md",
      status: "current",
    })));
    expect(await screen.findByText("Indexed WORKFLOW.md")).toBeInTheDocument();
    expect(index).toHaveTextContent("WORKFLOW.md");
  });

  it("clears stale knowledge source index status after validation failures and recovers on retry", async () => {
    knowledgeApi.upsertNativeKnowledgeSource
      .mockResolvedValueOnce({
        id: "ks_1",
        project_id: "proj_local",
        kind: "file",
        path_or_ref: "WORKFLOW.md",
        fingerprint: "local:WORKFLOW.md",
        status: "current",
      })
      .mockResolvedValueOnce({
        id: "ks_2",
        project_id: "proj_local",
        kind: "file",
        path_or_ref: "ARCHITECTURE.md",
        fingerprint: "local:ARCHITECTURE.md",
        status: "current",
      });

    render(<App />);

    const index = await screen.findByRole("region", { name: /knowledge source index/i });
    fireEvent.change(within(index).getByLabelText("Knowledge source path"), { target: { value: "WORKFLOW.md" } });
    fireEvent.click(within(index).getByRole("button", { name: "Index knowledge source" }));
    expect(await within(index).findByText("Indexed WORKFLOW.md")).toBeInTheDocument();

    fireEvent.click(within(index).getByRole("button", { name: "Index knowledge source" }));
    expect(within(index).getByText("Knowledge source path is required")).toBeInTheDocument();
    expect(within(index).queryByText("Indexed WORKFLOW.md")).not.toBeInTheDocument();

    fireEvent.change(within(index).getByLabelText("Knowledge source path"), { target: { value: "ARCHITECTURE.md" } });
    fireEvent.click(within(index).getByRole("button", { name: "Index knowledge source" }));
    expect(await within(index).findByText("Indexed ARCHITECTURE.md")).toBeInTheDocument();
    expect(index).not.toHaveTextContent("Knowledge source path is required");
  });

  it("surfaces knowledge source index degraded state", async () => {
    knowledgeApi.listNativeKnowledgeSources.mockRejectedValueOnce(new Error("native knowledge API unavailable"));

    render(<App />);

    expect(await screen.findByText("Knowledge source index unavailable · native knowledge API unavailable")).toBeInTheDocument();
  });

  it("runs OpenKB-style knowledge compile watch and lint automation", async () => {
    knowledgeApi.listNativeKnowledgeSources.mockResolvedValueOnce([
      {
        id: "ks_stale",
        project_id: "proj_local",
        kind: "file",
        path_or_ref: "docs/stale.md",
        fingerprint: "sha256:stale",
        status: "stale",
      },
    ]);
    knowledgeApi.searchNativeKnowledgePages.mockResolvedValueOnce([
      {
        id: "kp_gap",
        project_id: "proj_local",
        slug: "gap-page",
        title: "Gap Page",
        artifact_path: "artifacts/knowledge/gap-page.md",
        source_ids: [],
        freshness_state: "current",
        body_md: "# Gap Page",
      },
    ]);
    knowledgeApi.runNativeKnowledgeAutomation.mockResolvedValueOnce({
      project_id: "proj_local",
      status: "compiled",
      watch_enabled: true,
      source_count: 1,
      page_count: 1,
      stale_count: 1,
      gap_count: 1,
      lint_report_id: "klr_1",
    });

    render(<App />);

    const automation = await screen.findByRole("region", { name: /knowledge automation/i });
    fireEvent.click(screen.getByRole("checkbox", { name: "Watch knowledge changes" }));
    fireEvent.click(screen.getByRole("button", { name: "Compile knowledge vault" }));

    await waitFor(() => expect(knowledgeApi.runNativeKnowledgeAutomation).toHaveBeenCalledWith({
      projectId: "proj_local",
      watch: true,
    }));
    expect(automation).toHaveTextContent("compiled · watch on");
    expect(automation).toHaveTextContent("1 sources · 1 pages");
    expect(automation).toHaveTextContent("1 stale · 1 gaps");
    expect(automation).toHaveTextContent("lint klr_1");
  });

  it("records manual knowledge lint reports from the automation panel", async () => {
    knowledgeApi.recordNativeKnowledgeLintReport.mockResolvedValueOnce({
      id: "klr_manual",
      project_id: "proj_local",
      artifact_path: "artifacts/knowledge/lint/klr_manual.md",
      stale_count: 2,
      gap_count: 1,
      contradiction_count: 0,
      body_md: "Gap: rollback evidence is missing",
    });

    render(<App />);

    const automation = await screen.findByRole("region", { name: /knowledge automation/i });
    fireEvent.change(within(automation).getByLabelText("Knowledge lint stale count"), { target: { value: "2" } });
    fireEvent.change(within(automation).getByLabelText("Knowledge lint gap count"), { target: { value: "1" } });
    fireEvent.change(within(automation).getByLabelText("Knowledge lint contradiction count"), { target: { value: "0" } });
    fireEvent.change(within(automation).getByLabelText("Knowledge lint body"), {
      target: { value: "Gap: rollback evidence is missing" },
    });
    fireEvent.click(within(automation).getByRole("button", { name: "Record knowledge lint report" }));

    await waitFor(() => expect(knowledgeApi.recordNativeKnowledgeLintReport).toHaveBeenCalledWith({
      projectId: "proj_local",
      staleCount: 2,
      gapCount: 1,
      contradictionCount: 0,
      bodyMd: "Gap: rollback evidence is missing",
    }));
    expect(await within(automation).findByText("Recorded lint klr_manual · 2 stale · 1 gaps · 0 contradictions")).toBeInTheDocument();
    expect(within(automation).getByText("artifacts/knowledge/lint/klr_manual.md")).toBeInTheDocument();
  });

  it("clears stale knowledge automation results after failures and recovers on retry", async () => {
    knowledgeApi.runNativeKnowledgeAutomation
      .mockResolvedValueOnce({
        project_id: "proj_local",
        status: "compiled",
        watch_enabled: false,
        source_count: 2,
        page_count: 3,
        stale_count: 0,
        gap_count: 0,
        lint_report_id: "klr_success",
      })
      .mockRejectedValueOnce(new Error("lint worker crashed"))
      .mockResolvedValueOnce({
        project_id: "proj_local",
        status: "compiled",
        watch_enabled: false,
        source_count: 1,
        page_count: 1,
        stale_count: 1,
        gap_count: 0,
        lint_report_id: "klr_retry",
      });

    render(<App />);

    const automation = await screen.findByRole("region", { name: /knowledge automation/i });
    fireEvent.click(screen.getByRole("button", { name: "Compile knowledge vault" }));

    expect(await screen.findByText("compiled · watch off")).toBeInTheDocument();
    expect(automation).toHaveTextContent("lint klr_success");

    fireEvent.click(screen.getByRole("button", { name: "Compile knowledge vault" }));

    expect(await screen.findByText("Knowledge automation unavailable · lint worker crashed")).toBeInTheDocument();
    expect(automation).not.toHaveTextContent("lint klr_success");

    fireEvent.click(screen.getByRole("button", { name: "Compile knowledge vault" }));

    expect(await screen.findByText("lint klr_retry")).toBeInTheDocument();
    expect(automation).not.toHaveTextContent("Knowledge automation unavailable · lint worker crashed");
  });

  it("ingests long document and multimodal artifacts into knowledge pages", async () => {
    knowledgeApi.ingestNativeKnowledgeArtifact.mockResolvedValueOnce({
      project_id: "proj_local",
      source_id: "ks_1",
      page_id: "kp_1",
      slug: "release-runbook",
      modality: "pdf",
      chunk_count: 3,
      fingerprint: "local:pdf:2600:docs/release-runbook.pdf",
    });

    render(<App />);

    const ingestion = await screen.findByRole("region", { name: /long document and multimodal ingestion/i });
    fireEvent.change(screen.getByLabelText("Ingestion artifact path"), { target: { value: "docs/release-runbook.pdf" } });
    fireEvent.change(screen.getByLabelText("Ingestion artifact kind"), { target: { value: "pdf" } });
    fireEvent.change(screen.getByLabelText("Ingestion artifact title"), { target: { value: "Release Runbook" } });
    fireEvent.change(screen.getByLabelText("Ingestion artifact body"), {
      target: { value: "Release runbook ".repeat(160) },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ingest knowledge artifact" }));

    await waitFor(() => expect(knowledgeApi.ingestNativeKnowledgeArtifact).toHaveBeenCalledWith({
      projectId: "proj_local",
      kind: "pdf",
      pathOrRef: "docs/release-runbook.pdf",
      title: "Release Runbook",
      bodyMd: "Release runbook ".repeat(160),
      maxChunkChars: 1200,
    }));
    expect(ingestion).toHaveTextContent("Ingested docs/release-runbook.pdf");
    expect(ingestion).toHaveTextContent("pdf · 3 chunks");
    expect(ingestion).toHaveTextContent("page kp_1 · source ks_1");
  });

  it("clears stale knowledge ingestion results after validation failures and recovers on retry", async () => {
    knowledgeApi.ingestNativeKnowledgeArtifact
      .mockResolvedValueOnce({
        project_id: "proj_local",
        source_id: "ks_1",
        page_id: "kp_1",
        slug: "release-runbook",
        modality: "pdf",
        chunk_count: 3,
        fingerprint: "local:pdf:2600:docs/release-runbook.pdf",
      })
      .mockResolvedValueOnce({
        project_id: "proj_local",
        source_id: "ks_2",
        page_id: "kp_2",
        slug: "incident-replay",
        modality: "markdown",
        chunk_count: 2,
        fingerprint: "local:markdown:1200:docs/incident-replay.md",
      });

    render(<App />);

    const ingestion = await screen.findByRole("region", { name: /long document and multimodal ingestion/i });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact path"), { target: { value: "docs/release-runbook.pdf" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact kind"), { target: { value: "pdf" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact title"), { target: { value: "Release Runbook" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact body"), {
      target: { value: "Release runbook ".repeat(160) },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest knowledge artifact" }));
    expect(await within(ingestion).findByText("Ingested docs/release-runbook.pdf")).toBeInTheDocument();

    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact path"), { target: { value: "docs/incident-replay.md" } });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest knowledge artifact" }));
    expect(within(ingestion).getByText("Ingestion artifact path and body are required")).toBeInTheDocument();
    expect(within(ingestion).queryByText("Ingested docs/release-runbook.pdf")).not.toBeInTheDocument();

    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact kind"), { target: { value: "markdown" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact title"), { target: { value: "Incident Replay" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact body"), {
      target: { value: "Incident replay ".repeat(90) },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest knowledge artifact" }));
    expect(await within(ingestion).findByText("Ingested docs/incident-replay.md")).toBeInTheDocument();
    expect(ingestion).not.toHaveTextContent("Ingestion artifact path and body are required");
  });

  it("clears stale knowledge ingestion results after API failures and recovers on retry", async () => {
    knowledgeApi.ingestNativeKnowledgeArtifact
      .mockResolvedValueOnce({
        project_id: "proj_local",
        source_id: "ks_1",
        page_id: "kp_1",
        slug: "release-runbook",
        modality: "pdf",
        chunk_count: 3,
        fingerprint: "local:pdf:2600:docs/release-runbook.pdf",
      })
      .mockRejectedValueOnce(new Error("ingestion worker down"))
      .mockResolvedValueOnce({
        project_id: "proj_local",
        source_id: "ks_2",
        page_id: "kp_2",
        slug: "incident-replay",
        modality: "markdown",
        chunk_count: 2,
        fingerprint: "local:markdown:1200:docs/incident-replay.md",
      });

    render(<App />);

    const ingestion = await screen.findByRole("region", { name: /long document and multimodal ingestion/i });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact path"), { target: { value: "docs/release-runbook.pdf" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact kind"), { target: { value: "pdf" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact title"), { target: { value: "Release Runbook" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact body"), {
      target: { value: "Release runbook ".repeat(160) },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest knowledge artifact" }));
    expect(await within(ingestion).findByText("Ingested docs/release-runbook.pdf")).toBeInTheDocument();

    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact path"), { target: { value: "docs/incident-replay.md" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact kind"), { target: { value: "markdown" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact title"), { target: { value: "Incident Replay" } });
    fireEvent.change(within(ingestion).getByLabelText("Ingestion artifact body"), {
      target: { value: "Incident replay ".repeat(90) },
    });
    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest knowledge artifact" }));
    await waitFor(() => expect(ingestion).toHaveTextContent("Knowledge ingestion unavailable · ingestion worker down"));
    expect(within(ingestion).queryByText("Ingested docs/release-runbook.pdf")).not.toBeInTheDocument();

    fireEvent.click(within(ingestion).getByRole("button", { name: "Ingest knowledge artifact" }));
    expect(await within(ingestion).findByText("Ingested docs/incident-replay.md")).toBeInTheDocument();
    expect(ingestion).not.toHaveTextContent("Knowledge ingestion unavailable · ingestion worker down");
  });

  it("renders and saves markdown knowledge pages", async () => {
    knowledgeApi.searchNativeKnowledgePages.mockResolvedValueOnce([
      {
        id: "kp_1",
        project_id: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        artifact_path: "artifacts/knowledge/auth-flow.md",
        source_ids: ["ks_1"],
        freshness_state: "current",
        body_md: "# Auth Flow\n\n- Token rotation",
      },
    ]);
    knowledgeApi.saveNativeKnowledgePage.mockResolvedValueOnce({
      id: "kp_2",
      project_id: "proj_local",
      slug: "deploy-notes",
      title: "Deploy Notes",
      artifact_path: "artifacts/knowledge/deploy-notes.md",
      source_ids: [],
      freshness_state: "current",
      body_md: "# Deploy Notes\n\n- Rollout checklist",
    });

    render(<App />);

    const pages = await screen.findByRole("region", { name: /markdown knowledge pages/i });
    expect(pages).toHaveTextContent("Auth Flow");
    expect(pages).toHaveTextContent("Token rotation");

    fireEvent.change(screen.getByLabelText("Knowledge page slug"), { target: { value: "deploy-notes" } });
    fireEvent.change(screen.getByLabelText("Knowledge page title"), { target: { value: "Deploy Notes" } });
    fireEvent.change(screen.getByLabelText("Knowledge page markdown"), { target: { value: "# Deploy Notes\n\n- Rollout checklist" } });
    fireEvent.click(screen.getByRole("button", { name: "Save markdown knowledge page" }));

    await waitFor(() => expect(knowledgeApi.saveNativeKnowledgePage).toHaveBeenCalledWith({
      projectId: "proj_local",
      slug: "deploy-notes",
      title: "Deploy Notes",
      bodyMd: "# Deploy Notes\n\n- Rollout checklist",
      sourceIds: [],
      freshnessState: "current",
    }));
    expect(await screen.findByText("Saved deploy-notes")).toBeInTheDocument();
    expect(pages).toHaveTextContent("Deploy Notes");
    expect(pages).toHaveTextContent("Rollout checklist");
  });

  it("surfaces markdown knowledge page save errors", async () => {
    knowledgeApi.saveNativeKnowledgePage.mockRejectedValueOnce(new Error("slug already exists"));

    render(<App />);

    fireEvent.change(await screen.findByLabelText("Knowledge page slug"), { target: { value: "auth-flow" } });
    fireEvent.change(screen.getByLabelText("Knowledge page title"), { target: { value: "Auth Flow" } });
    fireEvent.change(screen.getByLabelText("Knowledge page markdown"), { target: { value: "# Auth Flow" } });
    fireEvent.click(screen.getByRole("button", { name: "Save markdown knowledge page" }));

    expect(await screen.findByText("Knowledge page save failed · slug already exists")).toBeInTheDocument();
  });

  it("clears stale markdown knowledge page status after validation failures and recovers on retry", async () => {
    knowledgeApi.saveNativeKnowledgePage
      .mockResolvedValueOnce({
        id: "kp_1",
        project_id: "proj_local",
        slug: "deploy-notes",
        title: "Deploy Notes",
        artifact_path: "artifacts/knowledge/deploy-notes.md",
        source_ids: [],
        freshness_state: "current",
        body_md: "# Deploy Notes",
      })
      .mockResolvedValueOnce({
        id: "kp_2",
        project_id: "proj_local",
        slug: "incident-notes",
        title: "Incident Notes",
        artifact_path: "artifacts/knowledge/incident-notes.md",
        source_ids: [],
        freshness_state: "current",
        body_md: "# Incident Notes",
      });

    render(<App />);

    const pages = await screen.findByRole("region", { name: /markdown knowledge pages/i });
    fireEvent.change(within(pages).getByLabelText("Knowledge page slug"), { target: { value: "deploy-notes" } });
    fireEvent.change(within(pages).getByLabelText("Knowledge page title"), { target: { value: "Deploy Notes" } });
    fireEvent.change(within(pages).getByLabelText("Knowledge page markdown"), { target: { value: "# Deploy Notes" } });
    fireEvent.click(within(pages).getByRole("button", { name: "Save markdown knowledge page" }));
    expect(await within(pages).findByText("Saved deploy-notes")).toBeInTheDocument();

    fireEvent.change(within(pages).getByLabelText("Knowledge page slug"), { target: { value: "incident-notes" } });
    fireEvent.click(within(pages).getByRole("button", { name: "Save markdown knowledge page" }));
    expect(within(pages).getByText("Knowledge page slug title and markdown are required")).toBeInTheDocument();
    expect(within(pages).queryByText("Saved deploy-notes")).not.toBeInTheDocument();

    fireEvent.change(within(pages).getByLabelText("Knowledge page title"), { target: { value: "Incident Notes" } });
    fireEvent.change(within(pages).getByLabelText("Knowledge page markdown"), { target: { value: "# Incident Notes" } });
    fireEvent.click(within(pages).getByRole("button", { name: "Save markdown knowledge page" }));
    expect(await within(pages).findByText("Saved incident-notes")).toBeInTheDocument();
    expect(pages).not.toHaveTextContent("Knowledge page slug title and markdown are required");
  });

  it("renders knowledge concepts with cross-links", async () => {
    knowledgeApi.listNativeKnowledgeConcepts.mockResolvedValueOnce([
      {
        slug: "auth-flow",
        title: "Auth Flow",
        page_id: "kp_1",
        outbound_slugs: ["jwt-rotation", "error-handling"],
        inbound_page_ids: ["kp_2"],
      },
      {
        slug: "jwt-rotation",
        title: "JWT Rotation",
        page_id: null,
        outbound_slugs: [],
        inbound_page_ids: ["kp_1"],
      },
    ]);

    render(<App />);

    const concepts = await screen.findByRole("region", { name: /knowledge concepts/i });
    expect(concepts).toHaveTextContent("Auth Flow");
    expect(concepts).toHaveTextContent("auth-flow · kp_1");
    expect(concepts).toHaveTextContent("Links jwt-rotation, error-handling");
    expect(concepts).toHaveTextContent("Backlinks kp_2");
    expect(concepts).toHaveTextContent("JWT Rotation");
    expect(concepts).toHaveTextContent("jwt-rotation · unresolved page");
  });

  it("exports Knowledge Vault pages as an Obsidian markdown vault", async () => {
    knowledgeApi.exportNativeKnowledgeObsidianMarkdown.mockResolvedValueOnce({
      project_id: "proj_local",
      status: "exported",
      export_root: "artifacts/knowledge/obsidian/proj_local",
      file_count: 3,
      files: ["Auth Flow.md", "JWT Rotation.md", "Knowledge Index.md"],
    });

    render(<App />);

    const concepts = await screen.findByRole("region", { name: /knowledge concepts/i });
    fireEvent.click(within(concepts).getByRole("button", { name: "Export Obsidian markdown" }));

    await waitFor(() => expect(knowledgeApi.exportNativeKnowledgeObsidianMarkdown).toHaveBeenCalledWith("proj_local"));
    expect(concepts).toHaveTextContent("exported · 3 files");
    expect(concepts).toHaveTextContent("artifacts/knowledge/obsidian/proj_local");
    expect(concepts).toHaveTextContent("Knowledge Index.md");
  });

  it("answers Knowledge Vault questions with local citations", async () => {
    knowledgeApi.listNativeContextPacks.mockResolvedValueOnce([
      {
        id: "ctx_auth",
        project_id: "proj_local",
        name: "auth-default",
        description: "Auth docs",
        sources_json: { sources: [{ type: "knowledge_page", id: "kp_1" }] },
      },
    ]);
    knowledgeApi.answerNativeKnowledgeQuestion.mockResolvedValueOnce({
      project_id: "proj_local",
      question: "How should rollback handle token rotation?",
      answer_md: "## Local knowledge answer draft\n\n- [[Auth Flow]] (kp_1) - Keep both issuers.",
      cited_page_ids: ["kp_1"],
      context_pack_id: "ctx_auth",
      source_count: 1,
    });

    render(<App />);

    const chat = await screen.findByRole("region", { name: /knowledge chat/i });
    fireEvent.change(within(chat).getByLabelText("Knowledge question"), {
      target: { value: "How should rollback handle token rotation?" },
    });
    fireEvent.change(within(chat).getByLabelText("Knowledge chat context pack"), { target: { value: "ctx_auth" } });
    fireEvent.click(within(chat).getByRole("button", { name: "Ask Knowledge Vault" }));

    await waitFor(() => expect(knowledgeApi.answerNativeKnowledgeQuestion).toHaveBeenCalledWith({
      projectId: "proj_local",
      question: "How should rollback handle token rotation?",
      contextPackId: "ctx_auth",
    }));
    expect(chat).toHaveTextContent("1 local citations");
    expect(chat).toHaveTextContent("kp_1");
    expect(chat).toHaveTextContent("Keep both issuers.");
  });

  it("clears stale Knowledge Chat answers after validation failures and recovers on retry", async () => {
    knowledgeApi.answerNativeKnowledgeQuestion
      .mockResolvedValueOnce({
        project_id: "proj_local",
        question: "How should rollback handle token rotation?",
        answer_md: "Keep both issuers during rollback.",
        cited_page_ids: ["kp_1"],
        context_pack_id: null,
        source_count: 1,
      })
      .mockResolvedValueOnce({
        project_id: "proj_local",
        question: "Which evidence should incident replay cite?",
        answer_md: "Cite the latest recovery drill.",
        cited_page_ids: ["kp_2", "kp_3"],
        context_pack_id: null,
        source_count: 2,
      });

    render(<App />);

    const chat = await screen.findByRole("region", { name: /knowledge chat/i });
    fireEvent.change(within(chat).getByLabelText("Knowledge question"), {
      target: { value: "How should rollback handle token rotation?" },
    });
    fireEvent.click(within(chat).getByRole("button", { name: "Ask Knowledge Vault" }));
    expect(await within(chat).findByText("1 local citations")).toBeInTheDocument();
    expect(chat).toHaveTextContent("Keep both issuers during rollback.");

    fireEvent.change(within(chat).getByLabelText("Knowledge question"), { target: { value: "" } });
    fireEvent.click(within(chat).getByRole("button", { name: "Ask Knowledge Vault" }));
    expect(within(chat).getByText("Knowledge question is required")).toBeInTheDocument();
    expect(chat).not.toHaveTextContent("Keep both issuers during rollback.");

    fireEvent.change(within(chat).getByLabelText("Knowledge question"), {
      target: { value: "Which evidence should incident replay cite?" },
    });
    fireEvent.click(within(chat).getByRole("button", { name: "Ask Knowledge Vault" }));
    expect(await within(chat).findByText("2 local citations")).toBeInTheDocument();
    expect(chat).toHaveTextContent("Cite the latest recovery drill.");
    expect(chat).not.toHaveTextContent("Knowledge question is required");
  });

  it("saves and lists knowledge explorations from the Knowledge Vault", async () => {
    knowledgeApi.searchNativeKnowledgePages.mockResolvedValueOnce([
      {
        id: "kp_1",
        project_id: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        artifact_path: "artifacts/knowledge/auth-flow.md",
        source_ids: ["ks_1"],
        freshness_state: "current",
        body_md: "# Auth Flow",
      },
    ]);
    knowledgeApi.listNativeContextPacks.mockResolvedValueOnce([
      {
        id: "ctx_auth",
        project_id: "proj_local",
        name: "auth-default",
        description: "Auth docs",
        sources_json: {
          sources: [{ type: "knowledge_page", id: "kp_1" }],
          budget: { max_tokens_hint: 24000 },
        },
      },
    ]);
    knowledgeApi.listNativeKnowledgeExplorations.mockResolvedValueOnce([
      {
        id: "kexp_1",
        project_id: "proj_local",
        title: "Token rollout investigation",
        question: "How should rollback handle token rotation?",
        answer_md: "Keep both issuers during rollback.",
        artifact_path: "artifacts/knowledge/explorations/kexp_1.md",
        page_ids: ["kp_1"],
        context_pack_id: "ctx_auth",
      },
    ]);
    knowledgeApi.saveNativeKnowledgeExploration.mockResolvedValueOnce({
      id: "kexp_2",
      project_id: "proj_local",
      title: "Deploy rollback answer",
      question: "Which context should release checks use?",
      answer_md: "Use ctx_auth with auth-flow as the cited page.",
      artifact_path: "artifacts/knowledge/explorations/kexp_2.md",
      page_ids: ["kp_1"],
      context_pack_id: "ctx_auth",
    });

    render(<App />);

    const explorations = await screen.findByRole("region", { name: /saved knowledge explorations/i });
    expect(explorations).toHaveTextContent("Token rollout investigation");
    expect(explorations).toHaveTextContent("How should rollback handle token rotation?");
    expect(explorations).toHaveTextContent("ctx_auth");

    fireEvent.change(screen.getByLabelText("Knowledge exploration title"), { target: { value: "Deploy rollback answer" } });
    fireEvent.change(screen.getByLabelText("Knowledge exploration question"), {
      target: { value: "Which context should release checks use?" },
    });
    fireEvent.change(screen.getByLabelText("Knowledge exploration answer"), {
      target: { value: "Use ctx_auth with auth-flow as the cited page." },
    });
    fireEvent.change(screen.getByLabelText("Knowledge exploration page"), { target: { value: "kp_1" } });
    fireEvent.change(screen.getByLabelText("Knowledge exploration context pack"), { target: { value: "ctx_auth" } });
    fireEvent.click(screen.getByRole("button", { name: "Save knowledge exploration" }));

    await waitFor(() => expect(knowledgeApi.saveNativeKnowledgeExploration).toHaveBeenCalledWith({
      projectId: "proj_local",
      title: "Deploy rollback answer",
      question: "Which context should release checks use?",
      answerMd: "Use ctx_auth with auth-flow as the cited page.",
      pageIds: ["kp_1"],
      contextPackId: "ctx_auth",
    }));
    expect(await screen.findByText("Saved exploration kexp_2")).toBeInTheDocument();
    expect(explorations).toHaveTextContent("Deploy rollback answer");
    expect(explorations).toHaveTextContent("Use ctx_auth with auth-flow as the cited page.");
  });

  it("clears stale knowledge exploration save status after validation failures and recovers on retry", async () => {
    knowledgeApi.saveNativeKnowledgeExploration
      .mockResolvedValueOnce({
        id: "kexp_1",
        project_id: "proj_local",
        title: "Deploy rollback answer",
        question: "Which context should release checks use?",
        answer_md: "Use ctx_auth with auth-flow as the cited page.",
        artifact_path: "artifacts/knowledge/explorations/kexp_1.md",
        page_ids: [],
        context_pack_id: null,
      })
      .mockResolvedValueOnce({
        id: "kexp_2",
        project_id: "proj_local",
        title: "Incident replay answer",
        question: "Which evidence should incident replay cite?",
        answer_md: "Cite the latest recovery drill.",
        artifact_path: "artifacts/knowledge/explorations/kexp_2.md",
        page_ids: [],
        context_pack_id: null,
      });

    render(<App />);

    const explorations = await screen.findByRole("region", { name: /saved knowledge explorations/i });
    fireEvent.change(within(explorations).getByLabelText("Knowledge exploration title"), {
      target: { value: "Deploy rollback answer" },
    });
    fireEvent.change(within(explorations).getByLabelText("Knowledge exploration question"), {
      target: { value: "Which context should release checks use?" },
    });
    fireEvent.change(within(explorations).getByLabelText("Knowledge exploration answer"), {
      target: { value: "Use ctx_auth with auth-flow as the cited page." },
    });
    fireEvent.click(within(explorations).getByRole("button", { name: "Save knowledge exploration" }));
    expect(await within(explorations).findByText("Saved exploration kexp_1")).toBeInTheDocument();

    fireEvent.change(within(explorations).getByLabelText("Knowledge exploration title"), {
      target: { value: "Incident replay answer" },
    });
    fireEvent.click(within(explorations).getByRole("button", { name: "Save knowledge exploration" }));
    expect(within(explorations).getByText("Knowledge exploration title question and answer are required")).toBeInTheDocument();
    expect(within(explorations).queryByText("Saved exploration kexp_1")).not.toBeInTheDocument();

    fireEvent.change(within(explorations).getByLabelText("Knowledge exploration question"), {
      target: { value: "Which evidence should incident replay cite?" },
    });
    fireEvent.change(within(explorations).getByLabelText("Knowledge exploration answer"), {
      target: { value: "Cite the latest recovery drill." },
    });
    fireEvent.click(within(explorations).getByRole("button", { name: "Save knowledge exploration" }));
    expect(await within(explorations).findByText("Saved exploration kexp_2")).toBeInTheDocument();
    expect(explorations).not.toHaveTextContent("Knowledge exploration title question and answer are required");
  });

  it("renders and builds context packs from knowledge pages", async () => {
    knowledgeApi.searchNativeKnowledgePages.mockResolvedValueOnce([
      {
        id: "kp_1",
        project_id: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        artifact_path: "artifacts/knowledge/auth-flow.md",
        source_ids: ["ks_1"],
        freshness_state: "current",
        body_md: "# Auth Flow",
      },
    ]);
    knowledgeApi.listNativeContextPacks.mockResolvedValueOnce([
      {
        id: "ctx_auth",
        project_id: "proj_local",
        name: "auth-default",
        description: "Auth docs",
        sources_json: {
          sources: [{ type: "knowledge_page", id: "kp_1" }],
          budget: { max_tokens_hint: 24000 },
        },
      },
    ]);
    knowledgeApi.upsertNativeContextPack.mockResolvedValueOnce({
      id: "ctx_run",
      project_id: "proj_local",
      name: "run-default",
      description: "Run docs",
      sources_json: {
        sources: [{ type: "knowledge_page", id: "kp_1" }],
        budget: { max_tokens_hint: 18000 },
      },
    });

    render(<App />);

    const builder = await screen.findByRole("region", { name: /context pack builder/i });
    expect(builder).toHaveTextContent("auth-default");
    expect(builder).toHaveTextContent("kp_1");
    expect(builder).toHaveTextContent("24000 tokens");

    fireEvent.change(screen.getByLabelText("Context pack name"), { target: { value: "run-default" } });
    fireEvent.change(screen.getByLabelText("Context pack description"), { target: { value: "Run docs" } });
    fireEvent.change(screen.getByLabelText("Context pack source"), { target: { value: "kp_1" } });
    fireEvent.change(screen.getByLabelText("Context pack max tokens"), { target: { value: "18000" } });
    fireEvent.click(screen.getByRole("button", { name: "Save context pack" }));

    await waitFor(() => expect(knowledgeApi.upsertNativeContextPack).toHaveBeenCalledWith({
      projectId: "proj_local",
      name: "run-default",
      description: "Run docs",
      sourcesJson: [{ type: "knowledge_page", id: "kp_1" }],
      maxTokensHint: 18000,
    }));
    expect(await screen.findByText("Saved context pack ctx_run")).toBeInTheDocument();
    expect(builder).toHaveTextContent("run-default");
  });

  it("clears stale context pack save status after validation failures and recovers on retry", async () => {
    knowledgeApi.searchNativeKnowledgePages.mockResolvedValueOnce([
      {
        id: "kp_1",
        project_id: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        artifact_path: "artifacts/knowledge/auth-flow.md",
        source_ids: ["ks_1"],
        freshness_state: "current",
        body_md: "# Auth Flow",
      },
    ]);
    knowledgeApi.upsertNativeContextPack
      .mockResolvedValueOnce({
        id: "ctx_run",
        project_id: "proj_local",
        name: "run-default",
        description: "Run docs",
        sources_json: {
          sources: [{ type: "knowledge_page", id: "kp_1" }],
          budget: { max_tokens_hint: 18000 },
        },
      })
      .mockResolvedValueOnce({
        id: "ctx_retry",
        project_id: "proj_local",
        name: "run-retry",
        description: "Retry docs",
        sources_json: {
          sources: [{ type: "knowledge_page", id: "kp_1" }],
          budget: { max_tokens_hint: 16000 },
        },
      });

    render(<App />);

    const builder = await screen.findByRole("region", { name: /context pack builder/i });
    fireEvent.change(within(builder).getByLabelText("Context pack name"), { target: { value: "run-default" } });
    fireEvent.change(within(builder).getByLabelText("Context pack description"), { target: { value: "Run docs" } });
    fireEvent.change(within(builder).getByLabelText("Context pack source"), { target: { value: "kp_1" } });
    fireEvent.change(within(builder).getByLabelText("Context pack max tokens"), { target: { value: "18000" } });
    fireEvent.click(within(builder).getByRole("button", { name: "Save context pack" }));
    expect(await within(builder).findByText("Saved context pack ctx_run")).toBeInTheDocument();

    fireEvent.change(within(builder).getByLabelText("Context pack name"), { target: { value: "run-retry" } });
    fireEvent.click(within(builder).getByRole("button", { name: "Save context pack" }));
    expect(within(builder).getByText("Context pack name and source are required")).toBeInTheDocument();
    expect(within(builder).queryByText("Saved context pack ctx_run")).not.toBeInTheDocument();

    fireEvent.change(within(builder).getByLabelText("Context pack description"), { target: { value: "Retry docs" } });
    fireEvent.change(within(builder).getByLabelText("Context pack source"), { target: { value: "kp_1" } });
    fireEvent.change(within(builder).getByLabelText("Context pack max tokens"), { target: { value: "16000" } });
    fireEvent.click(within(builder).getByRole("button", { name: "Save context pack" }));
    expect(await within(builder).findByText("Saved context pack ctx_retry")).toBeInTheDocument();
    expect(builder).not.toHaveTextContent("Context pack name and source are required");
  });

  it("surfaces context pack viewer degraded state", async () => {
    knowledgeApi.listNativeContextPacks.mockRejectedValueOnce(new Error("native context API unavailable"));

    render(<App />);

    expect(await screen.findByText("Context packs unavailable · native context API unavailable")).toBeInTheDocument();
  });

  it("surfaces and resolves pending policy approvals from the state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_policy",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_policy",
            project_id: "proj_local",
            lifecycle: "permission_requested",
            retry_count: 0,
            status_detail: "Permission requested: shell_command (rm -rf build/cache)",
          },
        ],
        counts_by_lifecycle: { permission_requested: 1 },
      },
      agents: [],
      reviews: [],
      attention: [
        {
          id: "policy_approval_1",
          label: "Policy approval required: shell_command",
          severity: "critical",
          detail: "rm -rf build/cache",
        },
      ],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    policyApi.decideNativePolicyApproval.mockResolvedValueOnce({
      id: "policy_approval_1",
      run_id: "run_1",
      state: "approved",
    });

    render(<App />);

    expect(await screen.findByText("Policy approval required: shell_command")).toBeInTheDocument();
    expect(screen.getByText("state detail Permission requested: shell_command (rm -rf build/cache)")).toBeInTheDocument();
    expect(screen.getByText("rm -rf build/cache")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve policy_approval_1" }));

    await waitFor(() =>
      expect(policyApi.decideNativePolicyApproval).toHaveBeenCalledWith({
        approvalId: "policy_approval_1",
        decision: "approved",
        decisionBy: "human",
        decisionNote: "Approved from Policy Approvals.",
      }),
    );
    expect(screen.queryByText("rm -rf build/cache")).not.toBeInTheDocument();
  });

  it("loads native policy approvals from the right rail", async () => {
    policyApi.listNativePolicyApprovals.mockResolvedValueOnce([
      {
        id: "policy_approval_loaded",
        project_id: "proj_local",
        task_id: "task_policy",
        run_id: "run_1",
        action_kind: "shell_command",
        command: "rm -rf build/cache",
        risk_level: "high",
        state: "pending",
        requested_by: "agent_codex",
        decision_by: null,
        decision_note: null,
        created_at: "2026-05-05T01:00:00Z",
        decided_at: null,
      },
    ]);

    render(<App />);

    const approvals = await screen.findByRole("region", { name: /policy approvals/i });
    fireEvent.click(within(approvals).getByRole("button", { name: /load policy approvals/i }));

    await waitFor(() => expect(policyApi.listNativePolicyApprovals).toHaveBeenCalledWith("proj_local", "pending"));
    expect(await within(approvals).findByText("Policy approval required: shell_command · critical")).toBeInTheDocument();
    expect(within(approvals).getByText("rm -rf build/cache · task task_policy · run run_1")).toBeInTheDocument();
  });

  it("creates native policy approvals from the policy action controls", async () => {
    policyApi.createNativePolicyApproval.mockResolvedValueOnce({
      id: "policy_approval_created",
      project_id: "proj_local",
      task_id: null,
      run_id: null,
      action_kind: "shell_command",
      command: "rm -rf build/cache",
      risk_level: "high",
      state: "pending",
      requested_by: "human",
      decision_by: null,
      decision_note: null,
      created_at: "2026-05-05T01:00:00Z",
      decided_at: null,
    });

    render(<App />);

    const policyModel = screen.getByText("Policy Pack Model").closest("section");
    expect(policyModel).not.toBeNull();
    fireEvent.change(within(policyModel as HTMLElement).getByLabelText("Policy action kind"), { target: { value: "shell_command" } });
    fireEvent.change(within(policyModel as HTMLElement).getByLabelText("Policy action command"), { target: { value: "rm -rf build/cache" } });
    fireEvent.click(within(policyModel as HTMLElement).getByRole("button", { name: "Create policy approval" }));

    await waitFor(() => expect(policyApi.createNativePolicyApproval).toHaveBeenCalledWith({
      projectId: "proj_local",
      actionKind: "shell_command",
      command: "rm -rf build/cache",
      riskLevel: "high",
      requestedBy: "human",
    }));

    const approvals = await screen.findByRole("region", { name: /policy approvals/i });
    expect(await within(approvals).findByText("Policy approval required: shell_command · critical")).toBeInTheDocument();
    expect(within(approvals).getByText("rm -rf build/cache")).toBeInTheDocument();
  });

  it("surfaces policy approval decision failures without clearing the pending approval", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_policy_error",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_policy",
            project_id: "proj_local",
            lifecycle: "permission_requested",
            retry_count: 0,
            status_detail: "Permission requested: shell_command (rm -rf build/cache)",
          },
        ],
        counts_by_lifecycle: { permission_requested: 1 },
      },
      agents: [],
      reviews: [],
      attention: [
        {
          id: "policy_approval_1",
          label: "Policy approval required: shell_command",
          severity: "critical",
          detail: "rm -rf build/cache",
        },
      ],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    policyApi.decideNativePolicyApproval.mockRejectedValueOnce(new Error("policy service offline"));

    render(<App />);

    expect(await screen.findByText("Policy approval required: shell_command")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Deny policy_approval_1" }));

    await waitFor(() =>
      expect(policyApi.decideNativePolicyApproval).toHaveBeenCalledWith({
        approvalId: "policy_approval_1",
        decision: "denied",
        decisionBy: "human",
        decisionNote: "Denied from Policy Approvals.",
      }),
    );
    expect(screen.getByText("rm -rf build/cache")).toBeInTheDocument();
    expect(await screen.findByText("Policy approval decision unavailable · policy service offline")).toBeInTheDocument();
  });

  it("renders and saves policy pack model settings from the right rail", async () => {
    policyApi.upsertNativePolicyPack.mockResolvedValueOnce({
      id: "policy_pack_proj_local_Ask_before_write",
      project_id: "proj_local",
      name: "Ask before write",
      sandbox_mode: "sandboxed",
      network: "blocked",
      network_profile: "local-only",
      file_write: "ask",
      tools: "ask",
      approval_required: ["shell_command"],
      forbidden_operations: ["network"],
      active: true,
      created_at: "2026-05-02T01:00:00Z",
      updated_at: "2026-05-02T01:00:00Z",
    });
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_policy_pack",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      security: {
        keychain: "local",
        secret_count: 0,
        policy_pack: {
          id: "policy_pack_proj_local_Ask_before_write",
          name: "Ask before write",
          sandbox_mode: "ask-before-write",
          network: "ask",
          network_profile: "internet",
          file_write: "ask",
          tools: "ask",
          approval_required_count: 1,
          forbidden_count: 0,
        },
      },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("Policy pack Ask before write · ask-before-write")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Policy pack name"), { target: { value: "Ask before write" } });
    fireEvent.change(screen.getByLabelText("Policy sandbox mode"), { target: { value: "sandboxed" } });
    fireEvent.change(screen.getByLabelText("Policy network permission"), { target: { value: "blocked" } });
    fireEvent.change(screen.getByLabelText("Policy network sandbox profile"), { target: { value: "local-only" } });
    fireEvent.change(screen.getByLabelText("Policy approval required actions"), { target: { value: "shell_command" } });
    fireEvent.change(screen.getByLabelText("Policy forbidden operations"), { target: { value: "network" } });
    fireEvent.click(screen.getByRole("button", { name: "Save policy pack" }));

    await waitFor(() => {
      expect(policyApi.upsertNativePolicyPack).toHaveBeenCalledWith({
        projectId: "proj_local",
        name: "Ask before write",
        sandboxMode: "sandboxed",
        network: "blocked",
        networkProfile: "local-only",
        fileWrite: "ask",
        tools: "ask",
        approvalRequired: ["shell_command"],
        forbiddenOperations: ["network"],
        setActive: true,
      });
    });
    expect(await screen.findByText("Saved policy pack Ask before write · sandboxed")).toBeInTheDocument();
    expect(screen.getByText("local-only network · ask file write")).toBeInTheDocument();
  });

  it("loads native policy packs from the policy pack model", async () => {
    policyApi.listNativePolicyPacks.mockResolvedValueOnce([
      {
        id: "policy_pack_1",
        project_id: "proj_local",
        name: "Ask before write",
        sandbox_mode: "ask-before-write",
        network: "ask",
        network_profile: "local-only",
        file_write: "ask",
        tools: "ask",
        approval_required: ["shell_command"],
        forbidden_operations: [],
        active: true,
        created_at: "2026-05-02T01:00:00Z",
        updated_at: "2026-05-02T01:00:00Z",
      },
      {
        id: "policy_pack_2",
        project_id: "proj_local",
        name: "Offline",
        sandbox_mode: "sandboxed",
        network: "blocked",
        network_profile: "offline",
        file_write: "blocked",
        tools: "ask",
        approval_required: [],
        forbidden_operations: ["network"],
        active: false,
        created_at: "2026-05-02T01:05:00Z",
        updated_at: "2026-05-02T01:05:00Z",
      },
    ]);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Load policy packs" }));

    await waitFor(() => expect(policyApi.listNativePolicyPacks).toHaveBeenCalledWith("proj_local"));
    expect(await screen.findByText("Ask before write · ask-before-write · active")).toBeInTheDocument();
    expect(screen.getByText("Offline · sandboxed · inactive")).toBeInTheDocument();
  });

  it("clears stale policy pack save results after failures and recovers on retry", async () => {
    policyApi.upsertNativePolicyPack
      .mockResolvedValueOnce({
        id: "policy_pack_proj_local_Ask_before_write",
        project_id: "proj_local",
        name: "Ask before write",
        sandbox_mode: "sandboxed",
        network: "blocked",
        network_profile: "local-only",
        file_write: "ask",
        tools: "ask",
        approval_required: ["shell_command"],
        forbidden_operations: ["network"],
        active: true,
        created_at: "2026-05-02T01:00:00Z",
        updated_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("policy store down"))
      .mockResolvedValueOnce({
        id: "policy_pack_proj_local_Offline",
        project_id: "proj_local",
        name: "Offline",
        sandbox_mode: "sandboxed",
        network: "blocked",
        network_profile: "offline",
        file_write: "blocked",
        tools: "ask",
        approval_required: ["tool_use"],
        forbidden_operations: ["network"],
        active: true,
        created_at: "2026-05-02T01:05:00Z",
        updated_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    fireEvent.change(screen.getByLabelText("Policy pack name"), { target: { value: "Ask before write" } });
    fireEvent.change(screen.getByLabelText("Policy sandbox mode"), { target: { value: "sandboxed" } });
    fireEvent.change(screen.getByLabelText("Policy network permission"), { target: { value: "blocked" } });
    fireEvent.change(screen.getByLabelText("Policy network sandbox profile"), { target: { value: "local-only" } });
    fireEvent.change(screen.getByLabelText("Policy approval required actions"), { target: { value: "shell_command" } });
    fireEvent.change(screen.getByLabelText("Policy forbidden operations"), { target: { value: "network" } });
    fireEvent.click(screen.getByRole("button", { name: "Save policy pack" }));
    expect(await screen.findByText("Saved policy pack Ask before write · sandboxed")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Policy pack name"), { target: { value: "Offline" } });
    fireEvent.change(screen.getByLabelText("Policy network sandbox profile"), { target: { value: "offline" } });
    fireEvent.change(screen.getByLabelText("Policy file write permission"), { target: { value: "blocked" } });
    fireEvent.change(screen.getByLabelText("Policy approval required actions"), { target: { value: "tool_use" } });
    fireEvent.click(screen.getByRole("button", { name: "Save policy pack" }));
    await waitFor(() => expect(screen.getByText("Policy pack unavailable · policy store down")).toBeInTheDocument());
    expect(screen.queryByText("Saved policy pack Ask before write · sandboxed")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Save policy pack" }));
    expect(await screen.findByText("Saved policy pack Offline · sandboxed")).toBeInTheDocument();
    expect(screen.getByText("offline network · blocked file write")).toBeInTheDocument();
    expect(screen.queryByText("Policy pack unavailable · policy store down")).not.toBeInTheDocument();
  });

  it("evaluates policy actions against the active network sandbox profile", async () => {
    policyApi.evaluateNativePolicyAction.mockResolvedValueOnce({
      audit_id: "permission_audit_1",
      project_id: "proj_local",
      policy_pack_id: "policy_pack_proj_local_Ask_before_write",
      action_kind: "network",
      decision: "forbidden",
      reason: "network profile blocks remote endpoint",
    });

    render(<App />);

    fireEvent.change(screen.getByLabelText("Policy action kind"), { target: { value: "network" } });
    fireEvent.change(screen.getByLabelText("Policy action command"), { target: { value: "curl https://api.example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Evaluate policy action" }));

    await waitFor(() => expect(policyApi.evaluateNativePolicyAction).toHaveBeenCalledWith({
      projectId: "proj_local",
      actionKind: "network",
      command: "curl https://api.example.com",
      requestedBy: "human",
    }));
    expect(await screen.findByText("Policy evaluation network · forbidden")).toBeInTheDocument();
    expect(screen.getByText("network profile blocks remote endpoint")).toBeInTheDocument();
  });

  it("clears stale policy action evaluation results after failures and recovers on retry", async () => {
    policyApi.evaluateNativePolicyAction
      .mockResolvedValueOnce({
        audit_id: "permission_audit_1",
        project_id: "proj_local",
        policy_pack_id: "policy_pack_proj_local_Ask_before_write",
        action_kind: "network",
        decision: "forbidden",
        reason: "network profile blocks remote endpoint",
      })
      .mockRejectedValueOnce(new Error("policy evaluator down"))
      .mockResolvedValueOnce({
        audit_id: "permission_audit_2",
        project_id: "proj_local",
        policy_pack_id: "policy_pack_proj_local_Ask_before_write",
        action_kind: "shell_command",
        decision: "approval_required",
        reason: "shell command requires approval",
      });

    render(<App />);

    fireEvent.change(screen.getByLabelText("Policy action kind"), { target: { value: "network" } });
    fireEvent.change(screen.getByLabelText("Policy action command"), { target: { value: "curl https://api.example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Evaluate policy action" }));
    expect(await screen.findByText("Policy evaluation network · forbidden")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Policy action kind"), { target: { value: "file_write" } });
    fireEvent.change(screen.getByLabelText("Policy action command"), { target: { value: "write /etc/hosts" } });
    fireEvent.click(screen.getByRole("button", { name: "Evaluate policy action" }));
    await waitFor(() => expect(screen.getByText("Policy evaluation unavailable · policy evaluator down")).toBeInTheDocument());
    expect(screen.queryByText("Policy evaluation network · forbidden")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Policy action kind"), { target: { value: "shell_command" } });
    fireEvent.change(screen.getByLabelText("Policy action command"), { target: { value: "rm -rf build/cache" } });
    fireEvent.click(screen.getByRole("button", { name: "Evaluate policy action" }));
    expect(await screen.findByText("Policy evaluation shell_command · approval_required")).toBeInTheDocument();
    expect(screen.getByText("shell command requires approval")).toBeInTheDocument();
    expect(screen.queryByText("Policy evaluation unavailable · policy evaluator down")).not.toBeInTheDocument();
  });

  it("persists provider model switching defaults and applies them to token adapter payloads", async () => {
    render(<App />);

    const switcher = await screen.findByRole("region", { name: /provider model switcher/i });
    expect(switcher).toHaveTextContent("Active openai · gpt-5.4 · agent agent_codex");

    fireEvent.change(screen.getByLabelText("Default provider"), { target: { value: "anthropic" } });
    fireEvent.change(screen.getByLabelText("Default model"), { target: { value: "claude-3-7-sonnet-latest" } });
    fireEvent.change(screen.getByLabelText("Provider model agent"), { target: { value: "agent_claude" } });
    fireEvent.click(screen.getByRole("button", { name: "Save provider model" }));

    await waitFor(() =>
      expect(providerModelApi.upsertNativeProviderModelSettings).toHaveBeenCalledWith({
        provider: "anthropic",
        model: "claude-3-7-sonnet-latest",
        agentProfileId: "agent_claude",
      }),
    );
    expect(switcher).toHaveTextContent("Saved anthropic · claude-3-7-sonnet-latest");
    expect(switcher).toHaveTextContent("Active anthropic · claude-3-7-sonnet-latest · agent agent_claude");
    expect(storage.get("haneulchi:provider-model")).toContain("claude-3-7-sonnet-latest");
    expect(screen.getByLabelText("Token usage adapter")).toHaveValue("local.usage-json");
    expect(screen.getByLabelText("Token usage adapter payload")).toHaveValue(
      JSON.stringify({ provider: "anthropic", model: "claude-3-7-sonnet-latest", input_tokens: 0, output_tokens: 0 }, null, 2),
    );
  });

  it("loads native provider model settings into the switcher", async () => {
    providerModelApi.getNativeProviderModelSettings.mockResolvedValueOnce({
      provider: "anthropic",
      model: "claude-3-7-sonnet-latest",
      agent_profile_id: "agent_claude",
    });

    render(<App />);

    const switcher = await screen.findByRole("region", { name: /provider model switcher/i });
    fireEvent.click(within(switcher).getByRole("button", { name: /load provider model/i }));

    await waitFor(() => expect(providerModelApi.getNativeProviderModelSettings).toHaveBeenCalled());
    expect(switcher).toHaveTextContent("Loaded anthropic · claude-3-7-sonnet-latest");
    expect(switcher).toHaveTextContent("Active anthropic · claude-3-7-sonnet-latest · agent agent_claude");
    expect(screen.getByLabelText("Default provider")).toHaveValue("anthropic");
    expect(screen.getByLabelText("Default model")).toHaveValue("claude-3-7-sonnet-latest");
    expect(screen.getByLabelText("Provider model agent")).toHaveValue("agent_claude");
    expect(storage.get("haneulchi:provider-model")).toContain("claude-3-7-sonnet-latest");
    expect(screen.getByLabelText("Token usage adapter")).toHaveValue("local.usage-json");
    expect(screen.getByLabelText("Token usage adapter agent")).toHaveValue("agent_claude");
    expect(screen.getByLabelText("Token usage adapter payload")).toHaveValue(
      JSON.stringify({ provider: "anthropic", model: "claude-3-7-sonnet-latest", input_tokens: 0, output_tokens: 0 }, null, 2),
    );
  });

  it("holds dangerous terminal input until the user explicitly approves it", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_dangerous_input",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pty_1",
          mode: "shell",
          title: "Danger shell",
          cwd: "/repo",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
          ports: [],
          created_at: "2026-05-02T01:00:00Z",
          updated_at: "2026-05-02T01:00:00Z",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.recordNativeSessionInput.mockResolvedValueOnce({
      session_id: "session_1",
      accepted: true,
      dangerous: true,
      input_len: 18,
      command_block_id: "cmdblk_session_input_1",
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Dangerous Danger shell" }));

    expect(await screen.findByText("Dangerous action approval required")).toBeInTheDocument();
    expect(screen.getByText("rm -rf /tmp/build")).toBeInTheDocument();
    expect(terminalPty.writeTerminalPtyInput).not.toHaveBeenCalledWith("pty_1", "rm -rf /tmp/build\r");

    fireEvent.click(screen.getByRole("button", { name: "Allow dangerous input session_1" }));

    await waitFor(() => {
      expect(sessionApi.recordNativeSessionInput).toHaveBeenCalledWith({
        sessionId: "session_1",
        text: "rm -rf /tmp/build\r",
        allowDangerous: true,
      });
    });
    expect(terminalPty.writeTerminalPtyInput).toHaveBeenCalledWith("pty_1", "rm -rf /tmp/build\r");
    expect(screen.queryByText("Dangerous action approval required")).not.toBeInTheDocument();
  });

  it("surfaces terminal input recording failures without blocking pty input", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_terminal_input_record_error",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pty_1",
          mode: "shell",
          title: "Danger shell",
          cwd: "/repo",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
          ports: [],
          created_at: "2026-05-02T01:00:00Z",
          updated_at: "2026-05-02T01:00:00Z",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.recordNativeSessionInput.mockRejectedValueOnce(new Error("session recorder offline"));

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Submit Danger shell" }));

    await waitFor(() =>
      expect(sessionApi.recordNativeSessionInput).toHaveBeenCalledWith({
        sessionId: "session_1",
        text: "npm test\r",
        allowDangerous: false,
      }),
    );
    expect(terminalPty.writeTerminalPtyInput).toHaveBeenCalledWith("pty_1", "npm test\r");
    expect(await screen.findByText("Terminal input recording unavailable · session recorder offline")).toBeInTheDocument();
  });

  it("surfaces agent profiles and updates availability from the right rail", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_agents",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [
        { id: "agent_codex", label: "Codex", available: false, last_heartbeat_at: null },
        { id: "agent_generic_shell", label: "Generic Shell", available: true },
      ],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    agentApi.updateNativeAgentProfileStatus.mockResolvedValueOnce({
      id: "agent_codex",
      name: "Codex",
      status: "available",
    });
    agentApi.heartbeatNativeAgentProfile.mockResolvedValueOnce({
      id: "agent_codex",
      name: "Codex",
      status: "available",
      last_heartbeat_at: "2026-05-02T01:02:00Z",
    });

    render(<App />);

    expect(await screen.findByText("Codex · paused")).toBeInTheDocument();
    expect(screen.getByText("Generic Shell · available")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Resume agent_codex" }));

    await waitFor(() =>
      expect(agentApi.updateNativeAgentProfileStatus).toHaveBeenCalledWith("agent_codex", "available"),
    );
    expect(await screen.findByText("Codex · available")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Heartbeat agent_codex" }));

    await waitFor(() =>
      expect(agentApi.heartbeatNativeAgentProfile).toHaveBeenCalledWith("agent_codex"),
    );
    expect(await screen.findByText("last heartbeat · 2026-05-02T01:02:00Z")).toBeInTheDocument();
  });

  it("keeps Claude Codex Gemini and generic presets visible after scanning agents", async () => {
    agentApi.scanNativeAgentProfiles.mockResolvedValueOnce([
      {
        id: "agent_claude",
        name: "Claude Code",
        runtime: "cli",
        command: "claude",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding", "review"],
        status: "unavailable",
        last_heartbeat_at: null,
      },
      {
        id: "agent_codex",
        name: "Codex",
        runtime: "cli",
        command: "codex",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding", "review"],
        status: "available",
        last_heartbeat_at: null,
      },
      {
        id: "agent_gemini",
        name: "Gemini CLI",
        runtime: "cli",
        command: "gemini",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding"],
        status: "unavailable",
        last_heartbeat_at: null,
      },
      {
        id: "agent_opencode",
        name: "OpenCode",
        runtime: "cli",
        command: "opencode",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding"],
        status: "unavailable",
        last_heartbeat_at: null,
      },
      {
        id: "agent_openclaw",
        name: "OpenClaw",
        runtime: "cli",
        command: "openclaw",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding"],
        status: "unavailable",
        last_heartbeat_at: null,
      },
      {
        id: "agent_cursor",
        name: "Cursor Agent",
        runtime: "cli",
        command: "cursor-agent",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding"],
        status: "unavailable",
        last_heartbeat_at: null,
      },
      {
        id: "agent_generic_shell",
        name: "Generic Shell",
        runtime: "generic-cli",
        command: "sh",
        args_json: [],
        env_policy_json: {},
        skills_json: ["fallback"],
        status: "available",
        last_heartbeat_at: null,
      },
    ]);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /scan agent profiles/i }));

    expect(await screen.findByText("Claude Code · paused")).toBeInTheDocument();
    expect(screen.getByText("Codex · available")).toBeInTheDocument();
    expect(screen.getByText("Gemini CLI · paused")).toBeInTheDocument();
    expect(screen.getByText("Generic Shell · available")).toBeInTheDocument();
  });

  it("loads native agent profiles from the agent directory", async () => {
    agentApi.listNativeAgentProfiles.mockResolvedValueOnce([
      {
        id: "agent_claude",
        name: "Claude Code",
        runtime: "cli",
        command: "claude",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding", "review"],
        status: "paused",
        last_heartbeat_at: "2026-05-02T01:00:00Z",
      },
      {
        id: "agent_codex",
        name: "Codex",
        runtime: "cli",
        command: "codex",
        args_json: [],
        env_policy_json: {},
        skills_json: ["coding"],
        status: "available",
        last_heartbeat_at: null,
      },
    ]);

    render(<App />);

    const directory = await screen.findByRole("region", { name: "Agent Directory" });
    fireEvent.click(within(directory).getByRole("button", { name: /load agent profiles/i }));

    await waitFor(() => expect(agentApi.listNativeAgentProfiles).toHaveBeenCalledWith());
    expect(await within(directory).findByText("Claude Code · paused")).toBeInTheDocument();
    expect(within(directory).getByText("Codex · available")).toBeInTheDocument();
    expect(within(directory).getByText("last heartbeat · 2026-05-02T01:00:00Z")).toBeInTheDocument();
  });

  it("surfaces agent heartbeat failures from the right rail", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_agent_heartbeat_error",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [{ id: "agent_codex", label: "Codex", available: true, last_heartbeat_at: null }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    agentApi.heartbeatNativeAgentProfile.mockRejectedValueOnce(new Error("agent heartbeat offline"));

    render(<App />);

    expect(await screen.findByText("Codex · available")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Heartbeat agent_codex" }));

    await waitFor(() => expect(agentApi.heartbeatNativeAgentProfile).toHaveBeenCalledWith("agent_codex"));
    expect(await screen.findByText("Agent directory unavailable · agent heartbeat offline")).toBeInTheDocument();
  });

  it("launches an agent profile as a raw terminal session from the right rail", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_agent_raw_terminal",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [{ id: "agent_acme", label: "Acme CLI", available: true }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.launchNativeAgentTerminal.mockResolvedValueOnce({
      session: {
        id: "session_agent_acme",
        project_id: "proj_auth",
        pane_id: "pane_session_agent_acme",
        mode: "agent",
        title: "Acme CLI raw agent",
        cwd: null,
        branch: null,
        agent_profile_id: "agent_acme",
        task_id: "task_agent_launch",
        run_id: "run_agent_launch",
        state: "running",
        attention_state: "none",
        token_budget_state: "unknown",
        created_at: "2026-05-02T01:01:00Z",
        updated_at: "2026-05-02T01:01:00Z",
      },
      pty_session: {
        id: "pty_agent_acme",
        title: "Acme CLI raw agent",
        command: "acme-agent",
        cols: 100,
        rows: 30,
      },
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Launch agent_acme raw terminal" }));

    await waitFor(() =>
      expect(sessionApi.launchNativeAgentTerminal).toHaveBeenCalledWith({
        projectId: "proj_auth",
        agentProfileId: "agent_acme",
        title: "Acme CLI raw agent",
        cols: 100,
        rows: 30,
      }),
    );
    expect(await screen.findByLabelText("Terminal pane Acme CLI raw agent")).toBeInTheDocument();
    expect(await screen.findByRole("region", { name: /session stack/i })).toHaveTextContent(
      "task task_agent_launch · run run_agent_launch · agent agent_acme",
    );
    expect(await screen.findByText("Launched Acme CLI raw terminal · pty_agent_acme")).toBeInTheDocument();
  });

  it("normalizes structured agent event payloads from the right rail", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_agent_event_normalizer",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [{ id: "session_1", project_id: "proj_auth", mode: "agent", title: "Codex run", state: "running", agent_profile_id: "agent_codex" }],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [{ id: "run_1", task_id: "task_1", project_id: "proj_auth", agent_profile_id: "agent_codex", lifecycle: "running", retry_count: 0 }], counts_by_lifecycle: {} },
      agents: [{ id: "agent_codex", label: "Codex", available: true }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    agentApi.ingestNativeAgentEvents.mockResolvedValueOnce({
      id: "agev_1",
      project_id: "proj_auth",
      session_id: "session_1",
      run_id: "run_1",
      agent_profile_id: "agent_codex",
      kind: "status",
      severity: "warning",
      detail: "Waiting for review",
      payload_json: {},
      source: "adapter:raw-jsonl",
      created_at: "2026-05-02T01:01:00Z",
    });

    render(<App />);

    const normalizer = await screen.findByRole("region", { name: /agent event normalizer/i });
    fireEvent.change(within(normalizer).getByLabelText("Agent event profile"), { target: { value: "agent_codex" } });
    fireEvent.change(within(normalizer).getByLabelText("Agent event session"), { target: { value: "session_1" } });
    fireEvent.change(within(normalizer).getByLabelText("Agent event run"), { target: { value: "run_1" } });
    fireEvent.change(within(normalizer).getByLabelText("Agent event payload"), {
      target: { value: "{\"type\":\"status\",\"status\":\"needs_input\",\"message\":\"Waiting for review\"}\n" },
    });
    fireEvent.click(within(normalizer).getByRole("button", { name: /ingest agent events/i }));

    await waitFor(() =>
      expect(agentApi.ingestNativeAgentEvents).toHaveBeenCalledWith({
        projectId: "proj_auth",
        sessionId: "session_1",
        runId: "run_1",
        agentProfileId: "agent_codex",
        adapter: "raw-jsonl",
        payload: { raw: "{\"type\":\"status\",\"status\":\"needs_input\",\"message\":\"Waiting for review\"}\n" },
      }),
    );
    expect(await within(normalizer).findByText("status · warning")).toBeInTheDocument();
    expect(screen.getByText("Waiting for review")).toBeInTheDocument();
  });

  it("clears stale agent event results after validation failures and recovers on retry", async () => {
    agentApi.ingestNativeAgentEvents
      .mockResolvedValueOnce({
        id: "agev_1",
        project_id: "proj_local",
        session_id: null,
        run_id: null,
        agent_profile_id: "agent_codex",
        kind: "status",
        severity: "warning",
        detail: "Waiting for review",
        payload_json: {},
        source: "adapter:raw-jsonl",
        created_at: "2026-05-02T01:01:00Z",
      })
      .mockResolvedValueOnce({
        id: "agev_2",
        project_id: "proj_local",
        session_id: null,
        run_id: null,
        agent_profile_id: "agent_codex",
        kind: "message",
        severity: "info",
        detail: "Review unblocked",
        payload_json: {},
        source: "adapter:raw-jsonl",
        created_at: "2026-05-02T01:02:00Z",
      });

    render(<App />);

    const normalizer = await screen.findByRole("region", { name: /agent event normalizer/i });
    fireEvent.change(within(normalizer).getByLabelText("Agent event profile"), { target: { value: "agent_codex" } });
    fireEvent.change(within(normalizer).getByLabelText("Agent event payload"), {
      target: { value: "{\"type\":\"status\",\"message\":\"Waiting for review\"}\n" },
    });
    fireEvent.click(within(normalizer).getByRole("button", { name: /ingest agent events/i }));
    expect(await within(normalizer).findByText("status · warning")).toBeInTheDocument();
    expect(normalizer).toHaveTextContent("Waiting for review");

    fireEvent.change(within(normalizer).getByLabelText("Agent event profile"), { target: { value: "" } });
    fireEvent.click(within(normalizer).getByRole("button", { name: /ingest agent events/i }));
    expect(within(normalizer).getByText("Agent event profile is required")).toBeInTheDocument();
    expect(within(normalizer).queryByText("status · warning")).not.toBeInTheDocument();
    expect(within(normalizer).queryByText("Waiting for review")).not.toBeInTheDocument();

    fireEvent.change(within(normalizer).getByLabelText("Agent event profile"), { target: { value: "agent_codex" } });
    fireEvent.change(within(normalizer).getByLabelText("Agent event payload"), {
      target: { value: "{\"type\":\"message\",\"message\":\"Review unblocked\"}\n" },
    });
    fireEvent.click(within(normalizer).getByRole("button", { name: /ingest agent events/i }));
    expect(await within(normalizer).findByText("message · info")).toBeInTheDocument();
    expect(normalizer).toHaveTextContent("Review unblocked");
    expect(normalizer).not.toHaveTextContent("Agent event profile is required");
  });

  it("registers a third-party CLI adapter profile from the right rail SDK panel", async () => {
    agentApi.upsertNativeAgentProfile.mockResolvedValueOnce({
      id: "agent_acme",
      name: "Acme CLI",
      runtime: "generic-cli",
      command: "acme-agent",
      status: "available",
    });

    render(<App />);

    const sdkPanel = await screen.findByRole("region", { name: /agent adapter sdk/i });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter profile id"), { target: { value: "agent_acme" } });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter profile name"), { target: { value: "Acme CLI" } });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter runtime"), { target: { value: "generic-cli" } });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter command"), { target: { value: "acme-agent" } });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter args JSON"), { target: { value: "[\"--json\"]" } });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter env policy JSON"), {
      target: { value: "{\"inherit\":false,\"allow\":[\"ACME_API_KEY\"]}" },
    });
    fireEvent.change(within(sdkPanel).getByLabelText("Adapter skills JSON"), { target: { value: "[\"code-review\"]" } });
    fireEvent.click(within(sdkPanel).getByRole("button", { name: /register adapter profile/i }));

    await waitFor(() =>
      expect(agentApi.upsertNativeAgentProfile).toHaveBeenCalledWith({
        id: "agent_acme",
        name: "Acme CLI",
        runtime: "generic-cli",
        command: "acme-agent",
        argsJson: ["--json"],
        envPolicyJson: { inherit: false, allow: ["ACME_API_KEY"] },
        skillsJson: ["code-review"],
        status: "available",
      }),
    );
    expect(await screen.findByText("Registered Acme CLI · generic-cli")).toBeInTheDocument();
    expect(screen.getByText("Acme CLI · available")).toBeInTheDocument();
  });

  it("renders an Agent Team mini dashboard with availability run load and budgets", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_agent_team",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_agent_current",
            title: "Implement auth retry",
            status: "running",
            priority: "high",
            project_id: "proj_local",
            assignee_type: "agent",
            assignee_id: "agent_codex",
            comment_count: 0,
            has_workpad: false,
          },
          {
            id: "task_agent_blocked",
            title: "Fix release blocker",
            status: "blocked",
            priority: "urgent",
            project_id: "proj_local",
            assignee_type: "agent",
            assignee_id: "agent_codex",
            comment_count: 0,
            has_workpad: false,
          },
        ],
        counts_by_status: { running: 1, blocked: 1 },
      },
      runs: {
        items: [
          {
            id: "run_codex_1",
            task_id: "task_agent_current",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "running",
            retry_count: 0,
          },
          {
            id: "run_codex_2",
            task_id: "task_agent_blocked",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "blocked",
            retry_count: 0,
          },
          {
            id: "run_claude_1",
            task_id: "task_3",
            project_id: "proj_local",
            agent_profile_id: "agent_claude",
            lifecycle: "review_ready",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { running: 1, blocked: 1, review_ready: 1 },
      },
      agents: [
        {
          id: "agent_codex",
          label: "Codex",
          available: true,
          token_usage: {
            input_tokens: 900,
            output_tokens: 600,
            total_tokens: 1500,
            cost_usd: 1.25,
          },
        },
        { id: "agent_claude", label: "Claude", available: false },
      ],
      reviews: [],
      attention: [],
      budgets: {
        workspace: {},
        projects: [],
        agents: [
          { scope_id: "agent_codex", used_usd: 4, max_usd: 5, state: "warn" },
          { scope_id: "agent_claude", used_usd: 1, max_usd: 5, state: "ok" },
        ],
      },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const dashboard = await screen.findByRole("region", { name: /agent team mini dashboard/i });
    expect(dashboard).toHaveTextContent("Available 1");
    expect(dashboard).toHaveTextContent("Paused 1");
    expect(dashboard).toHaveTextContent("Active runs 3");
    expect(dashboard).toHaveTextContent("Blocked 1");
    expect(dashboard).toHaveTextContent("Codex · available · 2 runs · budget warn · $1.25 · 1500 tokens");
    expect(dashboard).toHaveTextContent("Task Implement auth retry · run run_codex_1");
    expect(dashboard).toHaveTextContent("Blocker run_codex_2 · blocked");
    expect(dashboard).toHaveTextContent("Claude · paused · 1 run · budget ok");
  });

  it("renders roadmap timeline and calendar views from task planning metadata", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({ tasks: {} });
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_task_planning_views",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_auth_handoff",
            title: "Ship auth handoff",
            status: "ready",
            priority: "high",
            project_id: "proj_auth",
            cycle: "Sprint 8",
            module: "Authentication",
            token_usage: {
              input_tokens: 900,
              output_tokens: 600,
              total_tokens: 1500,
              cost_usd: 1.25,
            },
          },
          {
            id: "task_auth_review",
            title: "Review auth traces",
            status: "review",
            priority: "urgent",
            project_id: "proj_auth",
            cycle: "Sprint 8",
            module: "Authentication",
          },
          {
            id: "task_docs",
            title: "Publish onboarding docs",
            status: "blocked",
            priority: "medium",
            project_id: "proj_auth",
            cycle: "Sprint 9",
            module: "Docs",
          },
          {
            id: "task_platform",
            title: "Plan platform goal",
            status: "ready",
            priority: "low",
            project_id: "proj_auth",
            cycle: "Sprint 10",
            module: "Platform",
            initiative_id: "init_platform",
          },
        ],
        counts_by_status: { ready: 2, review: 1, blocked: 1 },
      },
      initiatives: [
        {
          id: "init_platform",
          project_id: "proj_auth",
          name: "Platform reliability goal",
          description: "Keep platform roadmap work tied to a visible goal",
          budget_id: "budget_platform",
          status: "active",
          token_usage: {
            input_tokens: 1400,
            output_tokens: 1000,
            total_tokens: 2400,
            cost_usd: 2,
          },
        },
      ],
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const roadmap = await screen.findByRole("region", { name: /roadmap timeline/i });
    expect(roadmap).toHaveTextContent("Sprint 8 · Authentication");
    expect(roadmap).toHaveTextContent("2 tasks · ready 1 · review 1");
    expect(roadmap).toHaveTextContent("Urgent 1");
    expect(roadmap).toHaveTextContent("Sprint 9 · Docs");
    expect(roadmap).toHaveTextContent("blocked 1");
    expect(roadmap).toHaveTextContent("Platform reliability goal · $2.00 · 2400 tokens · Sprint 10 · Platform");

    const calendar = screen.getByRole("region", { name: /calendar view/i });
    expect(calendar).toHaveTextContent("Sprint 8");
    expect(calendar).toHaveTextContent("Ship auth handoff · $1.25 · 1500 tokens");
    expect(calendar).toHaveTextContent("Review auth traces");
    expect(calendar).toHaveTextContent("Sprint 9");
    expect(calendar).toHaveTextContent("Publish onboarding docs");
    expect(calendar).toHaveTextContent("Plan platform goal");
  });

  it("hydrates roadmap initiative labels from the native initiative registry", async () => {
    taskApi.loadNativeTaskState.mockReturnValueOnce(immediateResolved({ tasks: {} }));
    initiativeApi.listNativeInitiatives.mockReturnValueOnce(immediateResolved([
      {
        id: "init_platform",
        project_id: "proj_auth",
        name: "Platform reliability goal",
        description: "Keep platform roadmap work tied to a visible goal",
        budget_id: "budget_platform",
        status: "active",
        token_usage: {
          input_tokens: 1800,
          output_tokens: 1200,
          total_tokens: 3000,
          cost_usd: 3,
        },
      },
    ]));
    stateSnapshot.getStateSnapshot.mockReturnValueOnce(immediateResolved({
      snapshot_id: "snap_task_planning_native_initiatives",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_platform",
            title: "Plan platform goal",
            status: "ready",
            priority: "low",
            project_id: "proj_auth",
            cycle: "Sprint 11",
            module: "Platform",
            initiative_id: "init_platform",
          },
        ],
        counts_by_status: { ready: 1 },
      },
      initiatives: [],
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    }));

    render(<App />);

    await waitFor(() => expect(initiativeApi.listNativeInitiatives).toHaveBeenCalledWith("proj_auth"));
    const roadmap = await screen.findByRole("region", { name: /roadmap timeline/i });
    expect(roadmap).toHaveTextContent("Platform reliability goal · $3.00 · 3000 tokens · Sprint 11 · Platform");
  });

  it("creates native roadmap initiatives from the roadmap panel", async () => {
    taskApi.loadNativeTaskState.mockReturnValueOnce(immediateResolved({ tasks: {} }));
    initiativeApi.createNativeInitiative.mockReturnValueOnce(immediateResolved({
      id: "init_launch",
      project_id: "proj_auth",
      name: "Launch readiness",
      description: null,
      budget_id: null,
      status: "planned",
    }));
    stateSnapshot.getStateSnapshot.mockReturnValueOnce(immediateResolved({
      snapshot_id: "snap_roadmap_create_initiative",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      initiatives: [],
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    }));

    render(<App />);

    const roadmap = await screen.findByRole("region", { name: /roadmap timeline/i });
    fireEvent.change(within(roadmap).getByLabelText("Roadmap initiative name"), {
      target: { value: "Launch readiness" },
    });
    fireEvent.change(within(roadmap).getByLabelText("Roadmap initiative status"), {
      target: { value: "planned" },
    });
    fireEvent.click(within(roadmap).getByRole("button", { name: /create roadmap initiative/i }));

    await waitFor(() =>
      expect(initiativeApi.createNativeInitiative).toHaveBeenCalledWith({
        projectId: "proj_auth",
        name: "Launch readiness",
        status: "planned",
      }),
    );
    expect(roadmap).toHaveTextContent("Launch readiness · planned");
  });

  it("renders skill pack registry and runtime pool summaries from context packs agents sessions and runs", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({ tasks: {} });
    knowledgeApi.listNativeContextPacks.mockResolvedValueOnce([
      {
        id: "ctx_auth_skill",
        project_id: "proj_auth",
        name: "auth-skill-pack",
        description: "Reusable auth implementation context",
        sources_json: {
          sources: [{ type: "knowledge_page", id: "kp_auth" }],
          budget: { max_tokens_hint: 18000 },
        },
      },
    ]);
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_runtime_pool",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [
        { id: "session_local", project_id: "proj_auth", mode: "shell", title: "Local zsh", state: "running" },
        { id: "session_ssh", project_id: "proj_auth", mode: "ssh", title: "Deploy SSH", state: "running" },
        { id: "session_cloud", project_id: "proj_auth", mode: "agent", title: "Cloud agent", state: "blocked" },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_auth",
            title: "Wire auth flow",
            status: "running",
            priority: "high",
            project_id: "proj_auth",
            context_pack_id: "ctx_auth_skill",
          },
        ],
        counts_by_status: { running: 1 },
      },
      runs: {
        items: [
          {
            id: "run_auth",
            task_id: "task_auth",
            project_id: "proj_auth",
            agent_profile_id: "agent_codex",
            session_id: "session_cloud",
            lifecycle: "running",
            retry_count: 0,
            context_pack_id: "ctx_auth_skill",
          },
          {
            id: "run_blocked",
            task_id: "task_ops",
            project_id: "proj_auth",
            agent_profile_id: "agent_codex",
            session_id: "session_ssh",
            lifecycle: "blocked",
            retry_count: 1,
          },
        ],
        counts_by_lifecycle: { running: 1, blocked: 1 },
      },
      agents: [{ id: "agent_codex", label: "Codex", available: true }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const registry = await screen.findByRole("region", { name: /skill pack registry/i });
    expect(registry).toHaveTextContent("auth-skill-pack");
    expect(registry).toHaveTextContent("ctx_auth_skill · 18000 tokens");
    expect(registry).toHaveTextContent("Used by 2 active workloads");
    expect(registry).toHaveTextContent("kp_auth");

    const runtimePool = screen.getByRole("region", { name: /runtime pool/i });
    expect(runtimePool).toHaveTextContent("Local 1");
    expect(runtimePool).toHaveTextContent("Remote SSH 1");
    expect(runtimePool).toHaveTextContent("Cloud agents 1");
    expect(runtimePool).toHaveTextContent("shell · 1 session · 0 runs · 0 blocked");
    expect(runtimePool).toHaveTextContent("ssh · 1 session · 1 run · 1 blocked");
    expect(runtimePool).toHaveTextContent("agent · 1 session · 1 run · 0 blocked");
  });

  it("loads native runtime pool from the runtime pool panel", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({ tasks: {} });
    agentApi.listNativeRuntimePool.mockResolvedValueOnce([
      {
        id: "agent",
        label: "Cloud agents",
        session_count: 1,
        run_count: 2,
        blocked_count: 1,
      },
      {
        id: "ssh",
        label: "Remote SSH",
        session_count: 1,
        run_count: 0,
        blocked_count: 0,
      },
    ]);
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_runtime_pool_native",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [{ id: "agent_codex", label: "Codex", available: true }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const runtimePool = await screen.findByRole("region", { name: /runtime pool/i });
    fireEvent.click(within(runtimePool).getByRole("button", { name: /load runtime pool/i }));

    await waitFor(() => expect(agentApi.listNativeRuntimePool).toHaveBeenCalledWith("proj_auth"));
    expect(runtimePool).toHaveTextContent("Loaded 2 runtimes");
    expect(runtimePool).toHaveTextContent("Cloud agents 1");
    expect(runtimePool).toHaveTextContent("agent · 1 session · 2 runs · 1 blocked");
    expect(runtimePool).toHaveTextContent("ssh · 1 session · 0 runs · 0 blocked");
  });

  it("loads and creates durable skill packs from the registry panel", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({ tasks: {} });
    agentApi.listNativeSkillPacks.mockResolvedValueOnce([
      {
        id: "skill_pack_1",
        project_id: "proj_auth",
        name: "Auth reviewer",
        description: "Review auth flows",
        skills_json: ["code-review", "auth"],
        source_context_pack_id: "ctx_auth",
        created_at: "2026-05-02T01:00:00Z",
        updated_at: "2026-05-02T01:00:00Z",
      },
    ]);
    agentApi.upsertNativeSkillPack.mockResolvedValueOnce({
      id: "skill_pack_2",
      project_id: "proj_auth",
      name: "Security reviewer",
      description: "Review security flows",
      skills_json: ["security", "auth"],
      source_context_pack_id: "ctx_security",
      created_at: "2026-05-02T01:05:00Z",
      updated_at: "2026-05-02T01:05:00Z",
    });
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_skill_packs",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [{ id: "agent_codex", label: "Codex", available: true }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const registry = await screen.findByRole("region", { name: /skill pack registry/i });
    fireEvent.click(within(registry).getByRole("button", { name: /load skill packs/i }));

    await waitFor(() => expect(agentApi.listNativeSkillPacks).toHaveBeenCalledWith("proj_auth"));
    expect(registry).toHaveTextContent("Auth reviewer");
    expect(registry).toHaveTextContent("skill_pack_1 · code-review, auth");
    expect(registry).toHaveTextContent("Review auth flows · ctx_auth");
    expect(registry).toHaveTextContent("Loaded 1 skill pack");

    fireEvent.change(within(registry).getByLabelText("Skill pack name"), {
      target: { value: "Security reviewer" },
    });
    fireEvent.change(within(registry).getByLabelText("Skill pack description"), {
      target: { value: "Review security flows" },
    });
    fireEvent.change(within(registry).getByLabelText("Skill pack skills JSON"), {
      target: { value: "[\"security\",\"auth\"]" },
    });
    fireEvent.change(within(registry).getByLabelText("Skill pack context pack"), {
      target: { value: "ctx_security" },
    });
    fireEvent.click(within(registry).getByRole("button", { name: /create skill pack/i }));

    await waitFor(() =>
      expect(agentApi.upsertNativeSkillPack).toHaveBeenCalledWith({
        projectId: "proj_auth",
        name: "Security reviewer",
        description: "Review security flows",
        skillsJson: ["security", "auth"],
        sourceContextPackId: "ctx_security",
      }),
    );
    expect(registry).toHaveTextContent("Security reviewer");
    expect(registry).toHaveTextContent("Saved skill pack Security reviewer");
  });

  it("renders a recent evidence activity table with task run and project context", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_recent_evidence",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          { id: "task_104", title: "Refresh auth harness", status: "review", priority: "high", project_id: "proj_auth" },
          { id: "task_118", title: "Update docs proof", status: "done", priority: "medium", project_id: "proj_docs" },
        ],
        counts_by_status: { review: 1, done: 1 },
      },
      runs: {
        items: [
          { id: "run_104", task_id: "task_104", project_id: "proj_auth", lifecycle: "review_ready", retry_count: 0 },
          { id: "run_118", task_id: "task_118", project_id: "proj_docs", lifecycle: "completed", retry_count: 1 },
        ],
        counts_by_lifecycle: { review_ready: 1, completed: 1 },
      },
      agents: [],
      reviews: [
        {
          id: "review_ev_104",
          evidence_pack_id: "ev_auth_104",
          task_id: "task_104",
          run_id: "run_104",
          state: "pending",
          completeness_state: "complete",
        },
        {
          id: "review_ev_118",
          evidence_pack_id: "ev_docs_118",
          task_id: "task_118",
          run_id: "run_118",
          state: "approved",
          completeness_state: "incomplete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const table = await screen.findByRole("table", { name: /recent evidence activity/i });
    await waitFor(() => expect(table).toHaveTextContent("ev_auth_104"));
    expect(table).toHaveTextContent("Refresh auth harness");
    expect(table).toHaveTextContent("Auth Service");
    expect(table).toHaveTextContent("pending · complete");
    expect(table).toHaveTextContent("review ready");
    expect(table).toHaveTextContent("ev_docs_118");
    expect(table).toHaveTextContent("Update docs proof");
    expect(table).toHaveTextContent("Docs Workspace");
    expect(table).toHaveTextContent("approved · incomplete");
    expect(table).toHaveTextContent("completed");
  });

  it("renders exported token usage in recent evidence activity", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_recent_evidence_usage",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          { id: "task_104", title: "Refresh auth harness", status: "review", priority: "high", project_id: "proj_auth" },
        ],
        counts_by_status: { review: 1 },
      },
      runs: {
        items: [
          { id: "run_104", task_id: "task_104", project_id: "proj_auth", lifecycle: "review_ready", retry_count: 0 },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
      agents: [],
      reviews: [
        {
          id: "review_ev_104",
          evidence_pack_id: "ev_auth_104",
          task_id: "task_104",
          run_id: "run_104",
          state: "pending",
          completeness_state: "complete",
          token_usage: {
            scope_type: "run",
            scope_id: "run_104",
            input_tokens: 1200,
            output_tokens: 800,
            total_tokens: 2000,
            cost_usd: 8.5,
            records: [],
          },
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const table = await screen.findByRole("table", { name: /recent evidence activity/i });
    await waitFor(() => expect(table).toHaveTextContent("review ready"));
    expect(table).toHaveTextContent("$8.50 · 2000 tokens");
  });

  it("surfaces degraded state for recent evidence activity", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_recent_evidence_degraded",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "degraded", pty: "ok", api: "degraded" },
    });

    render(<App />);

    const panel = await screen.findByRole("region", { name: /recent evidence activity/i });
    expect(panel).toHaveTextContent("Evidence activity degraded · api degraded · db degraded");
    expect(panel).toHaveTextContent("No recent evidence activity");
  });

  it("surfaces session stack and supports takeover from the right rail", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_sessions",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pane_session_1",
          mode: "agent",
          title: "Codex AUTH-104",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          state: "running",
          attention_state: "unread",
          token_budget_state: "unknown",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.takeoverNativeSession.mockResolvedValueOnce({
      id: "session_1",
      attention_state: "needs_input",
      state: "running",
    });

    render(<App />);

    expect(await screen.findByText("Codex AUTH-104 · agent · unread")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Take over session_1" }));

    await waitFor(() => expect(sessionApi.takeoverNativeSession).toHaveBeenCalledWith("session_1"));
    expect(await screen.findByText("Codex AUTH-104 · agent · needs_input")).toBeInTheDocument();
  });

  it("loads persisted native sessions from the session stack", async () => {
    sessionApi.listNativeSessions.mockResolvedValueOnce([
      {
        id: "session_loaded",
        project_id: "proj_local",
        pane_id: "pane_loaded",
        mode: "agent",
        title: "Loaded Codex session",
        cwd: "/repo/haneulchi",
        branch: "feature/native-session-list",
        agent_profile_id: "agent_codex",
        task_id: "task_loaded",
        run_id: "run_loaded",
        state: "running",
        attention_state: "needs_input",
        token_budget_state: "ok",
        created_at: "2026-05-05T01:00:00Z",
        updated_at: "2026-05-05T01:10:00Z",
      },
    ]);

    render(<App />);

    const stack = await screen.findByRole("region", { name: /session stack/i });
    fireEvent.click(within(stack).getByRole("button", { name: /load sessions/i }));

    await waitFor(() => expect(sessionApi.listNativeSessions).toHaveBeenCalledWith("proj_local"));
    expect(await within(stack).findByText("Loaded Codex session · agent · needs_input")).toBeInTheDocument();
    expect(within(stack).getByText("/repo/haneulchi · branch feature/native-session-list · task task_loaded · run run_loaded · agent agent_codex · heartbeat 2026-05-05T01:10:00Z")).toBeInTheDocument();
  });

  it("loads redacted terminal transcript chunks from the session stack", async () => {
    sessionApi.listNativeTerminalStreamChunks.mockResolvedValueOnce([
      {
        id: "stream_chunk_1",
        session_id: "session_1",
        seq_start: 1,
        seq_end: 8,
        artifact_path: "artifacts/transcripts/session_1/stream_chunk_1.txt",
        body: "OPENAI_API_KEY=redacted-secret-fixture\nPASS auth harness",
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_session_transcript_chunks",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pane_session_1",
          mode: "agent",
          title: "Codex AUTH-104",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const stack = await screen.findByRole("region", { name: /session stack/i });
    fireEvent.click(within(stack).getByRole("button", { name: "Load transcript session_1" }));

    await waitFor(() => expect(sessionApi.listNativeTerminalStreamChunks).toHaveBeenCalledWith("session_1", 10));
    expect(await within(stack).findByText("transcript 1-8 · OPENAI_API_KEY=[redacted]")).toBeInTheDocument();
    expect(stack).not.toHaveTextContent("redacted-secret-fixture");
  });

  it("scopes the session stack to the active project", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_project_session_stack",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [
        { id: "proj_auth", name: "Auth Service", state: "active" },
        { id: "proj_docs", name: "Docs Workspace", state: "idle" },
      ],
      project_tabs: [
        { id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true },
        { id: "tab_proj_docs", project_id: "proj_docs", label: "Docs Workspace", active: false },
      ],
      sessions: [
        {
          id: "session_auth",
          project_id: "proj_auth",
          pane_id: "pane_session_auth",
          mode: "agent",
          title: "Auth agent",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          state: "running",
          attention_state: "unread",
          token_budget_state: "ok",
        },
        {
          id: "session_docs",
          project_id: "proj_docs",
          pane_id: "pane_session_docs",
          mode: "shell",
          title: "Docs shell",
          cwd: "/repo/docs",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const stack = await screen.findByRole("region", { name: /session stack/i });
    expect(stack).toHaveTextContent("Auth agent · agent · unread");
    expect(stack).not.toHaveTextContent("Docs shell");
  });

  it("surfaces session takeover failures in the session stack", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_session_takeover_error",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pane_session_1",
          mode: "agent",
          title: "Codex AUTH-104",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          state: "running",
          attention_state: "unread",
          token_budget_state: "unknown",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.takeoverNativeSession.mockRejectedValueOnce(new Error("session daemon offline"));

    render(<App />);

    expect(await screen.findByText("Codex AUTH-104 · agent · unread")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Take over session_1" }));

    await waitFor(() => expect(sessionApi.takeoverNativeSession).toHaveBeenCalledWith("session_1"));
    expect(await screen.findByText("Session control unavailable · session daemon offline")).toBeInTheDocument();
  });

  it("keeps the previous terminal focused when session focus fails", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_session_focus_error",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pane_session_1",
          mode: "shell",
          title: "Shell 1",
          cwd: "/repo",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
        {
          id: "session_2",
          project_id: "proj_local",
          pane_id: "pane_session_2",
          mode: "agent",
          title: "Agent 2",
          cwd: "/repo",
          branch: "main",
          state: "running",
          attention_state: "unread",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.focusNativeSession.mockRejectedValueOnce(new Error("session focus offline"));

    render(<App />);

    expect(await screen.findByLabelText("Terminal pane Shell 1")).toHaveAttribute("data-highlighted", "true");
    fireEvent.click(screen.getByRole("button", { name: "Focus session_2" }));

    await waitFor(() => expect(sessionApi.focusNativeSession).toHaveBeenCalledWith("session_2"));
    expect(await screen.findByText("Session control unavailable · session focus offline")).toBeInTheDocument();
    expect(screen.getByLabelText("Terminal pane Shell 1")).toHaveAttribute("data-highlighted", "true");
    expect(screen.getByLabelText("Terminal pane Agent 2")).toHaveAttribute("data-highlighted", "false");
  });

  it("releases a taken-over session from the session stack", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snapshot_session_release",
      generated_at: "2026-05-03T00:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pane_session_1",
          mode: "agent",
          title: "Codex AUTH-104",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          state: "running",
          attention_state: "needs_input",
          token_budget_state: "unknown",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.releaseNativeSession.mockResolvedValueOnce({
      id: "session_1",
      attention_state: "none",
      state: "running",
    });

    render(<App />);

    expect(await screen.findByText("Codex AUTH-104 · agent · needs_input")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Release session_1" }));

    await waitFor(() => expect(sessionApi.releaseNativeSession).toHaveBeenCalledWith("session_1"));
    expect(await screen.findByText("Codex AUTH-104 · agent · none")).toBeInTheDocument();
  });

  it("attaches and detaches the selected task from the session stack", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      ...defaultStateSnapshot(),
      sessions: [
        {
          id: "session_1",
          project_id: "proj_local",
          pane_id: "pty_session_1",
          mode: "agent",
          title: "Codex AUTH-104",
          cwd: "/repo/auth-service",
          branch: "fix/auth",
          state: "running",
          attention_state: "none",
          token_budget_state: "unknown",
          task_id: null,
          run_id: null,
          agent_profile_id: "agent_codex",
        },
      ],
    });
    sessionApi.attachNativeSessionTask.mockResolvedValueOnce({
      id: "session_1",
      title: "Codex AUTH-104",
      mode: "agent",
      state: "running",
      attention_state: "none",
      token_budget_state: "unknown",
      task_id: "task_review",
    });
    sessionApi.detachNativeSessionTask.mockResolvedValueOnce({
      id: "session_1",
      title: "Codex AUTH-104",
      mode: "agent",
      state: "running",
      attention_state: "none",
      token_budget_state: "unknown",
      task_id: null,
    });

    render(<App />);

    expect(await screen.findByText("Codex AUTH-104 · agent · none")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.click(screen.getByRole("button", { name: "Attach task_review to session_1" }));

    await waitFor(() => expect(sessionApi.attachNativeSessionTask).toHaveBeenCalledWith("session_1", "task_review"));
    expect(await screen.findByText(/task task_review/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Detach task from session_1" }));

    await waitFor(() => expect(sessionApi.detachNativeSessionTask).toHaveBeenCalledWith("session_1"));
    await waitFor(() => expect(screen.queryByText(/task task_review/i)).not.toBeInTheDocument());
  });

  it("surfaces MVP task board counts in the workspace snapshot", async () => {
    render(<App />);

    expect(await screen.findByText("Task Board")).toBeInTheDocument();
    expect(screen.getByText("Inbox 1")).toBeInTheDocument();
    expect(screen.getByText("Ready 1")).toBeInTheDocument();
    expect(screen.getByText("Running 1")).toBeInTheDocument();
    expect(screen.getByText("Review 1")).toBeInTheDocument();
    expect(screen.getByText("Blocked 1")).toBeInTheDocument();
    expect(screen.getByText("Done 1")).toBeInTheDocument();
    expect(screen.getByText(/State tasks 6 tracked/i)).toBeInTheDocument();
  });

  it("loads persisted task board state when available", async () => {
    window.localStorage.setItem(
      "haneulchi:task-state:proj_local",
      JSON.stringify({
        tasks: {
          persisted_done: {
            id: "persisted_done",
            title: "Persisted task",
            status: "done",
            priority: "high",
            projectId: "proj_local",
          },
        },
      }),
    );

    render(<App />);

    expect(await screen.findByText("Task Board")).toBeInTheDocument();
    expect(screen.getByText("Inbox 0")).toBeInTheDocument();
    expect(screen.getByText("Done 1")).toBeInTheDocument();
    expect(screen.getByText(/State tasks 1 tracked/i)).toBeInTheDocument();
  });

  it("surfaces run lifecycle controls and updates the snapshot optimistically", async () => {
    const snapshotWithRuns: StateSnapshot = {
      snapshot_id: "snap_runs",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "running",
            retry_count: 0,
            next_retry_at: undefined,
            context_pack_id: "ctx_default",
          },
        ],
        counts_by_lifecycle: { running: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    };
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce(snapshotWithRuns);
    runApi.cancelNativeRun.mockResolvedValueOnce({
      id: "run_1",
      task_id: "task_ready",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      session_id: null,
      workflow_version_id: null,
      context_pack_id: "ctx_default",
      workspace_path: null,
      lifecycle: "cancelled",
      retry_count: 0,
      next_retry_at: null,
      budget_id: null,
      started_at: "2026-04-30T01:00:01Z",
      ended_at: "2026-04-30T01:00:02Z",
    });
    runApi.retryNativeRun.mockResolvedValueOnce({
      id: "run_1",
      task_id: "task_ready",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      session_id: null,
      workflow_version_id: null,
      context_pack_id: "ctx_default",
      workspace_path: null,
      lifecycle: "queued",
      retry_count: 1,
      next_retry_at: "2026-04-30T01:00:10Z",
      budget_id: null,
      started_at: null,
      ended_at: null,
    });

    render(<App />);

    expect(await screen.findByText("Run Queue")).toBeInTheDocument();
    expect(screen.getByText("run_1 · running · retries 0")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel run_1" }));

    await waitFor(() => expect(runApi.cancelNativeRun).toHaveBeenCalledWith("run_1"));
    expect(await screen.findByText("run_1 · cancelled · retries 0")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Retry run_1" }));

    await waitFor(() => expect(runApi.retryNativeRun).toHaveBeenCalledWith("run_1"));
    expect(await screen.findByText("run_1 · queued · retries 1")).toBeInTheDocument();
    expect(screen.getByText("next retry 2026-04-30T01:00:10Z")).toBeInTheDocument();
  });

  it("shows every run lifecycle bucket in the Run Queue summary", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_run_lifecycle_counts",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [],
        counts_by_lifecycle: {
          queued: 1,
          claimed: 2,
          starting: 3,
          running: 4,
          waiting_input: 5,
          permission_requested: 6,
          blocked: 7,
          review_ready: 8,
          completed: 9,
          failed: 10,
          cancelled: 11,
        },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const runQueue = await screen.findByRole("region", { name: "Run Queue" });
    expect(runQueue).toHaveTextContent("queued 1");
    expect(runQueue).toHaveTextContent("claimed 2");
    expect(runQueue).toHaveTextContent("starting 3");
    expect(runQueue).toHaveTextContent("running 4");
    expect(runQueue).toHaveTextContent("waiting input 5");
    expect(runQueue).toHaveTextContent("permission requested 6");
    expect(runQueue).toHaveTextContent("blocked 7");
    expect(runQueue).toHaveTextContent("review ready 8");
    expect(runQueue).toHaveTextContent("completed 9");
    expect(runQueue).toHaveTextContent("failed 10");
    expect(runQueue).toHaveTextContent("cancelled 11");
  });

  it("blocks dispatch for ready tasks assigned to paused agents", async () => {
    window.localStorage.setItem(
      "haneulchi:task-state:proj_local",
      JSON.stringify({
        tasks: {
          task_paused_agent: {
            id: "task_paused_agent",
            title: "Ship paused agent task",
            status: "ready",
            priority: "high",
            projectId: "proj_local",
            assignee: "agent_codex",
          },
        },
      }),
    );
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_paused_agent_dispatch",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [{ id: "agent_codex", label: "Codex", available: false }],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const taskBoard = await screen.findByRole("region", { name: "Task Board" });
    const dispatch = within(taskBoard).getByRole("button", { name: "Dispatch Ship paused agent task" });
    expect(dispatch).toBeDisabled();
    expect(taskBoard).toHaveTextContent("Codex unavailable");

    fireEvent.click(dispatch);

    expect(runApi.dispatchNativeRun).not.toHaveBeenCalled();
  });

  it("surfaces run lifecycle mutation failures and clears them after recovery", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_run_lifecycle_error",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_error",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "running",
            retry_count: 0,
            next_retry_at: undefined,
          },
        ],
        counts_by_lifecycle: { running: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    runApi.cancelNativeRun.mockRejectedValueOnce(new Error("cancel denied"));
    runApi.updateNativeRunLifecycle.mockResolvedValueOnce({
      id: "run_error",
      task_id: "task_ready",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      session_id: null,
      workflow_version_id: null,
      context_pack_id: null,
      workspace_path: null,
      lifecycle: "review_ready",
      retry_count: 0,
      next_retry_at: null,
      budget_id: null,
      started_at: "2026-04-30T01:00:01Z",
      ended_at: null,
    });

    render(<App />);

    expect(await screen.findByText("run_error · running · retries 0")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel run_error" }));

    expect(await screen.findByText("Run lifecycle unavailable · cancel denied")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Move run_error to review" }));

    await waitFor(() => expect(runApi.updateNativeRunLifecycle).toHaveBeenCalledWith("run_error", "review_ready"));
    expect(await screen.findByText("run_error · review_ready · retries 0")).toBeInTheDocument();
    expect(screen.queryByText(/Run lifecycle unavailable/i)).not.toBeInTheDocument();
  });

  it("moves runs into blocked state with a status detail from the Run Queue", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_run_block_detail",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_block_detail",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "running",
            retry_count: 0,
            next_retry_at: undefined,
          },
        ],
        counts_by_lifecycle: { running: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    runApi.updateNativeRunLifecycle.mockResolvedValueOnce({
      id: "run_block_detail",
      task_id: "task_ready",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      session_id: null,
      workflow_version_id: null,
      context_pack_id: null,
      workspace_path: null,
      lifecycle: "blocked",
      retry_count: 0,
      next_retry_at: null,
      status_detail: "Needs OAuth test account",
      budget_id: null,
      started_at: "2026-04-30T01:00:01Z",
      ended_at: null,
    });

    render(<App />);

    expect(await screen.findByText("run_block_detail · running · retries 0")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Status detail for run_block_detail"), {
      target: { value: "Needs OAuth test account" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Block run_block_detail" }));

    await waitFor(() =>
      expect(runApi.updateNativeRunLifecycle).toHaveBeenCalledWith(
        "run_block_detail",
        "blocked",
        "Needs OAuth test account",
      ),
    );
    expect(await screen.findByText("run_block_detail · blocked · retries 0")).toBeInTheDocument();
    expect(screen.getByText("state detail Needs OAuth test account")).toBeInTheDocument();
  });

  it("summarizes blocked waiting and permission run states in the Run Queue", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_run_exception_states",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_waiting",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "waiting_input",
            retry_count: 0,
            status_detail: "Needs OAuth test account",
          },
          {
            id: "run_permission",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "permission_requested",
            retry_count: 0,
            status_detail: "Permission requested: shell_command",
          },
          {
            id: "run_blocked",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "blocked",
            retry_count: 0,
            status_detail: "Blocked by release gate",
          },
        ],
        counts_by_lifecycle: { waiting_input: 1, permission_requested: 1, blocked: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const runQueue = await screen.findByText("Run Queue");
    const panel = runQueue.closest("section");
    expect(panel).toHaveTextContent("waiting input 1");
    expect(panel).toHaveTextContent("permission requested 1");
    expect(panel).toHaveTextContent("blocked 1");
  });

  it("loads native runs from the Run Queue", async () => {
    runApi.listNativeRuns.mockResolvedValueOnce([
      {
        id: "run_loaded",
        task_id: "task_loaded",
        project_id: "proj_local",
        agent_profile_id: "agent_codex",
        session_id: "session_loaded",
        workflow_version_id: "workflow_1",
        context_pack_id: "ctx_default",
        workspace_path: "/repo/.haneulchi/worktrees/run_loaded",
        lifecycle: "running",
        retry_count: 2,
        next_retry_at: "2026-05-05T01:15:00Z",
        status_detail: "Executing integration suite",
        budget_id: null,
        started_at: "2026-05-05T01:00:00Z",
        ended_at: null,
      },
    ]);

    render(<App />);

    const queue = await screen.findByRole("region", { name: /run queue/i });
    fireEvent.click(within(queue).getByRole("button", { name: /load runs/i }));

    await waitFor(() => expect(runApi.listNativeRuns).toHaveBeenCalledWith("proj_local"));
    expect(await within(queue).findByText("run_loaded · running · retries 2")).toBeInTheDocument();
    expect(within(queue).getByText("session session_loaded")).toBeInTheDocument();
    expect(within(queue).getByText("worktree /repo/.haneulchi/worktrees/run_loaded")).toBeInTheDocument();
    expect(within(queue).getByText("state detail Executing integration suite")).toBeInTheDocument();
    expect(within(queue).getByText("next retry 2026-05-05T01:15:00Z")).toBeInTheDocument();
  });

  it("generates native evidence packs from the Run Queue", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      ...defaultStateSnapshot(),
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_review",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "review_ready",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
      reviews: [],
    });
    commandBlockApi.generateNativeEvidencePackForRun.mockResolvedValueOnce({
      id: "ev_run_1",
      task_id: "task_review",
      run_id: "run_1",
      artifact_path: "artifacts/evidence/ev_run_1.json",
      completeness_state: "complete",
      body_json: { tests: [{ command_block_id: "cmdblk_1" }] },
    });

    render(<App />);

    const queue = await screen.findByRole("region", { name: /run queue/i });
    fireEvent.click(within(queue).getByRole("button", { name: "Generate evidence for run_1" }));

    await waitFor(() =>
      expect(commandBlockApi.generateNativeEvidencePackForRun).toHaveBeenCalledWith({
        runId: "run_1",
        evidencePackId: "ev_run_1",
      }),
    );
    expect(await within(queue).findByText("Evidence ev_run_1 · complete · artifacts/evidence/ev_run_1.json")).toBeInTheDocument();
    expect(await screen.findByText("review_ev_run_1 · pending · complete")).toBeInTheDocument();
  });

  it("loads run replay metadata from the Run Queue", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      ...defaultStateSnapshot(),
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_review",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "review_ready",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
    });
    workflowApi.getRunReplayMetadata.mockResolvedValueOnce({
      id: "replay_1",
      run_id: "run_1",
      artifact_path: "artifacts/replays/run_1.json",
      body_json: {
        events: 12,
        exit_code: 0,
      },
    });

    render(<App />);

    const queue = await screen.findByRole("region", { name: /run queue/i });
    fireEvent.click(within(queue).getByRole("button", { name: "Load replay for run_1" }));

    await waitFor(() => expect(workflowApi.getRunReplayMetadata).toHaveBeenCalledWith("run_1"));
    expect(await within(queue).findByText("Replay replay_1 · artifacts/replays/run_1.json · events, exit_code")).toBeInTheDocument();
  });

  it("posts agent status updates from the Run Queue", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_run_status_update",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_status_update",
            task_id: "task_ready",
            project_id: "proj_local",
            agent_profile_id: "agent_codex",
            lifecycle: "running",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { running: 1 },
      },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    runApi.recordNativeRunStatusUpdate.mockResolvedValueOnce({
      id: "comment_1",
      task_id: "task_ready",
      run_id: "run_status_update",
      author_type: "agent",
      author_id: "agent_codex",
      body_md: "Investigating OAuth fixture failure.",
      parent_id: null,
      created_at: "2026-04-30T01:00:00Z",
    });

    render(<App />);

    expect(await screen.findByText("run_status_update · running · retries 0")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Status update for run_status_update"), {
      target: { value: "Investigating OAuth fixture failure." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Post status for run_status_update" }));

    await waitFor(() =>
      expect(runApi.recordNativeRunStatusUpdate).toHaveBeenCalledWith({
        runId: "run_status_update",
        bodyMd: "Investigating OAuth fixture failure.",
      }),
    );
    expect(await screen.findByText("Status update posted · comment_1")).toBeInTheDocument();
    expect(screen.getByLabelText("Status update for run_status_update")).toHaveValue("");
  });

  it("dispatches a ready task into the native run queue from the board", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_ready: {
          id: "task_ready",
          title: "Ready dispatch task",
          status: "ready",
          priority: "high",
          projectId: "proj_local",
        },
      },
    });
    runApi.dispatchNativeRun.mockResolvedValueOnce({
      id: "run_1",
      task_id: "task_ready",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      session_id: "session_1",
      workflow_version_id: null,
      context_pack_id: "ctx_default",
      workspace_path: "/repo/.haneulchi/worktrees/run_1",
      lifecycle: "queued",
      retry_count: 0,
      budget_id: null,
      started_at: null,
      ended_at: null,
    });

    render(<App />);

    expect(await screen.findByText("Ready dispatch task")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Dispatch Ready dispatch task" }));

    await waitFor(() =>
      expect(runApi.dispatchNativeRun).toHaveBeenCalledWith({
        taskId: "task_ready",
        agentProfileId: "agent_codex",
        contextPackId: "ctx_default",
      }),
    );
    expect(await screen.findByText("run_1 · queued · retries 0")).toBeInTheDocument();
    expect(await screen.findByText("session session_1")).toBeInTheDocument();
    expect(await screen.findByText("worktree /repo/.haneulchi/worktrees/run_1")).toBeInTheDocument();
    expect(await screen.findByText("Agent session · agent · none")).toBeInTheDocument();
    expect(screen.getByText("Running 1")).toBeInTheDocument();
  });

  it("dispatches an assigned ready task to its agent assignee", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_assigned: {
          id: "task_assigned",
          title: "Assigned dispatch task",
          status: "ready",
          priority: "high",
          projectId: "proj_local",
          assignee: "agent_claude",
        },
      },
    });
    runApi.dispatchNativeRun.mockResolvedValueOnce({
      id: "run_assigned",
      task_id: "task_assigned",
      project_id: "proj_local",
      agent_profile_id: "agent_claude",
      session_id: "session_claude",
      workflow_version_id: null,
      context_pack_id: "ctx_default",
      workspace_path: "/repo/.haneulchi/worktrees/run_assigned",
      lifecycle: "queued",
      retry_count: 0,
      budget_id: null,
      started_at: null,
      ended_at: null,
    });

    render(<App />);

    expect(await screen.findByText("Assigned dispatch task")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Dispatch Assigned dispatch task" }));

    await waitFor(() =>
      expect(runApi.dispatchNativeRun).toHaveBeenCalledWith({
        taskId: "task_assigned",
        agentProfileId: "agent_claude",
        contextPackId: "ctx_default",
      }),
    );
    expect(await screen.findByText("run_assigned · queued · retries 0")).toBeInTheDocument();
    expect(await screen.findByText("session session_claude")).toBeInTheDocument();
  });

  it("surfaces and clears budget policy gate errors when dispatch is blocked", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_ready: {
          id: "task_ready",
          title: "Budget gated task",
          status: "ready",
          priority: "high",
          projectId: "proj_local",
        },
      },
    });
    runApi.dispatchNativeRun
      .mockRejectedValueOnce(new Error("project budget exceeded: used $10.00 of $10.00"))
      .mockResolvedValueOnce({
        id: "run_budget",
        task_id: "task_ready",
        project_id: "proj_local",
        agent_profile_id: "agent_codex",
        session_id: "session_budget",
        workflow_version_id: null,
        context_pack_id: "ctx_default",
        workspace_path: null,
        lifecycle: "queued",
        retry_count: 0,
        budget_id: null,
        started_at: null,
        ended_at: null,
      });

    render(<App />);

    expect(await screen.findByText("Budget gated task")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Dispatch Budget gated task" }));

    expect(await screen.findByText("Dispatch blocked · project budget exceeded: used $10.00 of $10.00")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Dispatch Budget gated task" }));

    await waitFor(() => expect(runApi.dispatchNativeRun).toHaveBeenCalledTimes(2));
    expect(screen.queryByText("Dispatch blocked · project budget exceeded: used $10.00 of $10.00")).not.toBeInTheDocument();
    expect(await screen.findByText("run_budget · queued · retries 0")).toBeInTheDocument();
  });

  it("attaches a context pack to a task and dispatches with that pack", async () => {
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_ready: {
          id: "task_ready",
          title: "Ready context task",
          status: "ready",
          priority: "high",
          projectId: "proj_local",
        },
      },
    });
    knowledgeApi.listNativeContextPacks.mockResolvedValueOnce([
      {
        id: "ctx_auth",
        project_id: "proj_local",
        name: "auth-default",
        description: "Auth docs",
        sources_json: { sources: [{ type: "knowledge_page", id: "kp_1" }] },
      },
    ]);
    taskApi.updateNativeTaskContext.mockResolvedValueOnce({
      id: "task_ready",
      key: "LOCAL-1",
      project_id: "proj_local",
      title: "Ready context task",
      description: null,
      status: "ready",
      priority: "high",
      assignee_type: null,
      assignee_id: null,
      cycle_id: null,
      module_id: null,
      context_pack_id: "ctx_auth",
    });
    runApi.dispatchNativeRun.mockResolvedValueOnce({
      id: "run_2",
      task_id: "task_ready",
      project_id: "proj_local",
      agent_profile_id: "agent_codex",
      session_id: "session_2",
      workflow_version_id: null,
      context_pack_id: "ctx_auth",
      workspace_path: null,
      lifecycle: "queued",
      retry_count: 0,
      budget_id: null,
      started_at: null,
      ended_at: null,
    });

    render(<App />);

    expect(await screen.findByText("Ready context task")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Open task Ready context task" }));
    fireEvent.change(await screen.findByLabelText("Task context pack"), { target: { value: "ctx_auth" } });
    fireEvent.click(screen.getByRole("button", { name: "Attach context pack to task" }));

    await waitFor(() => expect(taskApi.updateNativeTaskContext).toHaveBeenCalledWith({
      taskId: "task_ready",
      contextPackId: "ctx_auth",
    }));
    expect(await screen.findByText("Attached context pack ctx_auth")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Dispatch selected task with context" }));

    await waitFor(() =>
      expect(runApi.dispatchNativeRun).toHaveBeenCalledWith({
        taskId: "task_ready",
        agentProfileId: "agent_codex",
        contextPackId: "ctx_auth",
      }),
    );
    expect(await screen.findByText("run_2 · queued · retries 0")).toBeInTheDocument();
  });

  it("surfaces invalid workflow diagnostics and reloads a sample workflow from the UI", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_workflow",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: {
        valid: false,
        invalid_projects: ["proj_local"],
        current_version_id: "workflow_2",
        last_known_good_version_id: "workflow_1",
        diagnostics: {
          errors: [{ code: "hook_path_escapes_repo", message: "hook before_run must stay inside the repo" }],
        },
      },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    workflowApi.reloadWorkflow.mockResolvedValueOnce({
      id: "workflow_3",
      valid: true,
      diagnostics_json: { errors: [] },
    });

    render(<App />);

    expect(await screen.findByText("Workflow Contract")).toBeInTheDocument();
    expect(screen.getByText("Invalid · current workflow_2 · LKG workflow_1")).toBeInTheDocument();
    expect(screen.getByText("hook before_run must stay inside the repo")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reload sample workflow" }));

    await waitFor(() => expect(workflowApi.reloadWorkflow).toHaveBeenCalledWith(expect.objectContaining({
      projectId: "proj_local",
      sourcePath: "WORKFLOW.md",
    })));
    expect(await screen.findByText("Valid · current workflow_3 · LKG workflow_3")).toBeInTheDocument();
  });

  it("surfaces sample workflow reload failures from the workflow panel", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_workflow_reload_error",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: {
        valid: false,
        invalid_projects: ["proj_local"],
        current_version_id: "workflow_2",
        last_known_good_version_id: "workflow_1",
        diagnostics: {
          errors: [{ code: "hook_path_escapes_repo", message: "hook before_run must stay inside the repo" }],
        },
      },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    workflowApi.reloadWorkflow.mockRejectedValueOnce(new Error("workflow daemon offline"));

    render(<App />);

    expect(await screen.findByText("Workflow Contract")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reload sample workflow" }));

    await waitFor(() => expect(workflowApi.reloadWorkflow).toHaveBeenCalledWith(expect.objectContaining({
      projectId: "proj_local",
      sourcePath: "WORKFLOW.md",
    })));
    expect(await screen.findByText("Workflow reload unavailable · workflow daemon offline")).toBeInTheDocument();
  });

  it("validates sample workflows and refreshes runtime state from the workflow panel", async () => {
    workflowApi.validateWorkflow.mockResolvedValueOnce({
      project_id: "proj_local",
      source_path: "WORKFLOW.md",
      valid: false,
      parsed_json: {},
      diagnostics_json: {
        errors: [{ code: "frontmatter_missing", message: "frontmatter missing" }],
      },
    });
    workflowApi.getWorkflowRuntimeState.mockResolvedValueOnce({
      valid: true,
      current_version_id: "workflow_5",
      last_known_good_version_id: "workflow_5",
      diagnostics: { errors: [] },
    });

    render(<App />);

    const workflowPanel = await screen.findByRole("region", { name: /workflow contract/i });
    fireEvent.click(within(workflowPanel).getByRole("button", { name: /validate sample workflow/i }));

    await waitFor(() => expect(workflowApi.validateWorkflow).toHaveBeenCalledWith(expect.objectContaining({
      projectId: "proj_local",
      sourcePath: "WORKFLOW.md",
    })));
    expect(await within(workflowPanel).findByText("Workflow validation invalid · frontmatter missing")).toBeInTheDocument();
    expect(workflowPanel).toHaveTextContent("Invalid · current none · LKG none");

    fireEvent.click(within(workflowPanel).getByRole("button", { name: /refresh workflow runtime/i }));

    await waitFor(() => expect(workflowApi.getWorkflowRuntimeState).toHaveBeenCalledWith("proj_local"));
    expect(await within(workflowPanel).findByText("Workflow runtime refreshed · workflow_5")).toBeInTheDocument();
    expect(workflowPanel).toHaveTextContent("Valid · current workflow_5 · LKG workflow_5");
  });

  it("runs workflow hooks from the workflow panel", async () => {
    workflowApi.runWorkflowHook.mockResolvedValueOnce({
      run_id: "run_1",
      hook_name: "before_run",
      status: "completed",
      exit_code: 0,
      stdout: "ok",
      stderr: "",
      source_path: "/repo/.haneulchi/hooks/before_run.sh",
      mirrored_path: "/repo/.haneulchi/worktrees/run_1/.haneulchi/hooks/before_run.sh",
      workspace_path: "/repo/.haneulchi/worktrees/run_1",
      env_json: { HANEULCHI_RUN_ID: "run_1" },
    });

    render(<App />);

    const workflowPanel = await screen.findByRole("region", { name: /workflow contract/i });
    fireEvent.change(within(workflowPanel).getByLabelText("Workflow hook run id"), { target: { value: "run_1" } });
    fireEvent.change(within(workflowPanel).getByLabelText("Workflow hook name"), { target: { value: "before_run" } });
    fireEvent.change(within(workflowPanel).getByLabelText("Workflow hook repo root"), { target: { value: "/repo" } });
    fireEvent.change(within(workflowPanel).getByLabelText("Workflow hook workspace path"), {
      target: { value: "/repo/.haneulchi/worktrees/run_1" },
    });
    fireEvent.click(within(workflowPanel).getByRole("button", { name: /run workflow hook/i }));

    await waitFor(() => expect(workflowApi.runWorkflowHook).toHaveBeenCalledWith({
      runId: "run_1",
      hookName: "before_run",
      repoRoot: "/repo",
      workspacePath: "/repo/.haneulchi/worktrees/run_1",
    }));
    expect(await within(workflowPanel).findByText("Hook before_run · completed · exit 0")).toBeInTheDocument();
    expect(workflowPanel).toHaveTextContent("/repo/.haneulchi/worktrees/run_1");
  });

  it("renders a visual workflow debugger with runtime sequence and diagnostics", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_workflow_debugger",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: {
        valid: false,
        invalid_projects: ["proj_local"],
        current_version_id: "workflow_2",
        last_known_good_version_id: "workflow_1",
        diagnostics: {
          errors: [{ code: "template_namespace_not_allowed", message: "template namespace secret is not allowed" }],
        },
      },
      knowledge: { stale_count: 2, gap_count: 1, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    const debuggerPanel = await screen.findByRole("region", { name: /visual workflow debugger/i });
    expect(debuggerPanel).toHaveTextContent("Current workflow workflow_2 invalid");
    expect(debuggerPanel).toHaveTextContent("Last known good workflow_1");
    expect(debuggerPanel).toHaveTextContent("Load workflow");
    expect(debuggerPanel).toHaveTextContent("Resolve context");
    expect(debuggerPanel).toHaveTextContent("Prepare workspace");
    expect(debuggerPanel).toHaveTextContent("Run hooks");
    expect(debuggerPanel).toHaveTextContent("Launch agent");
    expect(debuggerPanel).toHaveTextContent("Generate evidence");
    expect(debuggerPanel).toHaveTextContent("template_namespace_not_allowed");
    expect(debuggerPanel).toHaveTextContent("template namespace secret is not allowed");
  });

  it("imports a built-in workflow preset from the workflow marketplace", async () => {
    workflowApi.reloadWorkflow.mockResolvedValueOnce({
      id: "workflow_4",
      valid: true,
      diagnostics_json: { errors: [] },
    });

    render(<App />);

    const marketplace = await screen.findByRole("region", { name: /workflow marketplace/i });
    expect(marketplace).toHaveTextContent("Harness Default");
    expect(marketplace).toHaveTextContent("Worktree agent run with before hook and evidence requirements");
    fireEvent.click(screen.getByRole("button", { name: "Import Harness Default workflow preset" }));

    await waitFor(() => expect(workflowApi.reloadWorkflow).toHaveBeenCalledWith(expect.objectContaining({
      projectId: "proj_local",
      sourcePath: "marketplace:harness-default/WORKFLOW.md",
    })));
    expect(await screen.findByText("Imported Harness Default workflow workflow_4")).toBeInTheDocument();
    expect(screen.getByText("Valid · current workflow_4 · LKG workflow_4")).toBeInTheDocument();
  });

  it("surfaces workflow marketplace import errors", async () => {
    workflowApi.reloadWorkflow.mockRejectedValueOnce(new Error("preset failed validation"));

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Import Harness Default workflow preset" }));

    expect(await screen.findByText("Import failed · preset failed validation")).toBeInTheDocument();
  });

  it("renders state snapshot review items and records review decisions", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_review",
            title: "Review generated evidence",
            status: "review",
            priority: "high",
            project_id: "proj_local",
          },
        ],
        counts_by_status: { review: 1 },
      },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_1",
          evidence_pack_id: "ev_run_1",
          task_id: "task_review",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
          diff_summary: {
            summary: "2 files changed, 14 insertions(+), 3 deletions(-)",
          },
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_review: {
          id: "task_review",
          title: "Review generated evidence",
          status: "review",
          priority: "high",
          projectId: "proj_local",
        },
      },
    });
    commandBlockApi.recordNativeEvidenceReviewDecision.mockResolvedValueOnce({
      id: "ev_run_1",
      task_id: "task_review",
      run_id: "run_1",
      artifact_path: "artifacts/evidence/ev_run_1.json",
      completeness_state: "complete",
      body_json: { review_decision: { decision: "approved" } },
    });

    render(<App />);

    expect(await screen.findByText("review_ev_run_1 · pending · complete")).toBeInTheDocument();
    expect(screen.getByText("Diff 2 files changed, 14 insertions(+), 3 deletions(-)")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reopen review_ev_run_1" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Block review_ev_run_1" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve review_ev_run_1" }));

    await waitFor(() =>
      expect(commandBlockApi.recordNativeEvidenceReviewDecision).toHaveBeenCalledWith({
        evidencePackId: "ev_run_1",
        decision: "approved",
        reviewerId: "human",
        bodyMd: "Approved from Review Queue.",
      }),
    );
    expect(await screen.findByText("review_ev_run_1 · approved · complete")).toBeInTheDocument();
    expect(screen.getByText("Done 1")).toBeInTheDocument();
  });

  it("surfaces review decision failures and recovers on retry", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_error",
      generated_at: "2026-05-03T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_review_error",
            title: "Review flaky evidence",
            status: "review",
            priority: "high",
            project_id: "proj_local",
          },
        ],
        counts_by_status: { review: 1 },
      },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [
        {
          id: "review_ev_error",
          evidence_pack_id: "ev_error",
          task_id: "task_review_error",
          run_id: "run_error",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("review_ev_error · pending · complete")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve review_ev_error" }));

    expect(await screen.findByText("Review decision unavailable · native evidence API unavailable")).toBeInTheDocument();
    expect(screen.getByText("review_ev_error · pending · complete")).toBeInTheDocument();

    commandBlockApi.recordNativeEvidenceReviewDecision.mockResolvedValueOnce({
      id: "ev_error",
      task_id: "task_review_error",
      run_id: "run_error",
      artifact_path: "artifacts/evidence/ev_error.json",
      completeness_state: "complete",
      body_json: { review_decision: { decision: "approved" } },
    });
    fireEvent.click(screen.getByRole("button", { name: "Approve review_ev_error" }));

    expect(await screen.findByText("Review decision recorded · approved review_ev_error")).toBeInTheDocument();
    expect(screen.queryByText("Review decision unavailable · native evidence API unavailable")).not.toBeInTheDocument();
    expect(screen.getByText("review_ev_error · approved · complete")).toBeInTheDocument();
  });

  it("opens the linked terminal and worktree diff from a review row", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_open",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_review",
          project_id: "proj_auth",
          pane_id: "pane_review",
          mode: "agent",
          title: "Review agent",
          cwd: "/repo/auth",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
          run_id: "run_1",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_review",
            project_id: "proj_auth",
            lifecycle: "review_ready",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_1",
          evidence_pack_id: "ev_run_1",
          task_id: "task_review",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.readNativeProjectDiff.mockResolvedValueOnce({
      project_id: "proj_auth",
      path: null,
      body: "diff --git a/src/App.tsx b/src/App.tsx\n+review action\n",
      file_count: 1,
      truncated: false,
      degraded_reason: null,
    });

    render(<App />);

    expect(await screen.findByText("review_ev_run_1 · pending · complete")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Open terminal review_ev_run_1" }));

    expect(screen.getByLabelText("Terminal pane Review agent")).toHaveAttribute("data-highlighted", "true");
    await waitFor(() => expect(sessionApi.focusNativeSession).toHaveBeenCalledWith("session_review"));

    fireEvent.click(screen.getByRole("button", { name: "Open worktree review_ev_run_1" }));

    await waitFor(() => expect(projectApi.readNativeProjectDiff).toHaveBeenCalledWith("proj_auth", undefined));
    expect(await screen.findByRole("region", { name: /review diff/i })).toHaveTextContent("+review action");
  });

  it("copies a review patch from the Review Queue row", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_copy_patch",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_review",
            project_id: "proj_auth",
            lifecycle: "review_ready",
            retry_count: 0,
            workspace_path: "/repo/.haneulchi/worktrees/run_1",
          },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_1",
          evidence_pack_id: "ev_run_1",
          task_id: "task_review",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.exportNativeProjectPatch.mockResolvedValueOnce({
      patch_id: "patch_auth_review",
      project_id: "proj_auth",
      body: "diff --git a/auth.ts b/auth.ts\n+export const ok = true;\n",
      status: "exported",
      checklist: ["copy patch for review handoff"],
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Copy patch review_ev_run_1" }));

    await waitFor(() => expect(projectApi.exportNativeProjectPatch).toHaveBeenCalledWith("proj_auth", undefined));
    expect(window.navigator.clipboard.writeText).toHaveBeenCalledWith("diff --git a/auth.ts b/auth.ts\n+export const ok = true;\n");
    expect(await screen.findByText("Copied patch patch_auth_review for review_ev_run_1")).toBeInTheDocument();
  });

  it("plans PR landing from a Review Queue row", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_pr_plan",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_review",
            project_id: "proj_auth",
            lifecycle: "review_ready",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_1",
          evidence_pack_id: "ev_run_1",
          task_id: "task_review",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.planNativeReviewPrLanding.mockResolvedValueOnce({
      review_id: "review_ev_run_1",
      evidence_pack_id: "ev_run_1",
      source_task_id: "task_review",
      source_run_id: "run_1",
      plan: {
        project_id: "proj_auth",
        provider: "github",
        title: "PR: review_ev_run_1",
        draft: true,
        checklist: ["link review review_ev_run_1 and evidence pack ev_run_1"],
        degraded_reason: "network push is intentionally not executed by local planner",
      },
    });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Plan PR review_ev_run_1" }));

    await waitFor(() =>
      expect(projectApi.planNativeReviewPrLanding).toHaveBeenCalledWith({
        reviewId: "review_ev_run_1",
        title: "PR: review_ev_run_1",
        draft: true,
      }),
    );
    expect(await screen.findByText("PR landing planned for review_ev_run_1")).toBeInTheDocument();
  });

  it("surfaces review PR landing planning failures", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_pr_plan_error",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_2",
          evidence_pack_id: "ev_run_2",
          task_id: "task_review",
          run_id: "run_missing",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    projectApi.planNativeReviewPrLanding.mockRejectedValueOnce(new Error("native review PR planner unavailable"));

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Plan PR review_ev_run_2" }));

    expect(await screen.findByText(/PR landing unavailable · native review PR planner unavailable/i)).toBeInTheDocument();
  });

  it("creates a native follow-up task from a review row", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_followup",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: {
        items: [
          {
            id: "run_1",
            task_id: "task_review",
            project_id: "proj_auth",
            lifecycle: "review_ready",
            retry_count: 0,
          },
        ],
        counts_by_lifecycle: { review_ready: 1 },
      },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_1",
          evidence_pack_id: "ev_run_1",
          task_id: "task_review",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    taskApi.createNativeReviewFollowUpTask.mockResolvedValueOnce({
      review_id: "review_ev_run_1",
      evidence_pack_id: "ev_run_1",
      source_task_id: "task_review",
      source_run_id: "run_1",
      task: {
        id: "task_followup_1",
        key: "LOCAL-12",
        project_id: "proj_auth",
        title: "Follow-up: review_ev_run_1",
        description: null,
        status: "inbox",
        priority: "high",
        assignee_type: null,
        assignee_id: null,
        cycle_id: null,
        module_id: null,
      },
      comment: {
        id: "comment_1",
        task_id: "task_followup_1",
        run_id: null,
        author_type: "system",
        author_id: "review_queue",
        body_md: "Created from review review_ev_run_1",
        parent_id: null,
      },
    });

    render(<App />);

    expect(await screen.findByText("review_ev_run_1 · pending · complete")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Create follow-up review_ev_run_1" }));

    await waitFor(() =>
      expect(taskApi.createNativeReviewFollowUpTask).toHaveBeenCalledWith({
        reviewId: "review_ev_run_1",
        title: "Follow-up: review_ev_run_1",
        priority: "high",
      }),
    );
    expect(await screen.findByText("Follow-up: review_ev_run_1")).toBeInTheDocument();
    expect(screen.getByText("Follow-up task created for review_ev_run_1")).toBeInTheDocument();
  });

  it("keeps review follow-up tasks local and visible when native task creation fails", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_followup_degraded",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_2",
          evidence_pack_id: "ev_run_2",
          task_id: "task_review",
          run_id: "run_missing",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    taskApi.createNativeReviewFollowUpTask.mockRejectedValueOnce(new Error("native review follow-up API unavailable"));

    render(<App />);

    expect(await screen.findByText("review_ev_run_2 · pending · complete")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Create follow-up review_ev_run_2" }));

    expect(await screen.findByText("Follow-up: review_ev_run_2", {}, { timeout: 5000 })).toBeInTheDocument();
    expect(
      await screen.findByText(/Follow-up saved locally; native sync unavailable/i, {}, { timeout: 5000 }),
    ).toBeInTheDocument();
  });

  it("reopens and blocks review queue tasks from decision actions", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_actions",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: {
        items: [
          {
            id: "task_review",
            title: "Review generated evidence",
            status: "review",
            priority: "high",
            project_id: "proj_local",
          },
        ],
        counts_by_status: { review: 1 },
      },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [
        {
          id: "review_ev_run_1",
          evidence_pack_id: "ev_run_1",
          task_id: "task_review",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_review: {
          id: "task_review",
          title: "Review generated evidence",
          status: "review",
          priority: "high",
          projectId: "proj_local",
        },
      },
    });
    commandBlockApi.recordNativeEvidenceReviewDecision
      .mockResolvedValueOnce({
        id: "ev_run_1",
        task_id: "task_review",
        run_id: "run_1",
        artifact_path: "artifacts/evidence/ev_run_1.json",
        completeness_state: "complete",
        body_json: { review_decision: { decision: "reopened" } },
      })
      .mockResolvedValueOnce({
        id: "ev_run_1",
        task_id: "task_review",
        run_id: "run_1",
        artifact_path: "artifacts/evidence/ev_run_1.json",
        completeness_state: "complete",
        body_json: { review_decision: { decision: "blocked" } },
      });

    render(<App />);

    expect(await screen.findByText("review_ev_run_1 · pending · complete")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reopen review_ev_run_1" }));

    await waitFor(() =>
      expect(commandBlockApi.recordNativeEvidenceReviewDecision).toHaveBeenCalledWith({
        evidencePackId: "ev_run_1",
        decision: "reopened",
        reviewerId: "human",
        bodyMd: "Reopened from Review Queue.",
      }),
    );
    expect(await screen.findByText("review_ev_run_1 · reopened · complete")).toBeInTheDocument();
    expect(screen.getByText("Ready 1")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Block review_ev_run_1" }));

    await waitFor(() =>
      expect(commandBlockApi.recordNativeEvidenceReviewDecision).toHaveBeenLastCalledWith({
        evidencePackId: "ev_run_1",
        decision: "blocked",
        reviewerId: "human",
        bodyMd: "Marked blocked from Review Queue.",
      }),
    );
    expect(await screen.findByText("review_ev_run_1 · blocked · complete")).toBeInTheDocument();
    expect(screen.getByText("Blocked 1")).toBeInTheDocument();
  });

  it("filters the review queue by review state and evidence completeness", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_review_filters",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [
        {
          id: "review_pending_complete",
          evidence_pack_id: "ev_pending_complete",
          task_id: "task_1",
          run_id: "run_1",
          state: "pending",
          completeness_state: "complete",
        },
        {
          id: "review_pending_incomplete",
          evidence_pack_id: "ev_pending_incomplete",
          task_id: "task_2",
          run_id: "run_2",
          state: "pending",
          completeness_state: "incomplete",
        },
        {
          id: "review_approved_complete",
          evidence_pack_id: "ev_approved_complete",
          task_id: "task_3",
          run_id: "run_3",
          state: "approved",
          completeness_state: "complete",
        },
        {
          id: "review_reopened_complete",
          evidence_pack_id: "ev_reopened_complete",
          task_id: "task_4",
          run_id: "run_4",
          state: "reopened",
          completeness_state: "complete",
        },
        {
          id: "review_blocked_complete",
          evidence_pack_id: "ev_blocked_complete",
          task_id: "task_5",
          run_id: "run_5",
          state: "blocked",
          completeness_state: "complete",
        },
      ],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("review_pending_complete · pending · complete")).toBeInTheDocument();
    expect(screen.getByText("review_pending_incomplete · pending · incomplete")).toBeInTheDocument();
    expect(screen.getByText("review_approved_complete · approved · complete")).toBeInTheDocument();
    expect(screen.getByText("review_reopened_complete · reopened · complete")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Review state filter"), {
      target: { value: "pending" },
    });

    expect(screen.getByText("review_pending_complete · pending · complete")).toBeInTheDocument();
    expect(screen.getByText("review_pending_incomplete · pending · incomplete")).toBeInTheDocument();
    expect(screen.queryByText("review_approved_complete · approved · complete")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Review completeness filter"), {
      target: { value: "complete" },
    });

    expect(screen.getByText("review_pending_complete · pending · complete")).toBeInTheDocument();
    expect(screen.queryByText("review_pending_incomplete · pending · incomplete")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Approve review_pending_complete" })).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Review state filter"), {
      target: { value: "blocked" },
    });

    expect(screen.getByText("review_blocked_complete · blocked · complete")).toBeInTheDocument();
    expect(screen.queryByText("review_pending_complete · pending · complete")).not.toBeInTheDocument();
  });

  it("summarizes terminal fidelity proof in the empty review queue", async () => {
    stateSnapshot.getStateSnapshot.mockReturnValueOnce(immediateResolved({
      ...defaultStateSnapshot(),
      snapshot_id: "snap_review_terminal_proof",
      terminal_fidelity: {
        last_run_id: "terminal_smoke_7",
        last_status: "passed",
        last_pass_count: 4,
        last_fail_count: 0,
        last_warning_count: 0,
        diagnostics: {
          status: "passed",
          case_count: 4,
          created_at: "2026-05-05T01:00:00Z",
        },
      },
    }));

    await renderApp();

    const reviewHeading = await screen.findByText("Review Queue");
    const reviewPanel = reviewHeading.closest("section");
    expect(reviewPanel).toHaveTextContent("Terminal proof terminal_smoke_7 · passed · 4 pass · 0 fail · 0 warning");
    expect(reviewPanel).not.toHaveTextContent("Terminal proof requires fidelity harness before HC10-013 closes");
  });

  it("renders Keychain secret storage state and clears plaintext after saving", async () => {
    secretApi.upsertNativeSecret.mockResolvedValueOnce({
      id: "secret_proj_local_OPENAI_API_KEY",
      project_id: "proj_local",
      name: "OPENAI_API_KEY",
      keychain_ref: "haneulchi:proj_local:OPENAI_API_KEY",
      redacted: true,
      created_at: "2026-05-02T01:00:00Z",
      updated_at: "2026-05-02T01:00:00Z",
    });
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_secret_storage",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      security: { keychain: "local", secret_count: 1 },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });

    render(<App />);

    expect(await screen.findByText("Keychain local · 1 secret")).toBeInTheDocument();
    expect(screen.getByText("Redaction active · 1 protected value")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Secret name"), { target: { value: "OPENAI_API_KEY" } });
    fireEvent.change(screen.getByLabelText("Secret value"), { target: { value: "sk-hidden-test-value" } });
    fireEvent.click(screen.getByRole("button", { name: "Save Keychain secret" }));

    await waitFor(() => {
      expect(secretApi.upsertNativeSecret).toHaveBeenCalledWith({
        projectId: "proj_local",
        name: "OPENAI_API_KEY",
        value: "sk-hidden-test-value",
      });
    });
    expect(await screen.findByText("Saved OPENAI_API_KEY · redacted")).toBeInTheDocument();
    expect(screen.queryByDisplayValue("sk-hidden-test-value")).not.toBeInTheDocument();
  });

  it("loads redacted Keychain secret metadata without exposing plaintext", async () => {
    secretApi.listNativeSecrets.mockResolvedValueOnce([
      {
        id: "secret_proj_local_OPENAI_API_KEY",
        project_id: "proj_local",
        name: "OPENAI_API_KEY",
        keychain_ref: "haneulchi:proj_local:OPENAI_API_KEY",
        redacted: true,
        created_at: "2026-05-02T01:00:00Z",
        updated_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const panel = await screen.findByRole("region", { name: /keychain secret storage/i });
    fireEvent.click(within(panel).getByRole("button", { name: /load keychain secrets/i }));

    await waitFor(() => expect(secretApi.listNativeSecrets).toHaveBeenCalledWith("proj_local"));
    expect(
      await within(panel).findByText("OPENAI_API_KEY · haneulchi:proj_local:OPENAI_API_KEY · redacted"),
    ).toBeInTheDocument();
    expect(panel).not.toHaveTextContent("sk-hidden-test-value");
  });

  it("clears stale Keychain secret results after failures and recovers on retry", async () => {
    secretApi.upsertNativeSecret
      .mockResolvedValueOnce({
        id: "secret_proj_local_OPENAI_API_KEY",
        project_id: "proj_local",
        name: "OPENAI_API_KEY",
        keychain_ref: "haneulchi:proj_local:OPENAI_API_KEY",
        redacted: true,
        created_at: "2026-05-02T01:00:00Z",
        updated_at: "2026-05-02T01:00:00Z",
      })
      .mockRejectedValueOnce(new Error("keychain locked"))
      .mockResolvedValueOnce({
        id: "secret_proj_local_ANTHROPIC_API_KEY",
        project_id: "proj_local",
        name: "ANTHROPIC_API_KEY",
        keychain_ref: "haneulchi:proj_local:ANTHROPIC_API_KEY",
        redacted: true,
        created_at: "2026-05-02T01:05:00Z",
        updated_at: "2026-05-02T01:05:00Z",
      });

    render(<App />);

    const panel = await screen.findByRole("region", { name: /keychain secret storage/i });
    fireEvent.change(within(panel).getByLabelText("Secret name"), { target: { value: "OPENAI_API_KEY" } });
    fireEvent.change(within(panel).getByLabelText("Secret value"), { target: { value: "sk-openai-hidden" } });
    fireEvent.click(within(panel).getByRole("button", { name: /save keychain secret/i }));
    expect(await within(panel).findByText("Saved OPENAI_API_KEY · redacted")).toBeInTheDocument();

    fireEvent.change(within(panel).getByLabelText("Secret name"), { target: { value: "ANTHROPIC_API_KEY" } });
    fireEvent.change(within(panel).getByLabelText("Secret value"), { target: { value: "sk-anthropic-hidden" } });
    fireEvent.click(within(panel).getByRole("button", { name: /save keychain secret/i }));
    await waitFor(() => expect(panel).toHaveTextContent("Keychain secret unavailable · keychain locked"));
    expect(within(panel).queryByText("Saved OPENAI_API_KEY · redacted")).not.toBeInTheDocument();
    expect(within(panel).getByDisplayValue("sk-anthropic-hidden")).toBeInTheDocument();

    fireEvent.click(within(panel).getByRole("button", { name: /save keychain secret/i }));
    expect(await within(panel).findByText("Saved ANTHROPIC_API_KEY · redacted")).toBeInTheDocument();
    expect(panel).not.toHaveTextContent("Keychain secret unavailable · keychain locked");
    expect(within(panel).queryByDisplayValue("sk-anthropic-hidden")).not.toBeInTheDocument();
  });

  it("renders security diagnostics from the state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_security_diagnostics",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      security: {
        keychain: "local",
        secret_count: 2,
        redaction: { status: "active", protected_secret_count: 2 },
        permission_audit: {
          recent_count: 4,
          allowed_count: 2,
          approval_required_count: 1,
          forbidden_count: 1,
          latest_decision: "forbidden",
          latest_action_kind: "network",
        },
        diagnostics: {
          status: "warning",
          pending_policy_approvals: 1,
          checks: [
            { id: "keychain", label: "Keychain", status: "ok", detail: "local secret storage available" },
            { id: "redaction", label: "Secret redaction", status: "ok", detail: "2 protected values" },
            { id: "policy-pack", label: "Policy pack", status: "ok", detail: "Ask before write · ask-before-write" },
            { id: "permission-audit", label: "Permission audit", status: "warning", detail: "1 forbidden decision in recent audit" },
            { id: "policy-approvals", label: "Policy approvals", status: "warning", detail: "1 pending approval" },
            { id: "control-plane", label: "Control plane", status: "warning", detail: "api degraded · db ok" },
          ],
        },
        policy_pack: {
          name: "Ask before write",
          sandbox_mode: "ask-before-write",
          network: "ask",
          file_write: "ask",
          tools: "ask",
          approval_required_count: 1,
          forbidden_count: 0,
        },
      },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "degraded" },
    });

    render(<App />);

    const panel = await screen.findByRole("region", { name: "Security diagnostics" });
    expect(panel).toHaveTextContent("Security diagnostics · warning");
    expect(panel).toHaveTextContent("Secret redaction · ok");
    expect(panel).toHaveTextContent("2 protected values");
    expect(panel).toHaveTextContent("Policy approvals · warning");
    expect(panel).toHaveTextContent("1 pending approval");
    expect(panel).toHaveTextContent("Permission audit · warning");
    expect(panel).toHaveTextContent("1 forbidden decision in recent audit");
    expect(panel).toHaveTextContent("Audit 4 recent · 1 blocked · latest network forbidden");
    expect(panel).toHaveTextContent("Control plane · warning");
  });

  it("filters advanced permission audit events from security diagnostics", async () => {
    policyApi.listNativePermissionAudits.mockResolvedValueOnce([
      {
        id: "permission_audit_2",
        project_id: "proj_local",
        task_id: "task_policy",
        run_id: "run_policy",
        policy_pack_id: "policy_pack_1",
        action_kind: "network",
        command: "curl https://example.com",
        decision: "forbidden",
        reason: "network profile blocks remote endpoint",
        requested_by: "agent_codex",
        created_at: "2026-05-02T01:00:00Z",
      },
    ]);

    render(<App />);

    const panel = await screen.findByRole("region", { name: "Security diagnostics" });
    fireEvent.change(within(panel).getByLabelText("Permission audit decision"), { target: { value: "forbidden" } });
    fireEvent.change(within(panel).getByLabelText("Permission audit action"), { target: { value: "network" } });
    fireEvent.change(within(panel).getByLabelText("Permission audit run"), { target: { value: "run_policy" } });
    fireEvent.change(within(panel).getByLabelText("Permission audit task"), { target: { value: "task_policy" } });
    fireEvent.click(within(panel).getByRole("button", { name: "Load permission audit" }));

    await waitFor(() => expect(policyApi.listNativePermissionAudits).toHaveBeenCalledWith("proj_local", {
      decision: "forbidden",
      actionKind: "network",
      runId: "run_policy",
      taskId: "task_policy",
    }));
    await waitFor(() => expect(panel).toHaveTextContent("permission_audit_2 · network · forbidden"));
    expect(panel).toHaveTextContent("run run_policy · task task_policy");
    expect(panel).toHaveTextContent("network profile blocks remote endpoint");
  });

  it("loads native SQLite task state before falling back to localStorage", async () => {
    window.localStorage.setItem(
      "haneulchi:task-state:proj_local",
      JSON.stringify({
        tasks: {
          stale_local: {
            id: "stale_local",
            title: "Stale local task",
            status: "done",
            priority: "high",
            projectId: "proj_local",
          },
        },
      }),
    );
    taskApi.loadNativeTaskState.mockResolvedValueOnce({
      tasks: {
        task_native: {
          id: "task_native",
          title: "Native SQLite task",
          status: "blocked",
          priority: "urgent",
          projectId: "proj_local",
        },
      },
    });

    render(<App />);

    await waitFor(() => expect(taskApi.loadNativeTaskState).toHaveBeenCalledWith("proj_local"));
    expect(screen.getByText("Native SQLite task")).toBeInTheDocument();
    expect(screen.getByText("Blocked 1")).toBeInTheDocument();
    expect(screen.getByText("Done 0")).toBeInTheDocument();
    expect(screen.queryByText("Stale local task")).not.toBeInTheDocument();
    expect(screen.getByText(/State tasks 1 tracked/i)).toBeInTheDocument();
  });

  it("loads the task board for the active state snapshot project", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_active_project_tasks",
      generated_at: "2026-05-02T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    taskApi.loadNativeTaskState.mockImplementation(async (projectId: string) => ({
      tasks: projectId === "proj_auth"
        ? {
            task_auth_project: {
              id: "task_auth_project",
              title: "Auth project task",
              status: "ready",
              priority: "high",
              projectId: "proj_auth",
            },
          }
        : {},
    }));

    render(<App />);

    await waitFor(() => expect(taskApi.loadNativeTaskState).toHaveBeenCalledWith("proj_auth"));
    expect(await screen.findByText("Auth project task")).toBeInTheDocument();
    expect(screen.getByText("Ready 1")).toBeInTheDocument();
  });

  it("surfaces native task state load failures while keeping the local task cache", async () => {
    window.localStorage.setItem(
      "haneulchi:task-state:proj_local",
      JSON.stringify({
        tasks: {
          task_cached: {
            id: "task_cached",
            title: "Cached local task",
            status: "running",
            priority: "high",
            projectId: "proj_local",
          },
        },
      }),
    );
    taskApi.loadNativeTaskState.mockRejectedValueOnce(new Error("task db offline"));

    render(<App />);

    expect(screen.getByText("Cached local task")).toBeInTheDocument();
    await waitFor(() => expect(taskApi.loadNativeTaskState).toHaveBeenCalledWith("proj_local"));
    expect(await screen.findByText("Task board loaded from local cache · task db offline")).toBeInTheDocument();
    expect(screen.getByText("Running 1")).toBeInTheDocument();
  });

  it("quick-creates inbox tasks and persists the board state", async () => {
    render(<App />);

    fireEvent.change(screen.getByLabelText("Quick task title"), {
      target: { value: "Implement quick task create" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create task/i }));

    expect(screen.getByText("Inbox 2")).toBeInTheDocument();
    expect(screen.getByText(/State tasks 7 tracked/i)).toBeInTheDocument();
    await waitFor(() =>
      expect(JSON.parse(window.localStorage.getItem("haneulchi:task-state:proj_local") ?? "{}").tasks.task_7).toMatchObject({
        title: "Implement quick task create",
        status: "inbox",
      }),
    );
  });

  it("quick-creates tasks through the native SQLite task command when available", async () => {
    taskApi.createNativeTask.mockResolvedValueOnce({
      id: "task_native_1",
      key: "LOCAL-1",
      project_id: "proj_local",
      title: "Native write-through task",
      description: null,
      status: "inbox",
      priority: "medium",
      assignee_type: null,
      assignee_id: null,
      cycle_id: null,
      module_id: null,
    });
    render(<App />);

    fireEvent.change(screen.getByLabelText("Quick task title"), {
      target: { value: "Native write-through task" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create task/i }));

    await waitFor(() =>
      expect(taskApi.createNativeTask).toHaveBeenCalledWith({
        projectId: "proj_local",
        title: "Native write-through task",
      }),
    );
    expect(screen.getByText("Native write-through task")).toBeInTheDocument();
    expect(screen.getByText(/State tasks 7 tracked/i)).toBeInTheDocument();
  });

  it("focuses quick task create from the task board keyboard shortcut", async () => {
    render(<App />);

    fireEvent.keyDown(window, { key: "B", metaKey: true, shiftKey: true });

    expect(screen.getByLabelText("Quick task title")).toHaveFocus();
  });

  it("surfaces quick task native persistence failures while keeping the local task", async () => {
    taskApi.createNativeTask.mockRejectedValueOnce(new Error("database offline"));
    render(<App />);

    fireEvent.change(screen.getByLabelText("Quick task title"), {
      target: { value: "Offline quick task" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create task/i }));

    expect(screen.getByText("Offline quick task")).toBeInTheDocument();
    expect(await screen.findByText("Quick task saved locally · database offline")).toBeInTheDocument();
  });

  it("advances an inbox task to ready and persists the transition", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /advance capture project setup requirements/i }));

    expect(screen.getByText("Inbox 0")).toBeInTheDocument();
    expect(screen.getByText("Ready 2")).toBeInTheDocument();
    await waitFor(() =>
      expect(JSON.parse(window.localStorage.getItem("haneulchi:task-state:proj_local") ?? "{}").tasks.task_inbox).toMatchObject({
        status: "ready",
      }),
    );
  });

  it("moves tasks through the native SQLite task command when available", async () => {
    taskApi.moveNativeTask.mockResolvedValueOnce({
      id: "task_inbox",
      key: "LOCAL-1",
      project_id: "proj_local",
      title: "Capture project setup requirements",
      description: null,
      status: "ready",
      priority: "medium",
      assignee_type: null,
      assignee_id: null,
      cycle_id: null,
      module_id: null,
    });
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /advance capture project setup requirements/i }));

    await waitFor(() => expect(taskApi.moveNativeTask).toHaveBeenCalledWith("task_inbox", "ready"));
    expect(screen.getByText("Inbox 0")).toBeInTheDocument();
    expect(screen.getByText("Ready 2")).toBeInTheDocument();
  });

  it("surfaces task status persistence failures after local advancement", async () => {
    taskApi.moveNativeTask.mockRejectedValueOnce(new Error("task store offline"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /advance capture project setup requirements/i }));

    expect(screen.getByText("Inbox 0")).toBeInTheDocument();
    expect(screen.getByText("Ready 2")).toBeInTheDocument();
    expect(await screen.findByText("Task status saved locally · task store offline")).toBeInTheDocument();
  });

  it("filters the task mini list by query", async () => {
    render(<App />);

    fireEvent.change(screen.getByLabelText("Filter tasks"), {
      target: { value: "DMG" },
    });

    expect(screen.getByText("Resolve DMG packaging gate")).toBeInTheDocument();
    expect(screen.queryByText("Capture project setup requirements")).not.toBeInTheDocument();
    expect(screen.getByText("Task filter 1 match")).toBeInTheDocument();
  });

  it("filters the task mini list by cycle and module metadata", async () => {
    render(<App />);
    const taskBoard = screen.getByRole("region", { name: "Task Board" });

    fireEvent.change(within(taskBoard).getByLabelText("Filter tasks"), {
      target: { value: "Sprint 5" },
    });

    expect(within(taskBoard).getByText("Wire state snapshot into CLI boundary")).toBeInTheDocument();
    expect(within(taskBoard).queryByText("Capture project setup requirements")).not.toBeInTheDocument();
    expect(within(taskBoard).getByText("Task filter 1 match")).toBeInTheDocument();

    fireEvent.change(within(taskBoard).getByLabelText("Filter tasks"), {
      target: { value: "Control API" },
    });

    expect(within(taskBoard).getByText("Wire state snapshot into CLI boundary")).toBeInTheDocument();
    expect(within(taskBoard).queryByText("Resolve DMG packaging gate")).not.toBeInTheDocument();
    expect(within(taskBoard).getByText("Task filter 1 match")).toBeInTheDocument();
  });

  it("exposes task board and agent directory as separate rail regions", async () => {
    render(<App />);

    expect(screen.getByRole("region", { name: "Task Board" })).toHaveTextContent("Inbox 1");
    expect(screen.getByRole("region", { name: "Agent Directory" })).toHaveTextContent("Scan agents");
  });

  it("shows status priority cycle and module metadata in task list rows", async () => {
    render(<App />);

    const taskBoard = screen.getByRole("region", { name: "Task Board" });

    expect(taskBoard).toHaveTextContent("Ready · high · Sprint 5 · Control API");
    expect(taskBoard).toHaveTextContent("Running · urgent · agent_codex");
  });

  it("groups task rows under the MVP board status lanes", async () => {
    render(<App />);

    const taskBoard = screen.getByRole("region", { name: "Task Board" });
    const inboxLane = within(taskBoard).getByRole("group", { name: "Inbox lane" });
    const readyLane = within(taskBoard).getByRole("group", { name: "Ready lane" });
    const reviewLane = within(taskBoard).getByRole("group", { name: "Review lane" });

    expect(inboxLane).toHaveTextContent("Capture project setup requirements");
    expect(readyLane).toHaveTextContent("Wire state snapshot into CLI boundary");
    expect(reviewLane).toHaveTextContent("Review evidence pack workflow");
    expect(readyLane).not.toHaveTextContent("Capture project setup requirements");
  });

  it("saves and reapplies local task filter views", async () => {
    render(<App />);

    fireEvent.change(screen.getByLabelText("Filter tasks"), {
      target: { value: "blocked" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task view/i }));

    expect(screen.getByRole("button", { name: /apply task view blocked/i })).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Filter tasks"), {
      target: { value: "" },
    });
    expect(screen.getByText("Capture project setup requirements")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /apply task view blocked/i }));

    expect(screen.getByText("Resolve DMG packaging gate")).toBeInTheDocument();
    expect(screen.queryByText("Capture project setup requirements")).not.toBeInTheDocument();
    expect(JSON.parse(window.localStorage.getItem("haneulchi:task-views:proj_local") ?? "[]")).toContainEqual({
      id: "view_blocked",
      label: "blocked",
      query: "blocked",
    });
  });

  it("removes saved local task filter views", async () => {
    window.localStorage.setItem(
      "haneulchi:task-views:proj_local",
      JSON.stringify([{ id: "view_blocked", label: "blocked", query: "blocked" }]),
    );
    render(<App />);

    expect(screen.getByRole("button", { name: /apply task view blocked/i })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /delete task view blocked/i }));

    expect(screen.queryByRole("button", { name: /apply task view blocked/i })).not.toBeInTheDocument();
    expect(JSON.parse(window.localStorage.getItem("haneulchi:task-views:proj_local") ?? "[]")).toEqual([]);
  });

  it("surfaces saved local task view persistence failures", async () => {
    render(<App />);

    fireEvent.change(screen.getByLabelText("Filter tasks"), {
      target: { value: "blocked" },
    });
    vi.spyOn(window.localStorage, "setItem").mockImplementationOnce(() => {
      throw new Error("quota exceeded");
    });
    fireEvent.click(screen.getByRole("button", { name: /save task view/i }));

    expect(await screen.findByText("Task view saved for this session · quota exceeded")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /apply task view blocked/i })).toBeInTheDocument();
  });

  it("opens a task drawer overview from the mini list", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));

    expect(screen.getByText("Task Drawer")).toBeInTheDocument();
    expect(screen.getByText("Review evidence pack workflow")).toBeInTheDocument();
    expect(screen.getByText(/Status review · Priority high/i)).toBeInTheDocument();
    expect(screen.getByText("Validate command block attachments before marking review-ready.")).toBeInTheDocument();
    expect(screen.getByText("Workpad")).toBeInTheDocument();
    expect(screen.getAllByText(/Confirm attached evidence references the latest terminal proof/i).length).toBeGreaterThan(0);
    expect(screen.getByText("Comments 2")).toBeInTheDocument();
    expect(screen.getByText("human")).toBeInTheDocument();
    expect(screen.getByText("Needs release gate evidence.")).toBeInTheDocument();
    expect(screen.getByText("agent_codex")).toBeInTheDocument();
    expect(screen.getByText("Command blocks attached.")).toBeInTheDocument();
  });

  it("updates drawer workpad and adds comments with persisted task state", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("Task workpad"), {
      target: { value: "Confirm screenshots and terminal proof are linked." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task workpad/i }));
    fireEvent.change(screen.getByLabelText("New task comment"), {
      target: { value: "  Screenshot evidence added.  " },
    });
    fireEvent.click(screen.getByRole("button", { name: /add task comment/i }));

    expect(screen.getByText("Comments 3")).toBeInTheDocument();
    expect(screen.getByLabelText("New task comment")).toHaveValue("");
    await waitFor(() => {
      const stored = JSON.parse(window.localStorage.getItem("haneulchi:task-state:proj_local") ?? "{}");
      expect(stored.tasks.task_review.workpad).toBe("Confirm screenshots and terminal proof are linked.");
      expect(stored.tasks.task_review.comments).toContainEqual({
        id: "comment_3",
        author: "human",
        body: "Screenshot evidence added.",
      });
    });
  });

  it("adds and completes task drawer subtasks with persisted task state", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("New task subtask"), {
      target: { value: "  Confirm screenshots  " },
    });
    fireEvent.click(screen.getByRole("button", { name: /add task subtask/i }));

    expect(screen.getByText("Subtasks 1 open / 1 total")).toBeInTheDocument();
    expect(screen.getByText("Confirm screenshots")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /mark confirm screenshots done/i }));

    expect(screen.getByText("Subtasks 0 open / 1 total")).toBeInTheDocument();
    await waitFor(() => {
      const stored = JSON.parse(window.localStorage.getItem("haneulchi:task-state:proj_local") ?? "{}");
      expect(stored.tasks.task_review.subtasks).toContainEqual({
        id: "subtask_1",
        title: "Confirm screenshots",
        status: "done",
      });
    });
  });

  it("adds task drawer subtasks through the native SQLite task command when available", async () => {
    taskApi.addNativeTaskSubtask.mockResolvedValueOnce({
      id: "subtask_native_1",
      task_id: "task_review",
      title: "Native subtask.",
      status: "open",
      order_index: 0,
    });
    taskApi.updateNativeTaskSubtaskStatus.mockResolvedValueOnce({
      id: "subtask_native_1",
      task_id: "task_review",
      title: "Native subtask.",
      status: "done",
      order_index: 0,
    });
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("New task subtask"), {
      target: { value: "Native subtask." },
    });
    fireEvent.click(screen.getByRole("button", { name: /add task subtask/i }));

    await waitFor(() =>
      expect(taskApi.addNativeTaskSubtask).toHaveBeenCalledWith({
        taskId: "task_review",
        title: "Native subtask.",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: /mark native subtask\. done/i }));
    await waitFor(() =>
      expect(taskApi.updateNativeTaskSubtaskStatus).toHaveBeenCalledWith({
        taskId: "task_review",
        subtaskId: "subtask_native_1",
        status: "done",
      }),
    );
    expect(screen.getByText("Subtasks 0 open / 1 total")).toBeInTheDocument();
  });

  it("adds task drawer comments through the native SQLite task command when available", async () => {
    taskApi.addNativeTaskComment.mockResolvedValueOnce({
      id: "comment_native_3",
      task_id: "task_review",
      run_id: null,
      author_type: "human",
      author_id: "local_user",
      body_md: "Native drawer comment.",
      parent_id: null,
    });
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("New task comment"), {
      target: { value: "Native drawer comment." },
    });
    fireEvent.click(screen.getByRole("button", { name: /add task comment/i }));

    await waitFor(() =>
      expect(taskApi.addNativeTaskComment).toHaveBeenCalledWith({
        taskId: "task_review",
        body: "Native drawer comment.",
      }),
    );
    expect(screen.getByText("Comments 3")).toBeInTheDocument();
  });

  it("surfaces task comment persistence failures in the drawer", async () => {
    taskApi.addNativeTaskComment.mockRejectedValueOnce(new Error("database offline"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("New task comment"), {
      target: { value: "Native drawer comment." },
    });
    fireEvent.click(screen.getByRole("button", { name: /add task comment/i }));

    expect(await screen.findByText("Task comment save failed · database offline")).toBeInTheDocument();
  });

  it("loads persisted task drawer comments through the native SQLite task command when available", async () => {
    taskApi.listNativeTaskComments.mockResolvedValueOnce([
      {
        id: "comment_native_1",
        task_id: "task_review",
        run_id: null,
        author_type: "human",
        author_id: "local_user",
        body_md: "Persisted drawer comment.",
        parent_id: null,
      },
    ]);
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));

    await waitFor(() => expect(taskApi.listNativeTaskComments).toHaveBeenCalledWith("task_review"));
    expect(screen.getByText("Persisted drawer comment.")).toBeInTheDocument();
    expect(screen.getByText("Comments 1")).toBeInTheDocument();
  });

  it("surfaces task comment load failures in the drawer", async () => {
    taskApi.listNativeTaskComments.mockRejectedValueOnce(new Error("comment store offline"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));

    await waitFor(() => expect(taskApi.listNativeTaskComments).toHaveBeenCalledWith("task_review"));
    expect(await screen.findByText("Task comments unavailable · comment store offline")).toBeInTheDocument();
  });

  it("saves task drawer workpads through the native SQLite workpad command when available", async () => {
    taskApi.saveNativeTaskWorkpad.mockResolvedValueOnce({
      id: "workpad_task_review",
      task_id: "task_review",
      artifact_path: "artifacts/workpads/task_review.md",
      title: "Task workpad",
      body_md: "Native workpad evidence.",
    });
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("Task workpad"), {
      target: { value: "Native workpad evidence." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task workpad/i }));

    await waitFor(() =>
      expect(taskApi.saveNativeTaskWorkpad).toHaveBeenCalledWith({
        taskId: "task_review",
        body: "Native workpad evidence.",
      }),
    );
    expect(screen.getByDisplayValue("Native workpad evidence.")).toBeInTheDocument();
  });

  it("surfaces task workpad persistence failures in the drawer", async () => {
    taskApi.saveNativeTaskWorkpad.mockRejectedValueOnce(new Error("disk full"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    fireEvent.change(screen.getByLabelText("Task workpad"), {
      target: { value: "Native workpad evidence." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task workpad/i }));

    expect(await screen.findByText("Task workpad save failed · disk full")).toBeInTheDocument();
  });

  it("renders a markdown preview for the task workpad editor", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);

    try {
      await renderApp();

      await act(async () => {
        fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
      });
      const workpad = await screen.findByLabelText("Task workpad");
      await act(async () => {
        fireEvent.change(workpad, {
          target: { value: "## Evidence\n- Attach terminal proof\n- Link screenshots" },
        });
        fireEvent.click(screen.getByRole("button", { name: /preview task workpad/i }));
      });

      const preview = screen.getByRole("region", { name: /task workpad markdown preview/i });
      expect(preview).toHaveTextContent("Evidence");
      expect(preview).toHaveTextContent("Attach terminal proof");
      expect(preview.querySelector("h2")).toHaveTextContent("Evidence");
      expect(preview.querySelectorAll("li")).toHaveLength(2);
      await Promise.resolve();
      await Promise.resolve();
      expect(
        consoleError.mock.calls.filter(([message]) => String(message).includes("not wrapped in act")),
      ).toHaveLength(0);
    } finally {
      consoleError.mockRestore();
    }
  });

  it("edits task workpads through a CodeMirror markdown editor surface", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task review evidence pack workflow/i }));
    const editor = screen.getByRole("region", { name: /codemirror markdown workpad/i });
    expect(editor).toHaveAttribute("data-language", "markdown");

    fireEvent.change(screen.getByLabelText("Task workpad"), {
      target: { value: "## CodeMirror notes\n- Persist markdown evidence" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task workpad/i }));

    await waitFor(() => {
      const stored = JSON.parse(window.localStorage.getItem("haneulchi:task-state:proj_local") ?? "{}");
      expect(stored.tasks.task_review.workpad).toBe("## CodeMirror notes\n- Persist markdown evidence");
    });
  });

  it("updates drawer cycle and module metadata with persisted task state", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task wire state snapshot into cli boundary/i }));
    fireEvent.change(screen.getByLabelText("Task cycle"), {
      target: { value: "Sprint 5" },
    });
    fireEvent.change(screen.getByLabelText("Task module"), {
      target: { value: "Control API" },
    });
    fireEvent.change(screen.getByLabelText("Task labels"), {
      target: { value: "release, evidence" },
    });
    fireEvent.change(screen.getByLabelText("Task due date"), {
      target: { value: "2026-05-15" },
    });
    fireEvent.change(screen.getByLabelText("Task estimate"), {
      target: { value: "3 pts" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task planning properties/i }));

    expect(screen.getByText(/Cycle Sprint 5 · Module Control API/i)).toBeInTheDocument();
    expect(screen.getByText(/Labels release, evidence/i)).toBeInTheDocument();
    expect(screen.getByText(/Due 2026-05-15/i)).toBeInTheDocument();
    expect(screen.getByText(/Estimate 3 pts/i)).toBeInTheDocument();
    await waitFor(() => {
      const stored = JSON.parse(window.localStorage.getItem("haneulchi:task-state:proj_local") ?? "{}");
      expect(stored.tasks.task_ready).toMatchObject({
        cycle: "Sprint 5",
        module: "Control API",
        labels: ["release", "evidence"],
        dueDate: "2026-05-15",
        estimate: "3 pts",
      });
    });
  });

  it("saves task drawer planning properties through the native SQLite task command when available", async () => {
    taskApi.updateNativeTaskPlanning.mockResolvedValueOnce({
      id: "task_ready",
      key: "HC-READY",
      project_id: "proj_local",
      title: "Wire state snapshot into CLI boundary",
      description: "Expose native health and board counts.",
      status: "ready",
      priority: "high",
      assignee_type: "agent",
      assignee_id: "agent_codex",
      cycle_id: "Sprint 5",
      module_id: "Control API",
      initiative_id: "init_auth",
      due_at: "2026-05-15",
      estimate: "3 pts",
      labels: ["release", "evidence"],
      workpad_md: null,
    });
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task wire state snapshot into cli boundary/i }));
    fireEvent.change(screen.getByLabelText("Task cycle"), {
      target: { value: "Sprint 5" },
    });
    fireEvent.change(screen.getByLabelText("Task module"), {
      target: { value: "Control API" },
    });
    fireEvent.change(screen.getByLabelText("Task initiative"), {
      target: { value: "init_auth" },
    });
    fireEvent.change(screen.getByLabelText("Task labels"), {
      target: { value: "release, evidence" },
    });
    fireEvent.change(screen.getByLabelText("Task due date"), {
      target: { value: "2026-05-15" },
    });
    fireEvent.change(screen.getByLabelText("Task estimate"), {
      target: { value: "3 pts" },
    });
    fireEvent.change(screen.getByLabelText("Task assignee"), {
      target: { value: "agent_codex" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task planning properties/i }));

    await waitFor(() =>
      expect(taskApi.updateNativeTaskPlanning).toHaveBeenCalledWith({
        taskId: "task_ready",
        cycle: "Sprint 5",
        module: "Control API",
        initiative: "init_auth",
        dueDate: "2026-05-15",
        estimate: "3 pts",
        labels: ["release", "evidence"],
        assignee: "agent_codex",
      }),
    );
    expect(screen.getByText(/Cycle Sprint 5 · Module Control API/i)).toBeInTheDocument();
    expect(screen.getByText(/Initiative init_auth/i)).toBeInTheDocument();
    expect(screen.getByText(/Labels release, evidence/i)).toBeInTheDocument();
    expect(screen.getByText(/Due 2026-05-15/i)).toBeInTheDocument();
    expect(screen.getByText(/Estimate 3 pts/i)).toBeInTheDocument();
    expect(screen.getByText(/Assignee agent_codex/i)).toBeInTheDocument();
  });

  it("creates lightweight task cycle and module properties from the drawer", async () => {
    taskApi.listNativeTaskCycles.mockReturnValueOnce(immediateResolved([
      {
        id: "cycle_sprint_12",
        project_id: "proj_local",
        name: "Sprint 12",
        starts_at: null,
        ends_at: null,
        status: "active",
      },
    ]));
    taskApi.listNativeTaskModules.mockReturnValueOnce(immediateResolved([
      {
        id: "module_control_api",
        project_id: "proj_local",
        name: "Control API",
        description: "Native boundary",
        status: "active",
      },
    ]));
    taskApi.createNativeTaskCycle.mockReturnValueOnce(immediateResolved({
      id: "cycle_sprint_13",
      project_id: "proj_local",
      name: "Sprint 13",
      starts_at: null,
      ends_at: null,
      status: "planned",
    }));
    taskApi.createNativeTaskModule.mockReturnValueOnce(immediateResolved({
      id: "module_release",
      project_id: "proj_local",
      name: "Release",
      description: null,
      status: "active",
    }));
    taskApi.updateNativeTaskPlanning.mockResolvedValueOnce({
      id: "task_ready",
      key: "HC-READY",
      project_id: "proj_local",
      title: "Wire state snapshot into CLI boundary",
      description: null,
      status: "ready",
      priority: "high",
      assignee_type: null,
      assignee_id: null,
      cycle_id: "Sprint 13",
      module_id: "Release",
      initiative_id: null,
      due_at: null,
      estimate: null,
      labels: [],
      workpad_md: null,
    });

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task wire state snapshot into cli boundary/i }));
    await waitFor(() => expect(taskApi.listNativeTaskCycles).toHaveBeenCalledWith("proj_local"));
    await waitFor(() => expect(taskApi.listNativeTaskModules).toHaveBeenCalledWith("proj_local"));

    fireEvent.change(screen.getByLabelText("Task cycle"), { target: { value: "Sprint 13" } });
    fireEvent.click(screen.getByRole("button", { name: /create task cycle/i }));
    await waitFor(() =>
      expect(taskApi.createNativeTaskCycle).toHaveBeenCalledWith({
        projectId: "proj_local",
        name: "Sprint 13",
      }),
    );
    expect(await screen.findByText("Created cycle Sprint 13 · planned")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Task module"), { target: { value: "Release" } });
    fireEvent.click(screen.getByRole("button", { name: /create task module/i }));
    await waitFor(() =>
      expect(taskApi.createNativeTaskModule).toHaveBeenCalledWith({
        projectId: "proj_local",
        name: "Release",
      }),
    );
    expect(await screen.findByText("Created module Release · active")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /save task planning properties/i }));
    await waitFor(() =>
      expect(taskApi.updateNativeTaskPlanning).toHaveBeenCalledWith(expect.objectContaining({
        taskId: "task_ready",
        cycle: "Sprint 13",
        module: "Release",
      })),
    );
  });

  it("surfaces task planning persistence failures in the drawer", async () => {
    taskApi.updateNativeTaskPlanning.mockRejectedValueOnce(new Error("database offline"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /open task wire state snapshot into cli boundary/i }));
    fireEvent.change(screen.getByLabelText("Task cycle"), {
      target: { value: "Sprint 5" },
    });
    fireEvent.change(screen.getByLabelText("Task module"), {
      target: { value: "Control API" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save task planning properties/i }));

    expect(await screen.findByText("Task planning unavailable · database offline")).toBeInTheDocument();
  });

  it("copies command blocks and jumps the owning terminal pane into focus", async () => {
    await renderApp();

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    });
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));
    });
    await waitFor(() => expect(terminalPty.writeTerminalPtyInput).toHaveBeenCalledWith("pty_1", "npm test\r"));

    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "PASS App command block workflow\n",
      });
    });

    expect(await screen.findByText(/npm test .* seq 1-1/i)).toBeInTheDocument();

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /copy npm test/i }));
    });
    expect(window.navigator.clipboard.writeText).toHaveBeenCalledWith(expect.stringContaining("$ npm test"));
    expect(window.navigator.clipboard.writeText).toHaveBeenCalledWith(expect.stringContaining("PASS App command block workflow"));

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /jump to npm test/i }));
    });
    expect(screen.getByLabelText(/terminal pane 1\. haneulchi/i)).toHaveAttribute("data-highlighted", "true");
    expect(screen.getByText(/State command blocks 1 unread/i)).toBeInTheDocument();
  });

  it("hydrates persisted command block summaries from the native state snapshot", async () => {
    stateSnapshot.getStateSnapshot.mockReturnValue(immediateResolved({
      ...defaultStateSnapshot(),
      command_blocks: {
        recent: [
          {
            id: "cmdblk_7",
            session_id: "session_native",
            command: "cargo test",
            status: "completed",
            seq_start: 12,
            seq_end: 18,
          },
        ],
        unread_count: 1,
      },
    }));

    await renderApp();

    expect(await screen.findByRole("button", { name: /command blocks 1/i })).toBeInTheDocument();
    expect(screen.getByText(/cargo test .* completed .* persisted .* seq 12-18/i)).toBeInTheDocument();
    expect(screen.getByText(/State command blocks 1 unread/i)).toBeInTheDocument();
  });

  it("loads persisted command block search results from the native command history", async () => {
    commandBlockApi.searchNativeCommandBlocks.mockResolvedValueOnce([
      {
        id: "cmdblk_7",
        session_id: "session_native",
        task_id: null,
        run_id: null,
        seq_start: 12,
        seq_end: 18,
        command: "cargo test",
        cwd: "/repo/src-tauri",
        branch: "main",
        exit_code: 0,
        duration_ms: 1200,
        summary: "PASS cargo tests",
      },
    ]);

    await renderApp();

    fireEvent.change(screen.getByLabelText("Search command blocks"), { target: { value: "cargo" } });

    await waitFor(() => expect(commandBlockApi.searchNativeCommandBlocks).toHaveBeenCalledWith("cargo", 50));
    expect(await screen.findByText(/cargo test .* completed .* main .* seq 12-18/i)).toBeInTheDocument();
  });

  it("closes live PTY sessions from the terminal pane", async () => {
    await renderApp();

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    });
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /close 1\. haneulchi/i }));
    });

    await waitFor(() => expect(terminalPty.closeTerminalPtySession).toHaveBeenCalledWith("pty_1"));
    expect(screen.queryByLabelText(/terminal pane 1\. haneulchi/i)).not.toBeInTheDocument();
  });

  it("explains and exports a command block bundle from the command block log", async () => {
    commandBlockApi.explainNativeCommandBlock.mockResolvedValueOnce({
      id: "explain_cmdblk_1",
      command_block_id: "cmdblk_1",
      command: "npm test",
      summary: "AI explanation draft for npm test: unknown in ~/develop/applications/haneulchi on main",
      evidence: [
        "ai route openai/gpt-5.4",
        "agent agent_codex",
        "context: npm test unknown in ~/develop/applications/haneulchi on main",
        "sequence 1-1",
        "status unknown",
        "output: PASS command block export workflow",
      ],
      provider: "openai",
      model: "gpt-5.4",
      agent_profile_id: "agent_codex",
      prompt: "Explain this command block for review evidence.",
      diagnostics: ["No external AI call has been made; draft is ready for the selected agent."],
    });
    commandBlockApi.exportNativeCommandBlockBundle.mockResolvedValueOnce({
      kind: "haneulchi.command_block_bundle",
      version: 1,
      exported_at: "2026-05-05T01:00:00Z",
      command_block: {
        id: "cmdblk_1",
        session_id: "pty_1",
        task_id: null,
        run_id: null,
        seq_start: 1,
        seq_end: 1,
        command: "npm test",
        cwd: "~/develop/applications/haneulchi",
        branch: "main",
        exit_code: null,
        duration_ms: null,
        summary: "PASS command block export workflow",
      },
      explanation: {
        id: "explain_cmdblk_1",
        command_block_id: "cmdblk_1",
        command: "npm test",
        summary: "AI explanation draft for npm test: unknown in ~/develop/applications/haneulchi on main",
        evidence: ["context: npm test unknown in ~/develop/applications/haneulchi on main", "sequence 1-1"],
        provider: null,
        model: null,
        agent_profile_id: null,
        prompt: null,
        diagnostics: [],
      },
    });

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.writeTerminalPtyInput).toHaveBeenCalledWith("pty_1", "npm test\r"));

    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "PASS command block export workflow\n",
      });
    });

    expect(await screen.findByText(/npm test .* seq 1-1/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /explain npm test/i }));
    await waitFor(() => expect(commandBlockApi.explainNativeCommandBlock).toHaveBeenCalledWith("cmdblk_1", {
      provider: "openai",
      model: "gpt-5.4",
      agentProfileId: "agent_codex",
    }));

    const explanation = await screen.findByRole("region", { name: /command block explanation/i });
    expect(explanation).toHaveTextContent("AI explanation draft · openai/gpt-5.4 · agent agent_codex");
    expect(explanation).toHaveTextContent("No external AI call has been made");
    expect(explanation).toHaveTextContent("npm test unknown in ~/develop/applications/haneulchi on main");
    expect(explanation).toHaveTextContent("output: PASS command block export workflow");

    fireEvent.click(screen.getByRole("button", { name: /export npm test bundle/i }));

    await waitFor(() => expect(commandBlockApi.exportNativeCommandBlockBundle).toHaveBeenCalledWith("cmdblk_1"));
    expect(window.navigator.clipboard.writeText).toHaveBeenCalledWith(expect.stringContaining('"kind": "haneulchi.command_block_bundle"'));
    expect(window.navigator.clipboard.writeText).toHaveBeenCalledWith(expect.stringContaining('"command_block"'));
    expect(window.navigator.clipboard.writeText).toHaveBeenCalledWith(expect.stringContaining('"command": "npm test"'));
    expect(await screen.findByText("Exported bundle cmdblk_1")).toBeInTheDocument();
  });

  it("persists command blocks through the native SQLite command block command when terminal IO is indexed", async () => {
    commandBlockApi.upsertNativeCommandBlock.mockReturnValue(immediateResolved({}));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));

    await waitFor(() =>
      expect(commandBlockApi.upsertNativeCommandBlock).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "cmdblk_1",
          sessionId: "pty_1",
          command: "npm test",
          cwd: "~/develop/applications/haneulchi",
          branch: "main",
          status: "running",
        }),
      ),
    );

    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "PASS persisted command block\n",
      });
    });

    await waitFor(() =>
      expect(commandBlockApi.upsertNativeCommandBlock).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "cmdblk_1",
          seqStart: 1,
          seqEnd: 1,
          outputExcerpt: expect.stringContaining("PASS persisted command block"),
        }),
      ),
    );
  });

  it("records raw PTY output stream chunks for persisted terminal sessions", async () => {
    stateSnapshot.getStateSnapshot.mockResolvedValueOnce({
      snapshot_id: "snap_terminal_stream",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [{ id: "proj_auth", name: "Auth Service", state: "active" }],
      project_tabs: [{ id: "tab_proj_auth", project_id: "proj_auth", label: "Auth Service", active: true }],
      sessions: [
        {
          id: "session_1",
          project_id: "proj_auth",
          pane_id: "pane_session_1",
          mode: "shell",
          title: "Auth shell",
          cwd: "/repo/auth",
          branch: "main",
          state: "running",
          attention_state: "none",
          token_budget_state: "ok",
        },
      ],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [], current_version_id: null, last_known_good_version_id: null, diagnostics: { errors: [] } },
      provider_model: { provider: "openai", model: "gpt-5.4", agent_profile_id: "agent_codex" },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "ok", pty: "ok", api: "ok" },
    });
    sessionApi.recordNativeTerminalStreamChunk.mockReturnValue(immediateResolved({}));
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /run auth shell/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());

    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "\u001b]777;agent.completed:{}\u0007raw evidence output\n",
      });
    });

    await waitFor(() =>
      expect(sessionApi.recordNativeTerminalStreamChunk).toHaveBeenCalledWith({
        sessionId: "session_1",
        seqStart: 1,
        seqEnd: 1,
        body: "\u001b]777;agent.completed:{}\u0007raw evidence output\n",
      }),
    );
  });

  it("surfaces and recovers command block parser degraded state when native persistence fails", async () => {
    commandBlockApi.upsertNativeCommandBlock.mockRejectedValueOnce(new Error("native command block API unavailable"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));

    expect(await screen.findByText(/Command block parser degraded · native command block API unavailable/i)).toBeInTheDocument();
    expect(screen.getByText(/Local command block log remains available/i)).toBeInTheDocument();

    commandBlockApi.upsertNativeCommandBlock.mockReturnValue(immediateResolved({}));
    fireEvent.click(screen.getByRole("button", { name: /retry command block persistence/i }));

    await waitFor(() =>
      expect(screen.queryByText(/Command block parser degraded · native command block API unavailable/i)).not.toBeInTheDocument(),
    );
    expect(screen.getByText(/Command block persistence recovered/i)).toBeInTheDocument();
  });

  it("persists command block evidence attachments through the native evidence command", async () => {
    commandBlockApi.upsertNativeCommandBlock.mockReturnValue(immediateResolved({}));
    commandBlockApi.attachNativeCommandBlockToEvidence.mockReturnValue(immediateResolved({
      id: "ev_local",
      task_id: null,
      run_id: null,
      artifact_path: "artifacts/evidence/ev_local.json",
      completeness_state: "partial",
      body_json: { command_blocks: [{ id: "cmdblk_1" }] },
    }));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));
    expect(await screen.findByText(/npm test .* seq --/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /attach npm test to evidence pack/i }));

    await waitFor(() =>
      expect(commandBlockApi.attachNativeCommandBlockToEvidence).toHaveBeenCalledWith({
        evidencePackId: "ev_local",
        commandBlockId: "cmdblk_1",
      }),
    );
    expect(screen.getByText(/Evidence Pack 1 command blocks attached/i)).toBeInTheDocument();
  });

  it("surfaces command block evidence attachment sync failures while keeping the local evidence pack", async () => {
    commandBlockApi.upsertNativeCommandBlock.mockReturnValue(immediateResolved({}));
    commandBlockApi.attachNativeCommandBlockToEvidence.mockRejectedValueOnce(new Error("evidence store offline"));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));
    expect(await screen.findByText(/npm test .* seq --/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /attach npm test to evidence pack/i }));

    await waitFor(() =>
      expect(commandBlockApi.attachNativeCommandBlockToEvidence).toHaveBeenCalledWith({
        evidencePackId: "ev_local",
        commandBlockId: "cmdblk_1",
      }),
    );
    expect(screen.getByText(/Evidence Pack 1 command blocks attached/i)).toBeInTheDocument();
    expect(await screen.findByText("Evidence attachment saved locally · evidence store offline")).toBeInTheDocument();
  });

  it("supports manual mark merge and split actions from the command block log", async () => {
    commandBlockApi.upsertNativeCommandBlock.mockReturnValue(immediateResolved({}));
    commandBlockApi.markNativeCommandBlock.mockReturnValue(immediateResolved({}));
    commandBlockApi.mergeNativeCommandBlocks.mockReturnValue(immediateResolved({}));
    commandBlockApi.splitNativeCommandBlock.mockReturnValue(immediateResolved({
      updated_block: {},
      created_block: {},
    }));
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: /run 1\. haneulchi/i }));
    await waitFor(() => expect(terminalPty.spawnTerminalPtySession).toHaveBeenCalled());

    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));
    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "PASS first command\n",
      });
    });

    fireEvent.click(screen.getByRole("button", { name: /submit 1\. haneulchi/i }));
    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 2,
        chunk: "PASS second command\n",
      });
    });

    expect(await screen.findAllByText(/npm test .* seq/i)).toHaveLength(2);

    fireEvent.click(screen.getAllByRole("button", { name: /mark npm test passed/i })[0]);
    expect(await screen.findByText(/npm test · completed .* seq 1-1/i)).toBeInTheDocument();
    await waitFor(() =>
      expect(commandBlockApi.markNativeCommandBlock).toHaveBeenCalledWith("cmdblk_1", "completed"),
    );

    fireEvent.click(screen.getAllByRole("button", { name: /merge npm test with previous block/i })[1]);
    expect(await screen.findByText(/npm test && npm test .* seq 1-2/i)).toBeInTheDocument();
    await waitFor(() =>
      expect(commandBlockApi.mergeNativeCommandBlocks).toHaveBeenCalledWith("cmdblk_1", "cmdblk_2"),
    );

    fireEvent.click(screen.getByRole("button", { name: /split npm test && npm test/i }));
    expect(await screen.findByText(/npm test && npm test \(part 1\)/i)).toBeInTheDocument();
    expect(await screen.findByText(/npm test && npm test \(part 2\)/i)).toBeInTheDocument();
    await waitFor(() => expect(commandBlockApi.splitNativeCommandBlock).toHaveBeenCalledWith("cmdblk_1"));
  });

  it("shows OSC allowlist parser counts from terminal output", async () => {
    render(<App />);

    act(() => {
      terminalPty.outputHandler?.({
        sessionId: "pty_1",
        seq: 1,
        chunk: "\u001b]777;agent.completed:{}\u0007\u001b]1337;<script>alert(1)</script>\u0007plain output",
      });
    });

    expect(await screen.findByText(/OSC 1 allowed · 1 ignored · 0 rejected/i)).toBeInTheDocument();
  });
});
