import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  captureTerminalPtyCommand,
  closeTerminalPtySession,
  getTerminalPtySnapshot,
  listenToTerminalPtyOutput,
  resizeTerminalPtySession,
  spawnTerminalPtySession,
  writeTerminalPtyInput,
} from "./terminalPtyClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

describe("terminal PTY client", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("loads the PTY manager snapshot from the Tauri command surface", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ total: 0, sessions: [] });

    await expect(getTerminalPtySnapshot()).resolves.toEqual({ total: 0, sessions: [] });
    expect(invoke).toHaveBeenCalledWith("get_terminal_pty_snapshot");
  });

  it("captures one-shot PTY command output through Tauri", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      output: "haneulchi",
      exitCode: 0,
      exitSuccess: true,
    });

    await expect(
      captureTerminalPtyCommand({
        command: "sh",
        args: ["-lc", "printf haneulchi"],
        cols: 80,
        rows: 24,
      }),
    ).resolves.toEqual({
      output: "haneulchi",
      exitCode: 0,
      exitSuccess: true,
    });
    expect(invoke).toHaveBeenCalledWith("capture_terminal_pty_command", {
      request: {
        command: "sh",
        args: ["-lc", "printf haneulchi"],
        cols: 80,
        rows: 24,
      },
    });
  });

  it("spawns live PTY sessions through Tauri", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "pty_1",
      title: "shell",
      command: "zsh",
      cols: 120,
      rows: 32,
    });

    await expect(
      spawnTerminalPtySession({
        title: "shell",
        command: "zsh",
        args: [],
        cols: 120,
        rows: 32,
      }),
    ).resolves.toEqual({
      id: "pty_1",
      title: "shell",
      command: "zsh",
      cols: 120,
      rows: 32,
    });
    expect(invoke).toHaveBeenCalledWith("spawn_terminal_pty_session", {
      request: {
        title: "shell",
        command: "zsh",
        args: [],
        cols: 120,
        rows: 32,
      },
    });
  });

  it("writes input to live PTY sessions through Tauri", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await expect(writeTerminalPtyInput("pty_1", "npm test\r")).resolves.toBeUndefined();
    expect(invoke).toHaveBeenCalledWith("write_terminal_pty_input", {
      id: "pty_1",
      input: "npm test\r",
    });
  });

  it("resizes and closes live PTY sessions through Tauri", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await expect(resizeTerminalPtySession("pty_1", 132, 40)).resolves.toBeUndefined();
    await expect(closeTerminalPtySession("pty_1")).resolves.toBeUndefined();

    expect(invoke).toHaveBeenNthCalledWith(1, "resize_terminal_pty_session", {
      id: "pty_1",
      cols: 132,
      rows: 40,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "close_terminal_pty_session", {
      id: "pty_1",
    });
  });

  it("subscribes to sequenced PTY output events", async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockImplementationOnce(async (_eventName, handler) => {
      handler({
        event: "terminal://pty-output",
        id: 1,
        payload: {
          sessionId: "pty_1",
          seq: 7,
          chunk: "hello",
        },
      });
      return unlisten;
    });
    const handler = vi.fn();

    await expect(listenToTerminalPtyOutput(handler)).resolves.toBe(unlisten);

    expect(listen).toHaveBeenCalledWith("terminal://pty-output", expect.any(Function));
    expect(handler).toHaveBeenCalledWith({
      sessionId: "pty_1",
      seq: 7,
      chunk: "hello",
    });
  });
});
