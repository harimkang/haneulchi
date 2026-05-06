import { describe, expect, it } from "vitest";
import {
  appendTerminalOutput,
  bindPtySession,
  createTerminalSession,
  degradeRendererForHealthEvent,
  getRendererHealthLabel,
  markRendererDegraded,
  terminalSessions,
} from "./terminal";

describe("terminal session model", () => {
  it("starts terminal sessions with WebGL as the preferred renderer", () => {
    const session = createTerminalSession({
      id: "ses-test",
      title: "1. haneulchi (zsh)",
      cwd: "/repo",
      branch: "main",
      seedLines: ["$ npm test"],
    });

    expect(session.renderer.kind).toBe("webgl");
    expect(getRendererHealthLabel(session.renderer)).toBe("WebGL");
  });

  it("marks renderer fallback without dropping existing terminal output", () => {
    const session = createTerminalSession({
      id: "ses-fallback",
      title: "2. fallback",
      cwd: "/repo",
      branch: "main",
      seedLines: ["raw terminal output"],
    });

    const degraded = markRendererDegraded(session, "WebGL context unavailable");

    expect(degraded.renderer.kind).toBe("canvas");
    expect(degraded.renderer.reason).toBe("WebGL context unavailable");
    expect(degraded.lines).toEqual(["raw terminal output"]);
  });

  it("records renderer health degradation reasons for runtime failures", () => {
    const session = createTerminalSession({
      id: "ses-health",
      title: "2. health",
      cwd: "/repo",
      branch: "main",
      seedLines: ["stream stays readable"],
    });

    const degraded = degradeRendererForHealthEvent(session, {
      kind: "webgl-context-lost",
      detail: "context lost while scrolling",
    });

    expect(degraded.renderer.kind).toBe("canvas");
    expect(degraded.renderer.degraded).toBe(true);
    expect(degraded.renderer.reason).toBe("Renderer degraded: WebGL context lost · context lost while scrolling");
    expect(degraded.lines).toEqual(["stream stays readable"]);
  });

  it("keeps terminal output bounded for pane previews", () => {
    const session = createTerminalSession({
      id: "ses-output",
      title: "3. logs",
      cwd: "/repo",
      branch: "main",
      seedLines: ["first"],
    });

    const updated = appendTerminalOutput(session, ["second", "third", "fourth"], 3);

    expect(updated.lines).toEqual(["second", "third", "fourth"]);
  });

  it("binds frontend sessions to live PTY sessions without changing display identity", () => {
    const session = createTerminalSession({
      id: "ses-bind",
      title: "4. shell",
      cwd: "/repo",
      branch: "main",
    });

    const bound = bindPtySession(session, "pty_7");

    expect(bound.id).toBe("ses-bind");
    expect(bound.ptyId).toBe("pty_7");
  });

  it("ships four starter terminal sessions for the 2x2 deck", () => {
    expect(terminalSessions).toHaveLength(4);
    expect(terminalSessions.map((session) => session.id)).toEqual([
      "ses-shell",
      "ses-agent",
      "ses-preview",
      "ses-release",
    ]);
  });
});
