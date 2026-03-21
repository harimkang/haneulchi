import Foundation
import Testing
@testable import HaneulchiApp

@MainActor
@Test("live session controller drains terminal output from the Rust runtime")
func liveSessionControllerDrainsOutput() async throws {
    let bridge = CoreBridge.mockLiveSession(outputChunks: ["ready\n"])
    let controller = TerminalSessionController(bridge: bridge)

    try await controller.start(.defaultShell)

    #expect(controller.status == .running)
    #expect(controller.latestText.contains("ready"))
}
