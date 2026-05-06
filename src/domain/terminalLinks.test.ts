import { describe, expect, it } from "vitest";
import { extractTerminalLinks } from "./terminalLinks";

describe("terminal link safety", () => {
  it("extracts HTTP(S) terminal links without trailing punctuation", () => {
    const links = extractTerminalLinks(["server ready at http://localhost:3000/docs.", "docs: https://example.com/path)"]);

    expect(links).toEqual([
      {
        url: "http://localhost:3000/docs",
        status: "safe",
        reason: undefined,
      },
      {
        url: "https://example.com/path",
        status: "safe",
        reason: undefined,
      },
    ]);
  });

  it("blocks dangerous terminal link schemes", () => {
    const links = extractTerminalLinks(["blocked javascript:alert", "blocked file:///etc/passwd"]);

    expect(links).toEqual([
      {
        url: "javascript:alert",
        status: "blocked",
        reason: "scheme not allowed",
      },
      {
        url: "file:///etc/passwd",
        status: "blocked",
        reason: "scheme not allowed",
      },
    ]);
  });
});
