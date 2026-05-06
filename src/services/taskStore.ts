import { createTaskState, type HaneulchiTask, type TaskComment, type TaskPriority, type TaskState, type TaskStatus } from "../domain/tasks";

const storagePrefix = "haneulchi:task-state:";
const taskStatuses = new Set<TaskStatus>(["inbox", "ready", "running", "review", "blocked", "done", "archived"]);
const taskPriorities = new Set<TaskPriority>(["none", "low", "medium", "high", "urgent"]);

export function loadTaskState(projectId: string): TaskState {
  if (typeof window === "undefined" || !hasStorageApi()) {
    return fallbackTaskState(projectId);
  }

  try {
    const raw = window.localStorage.getItem(storageKey(projectId));
    if (!raw) return fallbackTaskState(projectId);

    const parsed = JSON.parse(raw) as TaskState;
    if (!isTaskState(parsed, projectId)) return fallbackTaskState(projectId);

    return parsed;
  } catch {
    return fallbackTaskState(projectId);
  }
}

export function saveTaskState(projectId: string, state: TaskState): void {
  if (typeof window === "undefined" || !hasStorageApi()) return;
  try {
    window.localStorage.setItem(storageKey(projectId), JSON.stringify(state));
  } catch {
    // Persistence is best-effort until the SQLite state store lands.
  }
}

function storageKey(projectId: string): string {
  return `${storagePrefix}${projectId}`;
}

function fallbackTaskState(projectId: string): TaskState {
  return projectId === "proj_local" ? createTaskState() : createTaskState([]);
}

function hasStorageApi(): boolean {
  return (
    typeof window.localStorage?.getItem === "function" &&
    typeof window.localStorage?.setItem === "function"
  );
}

function isTaskState(value: TaskState, projectId: string): boolean {
  return (
    value !== null &&
    typeof value === "object" &&
    value.tasks !== null &&
    typeof value.tasks === "object" &&
    Object.values(value.tasks).every((task) => isTask(task, projectId))
  );
}

function isTask(value: HaneulchiTask, projectId: string): boolean {
  return (
    value !== null &&
    typeof value === "object" &&
    typeof value.id === "string" &&
    value.id.trim().length > 0 &&
    typeof value.title === "string" &&
    value.title.trim().length > 0 &&
    taskStatuses.has(value.status) &&
    taskPriorities.has(value.priority) &&
    value.projectId === projectId &&
    (value.assignee === undefined || typeof value.assignee === "string") &&
    (value.description === undefined || typeof value.description === "string") &&
    (value.workpad === undefined || typeof value.workpad === "string") &&
    (value.cycle === undefined || typeof value.cycle === "string") &&
    (value.module === undefined || typeof value.module === "string") &&
    (value.comments === undefined || (Array.isArray(value.comments) && value.comments.every(isTaskComment)))
  );
}

function isTaskComment(value: TaskComment): boolean {
  return (
    value !== null &&
    typeof value === "object" &&
    typeof value.id === "string" &&
    value.id.trim().length > 0 &&
    typeof value.author === "string" &&
    value.author.trim().length > 0 &&
    typeof value.body === "string"
  );
}
