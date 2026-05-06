import { invoke } from "@tauri-apps/api/core";
import type { StateBenchmarkSuite } from "./stateSnapshotClient";

export interface NativeReleaseGateScenario {
  gate_id: string;
  name: string;
  status: string;
  detail: string;
  evidence: string[];
}

export interface NativeReleaseGateRun {
  id: string;
  project_id: string;
  status: string;
  scenario_count: number;
  pass_count: number;
  fail_count: number;
  warning_count: number;
  scenarios: NativeReleaseGateScenario[];
  created_at: string;
}

export interface NativeTerminalFidelitySmokeCase {
  case_id: string;
  name: string;
  status: string;
  detail: string;
}

export interface NativeTerminalFidelitySmokeRun {
  id: string;
  project_id: string;
  status: string;
  case_count: number;
  pass_count: number;
  fail_count: number;
  warning_count: number;
  cases: NativeTerminalFidelitySmokeCase[];
  created_at: string;
}

export interface NativeTaskLifecycleE2ETransition {
  step: string;
  task_status: string;
  run_lifecycle?: string | null;
  evidence_pack_id?: string | null;
}

export interface NativeTaskLifecycleE2ERun {
  id: string;
  project_id: string;
  status: string;
  task_id: string;
  run_id: string;
  evidence_pack_id: string;
  transitions: NativeTaskLifecycleE2ETransition[];
  created_at: string;
}

export interface NativeWorkflowNegativeTestCase {
  case_id: string;
  status: string;
  detail: string;
}

export interface NativeWorkflowNegativeTestRun {
  id: string;
  project_id: string;
  status: string;
  baseline_workflow_id: string;
  invalid_workflow_id: string;
  last_known_good_workflow_id: string;
  dispatch_run_id: string;
  cases: NativeWorkflowNegativeTestCase[];
  created_at: string;
}

export interface NativeDmgSmokeCase {
  case_id: string;
  name: string;
  status: string;
  detail: string;
}

export interface NativeDmgSmokeRun {
  id: string;
  project_id: string;
  status: string;
  explicit_blocker: boolean;
  dmg_path?: string | null;
  app_bundle_path?: string | null;
  case_count: number;
  pass_count: number;
  fail_count: number;
  warning_count: number;
  cases: NativeDmgSmokeCase[];
  created_at: string;
}

export interface NativeRecoveryDrill {
  drill_id: string;
  name: string;
  status: string;
  detail: string;
  evidence: string[];
}

export interface NativeRecoveryDrillRun {
  id: string;
  project_id: string;
  status: string;
  drill_count: number;
  pass_count: number;
  fail_count: number;
  warning_count: number;
  drills: NativeRecoveryDrill[];
  created_at: string;
}

export interface NativeBenchmarkRun {
  id: string;
  project_id: string;
  status: string;
  suite_count: number;
  pass_count: number;
  fail_count: number;
  warning_count: number;
  duration_ms: number;
  suites: StateBenchmarkSuite[];
  created_at: string;
}

export interface NativeDogfoodTelemetryFinding {
  finding_id: string;
  status: string;
  detail: string;
}

export interface NativeDogfoodTelemetryReview {
  id: string;
  project_id: string;
  status: string;
  evidence_pack_id: string;
  finding_count: number;
  pass_count: number;
  warning_count: number;
  fail_count: number;
  findings: NativeDogfoodTelemetryFinding[];
  created_at: string;
}

export function runNativeReleaseGates(projectId: string): Promise<NativeReleaseGateRun> {
  return invoke<NativeReleaseGateRun>("run_release_gates", { request: { projectId } });
}

export function listNativeReleaseGateRuns(projectId: string): Promise<NativeReleaseGateRun[]> {
  return invoke<NativeReleaseGateRun[]>("list_release_gate_runs", { projectId });
}

export function runNativeTerminalFidelitySmoke(projectId: string): Promise<NativeTerminalFidelitySmokeRun> {
  return invoke<NativeTerminalFidelitySmokeRun>("run_terminal_fidelity_smoke", { request: { projectId } });
}

export function listNativeTerminalFidelitySmokeRuns(projectId: string): Promise<NativeTerminalFidelitySmokeRun[]> {
  return invoke<NativeTerminalFidelitySmokeRun[]>("list_terminal_fidelity_smoke_runs", { projectId });
}

export function runNativeTaskLifecycleE2E(projectId: string): Promise<NativeTaskLifecycleE2ERun> {
  return invoke<NativeTaskLifecycleE2ERun>("run_task_lifecycle_e2e", { request: { projectId } });
}

export function listNativeTaskLifecycleE2ERuns(projectId: string): Promise<NativeTaskLifecycleE2ERun[]> {
  return invoke<NativeTaskLifecycleE2ERun[]>("list_task_lifecycle_e2e_runs", { projectId });
}

export function runNativeWorkflowNegativeTests(projectId: string): Promise<NativeWorkflowNegativeTestRun> {
  return invoke<NativeWorkflowNegativeTestRun>("run_workflow_negative_tests", { request: { projectId } });
}

export function listNativeWorkflowNegativeTestRuns(projectId: string): Promise<NativeWorkflowNegativeTestRun[]> {
  return invoke<NativeWorkflowNegativeTestRun[]>("list_workflow_negative_test_runs", { projectId });
}

export function runNativeDmgSmokeTest(
  projectId: string,
  dmgPath?: string,
  appBundlePath?: string,
): Promise<NativeDmgSmokeRun> {
  return invoke<NativeDmgSmokeRun>("run_dmg_smoke_test", {
    request: { projectId, dmgPath, appBundlePath },
  });
}

export function listNativeDmgSmokeRuns(projectId: string): Promise<NativeDmgSmokeRun[]> {
  return invoke<NativeDmgSmokeRun[]>("list_dmg_smoke_runs", { projectId });
}

export function runNativeRecoveryDrills(projectId: string): Promise<NativeRecoveryDrillRun> {
  return invoke<NativeRecoveryDrillRun>("run_recovery_drills", { request: { projectId } });
}

export function listNativeRecoveryDrillRuns(projectId: string): Promise<NativeRecoveryDrillRun[]> {
  return invoke<NativeRecoveryDrillRun[]>("list_recovery_drill_runs", { projectId });
}

export function runNativeBenchmarks(projectId: string): Promise<NativeBenchmarkRun> {
  return invoke<NativeBenchmarkRun>("run_benchmarks", { request: { projectId } });
}

export function listNativeBenchmarkRuns(projectId: string): Promise<NativeBenchmarkRun[]> {
  return invoke<NativeBenchmarkRun[]>("list_benchmark_runs", { projectId });
}

export function runNativeDogfoodTelemetryReview(projectId: string): Promise<NativeDogfoodTelemetryReview> {
  return invoke<NativeDogfoodTelemetryReview>("run_dogfood_telemetry_review", { request: { projectId } });
}

export function listNativeDogfoodTelemetryReviews(projectId: string): Promise<NativeDogfoodTelemetryReview[]> {
  return invoke<NativeDogfoodTelemetryReview[]>("list_dogfood_telemetry_reviews", { projectId });
}
