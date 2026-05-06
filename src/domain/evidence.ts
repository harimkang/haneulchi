import type { CommandBlock } from "./commandBlocks";

export interface EvidenceCommandBlock {
  id: string;
  sessionId: string;
  command: string;
  cwd: string;
  branch: string;
  seqStart?: number;
  seqEnd?: number;
  status: CommandBlock["status"];
  outputExcerpt: string;
}

export interface EvidencePack {
  id: string;
  taskId: string | null;
  runId: string | null;
  workflowVersion: string | null;
  contextSources: string[];
  diffSummary: Record<string, unknown>;
  commandBlocks: EvidenceCommandBlock[];
  tests: string[];
  screenshots: string[];
  transcriptSummary: string;
  tokenUsage: Record<string, unknown>;
  policyEvents: string[];
  reviewDecision: string | null;
}

export function createEvidencePack(id: string): EvidencePack {
  return {
    id,
    taskId: null,
    runId: null,
    workflowVersion: null,
    contextSources: [],
    diffSummary: {},
    commandBlocks: [],
    tests: [],
    screenshots: [],
    transcriptSummary: "",
    tokenUsage: {},
    policyEvents: [],
    reviewDecision: null,
  };
}

export function attachCommandBlockToEvidencePack(pack: EvidencePack, block: CommandBlock): EvidencePack {
  if (pack.commandBlocks.some((candidate) => candidate.id === block.id)) {
    return pack;
  }

  return {
    ...pack,
    commandBlocks: [
      ...pack.commandBlocks,
      {
        id: block.id,
        sessionId: block.sessionId,
        command: block.command,
        cwd: block.cwd,
        branch: block.branch,
        seqStart: block.seqStart,
        seqEnd: block.seqEnd,
        status: block.status,
        outputExcerpt: block.outputExcerpt,
      },
    ],
  };
}
