import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  addNativeTaskComment,
  addNativeTaskSubtask,
  createNativeTaskCycle,
  createNativeTaskModule,
  createNativeReviewFollowUpTask,
  createNativeTask,
  listNativeTaskComments,
  listNativeTaskCycles,
  listNativeTaskModules,
  listNativeTaskSubtasks,
  listNativeTasks,
  moveNativeTask,
  nativeTaskToHaneulchiTask,
  saveNativeTaskWorkpad,
  updateNativeTaskSubtaskStatus,
  updateNativeTaskContext,
  updateNativeTaskPlanning,
} from "./taskApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("task API client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("maps native task commands through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([
        {
          id: "task_1",
          key: "LOCAL-1",
          project_id: "proj_local",
          title: "Persisted task",
          description: null,
          status: "ready",
          priority: "high",
          assignee_type: "agent",
          assignee_id: "agent_codex",
          cycle_id: "cycle_sprint_5",
          module_id: "module_control_api",
          initiative_id: "init_platform",
          due_at: "2026-05-15",
          estimate: "3 pts",
          labels: ["release", "evidence"],
        },
      ])
      .mockResolvedValueOnce({
        id: "task_2",
        key: "LOCAL-2",
        project_id: "proj_local",
        title: "New task",
        description: null,
        status: "inbox",
        priority: "medium",
        assignee_type: null,
        assignee_id: null,
        cycle_id: null,
        module_id: null,
      })
      .mockResolvedValueOnce({
        id: "task_2",
        key: "LOCAL-2",
        project_id: "proj_local",
        title: "New task",
        description: null,
        status: "ready",
        priority: "medium",
        assignee_type: null,
        assignee_id: null,
        cycle_id: null,
        module_id: null,
      })
      .mockResolvedValueOnce({
        id: "comment_1",
        task_id: "task_2",
        run_id: null,
        author_type: "human",
        author_id: "local_user",
        body_md: "Native comment",
        parent_id: null,
      })
      .mockResolvedValueOnce([
        {
          id: "comment_1",
          task_id: "task_2",
          run_id: null,
          author_type: "human",
          author_id: "local_user",
          body_md: "Native comment",
          parent_id: null,
        },
      ])
      .mockResolvedValueOnce({
        id: "workpad_task_2",
        task_id: "task_2",
        artifact_path: "artifacts/workpads/task_2.md",
        title: "Task workpad",
        body_md: "Native workpad",
      })
      .mockResolvedValueOnce({
        id: "task_2",
        key: "LOCAL-2",
        project_id: "proj_local",
        title: "New task",
        description: null,
        status: "ready",
        priority: "medium",
        assignee_type: "agent",
        assignee_id: "agent_codex",
        cycle_id: "Sprint 5",
        module_id: "Control API",
        initiative_id: "init_auth",
        due_at: "2026-05-15",
        estimate: "3 pts",
        labels: ["release", "evidence"],
      })
      .mockResolvedValueOnce({
        id: "task_2",
        key: "LOCAL-2",
        project_id: "proj_local",
        title: "New task",
        description: null,
        status: "ready",
        priority: "medium",
        assignee_type: "agent",
        assignee_id: "agent_codex",
        cycle_id: "Sprint 5",
        module_id: "Control API",
        context_pack_id: "ctx_auth",
      });

    const tasks = await listNativeTasks("proj_local");
    const created = await createNativeTask({ projectId: "proj_local", title: "New task" });
    const moved = await moveNativeTask("task_2", "ready");
    const comment = await addNativeTaskComment({ taskId: "task_2", body: "Native comment" });
    const comments = await listNativeTaskComments("task_2");
    const workpad = await saveNativeTaskWorkpad({ taskId: "task_2", body: "Native workpad" });
    const planned = await updateNativeTaskPlanning({
      taskId: "task_2",
      cycle: "Sprint 5",
      module: "Control API",
      initiative: "init_auth",
      dueDate: "2026-05-15",
      estimate: "3 pts",
      labels: ["release", "evidence"],
      assignee: "agent_codex",
    } as Parameters<typeof updateNativeTaskPlanning>[0]);
    const contexted = await updateNativeTaskContext({ taskId: "task_2", contextPackId: "ctx_auth" });

    expect(invoke).toHaveBeenNthCalledWith(1, "list_tasks", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(2, "create_task", {
      request: { projectId: "proj_local", title: "New task", priority: undefined },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "move_task", { id: "task_2", status: "ready" });
    expect(invoke).toHaveBeenNthCalledWith(4, "add_task_comment", {
      request: { taskId: "task_2", authorType: "human", authorId: "local_user", bodyMd: "Native comment" },
    });
    expect(invoke).toHaveBeenNthCalledWith(5, "list_task_comments", { taskId: "task_2" });
    expect(invoke).toHaveBeenNthCalledWith(6, "save_task_workpad", {
      request: { taskId: "task_2", bodyMd: "Native workpad" },
    });
    expect(invoke).toHaveBeenNthCalledWith(7, "update_task_planning", {
      request: { taskId: "task_2", cycleId: "Sprint 5", moduleId: "Control API", initiativeId: "init_auth", dueAt: "2026-05-15", estimate: "3 pts", labels: ["release", "evidence"], assigneeType: "agent", assigneeId: "agent_codex" },
    });
    expect(invoke).toHaveBeenNthCalledWith(8, "update_task_context", {
      request: { taskId: "task_2", contextPackId: "ctx_auth" },
    });
    expect(nativeTaskToHaneulchiTask(tasks[0])).toMatchObject({
      id: "task_1",
      projectId: "proj_local",
      assignee: "agent_codex",
      cycle: "cycle_sprint_5",
      module: "module_control_api",
      initiative: "init_platform",
      dueDate: "2026-05-15",
      estimate: "3 pts",
      labels: ["release", "evidence"],
    });
    expect(created.status).toBe("inbox");
    expect(moved.status).toBe("ready");
    expect(comment.body_md).toBe("Native comment");
    expect(comments).toHaveLength(1);
    expect(workpad.body_md).toBe("Native workpad");
    expect(planned.cycle_id).toBe("Sprint 5");
    expect(planned.initiative_id).toBe("init_auth");
    expect(planned.due_at).toBe("2026-05-15");
    expect(planned.estimate).toBe("3 pts");
    expect(planned.assignee_id).toBe("agent_codex");
    expect(contexted.context_pack_id).toBe("ctx_auth");
  });

  it("converts native task lists into Haneulchi task state", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        id: "task_native",
        key: "LOCAL-3",
        project_id: "proj_local",
        title: "Native task only",
        description: "Loaded from SQLite",
        status: "blocked",
        priority: "urgent",
        assignee_type: null,
        assignee_id: null,
        cycle_id: null,
        module_id: "module_release",
        initiative_id: "init_release",
        due_at: "2026-06-01",
        estimate: "2 pts",
        labels: ["release", "blocked"],
        workpad_md: "Persisted native workpad",
      },
    ]);

    const { loadNativeTaskState } = await import("./taskApiClient");

    await expect(loadNativeTaskState("proj_local")).resolves.toEqual({
      tasks: {
        task_native: {
          id: "task_native",
          title: "Native task only",
          description: "Loaded from SQLite",
          status: "blocked",
          priority: "urgent",
          projectId: "proj_local",
          assignee: undefined,
          workpad: "Persisted native workpad",
          cycle: undefined,
          module: "module_release",
          initiative: "init_release",
          dueDate: "2026-06-01",
          estimate: "2 pts",
          labels: ["release", "blocked"],
          contextPackId: undefined,
        },
      },
    });
  });

  it("creates native review follow-up tasks through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      review_id: "review_ev_run_1",
      evidence_pack_id: "ev_run_1",
      source_task_id: "task_review",
      source_run_id: "run_1",
      task: {
        id: "task_followup_1",
        key: "LOCAL-12",
        project_id: "proj_auth",
        title: "Address reviewer notes",
        description: null,
        status: "inbox",
        priority: "urgent",
        assignee_type: null,
        assignee_id: null,
        cycle_id: null,
        module_id: null,
      },
      comment: {
        id: "comment_1",
        task_id: "task_followup_1",
        run_id: null,
        author_type: "system",
        author_id: "review_queue",
        body_md: "Created from review review_ev_run_1",
        parent_id: null,
      },
    });

    const receipt = await createNativeReviewFollowUpTask({
      reviewId: "review_ev_run_1",
      title: "Address reviewer notes",
      priority: "urgent",
    });

    expect(invoke).toHaveBeenCalledWith("create_review_follow_up_task", {
      request: {
        reviewId: "review_ev_run_1",
        title: "Address reviewer notes",
        priority: "urgent",
      },
    });
    expect(receipt.task.id).toBe("task_followup_1");
    expect(receipt.source_task_id).toBe("task_review");
  });

  it("maps native task subtask commands through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([
        { id: "subtask_1", task_id: "task_2", title: "Attach screenshots", status: "open", order_index: 0 },
      ])
      .mockResolvedValueOnce({ id: "subtask_2", task_id: "task_2", title: "Confirm terminal proof", status: "open", order_index: 1 })
      .mockResolvedValueOnce({ id: "subtask_2", task_id: "task_2", title: "Confirm terminal proof", status: "done", order_index: 1 });

    const subtasks = await listNativeTaskSubtasks("task_2");
    const added = await addNativeTaskSubtask({ taskId: "task_2", title: "Confirm terminal proof" });
    const completed = await updateNativeTaskSubtaskStatus({ taskId: "task_2", subtaskId: "subtask_2", status: "done" });

    expect(invoke).toHaveBeenNthCalledWith(1, "list_task_subtasks", { taskId: "task_2" });
    expect(invoke).toHaveBeenNthCalledWith(2, "add_task_subtask", {
      request: { taskId: "task_2", title: "Confirm terminal proof" },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "update_task_subtask_status", {
      request: { taskId: "task_2", subtaskId: "subtask_2", status: "done" },
    });
    expect(subtasks[0].title).toBe("Attach screenshots");
    expect(added.status).toBe("open");
    expect(completed.status).toBe("done");
  });

  it("maps native task cycle and module commands through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([
        { id: "cycle_1", project_id: "proj_local", name: "Sprint 12", starts_at: "2026-05-01", ends_at: "2026-05-15", status: "active" },
      ])
      .mockResolvedValueOnce({ id: "cycle_2", project_id: "proj_local", name: "Sprint 13", starts_at: null, ends_at: null, status: "planned" })
      .mockResolvedValueOnce([
        { id: "module_1", project_id: "proj_local", name: "Control API", description: "Native boundary", status: "active" },
      ])
      .mockResolvedValueOnce({ id: "module_2", project_id: "proj_local", name: "Release", description: null, status: "active" });

    const cycles = await listNativeTaskCycles("proj_local");
    const createdCycle = await createNativeTaskCycle({ projectId: "proj_local", name: "Sprint 13", status: "planned" });
    const modules = await listNativeTaskModules("proj_local");
    const createdModule = await createNativeTaskModule({ projectId: "proj_local", name: "Release" });

    expect(invoke).toHaveBeenNthCalledWith(1, "list_task_cycles", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(2, "create_task_cycle", {
      request: { projectId: "proj_local", name: "Sprint 13", startsAt: undefined, endsAt: undefined, status: "planned" },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "list_task_modules", { projectId: "proj_local" });
    expect(invoke).toHaveBeenNthCalledWith(4, "create_task_module", {
      request: { projectId: "proj_local", name: "Release", description: undefined, status: undefined },
    });
    expect(cycles[0].name).toBe("Sprint 12");
    expect(createdCycle.status).toBe("planned");
    expect(modules[0].description).toBe("Native boundary");
    expect(createdModule.name).toBe("Release");
  });
});
