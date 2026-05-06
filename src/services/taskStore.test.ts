import { beforeEach, describe, expect, it } from "vitest";
import { createTaskState, moveTask } from "../domain/tasks";
import { loadTaskState, saveTaskState } from "./taskStore";

describe("task store", () => {
  const storage = new Map<string, string>();

  beforeEach(() => {
    storage.clear();
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: {
        getItem: (key: string) => storage.get(key) ?? null,
        setItem: (key: string, value: string) => storage.set(key, value),
        clear: () => storage.clear(),
      },
    });
  });

  it("loads seeded task state when storage is empty", () => {
    expect(Object.keys(loadTaskState("proj_local").tasks)).toHaveLength(6);
  });

  it("does not seed proj_local tasks into another project cache", () => {
    expect(loadTaskState("proj_auth").tasks).toEqual({});
  });

  it("saves and reloads task board status changes", () => {
    const moved = moveTask(createTaskState(), "task_ready", "running");

    saveTaskState("proj_local", moved);

    expect(loadTaskState("proj_local").tasks.task_ready.status).toBe("running");
  });

  it("reloads task drawer planning metadata when persisted", () => {
    const state = createTaskState([
      {
        id: "task_1",
        title: "Wire state snapshot",
        status: "ready",
        priority: "high",
        projectId: "proj_local",
        cycle: "Sprint 5",
        module: "Control API",
      },
    ]);

    saveTaskState("proj_local", state);

    expect(loadTaskState("proj_local").tasks.task_1).toMatchObject({
      cycle: "Sprint 5",
      module: "Control API",
    });
  });

  it("falls back safely when stored task state is invalid", () => {
    window.localStorage.setItem("haneulchi:task-state:proj_local", "{bad json");

    expect(Object.keys(loadTaskState("proj_local").tasks)).toHaveLength(6);
  });
});
