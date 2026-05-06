import { invoke } from "@tauri-apps/api/core";
import type { CommandBlock } from "../domain/commandBlocks";

export interface NativeCommandBlock {
  id: string;
  session_id: string;
  task_id: string | null;
  run_id: string | null;
  seq_start: number | null;
  seq_end: number | null;
  command: string;
  cwd: string | null;
  branch: string | null;
  exit_code: number | null;
  duration_ms: number | null;
  summary: string | null;
}

export interface NativeEvidencePack {
  id: string;
  task_id: string | null;
  run_id: string | null;
  artifact_path: string;
  completeness_state: string;
  body_json: {
    command_blocks?: unknown[];
    [key: string]: unknown;
  };
}

export interface NativeCommandBlockExplanation {
  id: string;
  command_block_id: string;
  command: string;
  summary: string;
  evidence: string[];
  provider: string | null;
  model: string | null;
  agent_profile_id: string | null;
  prompt: string | null;
  diagnostics: string[];
}

export interface NativeCommandBlockBundle {
  kind: "haneulchi.command_block_bundle";
  version: number;
  exported_at: string;
  command_block: NativeCommandBlock;
  explanation: NativeCommandBlockExplanation;
}

export interface NativeCommandBlockSplitReceipt {
  updated_block: NativeCommandBlock;
  created_block: NativeCommandBlock;
  snapshot_id?: string;
}

interface AttachNativeCommandBlockEvidenceInput {
  evidencePackId: string;
  commandBlockId: string;
  taskId?: string;
  runId?: string;
}

interface ExplainNativeCommandBlockInput {
  provider?: string;
  model?: string;
  agentProfileId?: string;
}

interface GenerateNativeEvidencePackInput {
  runId: string;
  evidencePackId?: string;
}

interface RecordNativeEvidenceReviewDecisionInput {
  evidencePackId: string;
  decision: "approved" | "changes_requested" | "reopened" | "blocked";
  reviewerId?: string;
  bodyMd?: string;
}

export function upsertNativeCommandBlock(block: CommandBlock): Promise<NativeCommandBlock> {
  return invoke<NativeCommandBlock>("upsert_command_block", {
    request: {
      id: block.id,
      sessionId: block.sessionId,
      taskId: undefined,
      runId: undefined,
      seqStart: block.seqStart,
      seqEnd: block.seqEnd,
      command: block.command,
      cwd: block.cwd,
      branch: block.branch,
      exitCode: statusToExitCode(block.status),
      durationMs: undefined,
      summary: block.outputExcerpt.trim() || undefined,
    },
  });
}

export function searchNativeCommandBlocks(query?: string, limit = 50): Promise<NativeCommandBlock[]> {
  return invoke<NativeCommandBlock[]>("search_command_blocks", {
    query: query?.trim() || undefined,
    limit,
  });
}

export function markNativeCommandBlock(commandBlockId: string, status: CommandBlock["status"]): Promise<NativeCommandBlock> {
  return invoke<NativeCommandBlock>("mark_command_block", {
    commandBlockId,
    status,
  });
}

export function mergeNativeCommandBlocks(
  firstCommandBlockId: string,
  secondCommandBlockId: string,
): Promise<NativeCommandBlock> {
  return invoke<NativeCommandBlock>("merge_command_blocks", {
    firstCommandBlockId,
    secondCommandBlockId,
  });
}

export function splitNativeCommandBlock(commandBlockId: string): Promise<NativeCommandBlockSplitReceipt> {
  return invoke<NativeCommandBlockSplitReceipt>("split_command_block", {
    commandBlockId,
  });
}

export function nativeCommandBlockToCommandBlock(block: NativeCommandBlock): CommandBlock {
  return {
    id: block.id,
    sessionId: block.session_id,
    command: block.command,
    cwd: block.cwd ?? "native command history",
    branch: block.branch ?? "persisted",
    seqStart: block.seq_start ?? undefined,
    seqEnd: block.seq_end ?? undefined,
    status: exitCodeToStatus(block.exit_code),
    outputExcerpt: block.summary ?? "",
  };
}

export function attachNativeCommandBlockToEvidence(
  input: AttachNativeCommandBlockEvidenceInput,
): Promise<NativeEvidencePack> {
  return invoke<NativeEvidencePack>("attach_command_block_to_evidence", {
    request: {
      evidencePackId: input.evidencePackId,
      commandBlockId: input.commandBlockId,
      taskId: input.taskId,
      runId: input.runId,
    },
  });
}

export function explainNativeCommandBlock(
  commandBlockId: string,
  input: ExplainNativeCommandBlockInput,
): Promise<NativeCommandBlockExplanation> {
  return invoke<NativeCommandBlockExplanation>("explain_command_block", {
    commandBlockId,
    request: {
      provider: input.provider,
      model: input.model,
      agentProfileId: input.agentProfileId,
    },
  });
}

export function exportNativeCommandBlockBundle(commandBlockId: string): Promise<NativeCommandBlockBundle> {
  return invoke<NativeCommandBlockBundle>("export_command_block_bundle", {
    commandBlockId,
  });
}

export function generateNativeEvidencePackForRun(
  input: GenerateNativeEvidencePackInput,
): Promise<NativeEvidencePack> {
  return invoke<NativeEvidencePack>("generate_evidence_pack_for_run", {
    request: {
      runId: input.runId,
      evidencePackId: input.evidencePackId,
    },
  });
}

export function recordNativeEvidenceReviewDecision(
  input: RecordNativeEvidenceReviewDecisionInput,
): Promise<NativeEvidencePack> {
  return invoke<NativeEvidencePack>("record_evidence_review_decision", {
    request: {
      evidencePackId: input.evidencePackId,
      decision: input.decision,
      reviewerId: input.reviewerId,
      bodyMd: input.bodyMd,
    },
  });
}

function statusToExitCode(status: CommandBlock["status"]): number | undefined {
  if (status === "completed") return 0;
  if (status === "failed") return 1;
  return undefined;
}

function exitCodeToStatus(exitCode: number | null): CommandBlock["status"] {
  if (exitCode === 0) return "completed";
  if (typeof exitCode === "number") return "failed";
  return "unknown";
}
