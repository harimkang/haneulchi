import Foundation
import SwiftTerm
import Testing
@testable import HaneulchiApp

@MainActor
@Test("renderer host resets terminal state before replaying a new transcript")
func rendererHostResetsTerminalStateBeforeReplay() throws {
    let coordinator = TerminalRendererHost.Coordinator()
    let terminalView = TerminalView(frame: .zero)

    coordinator.render(transcript: "first\n", into: terminalView)
    coordinator.render(transcript: "second\n", into: terminalView)

    let rendered = try #require(
        String(data: terminalView.terminal.getBufferAsData(), encoding: .utf8)
    )
    let normalized = rendered.replacingOccurrences(of: "\n", with: "")

    #expect(normalized.contains("second"))
    #expect(normalized.contains("first") == false)
}
