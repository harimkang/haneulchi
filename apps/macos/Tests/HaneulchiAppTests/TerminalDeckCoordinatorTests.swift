import AppKit
@testable import HaneulchiApp
import Testing

@MainActor
private final class RecordingHostHandle: TerminalHostHandle {
    private(set) var focusCalls = 0
    private(set) var findCalls = 0
    private(set) var pasteCalls = 0
    private(set) var copyCalls = 0
    private(set) var selectAllCalls = 0
    private(set) var keyDownCalls = 0

    func focusTerminal() {
        focusCalls += 1
    }

    func showFind() {
        findCalls += 1
    }

    func pasteClipboard() {
        pasteCalls += 1
    }

    func copySelection() {
        copyCalls += 1
    }

    func selectAllText() {
        selectAllCalls += 1
    }

    func handleKeyDown(_: NSEvent) {
        keyDownCalls += 1
    }
}

@MainActor
@Test("deck coordinator routes find and paste to the focused pane")
func deckCoordinatorRoutesActionsToFocusedPane() {
    let first = RecordingHostHandle()
    let second = RecordingHostHandle()
    let coordinator = TerminalDeckCoordinator()

    coordinator.register(first, for: "pane-1")
    coordinator.register(second, for: "pane-2")

    coordinator.showFind(in: "pane-2")
    coordinator.pasteClipboard(in: "pane-2")

    #expect(first.findCalls == 0)
    #expect(second.findCalls == 1)
    #expect(second.pasteCalls == 1)
}

@MainActor
@Test("deck coordinator replays a pending focus request when the handle registers later")
func deckCoordinatorReplaysPendingFocusRequestOnRegistration() {
    let handle = RecordingHostHandle()
    let coordinator = TerminalDeckCoordinator()

    coordinator.focusPane("pane-1")
    coordinator.register(handle, for: "pane-1")

    #expect(handle.focusCalls == 1)
}

@MainActor
@Test("deck coordinator routes key events to the derived focused pane")
func deckCoordinatorRoutesKeyEventsToFocusedPane() throws {
    let first = RecordingHostHandle()
    let second = RecordingHostHandle()
    let coordinator = TerminalDeckCoordinator()

    coordinator.register(first, for: "pane-1")
    coordinator.register(second, for: "pane-2")
    coordinator.updateFocusedPane("pane-2")

    let event = try #require(
        NSEvent.keyEvent(
            with: .keyDown,
            location: .zero,
            modifierFlags: [],
            timestamp: 0,
            windowNumber: 0,
            context: nil,
            characters: "a",
            charactersIgnoringModifiers: "a",
            isARepeat: false,
            keyCode: 0,
        ),
    )

    let handled = coordinator.handleKeyDown(event)

    #expect(handled)
    #expect(first.keyDownCalls == 0)
    #expect(second.keyDownCalls == 1)
}
