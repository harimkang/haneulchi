import Foundation
import Testing
@testable import HaneulchiApp

@Test("spawn response can decode a session identifier before fetching a snapshot")
func spawnResponseDecodesSessionIdentifier() throws {
    let payload = Data(#"{"session_id":"session-0001"}"#.utf8)

    #expect(try decodeSpawnSessionID(from: payload) == "session-0001")
}

@Test("session snapshots can decode when launch geometry is omitted and top-level geometry is present")
func sessionSnapshotDecodesWithoutLaunchGeometry() throws {
    let payload = Data(
        #"""
        {
          "session_id": "session-0001",
          "launch": {
            "program": "/bin/zsh",
            "args": [],
            "current_directory": null
          },
          "geometry": {
            "cols": 80,
            "rows": 24
          },
          "running": true,
          "exit_code": null
        }
        """#.utf8
    )

    let snapshot = try JSONDecoder().decode(TerminalSessionSnapshot.self, from: payload)

    #expect(snapshot.sessionID == "session-0001")
    #expect(snapshot.launch.geometry == TerminalGridSize(cols: 80, rows: 24))
    #expect(snapshot.geometry == TerminalGridSize(cols: 80, rows: 24))
    #expect(snapshot.running)
}

@Test("app shell snapshot payload can decode the richer sprint 2 bridge contract")
func bridgeDecodesStateSnapshotPayload() throws {
    let payload = Data(
        #"""
        {
          "meta": {
            "snapshot_rev": 1,
            "runtime_rev": 1,
            "projection_rev": 1,
            "snapshot_at": "2026-03-22T00:00:00Z"
          },
          "ops": {
            "running_slots": 1,
            "max_slots": 4,
            "retry_queue_count": 0,
            "workflow_health": "ok"
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
            "focused_session_id": "ses_02",
            "degraded_flags": []
          },
          "projects": [],
          "sessions": [],
          "attention": [],
          "retry_queue": [],
          "warnings": []
        }
        """#.utf8
    )

    let snapshot = try JSONDecoder().decode(AppShellSnapshot.self, from: payload)

    #expect(snapshot.workflow?.state == .ok)
    #expect(snapshot.tracker?.health == "ok")
    #expect(snapshot.app.focusedSessionID == "ses_02")
}
