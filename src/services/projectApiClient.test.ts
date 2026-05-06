import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  addNativeProject,
  collectNativeProjectLspDiagnostics,
  exportNativeProjectPatch,
  focusNativeProject,
  importNativeProjectPatch,
  listNativeProjectLayoutPresets,
  listNativeProjectFiles,
  listNativeProjects,
  planNativeProjectDetach,
  planNativePrLanding,
  planNativeReviewPrLanding,
  readNativeProjectDiff,
  readNativeProjectFile,
  saveNativeProjectFile,
  saveNativeProjectLayoutPreset,
  searchNativeProjectFiles,
  upsertNativeProjectTabGroup,
} from "./projectApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("project API client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("lists native projects", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([{ id: "proj_auth", name: "Auth Service" }]);

    await expect(listNativeProjects()).resolves.toEqual([{ id: "proj_auth", name: "Auth Service" }]);

    expect(invoke).toHaveBeenCalledWith("list_projects");
  });

  it("adds and focuses native projects", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ id: "proj_auth", name: "Auth Service" })
      .mockResolvedValueOnce({ id: "proj_auth", name: "Auth Service", status: "active" });

    await addNativeProject({
      key: "AUTH",
      name: "Auth Service",
      path: "/repo/auth-service",
      color: "#059669",
    });
    await focusNativeProject("proj_auth");

    expect(invoke).toHaveBeenNthCalledWith(1, "add_project", {
      request: {
        key: "AUTH",
        name: "Auth Service",
        path: "/repo/auth-service",
        color: "#059669",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "focus_project", {
      projectId: "proj_auth",
    });
  });

  it("lists project file entries through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      root_path: "/repo/auth-service",
      relative_path: "src",
      degraded_reason: null,
      entries: [{ name: "main.rs", path: "src/main.rs", kind: "file", git_status: "modified" }],
    });

    const files = await listNativeProjectFiles("proj_auth", "src");

    expect(invoke).toHaveBeenCalledWith("list_project_files", {
      request: {
        projectId: "proj_auth",
        relativePath: "src",
      },
    });
    expect(files.entries[0].git_status).toBe("modified");
  });

  it("reads a project file preview through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/main.rs",
      name: "main.rs",
      language: "rust",
      body: "fn main() {}\n",
      size_bytes: 13,
      truncated: false,
    });

    const preview = await readNativeProjectFile("proj_auth", "src/main.rs");

    expect(invoke).toHaveBeenCalledWith("read_project_file", {
      request: {
        projectId: "proj_auth",
        path: "src/main.rs",
      },
    });
    expect(preview.language).toBe("rust");
    expect(preview.body).toContain("main");
  });

  it("saves a project file through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "src/main.ts",
      name: "main.ts",
      language: "typescript",
      body: "export const value = 2;\n",
      size_bytes: 24,
      truncated: false,
    });

    const preview = await saveNativeProjectFile("proj_auth", "src/main.ts", "export const value = 2;\n");

    expect(invoke).toHaveBeenCalledWith("write_project_file", {
      request: {
        projectId: "proj_auth",
        path: "src/main.ts",
        body: "export const value = 2;\n",
      },
    });
    expect(preview.body).toContain("value = 2");
  });

  it("searches project file entries through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      query: "login",
      degraded_reason: null,
      entries: [{ name: "login.ts", path: "src/auth/login.ts", kind: "file", git_status: "untracked" }],
    });

    const results = await searchNativeProjectFiles("proj_auth", "login");

    expect(invoke).toHaveBeenCalledWith("search_project_files", {
      request: {
        projectId: "proj_auth",
        query: "login",
      },
    });
    expect(results.entries[0].path).toBe("src/auth/login.ts");
  });

  it("reads project review diffs through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      path: "README.md",
      body: "diff --git a/README.md b/README.md\n+review notes\n",
      file_count: 1,
      truncated: false,
      degraded_reason: null,
    });

    const diff = await readNativeProjectDiff("proj_auth", "README.md");

    expect(invoke).toHaveBeenCalledWith("read_project_diff", {
      request: {
        projectId: "proj_auth",
        path: "README.md",
      },
    });
    expect(diff.body).toContain("+review notes");
  });

  it("collects project LSP diagnostics through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      diagnostics: [
        {
          path: "src/app.ts",
          line: 2,
          severity: "warning",
          message: "TypeScript explicit any weakens local LSP guarantees",
        },
      ],
      degraded_reason: null,
    });

    const diagnostics = await collectNativeProjectLspDiagnostics("proj_auth", "src/app.ts");

    expect(invoke).toHaveBeenCalledWith("collect_project_lsp_diagnostics", {
      request: {
        projectId: "proj_auth",
        path: "src/app.ts",
      },
    });
    expect(diagnostics.diagnostics[0].message).toContain("explicit any");
  });

  it("exports imports and plans PR landing through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        patch_id: "patch_123",
        body: "diff --git a/README.md b/README.md\n+review notes\n",
        file_count: 1,
        status: "exported",
        degraded_reason: null,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        patch_id: "patch_123",
        body: "diff --git a/README.md b/README.md\n+review notes\n",
        file_count: 1,
        status: "validated",
        degraded_reason: null,
      })
      .mockResolvedValueOnce({
        project_id: "proj_auth",
        provider: "github",
        title: "Ship review evidence",
        draft: true,
        checklist: ["export patch and review diff summary"],
        degraded_reason: "network push is intentionally not executed by local planner",
      });

    const patch = await exportNativeProjectPatch("proj_auth", "README.md");
    const imported = await importNativeProjectPatch("proj_auth", patch.body);
    const prPlan = await planNativePrLanding({ projectId: "proj_auth", title: "Ship review evidence", draft: true });

    expect(invoke).toHaveBeenNthCalledWith(1, "export_project_patch", {
      request: { projectId: "proj_auth", path: "README.md" },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "import_project_patch", {
      request: { projectId: "proj_auth", body: patch.body },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "plan_pr_landing", {
      request: { projectId: "proj_auth", title: "Ship review evidence", draft: true },
    });
    expect(imported.status).toBe("validated");
    expect(prPlan.checklist[0]).toContain("export patch");
  });

  it("plans native project detach through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      project_name: "Auth Service",
      window_id: "win_proj_auth",
      status: "planned",
      degraded_reason: null,
    });

    const plan = await planNativeProjectDetach("proj_auth");

    expect(invoke).toHaveBeenCalledWith("plan_project_detach", {
      projectId: "proj_auth",
    });
    expect(plan.window_id).toBe("win_proj_auth");
  });

  it("upserts native project tab groups through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      project_id: "proj_auth",
      group_name: "Backend",
      updated_at: "2026-05-03T01:00:00Z",
    });

    const group = await upsertNativeProjectTabGroup("proj_auth", "Backend");

    expect(invoke).toHaveBeenCalledWith("upsert_project_tab_group", {
      projectId: "proj_auth",
      groupName: "Backend",
    });
    expect(group.group_name).toBe("Backend");
  });

  it("saves and lists native project layout presets through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "layout_preset_1",
        project_id: "proj_auth",
        name: "Review grid",
        layout_json: {
          mode: "grid",
          focusedSessionId: "session_1",
          maximizedSessionId: null,
          panes: ["session_1"],
        },
      })
      .mockResolvedValueOnce([
        {
          id: "layout_preset_1",
          project_id: "proj_auth",
          name: "Review grid",
          layout_json: {
            mode: "grid",
            focusedSessionId: "session_1",
            maximizedSessionId: null,
            panes: ["session_1"],
          },
        },
      ]);

    const saved = await saveNativeProjectLayoutPreset("proj_auth", "Review grid", {
      mode: "grid",
      focusedSessionId: "session_1",
      maximizedSessionId: null,
      panes: ["session_1"],
    });
    const presets = await listNativeProjectLayoutPresets("proj_auth");

    expect(invoke).toHaveBeenNthCalledWith(1, "save_project_layout_preset", {
      request: {
        projectId: "proj_auth",
        name: "Review grid",
        layoutJson: {
          mode: "grid",
          focusedSessionId: "session_1",
          maximizedSessionId: null,
          panes: ["session_1"],
        },
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_project_layout_presets", {
      projectId: "proj_auth",
    });
    expect(saved.name).toBe("Review grid");
    expect(presets[0].layout_json.focusedSessionId).toBe("session_1");
  });

  it("plans native review PR landing through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      review_id: "review_ev_run_1",
      evidence_pack_id: "ev_run_1",
      source_task_id: "task_review",
      source_run_id: "run_1",
      plan: {
        project_id: "proj_auth",
        provider: "github",
        title: "Ship review evidence",
        draft: true,
        checklist: ["link review review_ev_run_1 and evidence pack ev_run_1"],
        degraded_reason: "network push is intentionally not executed by local planner",
      },
    });

    const receipt = await planNativeReviewPrLanding({
      reviewId: "review_ev_run_1",
      title: "Ship review evidence",
      draft: true,
    });

    expect(invoke).toHaveBeenCalledWith("plan_review_pr_landing", {
      request: {
        reviewId: "review_ev_run_1",
        title: "Ship review evidence",
        draft: true,
      },
    });
    expect(receipt.evidence_pack_id).toBe("ev_run_1");
    expect(receipt.plan.checklist[0]).toContain("review_ev_run_1");
  });
});
