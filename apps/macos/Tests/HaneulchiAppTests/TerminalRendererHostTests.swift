import Foundation
import SwiftTerm
import Testing
@testable import HaneulchiApp

private final class RecordingTerminalSessionController: @unchecked Sendable {
    private(set) var sentPayloads: [Data] = []
    private(set) var lastResize: TerminalGridSize?

    func write(_ data: Data) throws {
        sentPayloads.append(data)
    }

    func resize(_ geometry: TerminalGridSize) throws {
        lastResize = geometry
    }
}

@MainActor
@Test("renderer host resets terminal state before replaying a new transcript")
func rendererHostResetsTerminalStateBeforeReplay() throws {
    let coordinator = TerminalRendererHost.Coordinator()
    let terminalView = TerminalView(frame: .zero)

    coordinator.render(text: "first\n", mode: .replay, into: terminalView)
    coordinator.render(text: "second\n", mode: .replay, into: terminalView)

    let rendered = try #require(
        String(data: terminalView.terminal.getBufferAsData(), encoding: .utf8)
    )
    let normalized = rendered.replacingOccurrences(of: "\n", with: "")

    #expect(normalized.contains("second"))
    #expect(normalized.contains("first") == false)
}

@MainActor
@Test("renderer host forwards keystrokes and resize events to the session controller")
func rendererHostForwardsInputAndResize() throws {
    let controller = RecordingTerminalSessionController()
    let coordinator = TerminalRendererHost.Coordinator(
        writeHandler: { data in
            try? controller.write(data)
        },
        resizeHandler: { geometry in
            try? controller.resize(geometry)
        }
    )
    let terminalView = TerminalView(frame: .zero)

    coordinator.send(source: terminalView, data: Array("ls\n".utf8)[...])
    coordinator.sizeChanged(source: terminalView, newCols: 120, newRows: 40)

    #expect(controller.sentPayloads.last == Data("ls\n".utf8))
    #expect(controller.lastResize == .init(cols: 120, rows: 40))
}

@MainActor
@Test("renderer host retains scrollback markers after overflow and resize")
func rendererHostRetainsScrollbackMarkersAfterOverflowAndResize() async throws {
    let overflowTranscript = (1...200).map { "line \($0)\n" }.joined()
    let controller = TerminalSessionController(bridge: .mockLiveSession(outputChunks: [overflowTranscript]))
    let coordinator = TerminalRendererHost.Coordinator()
    let terminalView = TerminalView(frame: .zero)

    try await controller.start(.defaultShell)

    coordinator.render(text: controller.latestText, mode: .live, into: terminalView)
    coordinator.sizeChanged(source: terminalView, newCols: 100, newRows: 20)

    #expect(coordinator.containsScrollbackMarker("line 1", in: terminalView))
    #expect(coordinator.containsScrollbackMarker("line 200", in: terminalView))
}

@Test("live renderer appends incremental text instead of replaying the full transcript")
func liveRendererUsesAppendInstructionForStreamingUpdates() {
    let instruction = TerminalRendererHost.Coordinator.renderInstruction(
        previousText: "hello",
        nextText: "hello world",
        mode: .live
    )

    #expect(instruction == .append(" world"))
}
