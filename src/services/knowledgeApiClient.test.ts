import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import {
  answerNativeKnowledgeQuestion,
  exportNativeKnowledgeObsidianMarkdown,
  listNativeKnowledgeSources,
  listNativeContextPacks,
  listNativeKnowledgeConcepts,
  listNativeKnowledgeExplorations,
  ingestNativeKnowledgeArtifact,
  recordNativeKnowledgeLintReport,
  runNativeKnowledgeAutomation,
  saveNativeKnowledgeExploration,
  saveNativeKnowledgePage,
  searchNativeKnowledgePages,
  upsertNativeContextPack,
  upsertNativeKnowledgeSource,
} from "./knowledgeApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("knowledge API client", () => {
  it("maps knowledge source upserts through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "ks_1",
      project_id: "proj_local",
      kind: "file",
      path_or_ref: "docs/auth.md",
      fingerprint: "sha256:abc",
      status: "current",
    });

    const source = await upsertNativeKnowledgeSource({
      projectId: "proj_local",
      kind: "file",
      pathOrRef: "docs/auth.md",
      fingerprint: "sha256:abc",
      status: "current",
    });

    expect(invoke).toHaveBeenCalledWith("upsert_knowledge_source", {
      request: {
        projectId: "proj_local",
        kind: "file",
        pathOrRef: "docs/auth.md",
        fingerprint: "sha256:abc",
        status: "current",
      },
    });
    expect(source.id).toBe("ks_1");
  });

  it("maps knowledge source index listing through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        id: "ks_1",
        project_id: "proj_local",
        kind: "file",
        path_or_ref: "docs/auth.md",
        fingerprint: "sha256:abc",
        status: "current",
      },
    ]);

    const sources = await listNativeKnowledgeSources("proj_local");

    expect(invoke).toHaveBeenCalledWith("list_knowledge_sources", {
      projectId: "proj_local",
    });
    expect(sources[0].path_or_ref).toBe("docs/auth.md");
  });

  it("maps knowledge pages and searches through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "kp_1",
        project_id: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        artifact_path: "artifacts/knowledge/auth-flow.md",
        source_ids: ["ks_1"],
        freshness_state: "current",
        body_md: "# Auth Flow",
      })
      .mockResolvedValueOnce([
        {
          id: "kp_1",
          project_id: "proj_local",
          slug: "auth-flow",
          title: "Auth Flow",
          artifact_path: "artifacts/knowledge/auth-flow.md",
          source_ids: ["ks_1"],
          freshness_state: "current",
          body_md: "# Auth Flow",
        },
      ]);

    const page = await saveNativeKnowledgePage({
      projectId: "proj_local",
      slug: "auth-flow",
      title: "Auth Flow",
      bodyMd: "# Auth Flow",
      sourceIds: ["ks_1"],
      freshnessState: "current",
    });
    const pages = await searchNativeKnowledgePages("proj_local", "auth");

    expect(invoke).toHaveBeenCalledWith("save_knowledge_page", {
      request: {
        projectId: "proj_local",
        slug: "auth-flow",
        title: "Auth Flow",
        bodyMd: "# Auth Flow",
        sourceIds: ["ks_1"],
        freshnessState: "current",
      },
    });
    expect(invoke).toHaveBeenCalledWith("search_knowledge_pages", {
      projectId: "proj_local",
      query: "auth",
    });
    expect(page.slug).toBe("auth-flow");
    expect(pages[0].id).toBe("kp_1");
  });

  it("maps context packs and lint reports through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "ctx_auth",
        project_id: "proj_local",
        name: "auth-default",
        description: "Auth docs",
        sources_json: { sources: [{ type: "knowledge_page", id: "kp_1" }] },
      })
      .mockResolvedValueOnce([
        {
          id: "ctx_auth",
          project_id: "proj_local",
          name: "auth-default",
          description: "Auth docs",
          sources_json: { sources: [{ type: "knowledge_page", id: "kp_1" }] },
        },
      ])
      .mockResolvedValueOnce({
        id: "klr_1",
        project_id: "proj_local",
        artifact_path: "artifacts/knowledge/lint/klr_1.md",
        stale_count: 2,
        gap_count: 1,
        contradiction_count: 0,
        body_md: "Gap: rollback",
      });

    const pack = await upsertNativeContextPack({
      id: "ctx_auth",
      projectId: "proj_local",
      name: "auth-default",
      description: "Auth docs",
      sourcesJson: [{ type: "knowledge_page", id: "kp_1" }],
      maxTokensHint: 24000,
    });
    const packs = await listNativeContextPacks("proj_local");
    const lint = await recordNativeKnowledgeLintReport({
      projectId: "proj_local",
      staleCount: 2,
      gapCount: 1,
      contradictionCount: 0,
      bodyMd: "Gap: rollback",
    });

    expect(invoke).toHaveBeenCalledWith("upsert_context_pack", {
      request: {
        id: "ctx_auth",
        projectId: "proj_local",
        name: "auth-default",
        description: "Auth docs",
        sourcesJson: [{ type: "knowledge_page", id: "kp_1" }],
        maxTokensHint: 24000,
      },
    });
    expect(invoke).toHaveBeenCalledWith("record_knowledge_lint_report", {
      request: {
        projectId: "proj_local",
        staleCount: 2,
        gapCount: 1,
        contradictionCount: 0,
        bodyMd: "Gap: rollback",
      },
    });
    expect(pack.id).toBe("ctx_auth");
    expect(invoke).toHaveBeenCalledWith("list_context_packs", {
      projectId: "proj_local",
    });
    expect(packs[0].id).toBe("ctx_auth");
    expect(lint.gap_count).toBe(1);
  });

  it("maps saved knowledge explorations through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "kexp_1",
        project_id: "proj_local",
        title: "Token rollout investigation",
        question: "How should rollback handle token rotation?",
        answer_md: "Keep both issuers during rollback.",
        artifact_path: "artifacts/knowledge/explorations/kexp_1.md",
        page_ids: ["kp_1"],
        context_pack_id: "ctx_auth",
      })
      .mockResolvedValueOnce([
        {
          id: "kexp_1",
          project_id: "proj_local",
          title: "Token rollout investigation",
          question: "How should rollback handle token rotation?",
          answer_md: "Keep both issuers during rollback.",
          artifact_path: "artifacts/knowledge/explorations/kexp_1.md",
          page_ids: ["kp_1"],
          context_pack_id: "ctx_auth",
        },
      ]);

    const exploration = await saveNativeKnowledgeExploration({
      projectId: "proj_local",
      title: "Token rollout investigation",
      question: "How should rollback handle token rotation?",
      answerMd: "Keep both issuers during rollback.",
      pageIds: ["kp_1"],
      contextPackId: "ctx_auth",
    });
    const explorations = await listNativeKnowledgeExplorations("proj_local");

    expect(invoke).toHaveBeenCalledWith("save_knowledge_exploration", {
      request: {
        projectId: "proj_local",
        title: "Token rollout investigation",
        question: "How should rollback handle token rotation?",
        answerMd: "Keep both issuers during rollback.",
        pageIds: ["kp_1"],
        contextPackId: "ctx_auth",
      },
    });
    expect(invoke).toHaveBeenCalledWith("list_knowledge_explorations", {
      projectId: "proj_local",
    });
    expect(exploration.id).toBe("kexp_1");
    expect(explorations[0].context_pack_id).toBe("ctx_auth");
  });

  it("maps knowledge concepts and cross-links through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        slug: "auth-flow",
        title: "Auth Flow",
        page_id: "kp_1",
        outbound_slugs: ["jwt-rotation"],
        inbound_page_ids: ["kp_2"],
      },
    ]);

    const concepts = await listNativeKnowledgeConcepts("proj_local");

    expect(invoke).toHaveBeenCalledWith("list_knowledge_concepts", {
      projectId: "proj_local",
    });
    expect(concepts[0].slug).toBe("auth-flow");
    expect(concepts[0].outbound_slugs).toEqual(["jwt-rotation"]);
    expect(concepts[0].inbound_page_ids).toEqual(["kp_2"]);
  });

  it("maps Obsidian markdown knowledge exports through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_local",
      status: "exported",
      export_root: "artifacts/knowledge/obsidian/proj_local",
      file_count: 3,
      files: ["Auth Flow.md", "JWT Rotation.md", "Knowledge Index.md"],
    });

    const exported = await exportNativeKnowledgeObsidianMarkdown("proj_local");

    expect(invoke).toHaveBeenCalledWith("export_knowledge_obsidian_markdown", {
      projectId: "proj_local",
    });
    expect(exported.status).toBe("exported");
    expect(exported.files).toContain("Knowledge Index.md");
  });

  it("maps local knowledge chat answers through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_local",
      question: "How should rollback handle token rotation?",
      answer_md: "## Local knowledge answer draft\n\n- [[Auth Flow]]",
      cited_page_ids: ["kp_1"],
      context_pack_id: "ctx_auth",
      source_count: 1,
    });

    const answer = await answerNativeKnowledgeQuestion({
      projectId: "proj_local",
      question: "How should rollback handle token rotation?",
      contextPackId: "ctx_auth",
    });

    expect(invoke).toHaveBeenCalledWith("answer_knowledge_question", {
      request: {
        projectId: "proj_local",
        question: "How should rollback handle token rotation?",
        contextPackId: "ctx_auth",
      },
    });
    expect(answer.source_count).toBe(1);
    expect(answer.cited_page_ids).toEqual(["kp_1"]);
  });

  it("maps knowledge automation compile runs through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_local",
      status: "compiled",
      watch_enabled: true,
      source_count: 2,
      page_count: 1,
      stale_count: 1,
      gap_count: 1,
      lint_report_id: "klr_1",
    });

    const run = await runNativeKnowledgeAutomation({
      projectId: "proj_local",
      watch: true,
    });

    expect(invoke).toHaveBeenCalledWith("run_knowledge_automation", {
      request: {
        projectId: "proj_local",
        watch: true,
      },
    });
    expect(run.status).toBe("compiled");
    expect(run.watch_enabled).toBe(true);
    expect(run.lint_report_id).toBe("klr_1");
  });

  it("maps long document and multimodal ingestion through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_local",
      source_id: "ks_1",
      page_id: "kp_1",
      slug: "runbook-pdf",
      modality: "pdf",
      chunk_count: 3,
      fingerprint: "local:pdf:2400:docs/runbook.pdf",
    });

    const result = await ingestNativeKnowledgeArtifact({
      projectId: "proj_local",
      kind: "pdf",
      pathOrRef: "docs/runbook.pdf",
      title: "Runbook PDF",
      bodyMd: "Long runbook text",
      maxChunkChars: 1200,
    });

    expect(invoke).toHaveBeenCalledWith("ingest_knowledge_artifact", {
      request: {
        projectId: "proj_local",
        kind: "pdf",
        pathOrRef: "docs/runbook.pdf",
        title: "Runbook PDF",
        bodyMd: "Long runbook text",
        maxChunkChars: 1200,
      },
    });
    expect(result.modality).toBe("pdf");
    expect(result.chunk_count).toBe(3);
    expect(result.page_id).toBe("kp_1");
  });
});
