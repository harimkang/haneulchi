import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface TerminalPtySessionSnapshot {
  id: string;
  title: string;
  command: string;
  cols: number;
  rows: number;
}

export interface TerminalPtySnapshot {
  total: number;
  sessions: TerminalPtySessionSnapshot[];
}

export interface CaptureTerminalPtyRequest {
  command: string;
  args: string[];
  cols: number;
  rows: number;
}

export interface SpawnTerminalPtyRequest {
  title: string;
  command: string;
  args: string[];
  cols: number;
  rows: number;
}

export interface TerminalPtyOutputEvent {
  sessionId: string;
  seq: number;
  chunk: string;
}

export interface PtyCommandCapture {
  output: string;
  exitCode: number;
  exitSuccess: boolean;
}

export const fallbackTerminalPtySnapshot: TerminalPtySnapshot = {
  total: 0,
  sessions: [],
};

export function getTerminalPtySnapshot(): Promise<TerminalPtySnapshot> {
  return invoke<TerminalPtySnapshot>("get_terminal_pty_snapshot");
}

export function captureTerminalPtyCommand(request: CaptureTerminalPtyRequest): Promise<PtyCommandCapture> {
  return invoke<PtyCommandCapture>("capture_terminal_pty_command", { request });
}

export function spawnTerminalPtySession(request: SpawnTerminalPtyRequest): Promise<TerminalPtySessionSnapshot> {
  return invoke<TerminalPtySessionSnapshot>("spawn_terminal_pty_session", { request });
}

export function writeTerminalPtyInput(id: string, input: string): Promise<void> {
  return invoke<void>("write_terminal_pty_input", { id, input });
}

export function resizeTerminalPtySession(id: string, cols: number, rows: number): Promise<void> {
  return invoke<void>("resize_terminal_pty_session", { id, cols, rows });
}

export function closeTerminalPtySession(id: string): Promise<void> {
  return invoke<void>("close_terminal_pty_session", { id });
}

export function listenToTerminalPtyOutput(handler: (event: TerminalPtyOutputEvent) => void): Promise<UnlistenFn> {
  return listen<TerminalPtyOutputEvent>("terminal://pty-output", (event) => {
    handler(event.payload);
  });
}
