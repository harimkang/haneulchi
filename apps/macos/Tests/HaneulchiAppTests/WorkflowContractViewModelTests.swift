import Foundation
@testable import HaneulchiAppUI
import Testing

@Test("workflow status payload decodes summary and reload diagnostics")
func workflowStatusPayloadDecodesSummary() throws {
    let payload = Data(
        #"""
        {
          "state": "invalid_kept_last_good",
          "path": "/tmp/demo/WORKFLOW.md",
          "last_good_hash": "sha256:abc123",
          "last_reload_at": "2026-03-22T00:00:00Z",
          "last_error": "front matter parse error",
          "workflow": {
            "name": "Demo Workflow",
            "strategy": "worktree",
            "base_root": ".",
            "review_checklist": ["tests passed"],
            "allowed_agents": ["codex", "claude"],
            "hooks": ["after_create", "before_run"]
          }
        }
        """#.utf8,
    )

    let status = try JSONDecoder().decode(WorkflowStatusPayload.self, from: payload)

    #expect(status.state == .invalidKeptLastGood)
    #expect(status.workflow?.name == "Demo Workflow")
    #expect(status.workflow?.allowedAgents == ["codex", "claude"])
    #expect(status.workflow?.hooks == ["after_create", "before_run"])
}
