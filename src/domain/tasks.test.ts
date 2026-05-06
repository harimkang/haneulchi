import { describe, expect, it } from "vitest";
import {
  addTask,
  addTaskComment,
  addTaskSubtask,
  advanceTask,
  countTasksByStatus,
  createTaskState,
  filterTasks,
  getTaskOverview,
  listTasks,
  moveTask,
  taskBoardStatuses,
  updateTaskSubtaskStatus,
  updateTaskWorkpad,
  updateTaskPlanningProperties,
} from "./tasks";

describe("task management model", () => {
  it("counts tasks by the MVP board statuses", () => {
    const state = createTaskState([
      { id: "task_1", title: "Inbox item", status: "inbox", priority: "medium", projectId: "proj_local" },
      { id: "task_2", title: "Ready item", status: "ready", priority: "high", projectId: "proj_local" },
      { id: "task_3", title: "Blocked item", status: "blocked", priority: "urgent", projectId: "proj_local" },
    ]);

    expect(taskBoardStatuses).toEqual(["inbox", "ready", "running", "review", "blocked", "done"]);
    expect(countTasksByStatus(state)).toMatchObject({
      inbox: 1,
      ready: 1,
      running: 0,
      review: 0,
      blocked: 1,
      done: 0,
    });
  });

  it("moves a task between board statuses without mutating the previous state", () => {
    const state = createTaskState([
      { id: "task_1", title: "Dispatchable task", status: "ready", priority: "high", projectId: "proj_local" },
    ]);

    const moved = moveTask(state, "task_1", "running");

    expect(state.tasks.task_1.status).toBe("ready");
    expect(moved.tasks.task_1.status).toBe("running");
    expect(countTasksByStatus(moved)).toMatchObject({ ready: 0, running: 1 });
  });

  it("quick-creates trimmed inbox tasks with deterministic ids", () => {
    const state = createTaskState([]);
    const result = addTask(state, {
      title: "  Write task persistence tests  ",
      projectId: "proj_local",
    });

    expect(result.createdTask).toMatchObject({
      id: "task_1",
      title: "Write task persistence tests",
      status: "inbox",
      priority: "medium",
      projectId: "proj_local",
    });
    expect(listTasks(result.state)).toEqual([result.createdTask]);
  });

  it("does not create empty quick tasks", () => {
    const result = addTask(createTaskState([]), {
      title: "   ",
      projectId: "proj_local",
    });

    expect(result.createdTask).toBeUndefined();
    expect(listTasks(result.state)).toEqual([]);
  });

  it("advances tasks through the MVP board flow", () => {
    let state = createTaskState([
      { id: "task_1", title: "Plan task", status: "inbox", priority: "medium", projectId: "proj_local" },
    ]);

    state = advanceTask(state, "task_1");
    expect(state.tasks.task_1.status).toBe("ready");

    state = advanceTask(state, "task_1");
    expect(state.tasks.task_1.status).toBe("running");

    state = advanceTask(state, "task_1");
    expect(state.tasks.task_1.status).toBe("review");

    state = advanceTask(state, "task_1");
    expect(state.tasks.task_1.status).toBe("done");

    expect(advanceTask(state, "task_1").tasks.task_1.status).toBe("done");
  });

  it("keeps blocked tasks blocked when advancing", () => {
    const state = createTaskState([
      { id: "task_1", title: "Blocked task", status: "blocked", priority: "urgent", projectId: "proj_local" },
    ]);

    expect(advanceTask(state, "task_1").tasks.task_1.status).toBe("blocked");
  });

  it("filters tasks by title, status, priority, assignee, and labels", () => {
    const state = createTaskState([
      { id: "task_1", title: "Fix packaging gate", status: "blocked", priority: "urgent", projectId: "proj_local" },
      { id: "task_2", title: "Review terminal proof", status: "review", priority: "high", projectId: "proj_local", assignee: "agent_codex", labels: ["release", "evidence"] },
    ]);

    expect(filterTasks(state, "packaging").map((task) => task.id)).toEqual(["task_1"]);
    expect(filterTasks(state, "blocked").map((task) => task.id)).toEqual(["task_1"]);
    expect(filterTasks(state, "urgent").map((task) => task.id)).toEqual(["task_1"]);
    expect(filterTasks(state, "codex").map((task) => task.id)).toEqual(["task_2"]);
    expect(filterTasks(state, "evidence").map((task) => task.id)).toEqual(["task_2"]);
  });

  it("builds task drawer overview with workpad and comments metadata", () => {
    const state = createTaskState([
      {
        id: "task_1",
        title: "Review evidence workflow",
        status: "review",
        priority: "high",
        projectId: "proj_local",
        description: "Confirm evidence pack attachments before review.",
        labels: ["release", "evidence"],
        subtasks: [
          { id: "subtask_1", title: "Attach screenshots", status: "open" },
          { id: "subtask_2", title: "Confirm terminal proof", status: "done" },
        ],
        workpad: "## Notes\n- Check command blocks",
        comments: [
          { id: "comment_1", author: "human", body: "Needs screenshot evidence." },
          { id: "comment_2", author: "agent_codex", body: "Command blocks attached." },
        ],
      },
    ]);

    expect(getTaskOverview(state, "task_1")).toEqual({
      id: "task_1",
      title: "Review evidence workflow",
      status: "review",
      priority: "high",
      assignee: undefined,
      description: "Confirm evidence pack attachments before review.",
      labels: ["release", "evidence"],
      subtaskCount: 2,
      openSubtaskCount: 1,
      subtasks: [
        { id: "subtask_1", title: "Attach screenshots", status: "open" },
        { id: "subtask_2", title: "Confirm terminal proof", status: "done" },
      ],
      workpad: "## Notes\n- Check command blocks",
      commentCount: 2,
      comments: [
        { id: "comment_1", author: "human", body: "Needs screenshot evidence." },
        { id: "comment_2", author: "agent_codex", body: "Command blocks attached." },
      ],
    });
  });

  it("updates task workpad markdown without mutating previous state", () => {
    const state = createTaskState([
      {
        id: "task_1",
        title: "Review evidence workflow",
        status: "review",
        priority: "high",
        projectId: "proj_local",
        workpad: "old notes",
      },
    ]);

    const updated = updateTaskWorkpad(state, "task_1", "## Notes\n- Attach latest proof");

    expect(state.tasks.task_1.workpad).toBe("old notes");
    expect(updated.tasks.task_1.workpad).toBe("## Notes\n- Attach latest proof");
  });

  it("adds trimmed task comments with deterministic ids and ignores blank comments", () => {
    const state = createTaskState([
      {
        id: "task_1",
        title: "Review evidence workflow",
        status: "review",
        priority: "high",
        projectId: "proj_local",
        comments: [{ id: "comment_2", author: "agent_codex", body: "Command blocks attached." }],
      },
    ]);

    const result = addTaskComment(state, "task_1", {
      author: "human",
      body: "  Needs screenshot evidence.  ",
    });

    expect(result.createdComment).toEqual({
      id: "comment_3",
      author: "human",
      body: "Needs screenshot evidence.",
    });
    expect(state.tasks.task_1.comments).toHaveLength(1);
    expect(result.state.tasks.task_1.comments).toHaveLength(2);
    expect(addTaskComment(result.state, "task_1", { author: "human", body: "   " }).createdComment).toBeUndefined();
  });

  it("adds and completes subtasks without mutating previous state", () => {
    const state = createTaskState([
      {
        id: "task_1",
        title: "Review evidence workflow",
        status: "review",
        priority: "high",
        projectId: "proj_local",
        subtasks: [{ id: "subtask_2", title: "Attach command block", status: "open" }],
      },
    ]);

    const added = addTaskSubtask(state, "task_1", { title: "  Confirm screenshots  " });
    const completed = updateTaskSubtaskStatus(added.state, "task_1", "subtask_3", "done");

    expect(added.createdSubtask).toEqual({
      id: "subtask_3",
      title: "Confirm screenshots",
      status: "open",
    });
    expect(state.tasks.task_1.subtasks).toHaveLength(1);
    expect(completed.tasks.task_1.subtasks).toContainEqual({
      id: "subtask_3",
      title: "Confirm screenshots",
      status: "done",
    });
    expect(addTaskSubtask(completed, "task_1", { title: "   " }).createdSubtask).toBeUndefined();
  });

  it("updates lightweight planning and assignee properties without mutating previous state", () => {
    const state = createTaskState([
      {
        id: "task_1",
        title: "Wire state snapshot",
        status: "ready",
        priority: "high",
        projectId: "proj_local",
        cycle: "Sprint 3",
        module: "State",
        assignee: "agent_claude",
      },
    ]);

    const updated = updateTaskPlanningProperties(state, "task_1", {
      cycle: "Sprint 5",
      module: "Control API",
      labels: [" release ", "evidence", "", "release"],
      dueDate: "  2026-05-15  ",
      estimate: " 3 pts ",
      assignee: "agent_codex",
    } as Parameters<typeof updateTaskPlanningProperties>[2]);

    expect(state.tasks.task_1.cycle).toBe("Sprint 3");
    expect(state.tasks.task_1.assignee).toBe("agent_claude");
    expect(updated.tasks.task_1).toMatchObject({
      cycle: "Sprint 5",
      module: "Control API",
      labels: ["release", "evidence"],
      dueDate: "2026-05-15",
      estimate: "3 pts",
      assignee: "agent_codex",
    });
    expect(updateTaskPlanningProperties(updated, "task_1", { cycle: "   ", module: "", labels: [], dueDate: " ", estimate: " ", assignee: " " } as Parameters<typeof updateTaskPlanningProperties>[2]).tasks.task_1).toMatchObject({
      cycle: undefined,
      module: undefined,
      labels: undefined,
      dueDate: undefined,
      estimate: undefined,
      assignee: undefined,
    });
  });
});
