import AppKit
@preconcurrency import SwiftTerm
import SwiftUI

@MainActor
protocol TerminalCommandTarget: AnyObject {
    func focusTerminal()
    func showFind()
    func pasteClipboard()
    func copySelection()
    func selectAllText()
    func handleKeyDown(_ event: NSEvent)
    func isTerminalFirstResponder() -> Bool
}

@MainActor
final class SwiftTermTerminalHostHandle: TerminalHostHandle {
    private let commandTarget: TerminalCommandTarget

    init(commandTarget: TerminalCommandTarget) {
        self.commandTarget = commandTarget
    }

    func focusTerminal() {
        commandTarget.focusTerminal()
    }

    func showFind() {
        commandTarget.showFind()
    }

    func pasteClipboard() {
        commandTarget.pasteClipboard()
    }

    func copySelection() {
        commandTarget.copySelection()
    }

    func selectAllText() {
        commandTarget.selectAllText()
    }

    func handleKeyDown(_ event: NSEvent) {
        commandTarget.handleKeyDown(event)
    }

    func isTerminalFirstResponder() -> Bool {
        commandTarget.isTerminalFirstResponder()
    }
}

@MainActor
final class SwiftTermTerminalCommandTarget: TerminalCommandTarget {
    private weak var terminalView: TerminalView?
    private var pendingFocusAttempts = 0

    init(terminalView: TerminalView) {
        self.terminalView = terminalView
    }

    func focusTerminal() {
        attemptFocus()
    }

    func showFind() {
        let item = NSMenuItem()
        item.tag = NSTextFinder.Action.showFindInterface.rawValue
        terminalView?.performTextFinderAction(item)
    }

    func pasteClipboard() {
        terminalView?.paste(self)
    }

    func copySelection() {
        terminalView?.copy(self)
    }

    func selectAllText() {
        terminalView?.selectAll(self)
    }

    func handleKeyDown(_ event: NSEvent) {
        guard let terminalView else {
            return
        }

        let flags = event.modifierFlags

        if flags.contains(.command) {
            return
        }

        switch event.keyCode {
        case 36:
            terminalView.send(EscapeSequences.cmdRet)
            return
        case 48:
            terminalView
                .send(flags.contains(.shift) ? EscapeSequences.cmdBackTab : EscapeSequences.cmdTab)
            return
        case 51:
            terminalView.send(EscapeSequences.cmdDel)
            return
        case 53:
            terminalView.send(EscapeSequences.cmdEsc)
            return
        default:
            break
        }

        if let chars = event.charactersIgnoringModifiers,
           let scalar = chars.unicodeScalars.first
        {
            switch Int(scalar.value) {
            case NSUpArrowFunctionKey:
                terminalView
                    .send(terminalView.terminal.applicationCursor ? EscapeSequences
                        .moveUpApp : EscapeSequences.moveUpNormal)
                return
            case NSDownArrowFunctionKey:
                terminalView
                    .send(terminalView.terminal.applicationCursor ? EscapeSequences
                        .moveDownApp : EscapeSequences.moveDownNormal)
                return
            case NSLeftArrowFunctionKey:
                terminalView
                    .send(terminalView.terminal.applicationCursor ? EscapeSequences
                        .moveLeftApp : EscapeSequences.moveLeftNormal)
                return
            case NSRightArrowFunctionKey:
                terminalView
                    .send(terminalView.terminal.applicationCursor ? EscapeSequences
                        .moveRightApp : EscapeSequences.moveRightNormal)
                return
            case NSHomeFunctionKey:
                terminalView
                    .send(terminalView.terminal.applicationCursor ? EscapeSequences
                        .moveHomeApp : EscapeSequences.moveHomeNormal)
                return
            case NSEndFunctionKey:
                terminalView
                    .send(terminalView.terminal.applicationCursor ? EscapeSequences
                        .moveEndApp : EscapeSequences.moveEndNormal)
                return
            case NSPageUpFunctionKey:
                terminalView.send(EscapeSequences.cmdPageUp)
                return
            case NSPageDownFunctionKey:
                terminalView.send(EscapeSequences.cmdPageDown)
                return
            default:
                break
            }
        }

        if flags.contains(.control), let chars = event.charactersIgnoringModifiers,
           let scalar = chars.unicodeScalars.first
        {
            let value = scalar.value
            if value >= 0x40, value <= 0x7F {
                terminalView.send([UInt8(value & 0x1F)])
                return
            }
        }

        if flags.contains(.option), let chars = event.charactersIgnoringModifiers {
            terminalView.send(EscapeSequences.cmdEsc)
            terminalView.send(txt: chars)
            return
        }

        if let chars = event.characters, !chars.isEmpty {
            terminalView.send(txt: chars)
        }
    }

    func isTerminalFirstResponder() -> Bool {
        guard let terminalView else {
            return false
        }

        return terminalView.window?.firstResponder === terminalView
    }

    private func attemptFocus() {
        guard let terminalView else {
            return
        }

        if let window = terminalView.window {
            NSApp.activate(ignoringOtherApps: true)
            window.makeKeyAndOrderFront(nil)
            window.makeFirstResponder(terminalView)
            pendingFocusAttempts = 0
            return
        }

        guard pendingFocusAttempts < 10 else {
            pendingFocusAttempts = 0
            return
        }

        pendingFocusAttempts += 1
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
            self?.attemptFocus()
        }
    }
}

@MainActor
final class FocusingTerminalContainerView: NSView {
    let terminalView: TerminalView
    var onFocusRequested: (() -> Void)?
    private var mouseMonitor: Any?

    init(frame frameRect: NSRect, terminalView: TerminalView) {
        self.terminalView = terminalView
        super.init(frame: frameRect)
        terminalView.frame = bounds
        terminalView.autoresizingMask = [.width, .height]
        addSubview(terminalView)
    }

    @available(*, unavailable)
    required init?(coder _: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()

        if window == nil, let mouseMonitor {
            NSEvent.removeMonitor(mouseMonitor)
            self.mouseMonitor = nil
            return
        }

        guard mouseMonitor == nil else {
            return
        }

        mouseMonitor = NSEvent
            .addLocalMonitorForEvents(matching: [.leftMouseDown]) { [weak self] event in
                guard let self else {
                    return event
                }

                return handlePotentialMouseDown(event)
            }
    }

    override func acceptsFirstMouse(for _: NSEvent?) -> Bool {
        true
    }

    override func mouseDown(with event: NSEvent) {
        window?.makeFirstResponder(terminalView)
        onFocusRequested?()
        terminalView.mouseDown(with: event)
    }

    func handlePotentialMouseDown(_ event: NSEvent) -> NSEvent {
        guard let window, event.window === window else {
            return event
        }

        let point = terminalView.convert(event.locationInWindow, from: nil)
        guard terminalView.bounds.contains(point) else {
            return event
        }

        window.makeFirstResponder(terminalView)
        onFocusRequested?()
        return event
    }
}

struct TerminalRendererHost: NSViewRepresentable {
    typealias HostHandleReady = @MainActor (TerminalHostHandle) -> Void
    typealias FocusRequested = @MainActor () -> Void

    enum RenderMode {
        case replay
        case live
    }

    enum RenderInstruction: Equatable {
        case none
        case replace(String)
        case append(String)
    }

    private enum Source {
        case replay(String)
        case live(
            text: @Sendable () -> String,
            write: @Sendable (Data) -> Void,
            resize: @Sendable (TerminalGridSize) -> Void,
        )

        var text: String {
            switch self {
            case let .replay(transcript):
                transcript
            case let .live(text, _, _):
                text()
            }
        }

        var mode: RenderMode {
            switch self {
            case .replay:
                .replay
            case .live:
                .live
            }
        }
    }

    private let source: Source
    private let onHostHandleReady: HostHandleReady?
    private let onFocusRequested: FocusRequested?

    init(
        transcript: String,
        onHostHandleReady: HostHandleReady? = nil,
        onFocusRequested: FocusRequested? = nil,
    ) {
        source = .replay(transcript)
        self.onHostHandleReady = onHostHandleReady
        self.onFocusRequested = onFocusRequested
    }

    private init(
        source: Source,
        onHostHandleReady: HostHandleReady? = nil,
        onFocusRequested: FocusRequested? = nil,
    ) {
        self.source = source
        self.onHostHandleReady = onHostHandleReady
        self.onFocusRequested = onFocusRequested
    }

    static func live(
        controller: TerminalSessionController,
        onHostHandleReady: HostHandleReady? = nil,
        onFocusRequested: FocusRequested? = nil,
    ) -> Self {
        Self(
            source: .live(
                text: {
                    MainActor.assumeIsolated {
                        controller.latestText
                    }
                },
                write: { data in
                    Task { @MainActor in
                        try? controller.write(data)
                    }
                },
                resize: { geometry in
                    Task { @MainActor in
                        try? controller.resize(geometry)
                    }
                },
            ),
            onHostHandleReady: onHostHandleReady,
            onFocusRequested: onFocusRequested,
        )
    }

    func makeCoordinator() -> Coordinator {
        switch source {
        case .replay:
            Coordinator()
        case let .live(_, write, resize):
            Coordinator(writeHandler: write, resizeHandler: resize)
        }
    }

    func makeNSView(context: Context) -> FocusingTerminalContainerView {
        let terminalView = TerminalView(frame: .zero)
        terminalView.terminalDelegate = context.coordinator
        terminalView.nativeBackgroundColor = .textBackgroundColor
        terminalView.nativeForegroundColor = .textColor
        onHostHandleReady?(
            SwiftTermTerminalHostHandle(
                commandTarget: SwiftTermTerminalCommandTarget(terminalView: terminalView),
            ),
        )
        context.coordinator.render(text: source.text, mode: source.mode, into: terminalView)
        let container = FocusingTerminalContainerView(frame: .zero, terminalView: terminalView)
        container.onFocusRequested = onFocusRequested
        return container
    }

    func updateNSView(_ nsView: FocusingTerminalContainerView, context: Context) {
        nsView.onFocusRequested = onFocusRequested
        context.coordinator.render(text: source.text, mode: source.mode, into: nsView.terminalView)
    }

    final class Coordinator: NSObject, TerminalViewDelegate {
        private let writeHandler: @Sendable (Data) -> Void
        private let resizeHandler: @Sendable (TerminalGridSize) -> Void
        private var renderedTranscript: String?

        init(
            writeHandler: @escaping @Sendable (Data) -> Void = { _ in },
            resizeHandler: @escaping @Sendable (TerminalGridSize) -> Void = { _ in },
        ) {
            self.writeHandler = writeHandler
            self.resizeHandler = resizeHandler
        }

        static func renderInstruction(
            previousText: String?,
            nextText: String,
            mode: RenderMode,
        ) -> RenderInstruction {
            guard previousText != nextText else {
                return .none
            }

            switch mode {
            case .replay:
                return .replace(nextText)
            case .live:
                guard let previousText else {
                    return .replace(nextText)
                }

                if nextText.hasPrefix(previousText) {
                    let suffix = String(nextText.dropFirst(previousText.count))
                    return suffix.isEmpty ? .none : .append(suffix)
                }

                return .replace(nextText)
            }
        }

        @MainActor
        func render(text: String, mode: RenderMode, into terminalView: TerminalView) {
            let instruction = Self.renderInstruction(
                previousText: renderedTranscript,
                nextText: text,
                mode: mode,
            )
            renderedTranscript = text

            switch instruction {
            case .none:
                return
            case let .replace(fullText):
                terminalView.terminal.resetToInitialState()
                terminalView.feed(text: fullText)
            case let .append(delta):
                terminalView.feed(text: delta)
            }
        }

        func sizeChanged(source _: TerminalView, newCols: Int, newRows: Int) {
            resizeHandler(.init(cols: newCols, rows: newRows))
        }

        func setTerminalTitle(source _: TerminalView, title _: String) {}
        func hostCurrentDirectoryUpdate(source _: TerminalView, directory _: String?) {}
        func send(source _: TerminalView, data: ArraySlice<UInt8>) {
            writeHandler(Data(data))
        }

        func scrolled(source _: TerminalView, position _: Double) {}
        func requestOpenLink(source _: TerminalView, link: String, params _: [String: String]) {
            guard let url = URL(string: link) else {
                return
            }

            NSWorkspace.shared.open(url)
        }

        func bell(source _: TerminalView) {}
        func clipboardCopy(source _: TerminalView, content _: Data) {}
        func iTermContent(source _: TerminalView, content _: Data) {}
        func rangeChanged(source _: TerminalView, startY _: Int, endY _: Int) {}

        @MainActor
        func containsScrollbackMarker(_ marker: String, in terminalView: TerminalView) -> Bool {
            if let renderedTranscript, renderedTranscript.contains(marker) {
                return true
            }

            guard let rendered = String(
                data: terminalView.terminal.getBufferAsData(),
                encoding: .utf8,
            ) else {
                return false
            }

            return rendered.contains(marker)
        }
    }
}
