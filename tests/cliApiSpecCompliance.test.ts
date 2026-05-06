import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const workspaceRoot = resolve(__dirname, "..");
const cliApiSpecPath = resolve(
  workspaceRoot,
  "docs/Haneulchi_Document_Set_v10_Super_App_Pack/04_specs/Haneulchi_CLI_API_Spec_v4.md",
);
const stateApiSpecPath = resolve(
  workspaceRoot,
  "docs/Haneulchi_Document_Set_v10_Super_App_Pack/04_specs/Haneulchi_State_Snapshot_and_Session_Control_API_v3.md",
);
const readmePath = resolve(workspaceRoot, "README.md");
const includePrivateDocs =
  (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
    ?.HANEULCHI_INCLUDE_PRIVATE_DOCS === "1";
const docsAvailable = includePrivateDocs && [cliApiSpecPath, stateApiSpecPath].every(existsSync);
const itWithDocs = docsAvailable ? it : it.skip;

function readCliApiSpec(): string {
  return readFileSync(cliApiSpecPath, "utf8");
}

function readStateApiSpec(): string {
  return readFileSync(stateApiSpecPath, "utf8");
}

function readReadme(): string {
  return readFileSync(readmePath, "utf8");
}

describe("CLI/API spec compliance", () => {
  itWithDocs("documents control endpoints used by required CLI action commands", () => {
    const spec = readCliApiSpec();

    [
      "`/v1/projects/:id/focus`",
      "`/v1/sessions/:id/kill`",
      "`/v1/sessions/:id/release`",
      "`/v1/sessions/:id/stream-chunks`",
      "`/v1/command-blocks/:id/explain`",
      "`/v1/command-blocks/:id/bundle`",
      "`/v1/agents`",
      "`/v1/agents/:id/pause`",
      "`/v1/agents/:id/resume`",
      "`/v1/agents/:id/heartbeat`",
      "`/v1/agent-events/ingest`",
      "`/v1/provider-model`",
      "`/v1/policy/approvals`",
      "`/v1/policy/approvals/:id/decision`",
      "`/v1/policy/packs`",
      "`/v1/policy/audit`",
      "`/v1/secrets`",
      "`/v1/knowledge/:id`",
      "`/v1/knowledge/lint`",
      "`/v1/context-packs`",
      "`/v1/context-packs/:id`",
      "`/v1/tasks/:id/planning`",
      "`/v1/tasks/:id/context`",
      "`/v1/initiatives`",
      "`/v1/budgets/forecast`",
      "`/v1/provider-prices`",
      "`/v1/provider-prices/update`",
      "`/v1/token-usage`",
      "`/v1/token-usage/ingest`",
      "`/v1/runs/:id/token-usage`",
      "`/v1/workflow/status`",
    ].forEach((endpoint) => {
      expect(spec).toContain(endpoint);
    });
  });

  itWithDocs("keeps command block CLI commands aligned with the P1 explain and export actions", () => {
    const spec = readCliApiSpec();

    expect(spec).toContain("hc block search|show|explain|export|attach");
  });

  itWithDocs("keeps session CLI commands aligned with transcript stream capture actions", () => {
    const spec = readCliApiSpec();

    expect(spec).toContain("hc session list|new|focus|usage|input|stream|attach-task|detach-task|takeover|release|kill");
  });

  itWithDocs("documents hardening, quality, visual, and tracker CLI/API surfaces", () => {
    const spec = readCliApiSpec();

    [
      "`/v1/release-gates/run`",
      "`/v1/release-gates/runs`",
      "`/v1/terminal-fidelity/smoke/run`",
      "`/v1/terminal-fidelity/smoke/runs`",
      "`/v1/task-lifecycle/e2e/run`",
      "`/v1/task-lifecycle/e2e/runs`",
      "`/v1/workflow/negative-tests/run`",
      "`/v1/workflow/negative-tests/runs`",
      "`/v1/distribution/dmg-smoke/run`",
      "`/v1/distribution/dmg-smoke/runs`",
      "`/v1/recovery/drills/run`",
      "`/v1/recovery/drills/runs`",
      "`/v1/benchmarks/run`",
      "`/v1/benchmarks/runs`",
      "`/v1/dogfood/telemetry-review/run`",
      "`/v1/dogfood/telemetry-reviews`",
      "`/v1/visual-harness/graph`",
      "`/v1/visual-harness/links`",
      "`/v1/tracker-bindings`",
      "`/v1/tracker-sync/linear/run`",
      "`/v1/tracker-sync/github/run`",
      "`/v1/tracker-sync/plane/run`",
    ].forEach((endpoint) => {
      expect(spec).toContain(endpoint);
    });

    [
      "hc release-gate run|list",
      "hc terminal smoke|smoke-runs",
      "hc task lifecycle-e2e|lifecycle-e2e-runs",
      "hc workflow validate|reload|status|negative-tests|negative-test-runs",
      "hc distribution dmg-smoke|dmg-smoke-runs",
      "hc recovery drills|drill-runs",
      "hc benchmark run|runs",
      "hc dogfood telemetry-review|telemetry-reviews",
      "hc visual graph|links|link",
      "hc tracker bindings|bind|<linear|github|plane> sync",
    ].forEach((command) => {
      expect(spec).toContain(command);
    });
  });

  itWithDocs("documents the implemented daily-driver and local-ops CLI command groups", () => {
    const spec = readCliApiSpec();

    [
      "`/v1/projects/:id/files`",
      "`/v1/projects/:id/file`",
      "`/v1/projects/:id/diff`",
      "`/v1/projects/:id/lsp-diagnostics`",
      "`/v1/projects/:id/patch/export`",
      "`/v1/projects/:id/patch/import`",
      "`/v1/projects/:id/pr/landing-plan`",
      "`/v1/projects/:id/layout`",
      "`/v1/projects/:id/detach`",
      "`/v1/projects/:id/tab-group`",
      "`/v1/sessions/:id/token-usage`",
      "`/v1/sessions/:id/attach-task`",
      "`/v1/sessions/:id/detach-task`",
      "`/v1/runs/:id/replay`",
      "`/v1/runs/:id/transition`",
      "`/v1/runs/:id/status-updates`",
      "`/v1/terminal-theme`",
      "`/v1/policy/evaluate`",
      "`/v1/knowledge/automation/run`",
      "`/v1/knowledge/ingest`",
      "`/v1/browser-automation/run`",
    ].forEach((endpoint) => {
      expect(spec).toContain(endpoint);
    });

    [
      "hc health --json",
      "hc project list|add|focus|detach|tab-group|files|file|write-file|diff|lsp|patch-export|patch-import|pr-plan|search-files|layout",
      "hc session list|new|focus|usage|input|stream|attach-task|detach-task|takeover|release|kill",
      "hc run list|open|replay|usage|transition|cancel|retry|status|hook",
      "hc evidence generate|review",
      "hc policy approvals|packs|audit|pack set|request|evaluate|decide",
      "hc agent list|scan|register|pause|resume|heartbeat|events ingest",
      "hc terminal-theme get|set",
      "hc knowledge sources|source add|page create|search|open|explorations|exploration create|concepts|obsidian export|chat|lint|compile|ingest",
      "hc budget status|explain|dashboard|forecast|prices|record|set|export|ingest",
      "hc browser run",
    ].forEach((command) => {
      expect(spec).toContain(command);
    });
  });

  itWithDocs("documents the implemented knowledge vault endpoint families", () => {
    const spec = readCliApiSpec();

    [
      "`/v1/knowledge/sources`",
      "`/v1/knowledge/pages`",
      "`/v1/knowledge/explorations`",
      "`/v1/knowledge/concepts`",
      "`/v1/knowledge/obsidian/export`",
      "`/v1/knowledge/chat`",
    ].forEach((endpoint) => {
      expect(spec).toContain(endpoint);
    });
  });

  itWithDocs("documents evidence generation and review decision endpoints", () => {
    const spec = readCliApiSpec();

    [
      "`/v1/runs/:id/evidence/generate`",
      "`/v1/evidence/:id/review-decision`",
    ].forEach((endpoint) => {
      expect(spec).toContain(endpoint);
    });
  });

  itWithDocs("documents task drawer comment and workpad endpoint families", () => {
    const spec = readCliApiSpec();

    [
      "`/v1/tasks/:id/comments` | list task comments",
      "`/v1/tasks/:id/comments` | add comment",
      "`/v1/tasks/:id/workpad`",
    ].forEach((endpoint) => {
      expect(spec).toContain(endpoint);
    });
  });

  it("keeps the README CLI support summary aligned with implemented command groups", () => {
    const readme = readReadme();

    [
      "project list|add|focus|detach|tab-group|files|file|write-file|diff|lsp|patch-export|patch-import|pr-plan|search-files|layout",
      "session list|new|focus|usage|input|stream|attach-task|detach-task|takeover|release|kill",
      "block search|show|explain|export|attach",
      "task list|create|edit|move|assign|comment|comments|open|workpad|context|planning",
      "initiative list|create",
      "run list|open|replay|usage|transition|cancel|retry|status|hook",
      "agent list|scan|register|pause|resume|heartbeat|events ingest",
      "provider-model get|set",
      "terminal-theme get|set",
      "policy approvals|packs|audit|pack set|request|evaluate|decide",
      "knowledge sources|source add|page create|search|open|explorations|exploration create|concepts|obsidian export|chat|lint|compile|ingest",
      "budget status|explain|dashboard|forecast|prices|record|set|export|ingest",
      "workflow validate|reload|status|negative-tests|negative-test-runs",
      "release-gate run|list",
      "terminal smoke|smoke-runs",
      "task lifecycle-e2e|lifecycle-e2e-runs",
      "distribution dmg-smoke|dmg-smoke-runs",
      "recovery drills|drill-runs",
      "benchmark run|runs",
      "dogfood telemetry-review|telemetry-reviews",
      "visual graph|links|link",
      "tracker bindings|bind|<linear|github|plane> sync",
      "browser run",
      "review list|accept|changes|block",
      "secret list|set",
      "context list|create|show|attach",
      "update check",
    ].forEach((command) => {
      expect(readme).toContain(command);
    });
  });

  it("keeps the README feature summary aligned with implemented P1 runway surfaces", () => {
    const readme = readReadme();

    [
      "roadmap timeline and calendar views",
      "skill pack registry and runtime pool summaries",
      "historical analytics charts and dashboard widget visibility controls",
      "visual workflow debugger and workflow marketplace import",
      "network sandbox profiles and advanced permission audit filtering",
      "budget forecasts and provider price update workflows",
      "visual harness graph canvas with drag-to-create context/tool/task links",
      "Linear, GitHub, and Plane tracker sync adapters",
    ].forEach((surface) => {
      expect(readme).toContain(surface);
    });
  });

  itWithDocs("documents the implemented state snapshot sections and hardening summaries", () => {
    const spec = readStateApiSpec();

    [
      '"initiatives": []',
      '"provider_model": {"provider": "openai", "model": "gpt-5.4", "agent_profile_id": "agent_codex"}',
      '"security": {"keychain": "unknown", "secret_count": 0, "redaction": {}, "permission_audit": {}, "policy_pack": {}, "diagnostics": {}}',
      '"workflow_negative": {"last_run_id": null, "last_status": "not_run", "last_baseline_workflow_id": null, "last_invalid_workflow_id": null, "last_known_good_workflow_id": null, "diagnostics": {}}',
      '"task_lifecycle": {"last_run_id": null, "last_status": "not_run", "last_task_id": null, "last_agent_run_id": null, "last_evidence_pack_id": null, "diagnostics": {}}',
      '"terminal_fidelity": {"last_run_id": null, "last_status": "not_run", "last_pass_count": 0, "last_fail_count": 0, "last_warning_count": 0, "diagnostics": {}}',
      '"release_gates": {"last_run_id": null, "last_status": "not_run", "last_pass_count": 0, "last_fail_count": 0, "last_warning_count": 0, "diagnostics": {}}',
      '"distribution": {"last_dmg_smoke_run_id": null, "last_status": "not_run", "explicit_blocker": false, "last_pass_count": 0, "last_fail_count": 0, "last_warning_count": 0, "diagnostics": {}}',
      '"recovery": {"last_run_id": null, "last_status": "not_run", "last_pass_count": 0, "last_fail_count": 0, "last_warning_count": 0, "diagnostics": {}}',
      '"benchmarks": {"last_run_id": null, "last_status": "not_run", "last_pass_count": 0, "last_fail_count": 0, "last_warning_count": 0, "suites": [], "diagnostics": {}}',
      '"dogfood": {"last_review_id": null, "last_status": "not_run", "last_evidence_pack_id": null, "last_pass_count": 0, "last_warning_count": 0, "last_fail_count": 0, "diagnostics": {}}',
      '"visual_harness": {"nodes": [], "edges": [], "diagnostics": {}}',
      '"tracker": {"binding_count": 0, "bindings": [], "diagnostics": {}}',
    ].forEach((field) => {
      expect(spec).toContain(field);
    });
  });

  itWithDocs("documents snapshot ids on CLI list endpoint responses", () => {
    const spec = readCliApiSpec();

    expect(spec).toContain(
      'top-level `snapshot_id` alongside their `items` array',
    );
  });

  itWithDocs("documents snapshot ids on session mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Session control mutations preserve their resource or receipt fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on task mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Task mutations preserve their task/comment/workpad fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on project registry mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Project registry mutations preserve their project/tab/group/plan fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on initiative mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Initiative mutations preserve their initiative fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on run mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Run mutations preserve their run or status comment fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on policy mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Policy mutations preserve their approval, policy pack, or evaluation fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on operational mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Operational mutations preserve their agent, settings, budget, telemetry, price, or secret fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on knowledge mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Knowledge mutations preserve their source, page, exploration, context pack, export, answer, lint, automation, or ingestion fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on workflow integration mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Workflow integration mutations preserve their release gate, quality run, visual harness, tracker sync, or browser automation fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on evidence and workflow mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Evidence and workflow mutations preserve their workflow, hook result, evidence pack, or review decision fields and include a top-level `snapshot_id`",
    );
  });

  itWithDocs("documents snapshot ids on project tool mutation responses", () => {
    const spec = readStateApiSpec();

    expect(spec).toContain(
      "Project tool mutations preserve their file preview, patch artifact, or PR landing plan fields and include a top-level `snapshot_id`",
    );
  });
});
