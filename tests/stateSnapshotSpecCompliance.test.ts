import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const workspaceRoot = resolve(__dirname, "..");
const stateSnapshotSpecPath = resolve(
  workspaceRoot,
  "docs/Haneulchi_Document_Set_v10_Super_App_Pack/04_specs/Haneulchi_State_Snapshot_and_Session_Control_API_v3.md",
);
const stateSnapshotRustPath = resolve(workspaceRoot, "src-tauri/src/state_snapshot.rs");
const includePrivateDocs =
  (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
    ?.HANEULCHI_INCLUDE_PRIVATE_DOCS === "1";
const describeWithDocs = includePrivateDocs && existsSync(stateSnapshotSpecPath) ? describe : describe.skip;

function readStateSnapshotSpec(): string {
  return readFileSync(stateSnapshotSpecPath, "utf8");
}

function readStateSnapshotRust(): string {
  return readFileSync(stateSnapshotRustPath, "utf8");
}

function sampleStatePayload(): Record<string, unknown> {
  const json = readStateSnapshotSpec().match(/```json\n([\s\S]*?)\n```/)?.[1];
  if (!json) throw new Error("State snapshot spec is missing its JSON payload sample");
  return JSON.parse(json) as Record<string, unknown>;
}

function rustStructFields(source: string, structName: string): string[] {
  const body = source.match(new RegExp(`pub struct ${structName} \\{([\\s\\S]*?)\\n\\}`))?.[1];
  if (!body) throw new Error(`Missing Rust struct: ${structName}`);

  return [...body.matchAll(/^\s+pub\s+([a-z0-9_]+):/gm)].map(([, field]) => field);
}

function valueAtPath(payload: Record<string, unknown>, path: string[]): Record<string, unknown> {
  return path.reduce<unknown>((value, segment) => {
    if (!value || typeof value !== "object") return undefined;
    return (value as Record<string, unknown>)[segment];
  }, payload) as Record<string, unknown>;
}

describeWithDocs("state snapshot spec compliance", () => {
  it("documents every object field emitted by the Rust state snapshot payload", () => {
    const payload = sampleStatePayload();
    const rust = readStateSnapshotRust();
    const contracts: Array<[string, string[]]> = [
      ["StateSnapshot", []],
      ["StateSnapshotApp", ["app"]],
      ["StateTerminalTheme", ["app", "terminal_theme"]],
      ["StateCommandBlocks", ["command_blocks"]],
      ["StateCollection", ["tasks"]],
      ["StateRunCollection", ["runs"]],
      ["StateProviderModel", ["provider_model"]],
      ["StateBudgets", ["budgets"]],
      ["StateSecurity", ["security"]],
      ["StateWorkflow", ["workflow"]],
      ["StateWorkflowNegative", ["workflow_negative"]],
      ["StateKnowledge", ["knowledge"]],
      ["StateTaskLifecycle", ["task_lifecycle"]],
      ["StateTerminalFidelity", ["terminal_fidelity"]],
      ["StateReleaseGates", ["release_gates"]],
      ["StateDistribution", ["distribution"]],
      ["StateRecovery", ["recovery"]],
      ["StateBenchmarks", ["benchmarks"]],
      ["StateDogfood", ["dogfood"]],
      ["StateVisualHarness", ["visual_harness"]],
      ["StateTracker", ["tracker"]],
      ["StateHealth", ["health"]],
    ];

    const missing = contracts.flatMap(([structName, path]) => {
      const sampleObject = valueAtPath(payload, path);
      const sampleFields = new Set(Object.keys(sampleObject ?? {}));

      return rustStructFields(rust, structName)
        .filter((field) => !sampleFields.has(field))
        .map((field) => `${path.length ? path.join(".") : "$"}.${field}`);
    });

    expect(missing).toEqual([]);
  });
});
