import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  listNativeDmgSmokeRuns,
  listNativeRecoveryDrillRuns,
  listNativeBenchmarkRuns,
  listNativeDogfoodTelemetryReviews,
  listNativeReleaseGateRuns,
  listNativeTaskLifecycleE2ERuns,
  listNativeTerminalFidelitySmokeRuns,
  listNativeWorkflowNegativeTestRuns,
  runNativeDmgSmokeTest,
  runNativeBenchmarks,
  runNativeDogfoodTelemetryReview,
  runNativeRecoveryDrills,
  runNativeReleaseGates,
  runNativeTaskLifecycleE2E,
  runNativeTerminalFidelitySmoke,
  runNativeWorkflowNegativeTests,
} from "./qualityApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("quality API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("runs and lists release gates through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "release_gate_1",
        project_id: "proj_local",
        status: "blocked",
        scenario_count: 2,
        pass_count: 1,
        fail_count: 1,
        warning_count: 0,
        scenarios: [{ gate_id: "RG-01", name: "Evidence", status: "fail", detail: "missing", evidence: [] }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([]);

    const run = await runNativeReleaseGates("proj_local");
    const runs = await listNativeReleaseGateRuns("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "run_release_gates", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_release_gate_runs", { projectId: "proj_local" });
    expect(run.scenarios[0].gate_id).toBe("RG-01");
    expect(runs).toEqual([]);
  });

  it("runs and lists benchmarks through Tauri invoke", async () => {
    vi.mocked(invoke)
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
      .mockResolvedValueOnce([]);

    const run = await runNativeBenchmarks("proj_local");
    const runs = await listNativeBenchmarkRuns("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "run_benchmarks", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_benchmark_runs", { projectId: "proj_local" });
    expect(run.suites[0].suite_id).toBe("state_snapshot_latency");
    expect(runs).toEqual([]);
  });

  it("runs and lists dogfood telemetry reviews through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "dogfood_review_1",
        project_id: "proj_local",
        status: "warning",
        evidence_pack_id: "ev_dogfood_review_1",
        finding_count: 1,
        pass_count: 0,
        warning_count: 1,
        fail_count: 0,
        findings: [{ finding_id: "telemetry_command_blocks", status: "warning", detail: "empty" }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([]);

    const review = await runNativeDogfoodTelemetryReview("proj_local");
    const reviews = await listNativeDogfoodTelemetryReviews("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "run_dogfood_telemetry_review", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_dogfood_telemetry_reviews", { projectId: "proj_local" });
    expect(review.evidence_pack_id).toBe("ev_dogfood_review_1");
    expect(reviews).toEqual([]);
  });

  it("runs and lists ship-readiness drills through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "terminal_smoke_1",
        project_id: "proj_local",
        status: "warning",
        case_count: 3,
        pass_count: 2,
        fail_count: 0,
        warning_count: 1,
        cases: [{ case_id: "ansi_palette", name: "ANSI palette", status: "pass", detail: "ok" }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
        id: "task_lifecycle_e2e_1",
        project_id: "proj_local",
        status: "passed",
        task_id: "task_e2e_1",
        run_id: "run_e2e_1",
        evidence_pack_id: "ev_lifecycle_run_e2e_1",
        transitions: [{ step: "done", task_status: "done", run_lifecycle: "completed", evidence_pack_id: "ev_lifecycle_run_e2e_1" }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
        id: "workflow_negative_1",
        project_id: "proj_local",
        status: "passed",
        baseline_workflow_id: "wf_base",
        invalid_workflow_id: "wf_bad",
        last_known_good_workflow_id: "wf_base",
        dispatch_run_id: "run_lkg",
        cases: [{ case_id: "invalid_reload_preserves_lkg", status: "pass", detail: "ok" }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
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
        cases: [{ case_id: "dmg_artifact", name: "DMG artifact", status: "fail", detail: "missing signature" }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({
        id: "recovery_drill_1",
        project_id: "proj_local",
        status: "passed",
        drill_count: 4,
        pass_count: 4,
        fail_count: 0,
        warning_count: 0,
        drills: [{ drill_id: "invalid_workflow_lkg", name: "Invalid workflow recovery", status: "pass", detail: "ok", evidence: ["wf_base"] }],
        created_at: "2026-05-02T01:00:00Z",
      })
      .mockResolvedValueOnce([]);

    const terminal = await runNativeTerminalFidelitySmoke("proj_local");
    const terminalRuns = await listNativeTerminalFidelitySmokeRuns("proj_local");
    const lifecycle = await runNativeTaskLifecycleE2E("proj_local");
    const lifecycleRuns = await listNativeTaskLifecycleE2ERuns("proj_local");
    const workflow = await runNativeWorkflowNegativeTests("proj_local");
    const workflowRuns = await listNativeWorkflowNegativeTestRuns("proj_local");
    const dmg = await runNativeDmgSmokeTest("proj_local", "/tmp/Haneulchi.dmg", "/Applications/Haneulchi.app");
    const dmgRuns = await listNativeDmgSmokeRuns("proj_local");
    const recovery = await runNativeRecoveryDrills("proj_local");
    const recoveryRuns = await listNativeRecoveryDrillRuns("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "run_terminal_fidelity_smoke", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_terminal_fidelity_smoke_runs", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(3, "run_task_lifecycle_e2e", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(4, "list_task_lifecycle_e2e_runs", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(5, "run_workflow_negative_tests", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(6, "list_workflow_negative_test_runs", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(7, "run_dmg_smoke_test", {
      request: { projectId: "proj_local", dmgPath: "/tmp/Haneulchi.dmg", appBundlePath: "/Applications/Haneulchi.app" },
    });
    expect(invoke).toHaveBeenNthCalledWith(8, "list_dmg_smoke_runs", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(9, "run_recovery_drills", { request: { projectId: "proj_local" } });
    expect(invoke).toHaveBeenNthCalledWith(10, "list_recovery_drill_runs", { projectId: "proj_local" });
    expect(terminal.cases[0].case_id).toBe("ansi_palette");
    expect(terminalRuns).toEqual([]);
    expect(lifecycle.evidence_pack_id).toBe("ev_lifecycle_run_e2e_1");
    expect(lifecycleRuns).toEqual([]);
    expect(workflow.last_known_good_workflow_id).toBe("wf_base");
    expect(workflowRuns).toEqual([]);
    expect(dmg.explicit_blocker).toBe(true);
    expect(dmgRuns).toEqual([]);
    expect(recovery.drills[0].drill_id).toBe("invalid_workflow_lkg");
    expect(recoveryRuns).toEqual([]);
  });
});
