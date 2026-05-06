import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  listNativeAgentProfiles,
  listNativeRuntimePool,
  listNativeSkillPacks,
  scanNativeAgentProfiles,
  heartbeatNativeAgentProfile,
  ingestNativeAgentEvents,
  updateNativeAgentProfileStatus,
  upsertNativeAgentProfile,
  upsertNativeSkillPack,
} from "./agentApiClient";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("agent API client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and scans native agent profiles through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([{ id: "agent_codex", status: "available" }])
      .mockResolvedValueOnce([{ id: "agent_generic_shell", status: "available" }]);

    const listed = await listNativeAgentProfiles();
    const scanned = await scanNativeAgentProfiles();

    expect(invoke).toHaveBeenNthCalledWith(1, "list_agent_profiles");
    expect(invoke).toHaveBeenNthCalledWith(2, "scan_agent_profiles");
    expect(listed[0].id).toBe("agent_codex");
    expect(scanned[0].id).toBe("agent_generic_shell");
  });

  it("updates agent availability through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "agent_codex",
      status: "paused",
    });

    const agent = await updateNativeAgentProfileStatus("agent_codex", "paused");

    expect(invoke).toHaveBeenCalledWith("update_agent_profile_status", {
      agentId: "agent_codex",
      status: "paused",
    });
    expect(agent.status).toBe("paused");
  });

  it("records agent profile heartbeats through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "agent_codex",
      status: "available",
      last_heartbeat_at: "2026-05-02T01:00:00Z",
    });

    const agent = await heartbeatNativeAgentProfile("agent_codex");

    expect(invoke).toHaveBeenCalledWith("heartbeat_agent_profile", {
      agentId: "agent_codex",
    });
    expect(agent.last_heartbeat_at).toBe("2026-05-02T01:00:00Z");
  });

  it("registers third-party CLI adapter profiles through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "agent_acme",
      name: "Acme CLI",
      runtime: "generic-cli",
      command: "acme-agent",
      status: "available",
    });

    const agent = await upsertNativeAgentProfile({
      id: "agent_acme",
      name: "Acme CLI",
      runtime: "generic-cli",
      command: "acme-agent",
      argsJson: ["--json"],
      envPolicyJson: { inherit: false, allow: ["ACME_API_KEY"] },
      skillsJson: ["code-review"],
      status: "available",
    });

    expect(invoke).toHaveBeenCalledWith("upsert_agent_profile", {
      request: {
        id: "agent_acme",
        name: "Acme CLI",
        runtime: "generic-cli",
        command: "acme-agent",
        argsJson: ["--json"],
        envPolicyJson: { inherit: false, allow: ["ACME_API_KEY"] },
        skillsJson: ["code-review"],
        status: "available",
      },
    });
    expect(agent.id).toBe("agent_acme");
  });

  it("saves and lists native skill packs through Tauri invoke", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        id: "skill_pack_1",
        project_id: "proj_auth",
        name: "Auth reviewer",
        description: "Review auth flows",
        skills_json: ["code-review", "auth"],
        source_context_pack_id: "ctx_auth",
      })
      .mockResolvedValueOnce([
        {
          id: "skill_pack_1",
          project_id: "proj_auth",
          name: "Auth reviewer",
          skills_json: ["code-review", "auth"],
        },
      ]);

    const saved = await upsertNativeSkillPack({
      projectId: "proj_auth",
      name: "Auth reviewer",
      description: "Review auth flows",
      skillsJson: ["code-review", "auth"],
      sourceContextPackId: "ctx_auth",
    });
    const listed = await listNativeSkillPacks("proj_auth");

    expect(invoke).toHaveBeenNthCalledWith(1, "upsert_skill_pack", {
      request: {
        projectId: "proj_auth",
        name: "Auth reviewer",
        description: "Review auth flows",
        skillsJson: ["code-review", "auth"],
        sourceContextPackId: "ctx_auth",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "list_skill_packs", {
      projectId: "proj_auth",
    });
    expect(saved.id).toBe("skill_pack_1");
    expect(listed[0].name).toBe("Auth reviewer");
  });

  it("loads native runtime pool through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        id: "agent",
        label: "Cloud agents",
        session_count: 1,
        run_count: 2,
        blocked_count: 1,
      },
    ]);

    const pool = await listNativeRuntimePool("proj_auth");

    expect(invoke).toHaveBeenCalledWith("list_runtime_pool", {
      projectId: "proj_auth",
    });
    expect(pool[0].label).toBe("Cloud agents");
    expect(pool[0].blocked_count).toBe(1);
  });

  it("ingests structured agent events through Tauri invoke", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "agev_1",
      agent_profile_id: "agent_codex",
      kind: "status",
      severity: "warning",
      detail: "Waiting for review",
    });

    const event = await ingestNativeAgentEvents({
      projectId: "proj_local",
      sessionId: "session_1",
      runId: "run_1",
      agentProfileId: "agent_codex",
      adapter: "raw-jsonl",
      payload: {
        raw: "{\"type\":\"status\",\"status\":\"needs_input\",\"message\":\"Waiting for review\"}\n",
      },
    });

    expect(invoke).toHaveBeenCalledWith("ingest_agent_events", {
      request: {
        projectId: "proj_local",
        sessionId: "session_1",
        runId: "run_1",
        agentProfileId: "agent_codex",
        adapter: "raw-jsonl",
        payload: {
          raw: "{\"type\":\"status\",\"status\":\"needs_input\",\"message\":\"Waiting for review\"}\n",
        },
      },
    });
    expect(event.kind).toBe("status");
  });
});
