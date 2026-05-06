import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  attachNativeSessionTask,
  createNativeSession,
  detachNativeSessionTask,
  focusNativeSession,
  killNativeSession,
  launchNativeAgentTerminal,
  listNativeTerminalStreamChunks,
  listNativeSessions,
  recordNativeSessionInput,
  recordNativeTerminalStreamChunk,
  releaseNativeSession,
  resizeNativeSession,
  setNativeSessionAttention,
  takeoverNativeSession,
} from "./sessionApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("session API client", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("maps session create and list through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "session_1", title: "Codex AUTH-104" })
      .mockResolvedValueOnce([{ id: "session_1", title: "Codex AUTH-104" }]);

    const created = await createNativeSession({
      projectId: "proj_local",
      mode: "agent",
      title: "Codex AUTH-104",
      cwd: "/repo/auth-service",
      branch: "fix/auth",
      agentProfileId: "agent_codex",
    });
    const sessions = await listNativeSessions("proj_local");

    expect(invoke).toHaveBeenNthCalledWith(1, "create_session", {
      request: {
        projectId: "proj_local",
        mode: "agent",
        title: "Codex AUTH-104",
        cwd: "/repo/auth-service",
        branch: "fix/auth",
        agentProfileId: "agent_codex",
        taskId: undefined,
        runId: undefined,
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_sessions", {
      projectId: "proj_local",
    });
    expect(created.id).toBe("session_1");
    expect(sessions[0].title).toBe("Codex AUTH-104");
  });

  it("maps focus, input, takeover, release, and kill through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "session_1", attention_state: "none" })
      .mockResolvedValueOnce({ session_id: "session_1", accepted: true, dangerous: true })
      .mockResolvedValueOnce({ id: "session_1", attention_state: "needs_input" })
      .mockResolvedValueOnce({ id: "session_1", attention_state: "none", state: "running" })
      .mockResolvedValueOnce({ id: "session_1", state: "completed" });

    await focusNativeSession("session_1");
    const receipt = await recordNativeSessionInput({
      sessionId: "session_1",
      text: "rm -rf /tmp/build",
      allowDangerous: true,
    });
    await takeoverNativeSession("session_1");
    const released = await releaseNativeSession("session_1");
    const killed = await killNativeSession("session_1");

    expect(invoke).toHaveBeenNthCalledWith(1, "focus_session", { sessionId: "session_1" });
    expect(invoke).toHaveBeenNthCalledWith(2, "record_session_input", {
      request: {
        sessionId: "session_1",
        text: "rm -rf /tmp/build",
        allowDangerous: true,
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "takeover_session", { sessionId: "session_1" });
    expect(invoke).toHaveBeenNthCalledWith(4, "release_session", { sessionId: "session_1" });
    expect(invoke).toHaveBeenNthCalledWith(5, "kill_session", { sessionId: "session_1" });
    expect(receipt.dangerous).toBe(true);
    expect(released.attention_state).toBe("none");
    expect(killed.state).toBe("completed");
  });

  it("maps session attention updates through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ id: "session_1", attention_state: "unread" });

    const session = await setNativeSessionAttention("session_1", "unread");

    expect(invoke).toHaveBeenCalledWith("set_session_attention", {
      sessionId: "session_1",
      attentionState: "unread",
    });
    expect(session.attention_state).toBe("unread");
  });

  it("maps session resize through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ session_id: "session_1", cols: 132, rows: 40 });

    const receipt = await resizeNativeSession("session_1", 132, 40);

    expect(invoke).toHaveBeenCalledWith("resize_session", {
      sessionId: "session_1",
      cols: 132,
      rows: 40,
    });
    expect(receipt.rows).toBe(40);
  });

  it("maps task attach and detach through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "session_1", task_id: "task_104" })
      .mockResolvedValueOnce({ id: "session_1", task_id: null });

    const attached = await attachNativeSessionTask("session_1", "task_104");
    const detached = await detachNativeSessionTask("session_1");

    expect(invoke).toHaveBeenNthCalledWith(1, "attach_session_task", {
      sessionId: "session_1",
      taskId: "task_104",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "detach_session_task", {
      sessionId: "session_1",
    });
    expect(attached.task_id).toBe("task_104");
    expect(detached.task_id).toBeNull();
  });

  it("maps terminal stream chunk record and list through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "terminal_stream_chunk_1",
        session_id: "session_1",
        seq_start: 1,
        seq_end: 4,
        body: "npm test\n88 passed\n",
      })
      .mockResolvedValueOnce([
        {
          id: "terminal_stream_chunk_1",
          session_id: "session_1",
          seq_start: 1,
          seq_end: 4,
          body: "npm test\n88 passed\n",
        },
      ]);

    const chunk = await recordNativeTerminalStreamChunk({
      sessionId: "session_1",
      seqStart: 1,
      seqEnd: 4,
      body: "npm test\n88 passed\n",
    });
    const chunks = await listNativeTerminalStreamChunks("session_1", 10);

    expect(invoke).toHaveBeenNthCalledWith(1, "record_terminal_stream_chunk", {
      request: {
        sessionId: "session_1",
        seqStart: 1,
        seqEnd: 4,
        body: "npm test\n88 passed\n",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_terminal_stream_chunks", {
      sessionId: "session_1",
      limit: 10,
    });
    expect(chunk.id).toBe("terminal_stream_chunk_1");
    expect(chunks[0].body).toContain("88 passed");
  });

  it("launches raw agent terminal sessions through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      session: { id: "session_agent", agent_profile_id: "agent_acme", mode: "agent" },
      pty_session: { id: "pty_agent", command: "acme-agent" },
    });

    const launched = await launchNativeAgentTerminal({
      projectId: "proj_auth",
      agentProfileId: "agent_acme",
      title: "Acme CLI raw agent",
      cols: 120,
      rows: 34,
    });

    expect(invoke).toHaveBeenCalledWith("launch_agent_terminal", {
      request: {
        projectId: "proj_auth",
        agentProfileId: "agent_acme",
        title: "Acme CLI raw agent",
        cols: 120,
        rows: 34,
      },
    });
    expect(launched.session.agent_profile_id).toBe("agent_acme");
    expect(launched.pty_session.command).toBe("acme-agent");
  });
});
