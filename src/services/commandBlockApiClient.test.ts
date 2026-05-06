import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
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
} from "./commandBlockApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("command block API client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("maps command block persistence through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "cmdblk_1",
      session_id: "pty_1",
      task_id: null,
      run_id: null,
      seq_start: 4,
      seq_end: 9,
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      exit_code: null,
      duration_ms: null,
      summary: "PASS command workflow",
    });

    const persisted = await upsertNativeCommandBlock({
      id: "cmdblk_1",
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 4,
      seqEnd: 9,
      status: "running",
      outputExcerpt: "PASS command workflow",
    });

    expect(invoke).toHaveBeenCalledWith("upsert_command_block", {
      request: {
        id: "cmdblk_1",
        sessionId: "pty_1",
        taskId: undefined,
        runId: undefined,
        seqStart: 4,
        seqEnd: 9,
        command: "npm test",
        cwd: "/repo",
        branch: "main",
        exitCode: undefined,
        durationMs: undefined,
        summary: "PASS command workflow",
      },
    });
    expect(persisted.summary).toBe("PASS command workflow");
  });

  it("searches persisted command blocks and maps them into local command block shape", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
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

    const blocks = await searchNativeCommandBlocks("cargo", 25);

    expect(invoke).toHaveBeenCalledWith("search_command_blocks", { query: "cargo", limit: 25 });
    expect(blocks.map(nativeCommandBlockToCommandBlock)).toEqual([
      {
        id: "cmdblk_7",
        sessionId: "session_native",
        command: "cargo test",
        cwd: "/repo/src-tauri",
        branch: "main",
        seqStart: 12,
        seqEnd: 18,
        status: "completed",
        outputExcerpt: "PASS cargo tests",
      },
    ]);
  });

  it("maps command block evidence attachment through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "ev_local",
      task_id: "task_review",
      run_id: null,
      artifact_path: "artifacts/evidence/ev_local.json",
      completeness_state: "partial",
      body_json: { command_blocks: [{ id: "cmdblk_1" }] },
    });

    const evidence = await attachNativeCommandBlockToEvidence({
      evidencePackId: "ev_local",
      commandBlockId: "cmdblk_1",
      taskId: "task_review",
    });

    expect(invoke).toHaveBeenCalledWith("attach_command_block_to_evidence", {
      request: {
        evidencePackId: "ev_local",
        commandBlockId: "cmdblk_1",
        taskId: "task_review",
        runId: undefined,
      },
    });
    expect(evidence.body_json.command_blocks).toEqual([{ id: "cmdblk_1" }]);
  });

  it("maps command block mark, merge, and split actions through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "cmdblk_1", exit_code: 0 })
      .mockResolvedValueOnce({ id: "cmdblk_1", command: "npm test && cargo test" })
      .mockResolvedValueOnce({
        updated_block: { id: "cmdblk_1", command: "npm test && cargo test (part 1)" },
        created_block: { id: "cmdblk_2", command: "npm test && cargo test (part 2)" },
      });

    const marked = await markNativeCommandBlock("cmdblk_1", "completed");
    const merged = await mergeNativeCommandBlocks("cmdblk_1", "cmdblk_2");
    const split = await splitNativeCommandBlock("cmdblk_1");

    expect(invoke).toHaveBeenNthCalledWith(1, "mark_command_block", {
      commandBlockId: "cmdblk_1",
      status: "completed",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "merge_command_blocks", {
      firstCommandBlockId: "cmdblk_1",
      secondCommandBlockId: "cmdblk_2",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "split_command_block", {
      commandBlockId: "cmdblk_1",
    });
    expect(marked.exit_code).toBe(0);
    expect(merged.command).toBe("npm test && cargo test");
    expect(split.created_block.command).toContain("(part 2)");
  });

  it("maps command block explain and bundle export through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "explain_cmdblk_1",
        command_block_id: "cmdblk_1",
        command: "npm test",
        summary: "AI explanation draft for npm test: completed in /repo on main",
        evidence: ["ai route openai/gpt-5.4", "agent agent_codex"],
        provider: "openai",
        model: "gpt-5.4",
        agent_profile_id: "agent_codex",
        prompt: "Explain this command block.",
        diagnostics: ["No external AI call has been made."],
      })
      .mockResolvedValueOnce({
        kind: "haneulchi.command_block_bundle",
        version: 1,
        exported_at: "2026-05-05T01:00:00Z",
        command_block: { id: "cmdblk_1", command: "npm test" },
        explanation: { id: "explain_cmdblk_1" },
      });

    const explanation = await explainNativeCommandBlock("cmdblk_1", {
      provider: "openai",
      model: "gpt-5.4",
      agentProfileId: "agent_codex",
    });
    const bundle = await exportNativeCommandBlockBundle("cmdblk_1");

    expect(invoke).toHaveBeenNthCalledWith(1, "explain_command_block", {
      commandBlockId: "cmdblk_1",
      request: {
        provider: "openai",
        model: "gpt-5.4",
        agentProfileId: "agent_codex",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "export_command_block_bundle", {
      commandBlockId: "cmdblk_1",
    });
    expect(explanation.agent_profile_id).toBe("agent_codex");
    expect(bundle.kind).toBe("haneulchi.command_block_bundle");
  });

  it("maps generated run evidence through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "ev_run_1",
      task_id: "task_review",
      run_id: "run_1",
      artifact_path: "artifacts/evidence/ev_run_1.json",
      completeness_state: "complete",
      body_json: { tests: [{ command_block_id: "cmdblk_1" }] },
    });

    const evidence = await generateNativeEvidencePackForRun({
      runId: "run_1",
      evidencePackId: "ev_run_1",
    });

    expect(invoke).toHaveBeenCalledWith("generate_evidence_pack_for_run", {
      request: {
        runId: "run_1",
        evidencePackId: "ev_run_1",
      },
    });
    expect(evidence.completeness_state).toBe("complete");
  });

  it("maps review decisions through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "ev_run_1",
      task_id: "task_review",
      run_id: "run_1",
      artifact_path: "artifacts/evidence/ev_run_1.json",
      completeness_state: "complete",
      body_json: { review_decision: { decision: "approved" } },
    });

    const evidence = await recordNativeEvidenceReviewDecision({
      evidencePackId: "ev_run_1",
      decision: "reopened",
      reviewerId: "human",
      bodyMd: "Looks complete.",
    });

    expect(invoke).toHaveBeenCalledWith("record_evidence_review_decision", {
      request: {
        evidencePackId: "ev_run_1",
        decision: "reopened",
        reviewerId: "human",
        bodyMd: "Looks complete.",
      },
    });
    expect(evidence.body_json.review_decision).toEqual({ decision: "approved" });
  });
});
