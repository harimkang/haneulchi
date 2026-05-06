import { invoke } from "@tauri-apps/api/core";

export interface NativeSession {
  id: string;
  project_id: string;
  pane_id: string | null;
  mode: string;
  title: string;
  cwd: string | null;
  branch: string | null;
  agent_profile_id: string | null;
  task_id: string | null;
  run_id: string | null;
  state: string;
  attention_state: string;
  token_budget_state: string;
  created_at: string;
  updated_at: string;
}

export interface NativeSessionInputReceipt {
  session_id: string;
  accepted: boolean;
  dangerous: boolean;
  input_len: number;
  command_block_id: string | null;
}

export interface NativeSessionResizeReceipt {
  session_id: string;
  pane_id: string | null;
  cols: number;
  rows: number;
  snapshot_id?: string;
}

export interface NativeAgentTerminalLaunch {
  session: NativeSession;
  pty_session: {
    id: string;
    title: string;
    command: string;
    cols: number;
    rows: number;
  };
}

export interface NativeTerminalStreamChunk {
  id: string;
  session_id: string;
  seq_start: number;
  seq_end: number;
  artifact_path: string;
  body: string;
  created_at: string;
}

interface CreateNativeSessionInput {
  projectId: string;
  mode: string;
  title: string;
  cwd?: string;
  branch?: string;
  agentProfileId?: string;
  taskId?: string;
  runId?: string;
}

interface RecordNativeSessionInput {
  sessionId: string;
  text: string;
  allowDangerous: boolean;
}

interface RecordNativeTerminalStreamChunkInput {
  sessionId: string;
  seqStart: number;
  seqEnd: number;
  body: string;
}

interface LaunchNativeAgentTerminalInput {
  projectId: string;
  agentProfileId: string;
  title?: string;
  cols?: number;
  rows?: number;
}

export function createNativeSession(input: CreateNativeSessionInput): Promise<NativeSession> {
  return invoke<NativeSession>("create_session", {
    request: {
      projectId: input.projectId,
      mode: input.mode,
      title: input.title,
      cwd: input.cwd,
      branch: input.branch,
      agentProfileId: input.agentProfileId,
      taskId: input.taskId,
      runId: input.runId,
    },
  });
}

export function listNativeSessions(projectId: string): Promise<NativeSession[]> {
  return invoke<NativeSession[]>("list_sessions", {
    projectId,
  });
}

export function focusNativeSession(sessionId: string): Promise<NativeSession> {
  return invoke<NativeSession>("focus_session", { sessionId });
}

export function setNativeSessionAttention(sessionId: string, attentionState: string): Promise<NativeSession> {
  return invoke<NativeSession>("set_session_attention", { sessionId, attentionState });
}

export function resizeNativeSession(
  sessionId: string,
  cols: number,
  rows: number,
): Promise<NativeSessionResizeReceipt> {
  return invoke<NativeSessionResizeReceipt>("resize_session", { sessionId, cols, rows });
}

export function takeoverNativeSession(sessionId: string): Promise<NativeSession> {
  return invoke<NativeSession>("takeover_session", { sessionId });
}

export function releaseNativeSession(sessionId: string): Promise<NativeSession> {
  return invoke<NativeSession>("release_session", { sessionId });
}

export function killNativeSession(sessionId: string): Promise<NativeSession> {
  return invoke<NativeSession>("kill_session", { sessionId });
}

export function attachNativeSessionTask(sessionId: string, taskId: string): Promise<NativeSession> {
  return invoke<NativeSession>("attach_session_task", { sessionId, taskId });
}

export function detachNativeSessionTask(sessionId: string): Promise<NativeSession> {
  return invoke<NativeSession>("detach_session_task", { sessionId });
}

export function recordNativeSessionInput(input: RecordNativeSessionInput): Promise<NativeSessionInputReceipt> {
  return invoke<NativeSessionInputReceipt>("record_session_input", {
    request: {
      sessionId: input.sessionId,
      text: input.text,
      allowDangerous: input.allowDangerous,
    },
  });
}

export function recordNativeTerminalStreamChunk(
  input: RecordNativeTerminalStreamChunkInput,
): Promise<NativeTerminalStreamChunk> {
  return invoke<NativeTerminalStreamChunk>("record_terminal_stream_chunk", {
    request: {
      sessionId: input.sessionId,
      seqStart: input.seqStart,
      seqEnd: input.seqEnd,
      body: input.body,
    },
  });
}

export function listNativeTerminalStreamChunks(
  sessionId: string,
  limit?: number,
): Promise<NativeTerminalStreamChunk[]> {
  return invoke<NativeTerminalStreamChunk[]>("list_terminal_stream_chunks", {
    sessionId,
    limit,
  });
}

export function launchNativeAgentTerminal(input: LaunchNativeAgentTerminalInput): Promise<NativeAgentTerminalLaunch> {
  return invoke<NativeAgentTerminalLaunch>("launch_agent_terminal", {
    request: {
      projectId: input.projectId,
      agentProfileId: input.agentProfileId,
      title: input.title,
      cols: input.cols,
      rows: input.rows,
    },
  });
}
