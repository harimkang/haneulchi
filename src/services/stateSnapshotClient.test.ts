import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HC_DEFAULT_TERMINAL_THEME } from "../design/haneulchiDesignTokens";
import { createTaskState } from "../domain/tasks";
import {
  fallbackStateSnapshot,
  getStateSnapshot,
  mergeCommandBlocksIntoStateSnapshot,
  mergeTasksIntoStateSnapshot,
} from "./stateSnapshotClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("state snapshot client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads the spec-shaped state snapshot from Tauri", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      snapshot_id: "snap_20260430_010000Z_1",
      generated_at: "2026-04-30T01:00:00Z",
      app: { version: "0.1.0", renderer: "xterm-webgl", update_state: "current" },
      projects: [],
      project_tabs: [],
      sessions: [{ id: "pty_1", title: "shell", mode: "shell", state: "running" }],
      command_blocks: { recent: [], unread_count: 0 },
      tasks: { items: [], counts_by_status: {} },
      runs: { items: [], counts_by_lifecycle: {} },
      agents: [],
      reviews: [],
      attention: [],
      budgets: { workspace: {}, projects: [], agents: [] },
      workflow: { valid: true, invalid_projects: [] },
      knowledge: { stale_count: 0, gap_count: 0, recent_pages: [] },
      health: { db: "degraded", pty: "ok", api: "ok" },
    });

    await expect(getStateSnapshot()).resolves.toMatchObject({
      snapshot_id: "snap_20260430_010000Z_1",
      sessions: [expect.objectContaining({ id: "pty_1", mode: "shell" })],
      health: { db: "degraded", pty: "ok", api: "ok" },
    });
    expect(invoke).toHaveBeenCalledWith("get_state_snapshot");
  });

  it("loads a project-scoped state snapshot from Tauri", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      ...fallbackStateSnapshot,
      snapshot_id: "snap_proj_auth",
      tasks: {
        items: [{ id: "task_auth", title: "Implement auth", status: "ready", priority: "high", project_id: "proj_auth" }],
        counts_by_status: { ready: 1 },
      },
    });

    await expect(getStateSnapshot("proj_auth")).resolves.toMatchObject({
      snapshot_id: "snap_proj_auth",
      tasks: {
        items: [expect.objectContaining({ id: "task_auth", project_id: "proj_auth" })],
      },
    });
    expect(invoke).toHaveBeenCalledWith("get_state_snapshot", { projectId: "proj_auth" });
  });

  it("keeps an explicit degraded fallback when Tauri state is unavailable", () => {
    expect(fallbackStateSnapshot.snapshot_id).toBe("snap_fallback");
    expect(fallbackStateSnapshot.app.terminal_theme).toEqual({
      project_id: null,
      ...HC_DEFAULT_TERMINAL_THEME,
    });
    expect(fallbackStateSnapshot.health.api).toBe("degraded");
    expect(fallbackStateSnapshot.attention).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "state-api-unavailable",
          severity: "warning",
        }),
      ]),
    );
  });

  it("derives recent command block summaries for the state snapshot contract", () => {
    const snapshot = mergeCommandBlocksIntoStateSnapshot(fallbackStateSnapshot, [
      {
        id: "cmdblk_1",
        sessionId: "pty_1",
        command: "npm test",
        cwd: "/repo",
        branch: "main",
        seqStart: 4,
        seqEnd: 9,
        status: "running",
        outputExcerpt: "PASS",
      },
    ]);

    expect(snapshot.command_blocks.unread_count).toBe(1);
    expect(snapshot.command_blocks.recent).toEqual([
      {
        id: "cmdblk_1",
        session_id: "pty_1",
        command: "npm test",
        status: "running",
        seq_start: 4,
        seq_end: 9,
      },
    ]);
  });

  it("preserves native command block summaries when the local command log is empty", () => {
    const snapshot = mergeCommandBlocksIntoStateSnapshot(
      {
        ...fallbackStateSnapshot,
        command_blocks: {
          recent: [
            {
              id: "cmdblk_native",
              session_id: "session_native",
              command: "cargo test",
              status: "completed",
              seq_start: 12,
              seq_end: 18,
            },
          ],
          unread_count: 1,
        },
      },
      [],
    );

    expect(snapshot.command_blocks.unread_count).toBe(1);
    expect(snapshot.command_blocks.recent).toEqual([
      {
        id: "cmdblk_native",
        session_id: "session_native",
        command: "cargo test",
        status: "completed",
        seq_start: 12,
        seq_end: 18,
      },
    ]);
  });

  it("merges local command block summaries with the native snapshot without duplicate ids", () => {
    const snapshot = mergeCommandBlocksIntoStateSnapshot(
      {
        ...fallbackStateSnapshot,
        command_blocks: {
          recent: [
            {
              id: "cmdblk_1",
              session_id: "session_native",
              command: "npm test",
              status: "running",
              seq_start: 1,
              seq_end: 2,
            },
            {
              id: "cmdblk_native",
              session_id: "session_native",
              command: "cargo test",
              status: "completed",
              seq_start: 12,
              seq_end: 18,
            },
          ],
          unread_count: 2,
        },
      },
      [
        {
          id: "cmdblk_1",
          sessionId: "session_local",
          command: "npm test",
          cwd: "/repo",
          branch: "main",
          seqStart: 3,
          seqEnd: 4,
          status: "completed",
          outputExcerpt: "PASS",
        },
        {
          id: "cmdblk_2",
          sessionId: "session_local",
          command: "npm run build",
          cwd: "/repo",
          branch: "main",
          seqStart: 5,
          seqEnd: 8,
          status: "running",
          outputExcerpt: "building",
        },
      ],
    );

    expect(snapshot.command_blocks.unread_count).toBe(3);
    expect(snapshot.command_blocks.recent).toEqual([
      expect.objectContaining({ id: "cmdblk_1", session_id: "session_local", status: "completed", seq_start: 3 }),
      expect.objectContaining({ id: "cmdblk_native", session_id: "session_native", status: "completed" }),
      expect.objectContaining({ id: "cmdblk_2", session_id: "session_local", status: "running" }),
    ]);
  });

  it("derives task items and board counts for the state snapshot contract", () => {
    const taskState = createTaskState([
      { id: "task_1", title: "Review evidence pack", status: "review", priority: "high", projectId: "proj_local" },
      { id: "task_2", title: "Fix packaging gate", status: "blocked", priority: "urgent", projectId: "proj_local" },
    ]);

    const snapshot = mergeTasksIntoStateSnapshot(fallbackStateSnapshot, taskState);

    expect(snapshot.tasks.items).toEqual([
      expect.objectContaining({ id: "task_1", title: "Review evidence pack", status: "review" }),
      expect.objectContaining({ id: "task_2", title: "Fix packaging gate", status: "blocked" }),
    ]);
    expect(snapshot.tasks.counts_by_status).toMatchObject({
      inbox: 0,
      ready: 0,
      running: 0,
      review: 1,
      blocked: 1,
      done: 0,
    });
  });

  it("includes task drawer metadata in state snapshot task summaries", () => {
    const taskState = createTaskState([
      {
        id: "task_1",
        title: "Review evidence pack",
        status: "review",
        priority: "high",
        projectId: "proj_local",
        cycle: "Sprint 5",
        module: "Control API",
        dueDate: "2026-05-15",
        estimate: "3 pts",
        labels: ["release", "evidence"],
        subtasks: [
          { id: "subtask_1", title: "Attach screenshots", status: "open" },
          { id: "subtask_2", title: "Confirm terminal proof", status: "done" },
        ],
        workpad: "Attach latest command evidence",
        contextPackId: "ctx_release",
        comments: [
          { id: "comment_1", author: "human", body: "Needs screenshot evidence." },
          { id: "comment_2", author: "agent_codex", body: "Command blocks attached." },
        ],
      },
    ]);

    const snapshot = mergeTasksIntoStateSnapshot(fallbackStateSnapshot, taskState);

    expect(snapshot.tasks.items[0]).toMatchObject({
      id: "task_1",
      comment_count: 2,
      has_workpad: true,
      cycle: "Sprint 5",
      module: "Control API",
      due_at: "2026-05-15",
      estimate: "3 pts",
      labels: ["release", "evidence"],
      subtask_count: 2,
      open_subtask_count: 1,
      context_pack_id: "ctx_release",
    });
  });
});
