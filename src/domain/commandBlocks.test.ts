import { describe, expect, it } from "vitest";
import {
  buildCommandBlockExplanation,
  exportCommandBlockBundle,
  mergeCommandBlockSummaries,
  mergeCommandBlocks,
  splitCommandBlock,
  updateCommandBlockStatus,
  createCommandBlockState,
  formatCommandBlockForClipboard,
  ingestCommandInput,
  ingestCommandOutput,
  listCommandBlocks,
  manuallyCreateCommandBlock,
  searchCommandBlocks,
} from "./commandBlocks";

describe("command block indexer", () => {
  it("creates a running command block when submitted input ends with enter", () => {
    const result = ingestCommandInput(createCommandBlockState(), {
      sessionId: "pty_1",
      input: "npm test\r",
      cwd: "/repo",
      branch: "main",
    });

    const blocks = listCommandBlocks(result.state);
    expect(result.createdBlock?.command).toBe("npm test");
    expect(blocks).toHaveLength(1);
    expect(blocks[0]).toMatchObject({
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      status: "running",
    });
  });

  it("buffers partial input until enter is received", () => {
    let state = createCommandBlockState();

    let result = ingestCommandInput(state, {
      sessionId: "pty_1",
      input: "npm",
      cwd: "/repo",
      branch: "main",
    });
    expect(result.createdBlock).toBeUndefined();
    state = result.state;

    result = ingestCommandInput(state, {
      sessionId: "pty_1",
      input: " test\n",
      cwd: "/repo",
      branch: "main",
    });

    expect(result.createdBlock?.command).toBe("npm test");
  });

  it("updates sequence range and excerpt from PTY output", () => {
    let state = createCommandBlockState();
    state = ingestCommandInput(state, {
      sessionId: "pty_1",
      input: "npm test\r",
      cwd: "/repo",
      branch: "main",
    }).state;

    const result = ingestCommandOutput(state, {
      sessionId: "pty_1",
      seq: 42,
      chunk: "PASS src/domain/commandBlocks.test.ts\n",
    });
    const [block] = listCommandBlocks(result.state);

    expect(block.seqStart).toBe(42);
    expect(block.seqEnd).toBe(42);
    expect(block.outputExcerpt).toContain("PASS");
    expect(result.updatedBlock).toMatchObject({
      id: block.id,
      outputExcerpt: expect.stringContaining("PASS"),
    });
  });

  it("does not create blocks for empty submissions", () => {
    const result = ingestCommandInput(createCommandBlockState(), {
      sessionId: "pty_1",
      input: "\r",
      cwd: "/repo",
      branch: "main",
    });

    expect(result.createdBlock).toBeUndefined();
    expect(listCommandBlocks(result.state)).toEqual([]);
  });

  it("supports manual command block boundaries when parsing fails", () => {
    const result = manuallyCreateCommandBlock(createCommandBlockState(), {
      sessionId: "pty_1",
      command: "manual boundary",
      cwd: "/repo",
      branch: "main",
      seqStart: 10,
      seqEnd: 15,
    });

    expect(result.createdBlock).toMatchObject({
      command: "manual boundary",
      status: "unknown",
      seqStart: 10,
      seqEnd: 15,
    });
    expect(listCommandBlocks(result.state)).toHaveLength(1);
  });

  it("hydrates persisted command block summaries without replacing local excerpts", () => {
    let state = createCommandBlockState();
    state = manuallyCreateCommandBlock(state, {
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 1,
      seqEnd: 2,
    }).state;
    state = ingestCommandOutput(state, {
      sessionId: "pty_1",
      seq: 3,
      chunk: "PASS local excerpt\n",
    }).state;

    const hydrated = mergeCommandBlockSummaries(state, [
      {
        id: "cmdblk_1",
        sessionId: "session_native",
        command: "npm test",
        status: "completed",
        seqStart: 4,
        seqEnd: 5,
        cwd: "native state snapshot",
        branch: "persisted",
      },
      {
        id: "cmdblk_7",
        sessionId: "session_native",
        command: "cargo test",
        status: "completed",
        seqStart: 12,
        seqEnd: 18,
        cwd: "native state snapshot",
        branch: "persisted",
      },
    ]);

    expect(listCommandBlocks(hydrated)).toEqual([
      expect.objectContaining({
        id: "cmdblk_1",
        sessionId: "session_native",
        status: "completed",
        seqStart: 4,
        seqEnd: 5,
        outputExcerpt: "PASS local excerpt\n",
      }),
      expect.objectContaining({
        id: "cmdblk_7",
        sessionId: "session_native",
        command: "cargo test",
        cwd: "native state snapshot",
        branch: "persisted",
        status: "completed",
      }),
    ]);
    expect(manuallyCreateCommandBlock(hydrated, {
      sessionId: "pty_2",
      command: "npm run build",
      cwd: "/repo",
      branch: "main",
    }).createdBlock?.id).toBe("cmdblk_8");
  });

  it("searches command blocks by command, branch, cwd, and output excerpt", () => {
    let state = createCommandBlockState();
    state = manuallyCreateCommandBlock(state, {
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo/frontend",
      branch: "feature/search",
    }).state;
    state = manuallyCreateCommandBlock(state, {
      sessionId: "pty_1",
      command: "cargo check",
      cwd: "/repo/src-tauri",
      branch: "main",
    }).state;
    state = ingestCommandOutput(state, {
      sessionId: "pty_1",
      seq: 4,
      chunk: "TypeScript diagnostics clean",
    }).state;

    expect(searchCommandBlocks(state, "npm").map((block) => block.command)).toEqual(["npm test"]);
    expect(searchCommandBlocks(state, "src-tauri").map((block) => block.command)).toEqual(["cargo check"]);
    expect(searchCommandBlocks(state, "FEATURE").map((block) => block.command)).toEqual(["npm test"]);
    expect(searchCommandBlocks(state, "diagnostics").map((block) => block.command)).toEqual(["cargo check"]);
  });

  it("formats a command block for clipboard evidence handoff", () => {
    let state = createCommandBlockState();
    state = manuallyCreateCommandBlock(state, {
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 7,
    }).state;
    state = ingestCommandOutput(state, {
      sessionId: "pty_1",
      seq: 9,
      chunk: "PASS src/domain/commandBlocks.test.ts\n",
    }).state;

    expect(formatCommandBlockForClipboard(listCommandBlocks(state)[0])).toBe(
      [
        "$ npm test",
        "cwd: /repo",
        "branch: main",
        "status: unknown",
        "sequence: 7-9",
        "",
        "PASS src/domain/commandBlocks.test.ts",
      ].join("\n"),
    );
  });

  it("builds a local command block explanation without external AI calls", () => {
    const block = {
      id: "cmdblk_42",
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 7,
      seqEnd: 9,
      status: "completed" as const,
      outputExcerpt: "PASS src/App.test.tsx\nTest Files 1 passed",
    };

    expect(buildCommandBlockExplanation(block)).toMatchObject({
      id: "explain_cmdblk_42",
      command: "npm test",
      summary: "npm test completed in /repo on main",
      evidence: expect.arrayContaining([
        "sequence 7-9",
        "output: PASS src/App.test.tsx",
      ]),
    });
  });

  it("builds an AI-routed command block explanation draft without leaking secrets", () => {
    const block = {
      id: "cmdblk_43",
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 10,
      seqEnd: 12,
      status: "failed" as const,
      outputExcerpt: "OPENAI_API_KEY=redacted-secret-fixture\nFAIL src/App.test.tsx",
    };

    const explanation = buildCommandBlockExplanation(block, {
      provider: "openai",
      model: "gpt-5.4",
      agentProfileId: "agent_codex",
    });

    expect(explanation).toMatchObject({
      id: "explain_cmdblk_43",
      command: "npm test",
      provider: "openai",
      model: "gpt-5.4",
      agentProfileId: "agent_codex",
      summary: "AI explanation draft for npm test: failed in /repo on main",
      diagnostics: ["No external AI call has been made; draft is ready for the selected agent."],
      evidence: expect.arrayContaining([
        "ai route openai/gpt-5.4",
        "agent agent_codex",
        "output: OPENAI_API_KEY=[redacted]",
      ]),
    });
    expect(explanation.prompt).toContain("Explain this command block for review evidence.");
    expect(explanation.prompt).not.toContain("redacted-secret-fixture");
  });

  it("exports a shareable command block bundle with evidence-safe metadata", () => {
    const block = {
      id: "cmdblk_42",
      sessionId: "pty_1",
      command: "cargo test",
      cwd: "/repo/src-tauri",
      branch: "feature/blocks",
      seqStart: 3,
      seqEnd: 8,
      status: "failed" as const,
      outputExcerpt: "error: test failed",
    };

    const bundle = exportCommandBlockBundle(block);

    expect(bundle.kind).toBe("haneulchi.command_block_bundle");
    expect(bundle.command_block).toMatchObject({
      id: "cmdblk_42",
      command: "cargo test",
      status: "failed",
    });
    expect(JSON.stringify(bundle)).toContain("error: test failed");
  });

  it("supports manual mark merge and split operations on command blocks", () => {
    let state = createCommandBlockState();
    state = manuallyCreateCommandBlock(state, {
      sessionId: "pty_1",
      command: "npm test",
      cwd: "/repo",
      branch: "main",
      seqStart: 1,
      seqEnd: 2,
    }).state;
    state = ingestCommandOutput(state, {
      sessionId: "pty_1",
      seq: 2,
      chunk: "PASS unit tests\n",
    }).state;
    state = manuallyCreateCommandBlock(state, {
      sessionId: "pty_1",
      command: "cargo test",
      cwd: "/repo/src-tauri",
      branch: "main",
      seqStart: 3,
      seqEnd: 4,
    }).state;
    state = ingestCommandOutput(state, {
      sessionId: "pty_1",
      seq: 4,
      chunk: "PASS rust tests\n",
    }).state;

    const marked = updateCommandBlockStatus(state, "cmdblk_1", "completed");
    expect(marked.updatedBlock?.status).toBe("completed");

    const merged = mergeCommandBlocks(marked.state, "cmdblk_1", "cmdblk_2");
    expect(listCommandBlocks(merged.state)).toHaveLength(1);
    expect(merged.mergedBlock).toMatchObject({
      id: "cmdblk_1",
      command: "npm test && cargo test",
      seqStart: 1,
      seqEnd: 4,
      status: "completed",
      outputExcerpt: expect.stringContaining("PASS rust tests"),
    });

    const split = splitCommandBlock(merged.state, "cmdblk_1");
    expect(listCommandBlocks(split.state).map((block) => block.command)).toEqual([
      "npm test && cargo test (part 1)",
      "npm test && cargo test (part 2)",
    ]);
    expect(split.createdBlock?.id).toBe("cmdblk_3");
  });
});
