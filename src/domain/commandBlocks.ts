import type { TerminalPtyOutputEvent } from "../services/terminalPtyClient";

export type CommandBlockStatus = "running" | "completed" | "failed" | "unknown";

export interface CommandBlock {
  id: string;
  sessionId: string;
  command: string;
  cwd: string;
  branch: string;
  seqStart?: number;
  seqEnd?: number;
  status: CommandBlockStatus;
  outputExcerpt: string;
}

export interface CommandBlockState {
  blocks: Record<string, CommandBlock>;
  activeBlockBySession: Record<string, string>;
  inputBufferBySession: Record<string, string>;
  nextId: number;
}

export interface CommandBlockExplanation {
  id: string;
  commandBlockId: string;
  command: string;
  summary: string;
  evidence: string[];
  provider?: string;
  model?: string;
  agentProfileId?: string;
  prompt?: string;
  diagnostics?: string[];
}

export interface CommandBlockExplanationSettings {
  provider: string;
  model: string;
  agentProfileId: string;
}

export interface CommandBlockBundle {
  kind: "haneulchi.command_block_bundle";
  version: 1;
  exported_at: string;
  command_block: CommandBlock;
  explanation: CommandBlockExplanation;
}

interface CommandInputEvent {
  sessionId: string;
  input: string;
  cwd: string;
  branch: string;
}

interface CommandInputResult {
  state: CommandBlockState;
  createdBlock?: CommandBlock;
}

interface ManualCommandBlockInput {
  sessionId: string;
  command: string;
  cwd: string;
  branch: string;
  seqStart?: number;
  seqEnd?: number;
}

interface CommandBlockSummaryInput {
  id: string;
  sessionId: string;
  command: string;
  status: string;
  seqStart?: number;
  seqEnd?: number;
  cwd?: string;
  branch?: string;
  outputExcerpt?: string;
}

export function createCommandBlockState(): CommandBlockState {
  return {
    blocks: {},
    activeBlockBySession: {},
    inputBufferBySession: {},
    nextId: 1,
  };
}

export function ingestCommandInput(state: CommandBlockState, event: CommandInputEvent): CommandInputResult {
  const existingBuffer = state.inputBufferBySession[event.sessionId] ?? "";
  const combined = existingBuffer + event.input;
  const submitIndex = findSubmitIndex(combined);

  if (submitIndex === -1) {
    return {
      state: {
        ...state,
        inputBufferBySession: {
          ...state.inputBufferBySession,
          [event.sessionId]: combined,
        },
      },
    };
  }

  const submitted = combined.slice(0, submitIndex).trim();
  const remaining = combined.slice(submitIndex + 1);

  if (submitted.length === 0) {
    return {
      state: {
        ...state,
        inputBufferBySession: {
          ...state.inputBufferBySession,
          [event.sessionId]: remaining,
        },
      },
    };
  }

  const block: CommandBlock = {
    id: `cmdblk_${state.nextId}`,
    sessionId: event.sessionId,
    command: submitted,
    cwd: event.cwd,
    branch: event.branch,
    status: "running",
    outputExcerpt: "",
  };

  return {
    state: {
      ...state,
      blocks: {
        ...state.blocks,
        [block.id]: block,
      },
      activeBlockBySession: {
        ...state.activeBlockBySession,
        [event.sessionId]: block.id,
      },
      inputBufferBySession: {
        ...state.inputBufferBySession,
        [event.sessionId]: remaining,
      },
      nextId: state.nextId + 1,
    },
    createdBlock: block,
  };
}

export function ingestCommandOutput(
  state: CommandBlockState,
  event: TerminalPtyOutputEvent,
): { state: CommandBlockState; updatedBlock?: CommandBlock } {
  const activeBlockId = state.activeBlockBySession[event.sessionId];
  if (!activeBlockId) return { state };

  const block = state.blocks[activeBlockId];
  if (!block) return { state };

  const updatedBlock: CommandBlock = {
    ...block,
    seqStart: block.seqStart ?? event.seq,
    seqEnd: event.seq,
    outputExcerpt: appendExcerpt(block.outputExcerpt, event.chunk),
  };

  return {
    state: {
      ...state,
      blocks: {
        ...state.blocks,
        [updatedBlock.id]: updatedBlock,
      },
    },
    updatedBlock,
  };
}

export function manuallyCreateCommandBlock(state: CommandBlockState, input: ManualCommandBlockInput): CommandInputResult {
  const block: CommandBlock = {
    id: `cmdblk_${state.nextId}`,
    sessionId: input.sessionId,
    command: input.command,
    cwd: input.cwd,
    branch: input.branch,
    seqStart: input.seqStart,
    seqEnd: input.seqEnd,
    status: "unknown",
    outputExcerpt: "",
  };

  return {
    state: {
      ...state,
      blocks: {
        ...state.blocks,
        [block.id]: block,
      },
      activeBlockBySession: {
        ...state.activeBlockBySession,
        [input.sessionId]: block.id,
      },
      nextId: state.nextId + 1,
    },
    createdBlock: block,
  };
}

export function listCommandBlocks(state: CommandBlockState): CommandBlock[] {
  return Object.values(state.blocks).sort((a, b) => numericId(a.id) - numericId(b.id));
}

export function searchCommandBlocks(state: CommandBlockState, query: string): CommandBlock[] {
  const normalized = query.trim().toLowerCase();
  const blocks = listCommandBlocks(state);
  if (normalized.length === 0) return blocks;

  return blocks.filter((block) =>
    [block.command, block.cwd, block.branch, block.outputExcerpt]
      .join("\n")
      .toLowerCase()
      .includes(normalized),
  );
}

export function mergeCommandBlockSummaries(
  state: CommandBlockState,
  summaries: CommandBlockSummaryInput[],
): CommandBlockState {
  if (summaries.length === 0) return state;

  const blocks = { ...state.blocks };
  let nextId = state.nextId;

  summaries.forEach((summary) => {
    const existing = blocks[summary.id];
    blocks[summary.id] = {
      id: summary.id,
      sessionId: summary.sessionId,
      command: summary.command,
      cwd: summary.cwd ?? existing?.cwd ?? "native state snapshot",
      branch: summary.branch ?? existing?.branch ?? "persisted",
      seqStart: summary.seqStart ?? existing?.seqStart,
      seqEnd: summary.seqEnd ?? existing?.seqEnd,
      status: normalizeCommandBlockStatus(summary.status),
      outputExcerpt: summary.outputExcerpt ?? existing?.outputExcerpt ?? "",
    };

    const idNumber = numericId(summary.id);
    if (Number.isFinite(idNumber)) {
      nextId = Math.max(nextId, idNumber + 1);
    }
  });

  return {
    ...state,
    blocks,
    nextId,
  };
}

export function updateCommandBlockStatus(
  state: CommandBlockState,
  blockId: string,
  status: CommandBlockStatus,
): { state: CommandBlockState; updatedBlock?: CommandBlock } {
  const block = state.blocks[blockId];
  if (!block) return { state };

  const updatedBlock = {
    ...block,
    status,
  };

  return {
    state: {
      ...state,
      blocks: {
        ...state.blocks,
        [blockId]: updatedBlock,
      },
    },
    updatedBlock,
  };
}

export function mergeCommandBlocks(
  state: CommandBlockState,
  firstBlockId: string,
  secondBlockId: string,
): { state: CommandBlockState; mergedBlock?: CommandBlock } {
  const first = state.blocks[firstBlockId];
  const second = state.blocks[secondBlockId];
  if (!first || !second || first.id === second.id) return { state };

  const mergedBlock: CommandBlock = {
    ...first,
    command: `${first.command} && ${second.command}`,
    seqStart: minDefined(first.seqStart, second.seqStart),
    seqEnd: maxDefined(first.seqEnd, second.seqEnd),
    status: mergeStatuses(first.status, second.status),
    outputExcerpt: [first.outputExcerpt.trim(), second.outputExcerpt.trim()].filter(Boolean).join("\n\n"),
  };
  const remainingBlocks = { ...state.blocks };
  delete remainingBlocks[secondBlockId];

  return {
    state: {
      ...state,
      blocks: {
        ...remainingBlocks,
        [firstBlockId]: mergedBlock,
      },
      activeBlockBySession: Object.fromEntries(
        Object.entries(state.activeBlockBySession).map(([sessionId, activeBlockId]) => [
          sessionId,
          activeBlockId === secondBlockId ? firstBlockId : activeBlockId,
        ]),
      ),
    },
    mergedBlock,
  };
}

export function splitCommandBlock(
  state: CommandBlockState,
  blockId: string,
): { state: CommandBlockState; updatedBlock?: CommandBlock; createdBlock?: CommandBlock } {
  const block = state.blocks[blockId];
  if (!block) return { state };

  const nextId = `cmdblk_${state.nextId}`;
  const splitSeq = splitSequenceRange(block.seqStart, block.seqEnd);
  const [firstExcerpt, secondExcerpt] = splitExcerpt(block.outputExcerpt);
  const updatedBlock: CommandBlock = {
    ...block,
    command: `${block.command} (part 1)`,
    seqStart: splitSeq.firstStart,
    seqEnd: splitSeq.firstEnd,
    outputExcerpt: firstExcerpt,
  };
  const createdBlock: CommandBlock = {
    ...block,
    id: nextId,
    command: `${block.command} (part 2)`,
    seqStart: splitSeq.secondStart,
    seqEnd: splitSeq.secondEnd,
    outputExcerpt: secondExcerpt,
  };

  return {
    state: {
      ...state,
      blocks: {
        ...state.blocks,
        [blockId]: updatedBlock,
        [nextId]: createdBlock,
      },
      activeBlockBySession: Object.fromEntries(
        Object.entries(state.activeBlockBySession).map(([sessionId, activeBlockId]) => [
          sessionId,
          activeBlockId === blockId ? nextId : activeBlockId,
        ]),
      ),
      nextId: state.nextId + 1,
    },
    updatedBlock,
    createdBlock,
  };
}

export function formatCommandBlockForClipboard(block: CommandBlock): string {
  const lines = [
    `$ ${block.command}`,
    `cwd: ${block.cwd}`,
    `branch: ${block.branch}`,
    `status: ${block.status}`,
    `sequence: ${block.seqStart ?? "-"}-${block.seqEnd ?? "-"}`,
  ];
  const excerpt = block.outputExcerpt.trim();

  if (excerpt.length > 0) {
    lines.push("", excerpt);
  }

  return lines.join("\n");
}

export function buildCommandBlockExplanation(
  block: CommandBlock,
  settings?: CommandBlockExplanationSettings,
): CommandBlockExplanation {
  const sequence = `${block.seqStart ?? "-"}-${block.seqEnd ?? "-"}`;
  const redactedOutput = redactCommandBlockSecrets(block.outputExcerpt);
  const firstOutputLine = redactedOutput.trim().split(/\r?\n/).find((line) => line.trim().length > 0);
  const localSummary = `${block.command} ${block.status} in ${block.cwd} on ${block.branch}`;
  const evidence = [
    `sequence ${sequence}`,
    `status ${block.status}`,
    firstOutputLine ? `output: ${firstOutputLine.trim()}` : "output: none captured",
  ];

  if (!settings) {
    return {
      id: `explain_${block.id}`,
      commandBlockId: block.id,
      command: block.command,
      summary: localSummary,
      evidence,
    };
  }

  return {
    id: `explain_${block.id}`,
    commandBlockId: block.id,
    command: block.command,
    summary: `AI explanation draft for ${block.command}: ${block.status} in ${block.cwd} on ${block.branch}`,
    evidence: [
      `ai route ${settings.provider}/${settings.model}`,
      `agent ${settings.agentProfileId}`,
      `context: ${localSummary}`,
      ...evidence,
    ],
    provider: settings.provider,
    model: settings.model,
    agentProfileId: settings.agentProfileId,
    prompt: [
      "Explain this command block for review evidence.",
      `Command: ${block.command}`,
      `Status: ${block.status}`,
      `Cwd: ${block.cwd}`,
      `Branch: ${block.branch}`,
      `sequence ${sequence}`,
      `Output excerpt:\n${redactedOutput.trim() || "none captured"}`,
    ].join("\n"),
    diagnostics: ["No external AI call has been made; draft is ready for the selected agent."],
  };
}

export function exportCommandBlockBundle(block: CommandBlock): CommandBlockBundle {
  return {
    kind: "haneulchi.command_block_bundle",
    version: 1,
    exported_at: new Date().toISOString(),
    command_block: block,
    explanation: buildCommandBlockExplanation(block),
  };
}

function findSubmitIndex(input: string): number {
  const carriage = input.indexOf("\r");
  const newline = input.indexOf("\n");
  if (carriage === -1) return newline;
  if (newline === -1) return carriage;
  return Math.min(carriage, newline);
}

function appendExcerpt(existing: string, chunk: string): string {
  return `${existing}${chunk}`.slice(-500);
}

function redactCommandBlockSecrets(value: string): string {
  return value
    .replace(/\b([A-Z0-9_]*(?:API_KEY|TOKEN|SECRET|PASSWORD))=([^\s]+)/gi, "$1=[redacted]")
    .replace(/\bsk-[A-Za-z0-9_-]{8,}\b/g, "sk-[redacted]");
}

function minDefined(first?: number, second?: number): number | undefined {
  if (first === undefined) return second;
  if (second === undefined) return first;
  return Math.min(first, second);
}

function maxDefined(first?: number, second?: number): number | undefined {
  if (first === undefined) return second;
  if (second === undefined) return first;
  return Math.max(first, second);
}

function mergeStatuses(first: CommandBlockStatus, second: CommandBlockStatus): CommandBlockStatus {
  if (first === "failed" || second === "failed") return "failed";
  if (first === "running" || second === "running") return "running";
  if (first === "completed" || second === "completed") return "completed";
  return "unknown";
}

function normalizeCommandBlockStatus(status: string): CommandBlockStatus {
  if (status === "running" || status === "completed" || status === "failed") return status;
  return "unknown";
}

function splitSequenceRange(seqStart?: number, seqEnd?: number) {
  if (seqStart === undefined || seqEnd === undefined || seqEnd <= seqStart) {
    return {
      firstStart: seqStart,
      firstEnd: seqStart,
      secondStart: seqEnd,
      secondEnd: seqEnd,
    };
  }
  const midpoint = Math.floor((seqStart + seqEnd) / 2);
  return {
    firstStart: seqStart,
    firstEnd: midpoint,
    secondStart: midpoint + 1,
    secondEnd: seqEnd,
  };
}

function splitExcerpt(excerpt: string): [string, string] {
  const lines = excerpt.split(/\r?\n/).filter((line) => line.length > 0);
  if (lines.length <= 1) return [excerpt, ""];
  const midpoint = Math.ceil(lines.length / 2);
  return [lines.slice(0, midpoint).join("\n"), lines.slice(midpoint).join("\n")];
}

function numericId(id: string): number {
  return Number(id.replace("cmdblk_", ""));
}
