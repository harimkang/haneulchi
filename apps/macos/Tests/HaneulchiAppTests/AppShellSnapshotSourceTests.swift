import Foundation
import Testing
@testable import HaneulchiApp

@Test("snapshot decodes richer sprint 2 projection groups and session fields")
func snapshotDecodesRicherProjectionContract() throws {
    let payload = """
    {
      "meta": {
        "snapshot_rev": 7,
        "runtime_rev": 4,
        "projection_rev": 9,
        "snapshot_at": "2026-03-22T00:00:00Z"
      },
      "ops": {
        "automation": {
          "status": "running",
          "cadence_ms": 15000,
          "last_tick_at": "2026-03-22T00:00:00Z",
          "last_reconcile_at": null,
          "running_slots": 1,
          "max_slots": 4,
          "retry_due_count": 2,
          "queued_claim_count": 1,
          "paused": false
        },
        "workflow": {
          "state": "ok",
          "path": "/tmp/demo/WORKFLOW.md",
          "last_good_hash": "sha256:abc123",
          "last_reload_at": "2026-03-22T00:00:00Z",
          "last_error": null
        },
        "tracker": {
          "state": "local_only",
          "last_sync_at": null,
          "health": "ok"
        },
        "app": {
          "active_route": "project_focus",
          "focused_session_id": "ses_demo",
          "degraded_flags": []
        }
      },
      "projects": [
        {
          "project_id": "proj_demo",
          "name": "demo",
          "root_path": "/tmp/demo",
          "status": "active",
          "workflow_state": "ok",
          "session_count": 1,
          "task_counts": {
            "Inbox": 1,
            "Ready": 2
          },
          "attention_count": 0
        }
      ],
      "sessions": [
        {
          "session_id": "ses_demo",
          "project_id": "proj_demo",
          "task_id": null,
          "automation_mode": "auto_eligible",
          "tracker_binding_state": "bound",
          "mode": "generic",
          "runtime_state": "running",
          "manual_control": "none",
          "dispatch_state": "not_dispatchable",
          "claim_state": "none",
          "adapter_kind": null,
          "title": "Demo shell",
          "cwd": "/tmp/demo",
          "workspace_root": "/tmp/demo",
          "base_root": ".",
          "branch": "main",
          "latest_summary": "Ready",
          "provider_id": "anthropic",
          "model_id": "claude-sonnet-4",
          "dispatch_reason": "dispatchable",
          "latest_commentary": "Need confirmation before rerun.",
          "commentary_updated_at": "2026-03-22T00:00:02Z",
          "active_window_title": "Terminal 1",
          "unread_count": 0,
          "last_activity_at": "2026-03-22T00:00:01Z",
          "focus_state": "focused",
          "can_focus": true,
          "can_takeover": true,
          "can_release_takeover": false
        }
      ],
      "attention": [],
      "retry_queue": [
        {
          "task_id": "task_demo",
          "project_id": "proj_demo",
          "attempt": 2,
          "reason_code": "hook_failed",
          "due_at": "2026-03-22T00:10:00Z",
          "backoff_ms": 30000,
          "claim_state": "claimed"
        }
      ],
      "recent_artifacts": [
        {
          "task_id": "task_demo",
          "project_id": "proj_demo",
          "summary": "Review ready",
          "jump_target": "review_queue",
          "manifest_path": "evidence/manifest.json"
        }
      ],
      "warnings": []
    }
    """

    let data = try #require(payload.data(using: .utf8))
    let snapshot = try JSONDecoder().decode(AppShellSnapshot.self, from: data)

    #expect(snapshot.workflow?.state == .ok)
    #expect(snapshot.tracker?.health == "ok")
    #expect(snapshot.app.focusedSessionID == "ses_demo")
    #expect(snapshot.projects.first?.taskCounts["Inbox"] == 1)
    #expect(snapshot.sessions.first?.automationMode == .autoEligible)
    #expect(snapshot.sessions.first?.trackerBindingState == "bound")
    #expect(snapshot.sessions.first?.providerID == "anthropic")
    #expect(snapshot.sessions.first?.modelID == "claude-sonnet-4")
    #expect(snapshot.sessions.first?.latestCommentary == "Need confirmation before rerun.")
    #expect(snapshot.sessions.first?.activeWindowTitle == "Terminal 1")
    #expect(snapshot.sessions.first?.claimState == ClaimState.none)
    #expect(snapshot.sessions.first?.focusState == SessionFocusState.focused)
    #expect(snapshot.retryQueue.first?.claimState == .claimed)
    #expect(snapshot.recentArtifacts.first?.jumpTarget == "review_queue")
    #expect(snapshot.retryQueue.first?.reasonCode == "hook_failed")
}

@MainActor
@Test("local shell snapshot mirrors the accepted top-level groups, metadata, and enum vocabulary")
func localSnapshotUsesCurrentShellInputs() async throws {
    let restoreStore = TerminalSessionRestoreStore.inMemory
    try restoreStore.save([.genericShell(at: "/tmp/demo")])

    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let report = ReadinessReport(
        project: project,
        checks: [
            .init(name: .shell, status: .ready, headline: "Shell ready", detail: "zsh available", nextAction: nil),
            .init(name: .presetBinaries, status: .degraded, headline: "Preset binaries missing", detail: "Generic shell remains available.", nextAction: "Open Settings"),
        ]
    )

    let snapshot = try await LocalAppShellSnapshotSource(restoreStore: restoreStore).load(
        activeRoute: .projectFocus,
        selectedProject: project,
        readinessReport: report,
        recentProjects: [project]
    )

    #expect(snapshot.meta.snapshotRev >= 1)
    #expect(snapshot.meta.runtimeRev >= 1)
    #expect(snapshot.meta.projectionRev >= 1)
    #expect(snapshot.app.activeRoute == .projectFocus)
    #expect(snapshot.ops.runningSlots == 0)
    #expect(snapshot.ops.retryQueueCount == 0)
    #expect(snapshot.tracker?.health == "ok")
    #expect(snapshot.projects.map(\.rootPath) == ["/tmp/demo"])
    #expect(snapshot.sessions.count == 1)
    #expect(snapshot.sessions.first?.mode == .generic)
    #expect(snapshot.sessions.first?.runtimeState == .exited)
    #expect(snapshot.attention.count == 1)
    #expect(snapshot.attention.first?.headline == "Preset binaries missing")
    #expect(snapshot.attention.first?.targetRoute == .attentionCenter)
    #expect(snapshot.retryQueue.isEmpty)
    #expect(snapshot.recentArtifacts.isEmpty)
    #expect(snapshot.warnings.map(\.severity) == [.degraded])
}
