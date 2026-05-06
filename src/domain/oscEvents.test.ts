import { describe, expect, it } from "vitest";
import { createOscEventState, parseOscSequences } from "./oscEvents";

describe("OSC event parser", () => {
  it("extracts allowlisted OSC 9, 99, and 777 events while stripping them from display output", () => {
    const result = parseOscSequences(createOscEventState(), "before \u001b]777;agent.completed:{\"ok\":true}\u0007 after");

    expect(result.displayText).toBe("before  after");
    expect(result.events).toEqual([
      {
        code: "777",
        payload: 'agent.completed:{"ok":true}',
      },
    ]);
    expect(result.state.allowedEvents).toBe(1);
    expect(result.state.ignoredEvents).toBe(0);
  });

  it("ignores unknown OSC codes without exposing the payload as terminal text", () => {
    const result = parseOscSequences(createOscEventState(), "safe\u001b]1337;<script>alert(1)</script>\u0007text");

    expect(result.displayText).toBe("safetext");
    expect(result.events).toEqual([]);
    expect(result.state.allowedEvents).toBe(0);
    expect(result.state.ignoredEvents).toBe(1);
  });

  it("rejects oversized allowlisted payloads", () => {
    const result = parseOscSequences(createOscEventState({ maxPayloadBytes: 8 }), `\u001b]9;${"x".repeat(9)}\u0007`);

    expect(result.displayText).toBe("");
    expect(result.events).toEqual([]);
    expect(result.state.allowedEvents).toBe(0);
    expect(result.state.rejectedEvents).toBe(1);
  });

  it("keeps incomplete OSC sequences buffered until the terminator arrives", () => {
    const partial = parseOscSequences(createOscEventState(), "a\u001b]99;partial");
    expect(partial.displayText).toBe("a");
    expect(partial.state.pending).toBe("\u001b]99;partial");

    const completed = parseOscSequences(partial.state, " event\u001b\\b");
    expect(completed.displayText).toBe("b");
    expect(completed.events).toEqual([{ code: "99", payload: "partial event" }]);
    expect(completed.state.pending).toBe("");
  });
});
