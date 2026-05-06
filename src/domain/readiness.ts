export type ReadinessStatus = "ready" | "warning" | "missing";

export interface ReadinessCheck {
  id: string;
  label: string;
  detail: string;
  status: ReadinessStatus;
}

export interface ReadinessSummary {
  ready: number;
  warning: number;
  missing: number;
  total: number;
}

export interface ReadinessSnapshot {
  checks: ReadinessCheck[];
  summary: ReadinessSummary;
}

export const readinessChecks: ReadinessCheck[] = [
  {
    id: "shell",
    label: "Login shell",
    detail: "zsh detected with project PATH inheritance",
    status: "ready",
  },
  {
    id: "git",
    label: "Git",
    detail: "Repository metadata available for worktree tracking",
    status: "ready",
  },
  {
    id: "node",
    label: "Node runtime",
    detail: "Node and npm available for frontend harness runs",
    status: "ready",
  },
  {
    id: "agent-claude",
    label: "Claude Code",
    detail: "CLI adapter not configured yet",
    status: "warning",
  },
  {
    id: "agent-codex",
    label: "Codex CLI",
    detail: "Preset visible, structured adapter pending",
    status: "warning",
  },
  {
    id: "signing",
    label: "Signing identity",
    detail: "Developer ID certificate missing from release gate",
    status: "missing",
  },
  {
    id: "generic-shell",
    label: "Generic Shell fallback",
    detail: "Always available so terminal work can continue",
    status: "ready",
  },
];

export function getReadinessSummary(checks: ReadinessCheck[]): ReadinessSummary {
  return checks.reduce<ReadinessSummary>(
    (summary, check) => {
      summary[check.status] += 1;
      summary.total += 1;
      return summary;
    },
    { ready: 0, warning: 0, missing: 0, total: 0 },
  );
}

export const fallbackReadinessSnapshot: ReadinessSnapshot = {
  checks: readinessChecks,
  summary: getReadinessSummary(readinessChecks),
};
