import AppKit
import Combine
import Foundation

@MainActor
final class TerminalDeckCoordinator: ObservableObject {
    private var handles: [String: TerminalHostHandle] = [:]
    private var pendingFocusPaneID: String?
    private var focusedPaneID: String?

    func register(_ handle: TerminalHostHandle, for paneID: String) {
        handles[paneID] = handle

        if pendingFocusPaneID == paneID {
            handle.focusTerminal()
            pendingFocusPaneID = nil
        }
    }

    func focusPane(_ paneID: String) {
        focusedPaneID = paneID
        guard let handle = handles[paneID] else {
            pendingFocusPaneID = paneID
            return
        }

        handle.focusTerminal()
        pendingFocusPaneID = nil
    }

    func showFind(in paneID: String) {
        handles[paneID]?.showFind()
    }

    func pasteClipboard(in paneID: String) {
        handles[paneID]?.pasteClipboard()
    }

    func copySelection(in paneID: String) {
        handles[paneID]?.copySelection()
    }

    func selectAllText(in paneID: String) {
        handles[paneID]?.selectAllText()
    }

    func updateFocusedPane(_ paneID: String) {
        focusedPaneID = paneID
    }

    func handleKeyDown(_ event: NSEvent) -> Bool {
        guard
            !event.modifierFlags.contains(.command),
            let focusedPaneID,
            let handle = handles[focusedPaneID],
            !handle.isTerminalFirstResponder()
        else {
            return false
        }

        handle.handleKeyDown(event)
        return true
    }
}
