import { describe, expect, it } from "vitest";
import type { CommandBlock } from "./commandBlocks";
import { attachCommandBlockToEvidencePack, createEvidencePack } from "./evidence";

const commandBlock: CommandBlock = {
  id: "cmdblk_1",
  sessionId: "pty_1",
  command: "npm test",
  cwd: "/repo",
  branch: "main",
  seqStart: 4,
  seqEnd: 8,
  status: "running",
  outputExcerpt: "PASS commandBlocks.test.ts",
};

describe("evidence pack", () => {
  it("attaches a command block snapshot to an evidence pack", () => {
    const pack = attachCommandBlockToEvidencePack(createEvidencePack("ev_local"), commandBlock);

    expect(pack.commandBlocks).toEqual([
      {
        id: "cmdblk_1",
        sessionId: "pty_1",
        command: "npm test",
        cwd: "/repo",
        branch: "main",
        seqStart: 4,
        seqEnd: 8,
        status: "running",
        outputExcerpt: "PASS commandBlocks.test.ts",
      },
    ]);
  });

  it("does not attach duplicate command blocks", () => {
    const pack = attachCommandBlockToEvidencePack(
      attachCommandBlockToEvidencePack(createEvidencePack("ev_local"), commandBlock),
      commandBlock,
    );

    expect(pack.commandBlocks).toHaveLength(1);
  });

  it("does not mutate the original evidence pack", () => {
    const original = createEvidencePack("ev_local");
    const updated = attachCommandBlockToEvidencePack(original, commandBlock);

    expect(original.commandBlocks).toEqual([]);
    expect(updated.commandBlocks).toHaveLength(1);
  });
});
