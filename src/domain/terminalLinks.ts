export type TerminalLinkStatus = "safe" | "blocked";

export interface TerminalLink {
  url: string;
  status: TerminalLinkStatus;
  reason?: string;
}

const terminalLinkPattern = /\b(?:https?:\/\/|javascript:|data:|file:|mailto:|tel:)[^\s<>"'`]+/gi;
const safeTerminalLinkProtocols = new Set(["http:", "https:"]);

export function extractTerminalLinks(lines: string[]): TerminalLink[] {
  const links: TerminalLink[] = [];
  const seen = new Set<string>();

  for (const line of lines) {
    for (const match of line.matchAll(terminalLinkPattern)) {
      const url = trimTerminalLinkToken(match[0]);
      if (!url || seen.has(url)) continue;
      seen.add(url);
      links.push(classifyTerminalLink(url));
    }
  }

  return links;
}

function classifyTerminalLink(url: string): TerminalLink {
  try {
    const parsed = new URL(url);
    if (!safeTerminalLinkProtocols.has(parsed.protocol)) {
      return { url, status: "blocked", reason: "scheme not allowed" };
    }
    return { url: parsed.toString(), status: "safe", reason: undefined };
  } catch {
    return { url, status: "blocked", reason: "invalid URL" };
  }
}

function trimTerminalLinkToken(value: string): string {
  return value.replace(/[.,;!?)\]}]+$/g, "");
}
