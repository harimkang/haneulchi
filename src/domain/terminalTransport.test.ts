import { describe, expect, it } from "vitest";
import { createTerminalTransportState, ingestTerminalPtyOutput } from "./terminalTransport";

describe("terminal transport reducer", () => {
  it("releases contiguous chunks in sequence order", () => {
    let state = createTerminalTransportState();

    let result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 1, chunk: "a" });
    expect(result.releasedChunks).toEqual(["a"]);
    state = result.state;

    result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 2, chunk: "b" });
    expect(result.releasedChunks).toEqual(["b"]);
    expect(result.state.sessions.pty_1.nextSeq).toBe(3);
  });

  it("buffers out-of-order chunks until the gap arrives", () => {
    let state = createTerminalTransportState();

    let result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 2, chunk: "b" });
    expect(result.releasedChunks).toEqual([]);
    state = result.state;

    result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 1, chunk: "a" });
    expect(result.releasedChunks).toEqual(["a", "b"]);
    expect(result.state.sessions.pty_1.nextSeq).toBe(3);
  });

  it("drops duplicate or stale chunks", () => {
    let state = createTerminalTransportState();

    let result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 1, chunk: "a" });
    state = result.state;
    result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 1, chunk: "duplicate" });

    expect(result.releasedChunks).toEqual([]);
    expect(result.state.sessions.pty_1.nextSeq).toBe(2);
  });

  it("keeps independent ordering per PTY session", () => {
    let state = createTerminalTransportState();

    let result = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 2, chunk: "b1" });
    state = result.state;
    result = ingestTerminalPtyOutput(state, { sessionId: "pty_2", seq: 1, chunk: "a2" });
    state = result.state;

    expect(result.releasedChunks).toEqual(["a2"]);
    expect(state.sessions.pty_1.nextSeq).toBe(1);
    expect(state.sessions.pty_2.nextSeq).toBe(2);
  });

  it("bounds pending out-of-order chunks per session", () => {
    let state = createTerminalTransportState({ maxPendingPerSession: 2 });

    state = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 4, chunk: "d" }).state;
    state = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 3, chunk: "c" }).state;
    state = ingestTerminalPtyOutput(state, { sessionId: "pty_1", seq: 2, chunk: "b" }).state;

    expect(Object.keys(state.sessions.pty_1.pending).map(Number).sort()).toEqual([2, 3]);
    expect(state.sessions.pty_1.droppedChunks).toBe(1);
  });
});
