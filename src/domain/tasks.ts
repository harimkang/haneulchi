export type TaskStatus = "inbox" | "ready" | "running" | "review" | "blocked" | "done" | "archived";
export type TaskPriority = "none" | "low" | "medium" | "high" | "urgent";

export interface TaskComment {
  id: string;
  author: string;
  body: string;
}

export type TaskSubtaskStatus = "open" | "done";

export interface TaskSubtask {
  id: string;
  title: string;
  status: TaskSubtaskStatus;
}

export interface HaneulchiTask {
  id: string;
  title: string;
  status: TaskStatus;
  priority: TaskPriority;
  projectId: string;
  assignee?: string;
  description?: string;
  labels?: string[];
  workpad?: string;
  comments?: TaskComment[];
  subtasks?: TaskSubtask[];
  cycle?: string;
  module?: string;
  initiative?: string;
  dueDate?: string;
  estimate?: string;
  contextPackId?: string;
}

export interface TaskOverview {
  id: string;
  title: string;
  status: TaskStatus;
  priority: TaskPriority;
  assignee?: string;
  description: string;
  workpad: string;
  commentCount: number;
  comments: TaskComment[];
  subtaskCount: number;
  openSubtaskCount: number;
  subtasks: TaskSubtask[];
  labels?: string[];
  cycle?: string;
  module?: string;
  initiative?: string;
  dueDate?: string;
  estimate?: string;
  contextPackId?: string;
}

export interface TaskState {
  tasks: Record<string, HaneulchiTask>;
}

interface AddTaskInput {
  title: string;
  projectId: string;
  priority?: TaskPriority;
  assignee?: string;
}

interface AddTaskResult {
  state: TaskState;
  createdTask?: HaneulchiTask;
}

interface AddTaskCommentInput {
  author: string;
  body: string;
}

interface AddTaskCommentResult {
  state: TaskState;
  createdComment?: TaskComment;
}

interface AddTaskSubtaskInput {
  title: string;
}

interface AddTaskSubtaskResult {
  state: TaskState;
  createdSubtask?: TaskSubtask;
}

interface UpdateTaskPlanningPropertiesInput {
  cycle?: string;
  module?: string;
  initiative?: string;
  labels?: string[];
  dueDate?: string;
  estimate?: string;
  assignee?: string;
}

export type TaskStatusCounts = Record<TaskStatus, number>;

export const taskBoardStatuses = ["inbox", "ready", "running", "review", "blocked", "done"] as const satisfies readonly TaskStatus[];

export function createTaskState(tasks: HaneulchiTask[] = seedTasks): TaskState {
  return {
    tasks: Object.fromEntries(tasks.map((task) => [task.id, task])),
  };
}

export function listTasks(state: TaskState): HaneulchiTask[] {
  return Object.values(state.tasks).sort((a, b) => a.id.localeCompare(b.id));
}

export function filterTasks(state: TaskState, query: string): HaneulchiTask[] {
  const normalized = query.trim().toLowerCase();
  const tasks = listTasks(state);
  if (normalized.length === 0) return tasks;

  return tasks.filter((task) =>
    [task.title, task.status, task.priority, task.assignee ?? "", task.cycle ?? "", task.module ?? "", task.initiative ?? "", task.dueDate ?? "", task.estimate ?? "", ...(task.labels ?? [])]
      .join("\n")
      .toLowerCase()
      .includes(normalized),
  );
}

export function countTasksByStatus(state: TaskState): TaskStatusCounts {
  const counts: TaskStatusCounts = {
    inbox: 0,
    ready: 0,
    running: 0,
    review: 0,
    blocked: 0,
    done: 0,
    archived: 0,
  };

  for (const task of listTasks(state)) {
    counts[task.status] += 1;
  }

  return counts;
}

export function getTaskOverview(state: TaskState, taskId: string): TaskOverview | undefined {
  const task = state.tasks[taskId];
  if (!task) return undefined;

  return {
    id: task.id,
    title: task.title,
    status: task.status,
    priority: task.priority,
    assignee: task.assignee,
    description: task.description ?? "",
    workpad: task.workpad ?? "",
    commentCount: task.comments?.length ?? 0,
    comments: task.comments ?? [],
    subtaskCount: task.subtasks?.length ?? 0,
    openSubtaskCount: task.subtasks?.filter((subtask) => subtask.status !== "done").length ?? 0,
    subtasks: task.subtasks ?? [],
    ...(task.labels && task.labels.length > 0 ? { labels: task.labels } : {}),
    cycle: task.cycle,
    module: task.module,
    initiative: task.initiative,
    dueDate: task.dueDate,
    estimate: task.estimate,
    contextPackId: task.contextPackId,
  };
}

export function updateTaskContextPack(state: TaskState, taskId: string, contextPackId?: string): TaskState {
  const task = state.tasks[taskId];
  if (!task) return state;

  return {
    tasks: {
      ...state.tasks,
      [taskId]: {
        ...task,
        contextPackId,
      },
    },
  };
}

export function moveTask(state: TaskState, taskId: string, status: TaskStatus): TaskState {
  const task = state.tasks[taskId];
  if (!task) return state;

  return {
    tasks: {
      ...state.tasks,
      [taskId]: {
        ...task,
        status,
      },
    },
  };
}

export function advanceTask(state: TaskState, taskId: string): TaskState {
  const task = state.tasks[taskId];
  if (!task) return state;

  const nextStatusByStatus: Partial<Record<TaskStatus, TaskStatus>> = {
    inbox: "ready",
    ready: "running",
    running: "review",
    review: "done",
  };
  const nextStatus = nextStatusByStatus[task.status];
  if (!nextStatus) return state;

  return moveTask(state, taskId, nextStatus);
}

export function updateTaskWorkpad(state: TaskState, taskId: string, workpad: string): TaskState {
  const task = state.tasks[taskId];
  if (!task) return state;

  return {
    tasks: {
      ...state.tasks,
      [taskId]: {
        ...task,
        workpad,
      },
    },
  };
}

export function addTaskComment(state: TaskState, taskId: string, input: AddTaskCommentInput): AddTaskCommentResult {
  const task = state.tasks[taskId];
  const body = input.body.trim();
  const author = input.author.trim();
  if (!task || body.length === 0 || author.length === 0) {
    return { state };
  }

  const comment: TaskComment = {
    id: nextCommentId(task.comments ?? []),
    author,
    body,
  };

  return {
    state: {
      tasks: {
        ...state.tasks,
        [taskId]: {
          ...task,
          comments: [...(task.comments ?? []), comment],
        },
      },
    },
    createdComment: comment,
  };
}

export function addTaskSubtask(state: TaskState, taskId: string, input: AddTaskSubtaskInput): AddTaskSubtaskResult {
  const task = state.tasks[taskId];
  const title = input.title.trim();
  if (!task || title.length === 0) {
    return { state };
  }

  const subtask: TaskSubtask = {
    id: nextSubtaskId(task.subtasks ?? []),
    title,
    status: "open",
  };

  return {
    state: {
      tasks: {
        ...state.tasks,
        [taskId]: {
          ...task,
          subtasks: [...(task.subtasks ?? []), subtask],
        },
      },
    },
    createdSubtask: subtask,
  };
}

export function updateTaskSubtaskStatus(
  state: TaskState,
  taskId: string,
  subtaskId: string,
  status: TaskSubtaskStatus,
): TaskState {
  const task = state.tasks[taskId];
  if (!task) return state;

  return {
    tasks: {
      ...state.tasks,
      [taskId]: {
        ...task,
        subtasks: (task.subtasks ?? []).map((subtask) =>
          subtask.id === subtaskId ? { ...subtask, status } : subtask,
        ),
      },
    },
  };
}

export function updateTaskPlanningProperties(
  state: TaskState,
  taskId: string,
  input: UpdateTaskPlanningPropertiesInput,
): TaskState {
  const task = state.tasks[taskId];
  if (!task) return state;

  return {
    tasks: {
      ...state.tasks,
      [taskId]: {
        ...task,
        cycle: normalizeOptionalTaskText(input.cycle),
        module: normalizeOptionalTaskText(input.module),
        initiative: normalizeOptionalTaskText(input.initiative),
        labels: normalizeTaskLabels(input.labels),
        dueDate: normalizeOptionalTaskText(input.dueDate),
        estimate: normalizeOptionalTaskText(input.estimate),
        assignee: normalizeOptionalTaskText(input.assignee),
      },
    },
  };
}

export function addTask(state: TaskState, input: AddTaskInput): AddTaskResult {
  const title = input.title.trim();
  if (title.length === 0) {
    return { state };
  }

  const task: HaneulchiTask = {
    id: nextTaskId(state),
    title,
    status: "inbox",
    priority: input.priority ?? "medium",
    projectId: input.projectId,
    assignee: input.assignee,
  };

  return {
    state: {
      tasks: {
        ...state.tasks,
        [task.id]: task,
      },
    },
    createdTask: task,
  };
}

function nextTaskId(state: TaskState): string {
  const maxId = Object.keys(state.tasks).reduce((max, id) => {
    const match = /^task_(\d+)$/.exec(id);
    if (!match) return max;
    return Math.max(max, Number(match[1]));
  }, 0);

  return `task_${Math.max(maxId, Object.keys(state.tasks).length) + 1}`;
}

function nextCommentId(comments: TaskComment[]): string {
  const maxId = comments.reduce((max, comment) => {
    const match = /^comment_(\d+)$/.exec(comment.id);
    if (!match) return max;
    return Math.max(max, Number(match[1]));
  }, 0);

  return `comment_${Math.max(maxId, comments.length) + 1}`;
}

function nextSubtaskId(subtasks: TaskSubtask[]): string {
  const maxId = subtasks.reduce((max, subtask) => {
    const match = /^subtask_(\d+)$/.exec(subtask.id);
    if (!match) return max;
    return Math.max(max, Number(match[1]));
  }, 0);

  return `subtask_${Math.max(maxId, subtasks.length) + 1}`;
}

function normalizeOptionalTaskText(value: string | undefined): string | undefined {
  const normalized = value?.trim() ?? "";
  return normalized.length > 0 ? normalized : undefined;
}

function normalizeTaskLabels(labels: string[] | undefined): string[] | undefined {
  const normalized = Array.from(new Set((labels ?? []).map((label) => label.trim()).filter(Boolean)));
  return normalized.length > 0 ? normalized : undefined;
}

export const seedTasks: HaneulchiTask[] = [
  {
    id: "task_inbox",
    title: "Capture project setup requirements",
    status: "inbox",
    priority: "medium",
    projectId: "proj_local",
  },
  {
    id: "task_ready",
    title: "Wire state snapshot into CLI boundary",
    status: "ready",
    priority: "high",
    projectId: "proj_local",
    cycle: "Sprint 5",
    module: "Control API",
  },
  {
    id: "task_running",
    title: "Harden terminal runtime proof",
    status: "running",
    priority: "urgent",
    projectId: "proj_local",
    assignee: "agent_codex",
  },
  {
    id: "task_review",
    title: "Review evidence pack workflow",
    status: "review",
    priority: "high",
    projectId: "proj_local",
    description: "Validate command block attachments before marking review-ready.",
    workpad: "Confirm attached evidence references the latest terminal proof before routing the task out of review.",
    comments: [
      { id: "comment_1", author: "human", body: "Needs release gate evidence." },
      { id: "comment_2", author: "agent_codex", body: "Command blocks attached." },
    ],
  },
  {
    id: "task_blocked",
    title: "Resolve DMG packaging gate",
    status: "blocked",
    priority: "urgent",
    projectId: "proj_local",
  },
  {
    id: "task_done",
    title: "Establish Tauri app shell",
    status: "done",
    priority: "medium",
    projectId: "proj_local",
  },
];
