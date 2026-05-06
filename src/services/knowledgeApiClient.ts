import { invoke } from "@tauri-apps/api/core";

export interface NativeKnowledgeSource {
  id: string;
  project_id: string;
  kind: string;
  path_or_ref: string;
  fingerprint: string;
  status: string;
}

export interface NativeKnowledgePage {
  id: string;
  project_id: string;
  slug: string;
  title: string;
  artifact_path: string;
  source_ids: string[];
  freshness_state: string;
  body_md: string;
}

export interface NativeKnowledgeExploration {
  id: string;
  project_id: string;
  title: string;
  question: string;
  answer_md: string;
  artifact_path: string;
  page_ids: string[];
  context_pack_id: string | null;
}

export interface NativeKnowledgeConcept {
  slug: string;
  title: string;
  page_id: string | null;
  outbound_slugs: string[];
  inbound_page_ids: string[];
}

export interface NativeKnowledgeObsidianExport {
  project_id: string;
  status: string;
  export_root: string;
  file_count: number;
  files: string[];
}

export interface NativeKnowledgeChatAnswer {
  project_id: string;
  question: string;
  answer_md: string;
  cited_page_ids: string[];
  context_pack_id: string | null;
  source_count: number;
}

export interface NativeContextPack {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  sources_json: unknown;
}

export interface NativeKnowledgeLintReport {
  id: string;
  project_id: string;
  artifact_path: string;
  stale_count: number;
  gap_count: number;
  contradiction_count: number;
  body_md: string;
}

export interface NativeKnowledgeAutomationRun {
  project_id: string;
  status: string;
  watch_enabled: boolean;
  source_count: number;
  page_count: number;
  stale_count: number;
  gap_count: number;
  lint_report_id: string;
}

export interface NativeKnowledgeIngestionResult {
  project_id: string;
  source_id: string;
  page_id: string;
  slug: string;
  modality: string;
  chunk_count: number;
  fingerprint: string;
}

interface UpsertNativeKnowledgeSourceInput {
  projectId: string;
  kind: string;
  pathOrRef: string;
  fingerprint: string;
  status: string;
}

interface SaveNativeKnowledgePageInput {
  projectId: string;
  slug: string;
  title: string;
  bodyMd: string;
  sourceIds: string[];
  freshnessState: string;
}

interface SaveNativeKnowledgeExplorationInput {
  projectId: string;
  title: string;
  question: string;
  answerMd: string;
  pageIds: string[];
  contextPackId?: string;
}

interface AnswerNativeKnowledgeQuestionInput {
  projectId: string;
  question: string;
  contextPackId?: string;
}

interface UpsertNativeContextPackInput {
  id?: string;
  projectId: string;
  name: string;
  description?: string;
  sourcesJson: unknown[];
  maxTokensHint?: number;
}

interface RecordNativeKnowledgeLintReportInput {
  projectId: string;
  staleCount: number;
  gapCount: number;
  contradictionCount: number;
  bodyMd: string;
}

interface RunNativeKnowledgeAutomationInput {
  projectId: string;
  watch: boolean;
}

interface IngestNativeKnowledgeArtifactInput {
  projectId: string;
  kind: string;
  pathOrRef: string;
  title?: string;
  bodyMd: string;
  maxChunkChars?: number;
}

export function upsertNativeKnowledgeSource(
  input: UpsertNativeKnowledgeSourceInput,
): Promise<NativeKnowledgeSource> {
  return invoke<NativeKnowledgeSource>("upsert_knowledge_source", {
    request: {
      projectId: input.projectId,
      kind: input.kind,
      pathOrRef: input.pathOrRef,
      fingerprint: input.fingerprint,
      status: input.status,
    },
  });
}

export function listNativeKnowledgeSources(projectId: string): Promise<NativeKnowledgeSource[]> {
  return invoke<NativeKnowledgeSource[]>("list_knowledge_sources", {
    projectId,
  });
}

export function saveNativeKnowledgePage(input: SaveNativeKnowledgePageInput): Promise<NativeKnowledgePage> {
  return invoke<NativeKnowledgePage>("save_knowledge_page", {
    request: {
      projectId: input.projectId,
      slug: input.slug,
      title: input.title,
      bodyMd: input.bodyMd,
      sourceIds: input.sourceIds,
      freshnessState: input.freshnessState,
    },
  });
}

export function searchNativeKnowledgePages(projectId: string, query?: string): Promise<NativeKnowledgePage[]> {
  return invoke<NativeKnowledgePage[]>("search_knowledge_pages", {
    projectId,
    query,
  });
}

export function saveNativeKnowledgeExploration(
  input: SaveNativeKnowledgeExplorationInput,
): Promise<NativeKnowledgeExploration> {
  return invoke<NativeKnowledgeExploration>("save_knowledge_exploration", {
    request: {
      projectId: input.projectId,
      title: input.title,
      question: input.question,
      answerMd: input.answerMd,
      pageIds: input.pageIds,
      contextPackId: input.contextPackId,
    },
  });
}

export function listNativeKnowledgeExplorations(projectId: string): Promise<NativeKnowledgeExploration[]> {
  return invoke<NativeKnowledgeExploration[]>("list_knowledge_explorations", {
    projectId,
  });
}

export function listNativeKnowledgeConcepts(projectId: string): Promise<NativeKnowledgeConcept[]> {
  return invoke<NativeKnowledgeConcept[]>("list_knowledge_concepts", {
    projectId,
  });
}

export function exportNativeKnowledgeObsidianMarkdown(projectId: string): Promise<NativeKnowledgeObsidianExport> {
  return invoke<NativeKnowledgeObsidianExport>("export_knowledge_obsidian_markdown", {
    projectId,
  });
}

export function answerNativeKnowledgeQuestion(
  input: AnswerNativeKnowledgeQuestionInput,
): Promise<NativeKnowledgeChatAnswer> {
  return invoke<NativeKnowledgeChatAnswer>("answer_knowledge_question", {
    request: {
      projectId: input.projectId,
      question: input.question,
      contextPackId: input.contextPackId,
    },
  });
}

export function upsertNativeContextPack(input: UpsertNativeContextPackInput): Promise<NativeContextPack> {
  return invoke<NativeContextPack>("upsert_context_pack", {
    request: {
      id: input.id,
      projectId: input.projectId,
      name: input.name,
      description: input.description,
      sourcesJson: input.sourcesJson,
      maxTokensHint: input.maxTokensHint,
    },
  });
}

export function listNativeContextPacks(projectId: string): Promise<NativeContextPack[]> {
  return invoke<NativeContextPack[]>("list_context_packs", {
    projectId,
  });
}

export function recordNativeKnowledgeLintReport(
  input: RecordNativeKnowledgeLintReportInput,
): Promise<NativeKnowledgeLintReport> {
  return invoke<NativeKnowledgeLintReport>("record_knowledge_lint_report", {
    request: {
      projectId: input.projectId,
      staleCount: input.staleCount,
      gapCount: input.gapCount,
      contradictionCount: input.contradictionCount,
      bodyMd: input.bodyMd,
    },
  });
}

export function runNativeKnowledgeAutomation(
  input: RunNativeKnowledgeAutomationInput,
): Promise<NativeKnowledgeAutomationRun> {
  return invoke<NativeKnowledgeAutomationRun>("run_knowledge_automation", {
    request: {
      projectId: input.projectId,
      watch: input.watch,
    },
  });
}

export function ingestNativeKnowledgeArtifact(
  input: IngestNativeKnowledgeArtifactInput,
): Promise<NativeKnowledgeIngestionResult> {
  return invoke<NativeKnowledgeIngestionResult>("ingest_knowledge_artifact", {
    request: {
      projectId: input.projectId,
      kind: input.kind,
      pathOrRef: input.pathOrRef,
      title: input.title,
      bodyMd: input.bodyMd,
      maxChunkChars: input.maxChunkChars,
    },
  });
}
