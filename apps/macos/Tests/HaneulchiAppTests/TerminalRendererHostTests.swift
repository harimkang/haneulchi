import AppKit
import Foundation
@testable import HaneulchiApp
import SwiftTerm
import Testing

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
private final class RecordingTerminalCommandTarget: TerminalCommandTarget {
    private(set) var focusCalls = 0
    private(set) var findCalls = 0
    private(set) var pasteCalls = 0
    private(set) var copyCalls = 0
    private(set) var selectAllCalls = 0

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

    func handleKeyDown(_: NSEvent) {}
}

private final class CapturingTerminalDelegate: TerminalViewDelegate, @unchecked Sendable {
    private let lock = NSLock()
    private(set) var sentPayloads: [Data] = []

    func sizeChanged(source _: TerminalView, newCols _: Int, newRows _: Int) {}
    func setTerminalTitle(source _: TerminalView, title _: String) {}
    func hostCurrentDirectoryUpdate(source _: TerminalView, directory _: String?) {}
    func send(source _: TerminalView, data: ArraySlice<UInt8>) {
        lock.lock()
        sentPayloads.append(Data(data))
        lock.unlock()
    }

    func scrolled(source _: TerminalView, position _: Double) {}
    func requestOpenLink(source _: TerminalView, link _: String, params _: [String: String]) {}
    func bell(source _: TerminalView) {}
    func clipboardCopy(source _: TerminalView, content _: Data) {}
    func iTermContent(source _: TerminalView, content _: ArraySlice<UInt8>) {}
    func rangeChanged(source _: TerminalView, startY _: Int, endY _: Int) {}
}

@MainActor
@Test("renderer host resets terminal state before replaying a new transcript")
func rendererHostResetsTerminalStateBeforeReplay() throws {
    let coordinator = TerminalRendererHost.Coordinator()
    let terminalView = TerminalView(frame: .zero)

    coordinator.render(text: "first\n", mode: .replay, into: terminalView)
    coordinator.render(text: "second\n", mode: .replay, into: terminalView)

    let rendered = try #require(
        String(data: terminalView.terminal.getBufferAsData(), encoding: .utf8),
    )
    let normalized = rendered.replacingOccurrences(of: "\n", with: "")

    #expect(normalized.contains("second"))
    #expect(normalized.contains("first") == false)
}

@MainActor
@Test("renderer host forwards keystrokes and resize events to the session controller")
func rendererHostForwardsInputAndResize() {
    let controller = RecordingTerminalSessionController()
    let coordinator = TerminalRendererHost.Coordinator(
        writeHandler: { data in
            try? controller.write(data)
        },
        resizeHandler: { geometry in
            try? controller.resize(geometry)
        },
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
    let overflowTranscript = (1 ... 200).map { "line \($0)\n" }.joined()
    let controller =
        TerminalSessionController(bridge: .mockLiveSession(outputChunks: [overflowTranscript]))
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
        mode: .live,
    )

    #expect(instruction == .append(" world"))
}

@MainActor
@Test("host handle forwards actions to the wrapped command target")
func hostHandleForwardsActions() {
    let target = RecordingTerminalCommandTarget()
    let handle = SwiftTermTerminalHostHandle(commandTarget: target)

    handle.focusTerminal()
    handle.showFind()
    handle.pasteClipboard()

    #expect(target.focusCalls == 1)
    #expect(target.findCalls == 1)
    #expect(target.pasteCalls == 1)
}

@MainActor
@Test("command target sends plain text and return directly to the terminal delegate")
func commandTargetSendsPlainTextAndReturn() throws {
    let terminalView = TerminalView(frame: NSRect(x: 0, y: 0, width: 300, height: 200))
    let delegate = CapturingTerminalDelegate()
    terminalView.terminalDelegate = delegate
    let target = SwiftTermTerminalCommandTarget(terminalView: terminalView)

    let textEvent = try #require(
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
    let returnEvent = try #require(
        NSEvent.keyEvent(
            with: .keyDown,
            location: .zero,
            modifierFlags: [],
            timestamp: 0,
            windowNumber: 0,
            context: nil,
            characters: "\r",
            charactersIgnoringModifiers: "\r",
            isARepeat: false,
            keyCode: 36,
        ),
    )

    target.handleKeyDown(textEvent)
    target.handleKeyDown(returnEvent)

    #expect(delegate.sentPayloads.contains(Data("a".utf8)))
    #expect(delegate.sentPayloads.contains(Data([13])))
}

@MainActor
@Test("command target can focus a terminal view after it is attached to a window")
func commandTargetFocusesTerminalAfterWindowAttachment() throws {
    let window = NSWindow(
        contentRect: NSRect(x: 0, y: 0, width: 400, height: 300),
        styleMask: [.titled, .closable, .resizable],
        backing: .buffered,
        defer: false,
    )
    let container = try NSView(frame: #require(window.contentView?.bounds))
    let terminalView = TerminalView(frame: container.bounds)
    container.addSubview(terminalView)
    window.contentView = container

    let target = SwiftTermTerminalCommandTarget(terminalView: terminalView)
    target.focusTerminal()

    RunLoop.main.run(until: Date().addingTimeInterval(0.1))

    #expect(window.firstResponder === terminalView)
}

@MainActor
@Test("command target retries focus until the window becomes key")
func commandTargetRetriesFocusUntilWindowBecomesKey() throws {
    let window = NSWindow(
        contentRect: NSRect(x: 0, y: 0, width: 400, height: 300),
        styleMask: [.titled, .closable, .resizable],
        backing: .buffered,
        defer: false,
    )
    let terminalView = TerminalView(frame: NSRect(x: 0, y: 0, width: 300, height: 200))
    let container = try NSView(frame: #require(window.contentView?.bounds))
    container.addSubview(terminalView)
    window.contentView = container

    let target = SwiftTermTerminalCommandTarget(terminalView: terminalView)
    target.focusTerminal()

    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
        NSApp.activate(ignoringOtherApps: true)
        window.makeKeyAndOrderFront(nil)
    }

    RunLoop.main.run(until: Date().addingTimeInterval(0.6))

    #expect(window.firstResponder === terminalView)
}

@MainActor
@Test("focusing terminal container takes first responder on mouse down")
func focusingTerminalContainerTakesFirstResponderOnMouseDown() throws {
    let window = NSWindow(
        contentRect: NSRect(x: 0, y: 0, width: 400, height: 300),
        styleMask: [.titled, .closable, .resizable],
        backing: .buffered,
        defer: false,
    )
    let terminalView = TerminalView(frame: NSRect(x: 0, y: 0, width: 300, height: 200))
    let view = FocusingTerminalContainerView(
        frame: NSRect(x: 0, y: 0, width: 300, height: 200),
        terminalView: terminalView,
    )
    let container = try NSView(frame: #require(window.contentView?.bounds))
    container.addSubview(view)
    window.contentView = container
    window.makeKeyAndOrderFront(nil)

    let event = try #require(
        NSEvent.mouseEvent(
            with: .leftMouseDown,
            location: NSPoint(x: 10, y: 10),
            modifierFlags: [],
            timestamp: 0,
            windowNumber: window.windowNumber,
            context: nil,
            eventNumber: 1,
            clickCount: 1,
            pressure: 1,
        ),
    )

    view.mouseDown(with: event)

    #expect(window.firstResponder === terminalView)
}
