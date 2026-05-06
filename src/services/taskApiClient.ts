import { invoke } from "@tauri-apps/api/core";
import type { HaneulchiTask, TaskPriority, TaskStatus, TaskSubtaskStatus } from "../domain/tasks";
import type { TaskState } from "../domain/tasks";

export interface NativeTask {
  id: string;
  key: string;
  project_id: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  assignee_type: string | null;
  assignee_id: string | null;
  cycle_id: string | null;
  module_id: string | null;
  initiative_id?: string | null;
  due_at?: string | null;
  estimate?: string | null;
  labels?: string[] | null;
  context_pack_id?: string | null;
  workpad_md?: string | null;
}

export interface NativeTaskComment {
  id: string;
  task_id: string | null;
  run_id: string | null;
  author_type: string;
  author_id: string;
  body_md: string;
  parent_id: string | null;
}

export interface NativeTaskWorkpad {
  id: string;
  task_id: string;
  artifact_path: string;
  title: string;
  body_md: string;
}

export interface NativeTaskSubtask {
  id: string;
  task_id: string;
  title: string;
  status: TaskSubtaskStatus;
  order_index: number;
}

export interface NativeTaskCycle {
  id: string;
  project_id: string;
  name: string;
  starts_at: string | null;
  ends_at: string | null;
  status: string;
}

export interface NativeTaskModule {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  status: string;
}

interface CreateNativeTaskInput {
  projectId: string;
  title: string;
  priority?: TaskPriority;
}

interface CreateNativeReviewFollowUpTaskInput {
  reviewId: string;
  title?: string;
  priority?: TaskPriority;
}

export interface NativeReviewFollowUpTaskReceipt {
  review_id: string;
  evidence_pack_id: string;
  source_task_id: string | null;
  source_run_id: string | null;
  task: NativeTask;
  comment: NativeTaskComment;
}

interface AddNativeTaskCommentInput {
  taskId: string;
  body: string;
}

interface SaveNativeTaskWorkpadInput {
  taskId: string;
  body: string;
}

interface AddNativeTaskSubtaskInput {
  taskId: string;
  title: string;
}

interface CreateNativeTaskCycleInput {
  projectId: string;
  name: string;
  startsAt?: string;
  endsAt?: string;
  status?: string;
}

interface CreateNativeTaskModuleInput {
  projectId: string;
  name: string;
  description?: string;
  status?: string;
}

interface UpdateNativeTaskSubtaskStatusInput {
  taskId: string;
  subtaskId: string;
  status: TaskSubtaskStatus;
}

interface UpdateNativeTaskPlanningInput {
  taskId: string;
  cycle?: string;
  module?: string;
  initiative?: string;
  dueDate?: string;
  estimate?: string;
  labels?: string[];
  assignee?: string;
}

interface UpdateNativeTaskContextInput {
  taskId: string;
  contextPackId?: string;
}

export function listNativeTasks(projectId: string): Promise<NativeTask[]> {
  return invoke<NativeTask[]>("list_tasks", { projectId });
}

export async function loadNativeTaskState(projectId: string): Promise<TaskState> {
  const tasks = await listNativeTasks(projectId);
  return {
    tasks: Object.fromEntries(tasks.map((task) => {
      const mapped = nativeTaskToHaneulchiTask(task);
      return [mapped.id, mapped];
    })),
  };
}

export function createNativeTask(input: CreateNativeTaskInput): Promise<NativeTask> {
  return invoke<NativeTask>("create_task", {
    request: {
      projectId: input.projectId,
      title: input.title,
      priority: input.priority,
    },
  });
}

export function createNativeReviewFollowUpTask(
  input: CreateNativeReviewFollowUpTaskInput,
): Promise<NativeReviewFollowUpTaskReceipt> {
  return invoke<NativeReviewFollowUpTaskReceipt>("create_review_follow_up_task", {
    request: {
      reviewId: input.reviewId,
      title: input.title,
      priority: input.priority,
    },
  });
}

export function moveNativeTask(id: string, status: TaskStatus): Promise<NativeTask> {
  return invoke<NativeTask>("move_task", { id, status });
}

export function addNativeTaskComment(input: AddNativeTaskCommentInput): Promise<NativeTaskComment> {
  return invoke<NativeTaskComment>("add_task_comment", {
    request: {
      taskId: input.taskId,
      authorType: "human",
      authorId: "local_user",
      bodyMd: input.body,
    },
  });
}

export function listNativeTaskComments(taskId: string): Promise<NativeTaskComment[]> {
  return invoke<NativeTaskComment[]>("list_task_comments", { taskId });
}

export function saveNativeTaskWorkpad(input: SaveNativeTaskWorkpadInput): Promise<NativeTaskWorkpad> {
  return invoke<NativeTaskWorkpad>("save_task_workpad", {
    request: {
      taskId: input.taskId,
      bodyMd: input.body,
    },
  });
}

export function listNativeTaskSubtasks(taskId: string): Promise<NativeTaskSubtask[]> {
  return invoke<NativeTaskSubtask[]>("list_task_subtasks", { taskId });
}

export function addNativeTaskSubtask(input: AddNativeTaskSubtaskInput): Promise<NativeTaskSubtask> {
  return invoke<NativeTaskSubtask>("add_task_subtask", {
    request: {
      taskId: input.taskId,
      title: input.title,
    },
  });
}

export function listNativeTaskCycles(projectId: string): Promise<NativeTaskCycle[]> {
  return invoke<NativeTaskCycle[]>("list_task_cycles", { projectId });
}

export function createNativeTaskCycle(input: CreateNativeTaskCycleInput): Promise<NativeTaskCycle> {
  return invoke<NativeTaskCycle>("create_task_cycle", {
    request: {
      projectId: input.projectId,
      name: input.name,
      startsAt: input.startsAt,
      endsAt: input.endsAt,
      status: input.status,
    },
  });
}

export function listNativeTaskModules(projectId: string): Promise<NativeTaskModule[]> {
  return invoke<NativeTaskModule[]>("list_task_modules", { projectId });
}

export function createNativeTaskModule(input: CreateNativeTaskModuleInput): Promise<NativeTaskModule> {
  return invoke<NativeTaskModule>("create_task_module", {
    request: {
      projectId: input.projectId,
      name: input.name,
      description: input.description,
      status: input.status,
    },
  });
}

export function updateNativeTaskSubtaskStatus(input: UpdateNativeTaskSubtaskStatusInput): Promise<NativeTaskSubtask> {
  return invoke<NativeTaskSubtask>("update_task_subtask_status", {
    request: {
      taskId: input.taskId,
      subtaskId: input.subtaskId,
      status: input.status,
    },
  });
}

export function updateNativeTaskPlanning(input: UpdateNativeTaskPlanningInput): Promise<NativeTask> {
  return invoke<NativeTask>("update_task_planning", {
    request: {
      taskId: input.taskId,
      cycleId: input.cycle,
      moduleId: input.module,
      initiativeId: input.initiative,
      dueAt: input.dueDate,
      estimate: input.estimate,
      labels: input.labels,
      assigneeType: input.assignee ? "agent" : undefined,
      assigneeId: input.assignee,
    },
  });
}

export function updateNativeTaskContext(input: UpdateNativeTaskContextInput): Promise<NativeTask> {
  return invoke<NativeTask>("update_task_context", {
    request: {
      taskId: input.taskId,
      contextPackId: input.contextPackId,
    },
  });
}

export function nativeTaskToHaneulchiTask(task: NativeTask): HaneulchiTask {
  return {
    id: task.id,
    title: task.title,
    status: task.status,
    priority: task.priority,
    projectId: task.project_id,
    assignee: task.assignee_id ?? undefined,
    description: task.description ?? undefined,
    workpad: task.workpad_md ?? undefined,
    cycle: task.cycle_id ?? undefined,
    module: task.module_id ?? undefined,
    initiative: task.initiative_id ?? undefined,
    dueDate: task.due_at ?? undefined,
    estimate: task.estimate ?? undefined,
    labels: task.labels && task.labels.length > 0 ? task.labels : undefined,
    contextPackId: task.context_pack_id ?? undefined,
  };
}
