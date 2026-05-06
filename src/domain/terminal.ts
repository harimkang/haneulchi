export type TerminalSessionStatus = "ready" | "warning" | "missing";
export type TerminalRendererKind = "webgl" | "canvas" | "dom";

export interface TerminalRendererHealth {
  kind: TerminalRendererKind;
  degraded: boolean;
  reason?: string;
}

export type TerminalRendererHealthEventKind =
  | "webgl-context-lost"
  | "frame-stall"
  | "dropped-chunks"
  | "ime-failure"
  | "throughput-failure";

export interface TerminalRendererHealthEvent {
  kind: TerminalRendererHealthEventKind;
  detail?: string;
}

export interface TerminalSession {
  id: string;
  ptyId?: string;
  title: string;
  cwd: string;
  branch: string;
  status: TerminalSessionStatus;
  renderer: TerminalRendererHealth;
  lines: string[];
}

interface CreateTerminalSessionInput {
  id: string;
  ptyId?: string;
  title: string;
  cwd: string;
  branch: string;
  status?: TerminalSessionStatus;
  renderer?: TerminalRendererHealth;
  seedLines?: string[];
}

const defaultRenderer: TerminalRendererHealth = {
  kind: "webgl",
  degraded: false,
};

export function createTerminalSession(input: CreateTerminalSessionInput): TerminalSession {
  return {
    id: input.id,
    ptyId: input.ptyId,
    title: input.title,
    cwd: input.cwd,
    branch: input.branch,
    status: input.status ?? "ready",
    renderer: input.renderer ?? defaultRenderer,
    lines: input.seedLines ?? [],
  };
}

export function markRendererDegraded(session: TerminalSession, reason: string): TerminalSession {
  return {
    ...session,
    renderer: {
      kind: "canvas",
      degraded: true,
      reason,
    },
  };
}

export function degradeRendererForHealthEvent(session: TerminalSession, event: TerminalRendererHealthEvent): TerminalSession {
  const label = rendererHealthEventLabel(event.kind);
  const detail = event.detail?.trim();
  return markRendererDegraded(session, `Renderer degraded: ${label}${detail ? ` · ${detail}` : ""}`);
}

export function appendTerminalOutput(session: TerminalSession, output: string[], maxPreviewLines = 80): TerminalSession {
  return {
    ...session,
    lines: [...session.lines, ...output].slice(-maxPreviewLines),
  };
}

export function bindPtySession(session: TerminalSession, ptyId: string): TerminalSession {
  return {
    ...session,
    ptyId,
  };
}

export function getRendererHealthLabel(renderer: TerminalRendererHealth): string {
  if (renderer.kind === "webgl") return "WebGL";
  if (renderer.kind === "canvas") return "Fallback";
  return "DOM";
}

function rendererHealthEventLabel(kind: TerminalRendererHealthEventKind): string {
  switch (kind) {
    case "webgl-context-lost":
      return "WebGL context lost";
    case "frame-stall":
      return "frame stalls detected";
    case "dropped-chunks":
      return "dropped terminal chunks";
    case "ime-failure":
      return "IME composition failure";
    case "throughput-failure":
      return "high-throughput rendering failure";
  }
}

export const terminalSessions: TerminalSession[] = [
  createTerminalSession({
    id: "ses-shell",
    title: "1. haneulchi (zsh)",
    cwd: "~/develop/applications/haneulchi",
    branch: "main",
    seedLines: ["~/develop/applications/haneulchi main", "$ npm test", "2 test files queued for terminal proof", "ready: shell/git/node"],
  }),
  createTerminalSession({
    id: "ses-agent",
    title: "2. codex-agent",
    cwd: "~/develop/applications/haneulchi",
    branch: "main",
    status: "warning",
    seedLines: ["agent preset: Codex CLI", "adapter: pending structured events", "fallback: raw PTY session", "attention: request review gate"],
  }),
  createTerminalSession({
    id: "ses-preview",
    title: "3. preview server",
    cwd: "~/develop/applications/haneulchi",
    branch: "main",
    seedLines: ["vite dev server reserved", "localhost preview surface linked", "browser pane: ready for Sprint 2", "logs indexed as command blocks"],
  }),
  createTerminalSession({
    id: "ses-release",
    title: "4. release gate",
    cwd: "~/develop/applications/haneulchi",
    branch: "main",
    status: "missing",
    renderer: {
      kind: "canvas",
      degraded: true,
      reason: "DMG bundling gate is not cleared",
    },
    seedLines: ["signing identity: missing", "notarization: follow-up", "DMG pipeline: tracked", "generic shell remains available"],
  }),
];
