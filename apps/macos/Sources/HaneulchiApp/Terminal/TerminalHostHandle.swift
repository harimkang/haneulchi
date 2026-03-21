import AppKit
import Foundation

@MainActor
protocol TerminalHostHandle: AnyObject {
    func focusTerminal()
    func showFind()
    func pasteClipboard()
    func copySelection()
    func selectAllText()
    func handleKeyDown(_ event: NSEvent)
}
