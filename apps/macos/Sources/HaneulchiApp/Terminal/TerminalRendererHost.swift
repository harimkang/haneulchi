import AppKit
import SwiftTerm
import SwiftUI

struct TerminalRendererHost: NSViewRepresentable {
    let transcript: String

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> TerminalView {
        let terminalView = TerminalView(frame: .zero)
        terminalView.terminalDelegate = context.coordinator
        terminalView.nativeBackgroundColor = .textBackgroundColor
        terminalView.nativeForegroundColor = .textColor
        context.coordinator.render(transcript: transcript, into: terminalView)
        return terminalView
    }

    func updateNSView(_ nsView: TerminalView, context: Context) {
        context.coordinator.render(transcript: transcript, into: nsView)
    }

    final class Coordinator: NSObject, TerminalViewDelegate {
        private var renderedTranscript: String?

        @MainActor
        func render(transcript: String, into terminalView: TerminalView) {
            guard renderedTranscript != transcript else {
                return
            }

            renderedTranscript = transcript
            terminalView.terminal.resetToInitialState()
            terminalView.feed(text: transcript)
        }

        func sizeChanged(source: TerminalView, newCols: Int, newRows: Int) {}
        func setTerminalTitle(source: TerminalView, title: String) {}
        func hostCurrentDirectoryUpdate(source: TerminalView, directory: String?) {}
        func send(source: TerminalView, data: ArraySlice<UInt8>) {}
        func scrolled(source: TerminalView, position: Double) {}
        func requestOpenLink(source: TerminalView, link: String, params: [String: String]) {}
        func bell(source: TerminalView) {}
        func clipboardCopy(source: TerminalView, content: Data) {}
        func iTermContent(source: TerminalView, content: Data) {}
        func rangeChanged(source: TerminalView, startY: Int, endY: Int) {}
    }
}
