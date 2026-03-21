import AppKit
@preconcurrency import SwiftTerm
import SwiftUI

struct TerminalRendererHost: NSViewRepresentable {
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
            resize: @Sendable (TerminalGridSize) -> Void
        )

        var text: String {
            switch self {
            case let .replay(transcript):
                return transcript
            case let .live(text, _, _):
                return text()
            }
        }

        var mode: RenderMode {
            switch self {
            case .replay:
                return .replay
            case .live:
                return .live
            }
        }
    }

    private let source: Source

    init(transcript: String) {
        self.source = .replay(transcript)
    }

    private init(source: Source) {
        self.source = source
    }

    static func live(controller: TerminalSessionController) -> Self {
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
                }
            )
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

    func makeNSView(context: Context) -> TerminalView {
        let terminalView = TerminalView(frame: .zero)
        terminalView.terminalDelegate = context.coordinator
        terminalView.nativeBackgroundColor = .textBackgroundColor
        terminalView.nativeForegroundColor = .textColor
        context.coordinator.render(text: source.text, mode: source.mode, into: terminalView)
        return terminalView
    }

    func updateNSView(_ nsView: TerminalView, context: Context) {
        context.coordinator.render(text: source.text, mode: source.mode, into: nsView)
    }

    final class Coordinator: NSObject, TerminalViewDelegate {
        private let writeHandler: @Sendable (Data) -> Void
        private let resizeHandler: @Sendable (TerminalGridSize) -> Void
        private var renderedTranscript: String?

        init(
            writeHandler: @escaping @Sendable (Data) -> Void = { _ in },
            resizeHandler: @escaping @Sendable (TerminalGridSize) -> Void = { _ in }
        ) {
            self.writeHandler = writeHandler
            self.resizeHandler = resizeHandler
        }

        static func renderInstruction(
            previousText: String?,
            nextText: String,
            mode: RenderMode
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
                mode: mode
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

        func sizeChanged(source: TerminalView, newCols: Int, newRows: Int) {
            resizeHandler(.init(cols: newCols, rows: newRows))
        }
        func setTerminalTitle(source: TerminalView, title: String) {}
        func hostCurrentDirectoryUpdate(source: TerminalView, directory: String?) {}
        func send(source: TerminalView, data: ArraySlice<UInt8>) {
            writeHandler(Data(data))
        }
        func scrolled(source: TerminalView, position: Double) {}
        func requestOpenLink(source: TerminalView, link: String, params: [String: String]) {
            guard let url = URL(string: link) else {
                return
            }

            NSWorkspace.shared.open(url)
        }
        func bell(source: TerminalView) {}
        func clipboardCopy(source: TerminalView, content: Data) {}
        func iTermContent(source: TerminalView, content: Data) {}
        func rangeChanged(source: TerminalView, startY: Int, endY: Int) {}

        @MainActor
        func containsScrollbackMarker(_ marker: String, in terminalView: TerminalView) -> Bool {
            if let renderedTranscript, renderedTranscript.contains(marker) {
                return true
            }

            guard let rendered = String(data: terminalView.terminal.getBufferAsData(), encoding: .utf8) else {
                return false
            }

            return rendered.contains(marker)
        }
    }
}
