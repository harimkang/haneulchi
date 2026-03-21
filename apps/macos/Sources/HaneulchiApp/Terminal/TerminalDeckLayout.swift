import Foundation

enum TerminalDeckAxis: String, Equatable, Sendable {
    case horizontal
    case vertical
}

struct TerminalPaneModel: Equatable, Identifiable, Sendable {
    let id: String
    let surface: TerminalSurfaceConfiguration
}

indirect enum TerminalDeckNode: Equatable, Sendable {
    case pane(TerminalPaneModel)
    case split(
        id: String,
        axis: TerminalDeckAxis,
        ratio: Double,
        first: TerminalDeckNode,
        second: TerminalDeckNode
    )
}

struct TerminalDeckLayout: Equatable, Sendable {
    var root: TerminalDeckNode
    var focusedPaneID: String
    private var nextPaneNumber: Int
    private var nextSplitNumber: Int

    static let singleDemo = Self(
        root: .pane(.init(id: "pane-1", surface: .projectFocusDemo)),
        focusedPaneID: "pane-1",
        nextPaneNumber: 2,
        nextSplitNumber: 1
    )

    static let singleLiveDemo = Self(
        root: .pane(.init(id: "pane-1", surface: .projectFocusLiveDemo)),
        focusedPaneID: "pane-1",
        nextPaneNumber: 2,
        nextSplitNumber: 1
    )

    static func singleLive(_ bundle: TerminalRestoreBundle) -> Self {
        Self(
            root: .pane(.init(id: "pane-1", surface: .liveSurface(id: "project-focus-live-demo", bundle: bundle))),
            focusedPaneID: "pane-1",
            nextPaneNumber: 2,
            nextSplitNumber: 1
        )
    }

    static func singleLiveShell(at rootPath: String?) -> Self {
        singleLive(.genericShell(at: rootPath))
    }

    var paneIDs: [String] {
        root.paneIDs
    }

    var rootSplitID: String? {
        guard case let .split(id, _, _, _, _) = root else {
            return nil
        }

        return id
    }

    var rootSplitRatio: Double? {
        guard case let .split(_, _, ratio, _, _) = root else {
            return nil
        }

        return ratio
    }

    var focusedSurface: TerminalSurfaceConfiguration? {
        root.pane(for: focusedPaneID)?.surface
    }

    mutating func focusPane(_ paneID: String) {
        guard paneIDs.contains(paneID) else {
            return
        }

        focusedPaneID = paneID
    }

    mutating func splitFocusedPane(axis: TerminalDeckAxis) {
        let originalPaneID = focusedPaneID
        let newPaneID = "pane-\(nextPaneNumber)"
        let splitID = "split-\(nextSplitNumber)"
        nextPaneNumber += 1
        nextSplitNumber += 1

        let focusedSurface = root.pane(for: originalPaneID)?.surface ?? .projectFocusDemo
        let newPane = TerminalPaneModel(
            id: newPaneID,
            surface: focusedSurface.duplicated(withID: "surface-\(newPaneID)")
        )
        root = root.replacingPane(
            id: originalPaneID,
            with: .split(
                id: splitID,
                axis: axis,
                ratio: 0.5,
                first: .pane(root.pane(for: originalPaneID)!),
                second: .pane(newPane)
            )
        )
        focusedPaneID = newPaneID
    }

    mutating func moveFocusForward() {
        let ids = paneIDs
        guard let currentIndex = ids.firstIndex(of: focusedPaneID), !ids.isEmpty else {
            return
        }

        let nextIndex = ids.index(after: currentIndex)
        focusedPaneID = ids[nextIndex == ids.endIndex ? ids.startIndex : nextIndex]
    }

    mutating func moveFocusBackward() {
        let ids = paneIDs
        guard let currentIndex = ids.firstIndex(of: focusedPaneID), !ids.isEmpty else {
            return
        }

        let previousIndex = currentIndex == ids.startIndex
            ? ids.index(before: ids.endIndex)
            : ids.index(before: currentIndex)
        focusedPaneID = ids[previousIndex]
    }

    mutating func setSplitRatio(_ ratio: Double, for splitID: String) {
        root = root.settingRatio(ratio, for: splitID)
    }
}

private extension TerminalDeckNode {
    var paneIDs: [String] {
        switch self {
        case let .pane(pane):
            return [pane.id]
        case let .split(_, _, _, first, second):
            return first.paneIDs + second.paneIDs
        }
    }

    func pane(for paneID: String) -> TerminalPaneModel? {
        switch self {
        case let .pane(pane):
            return pane.id == paneID ? pane : nil
        case let .split(_, _, _, first, second):
            return first.pane(for: paneID) ?? second.pane(for: paneID)
        }
    }

    func replacingPane(id paneID: String, with replacement: TerminalDeckNode) -> TerminalDeckNode {
        switch self {
        case let .pane(pane):
            return pane.id == paneID ? replacement : self
        case let .split(id, axis, ratio, first, second):
            return .split(
                id: id,
                axis: axis,
                ratio: ratio,
                first: first.replacingPane(id: paneID, with: replacement),
                second: second.replacingPane(id: paneID, with: replacement)
            )
        }
    }

    func settingRatio(_ ratio: Double, for splitID: String) -> TerminalDeckNode {
        switch self {
        case .pane:
            return self
        case let .split(id, axis, currentRatio, first, second):
            let updatedRatio = id == splitID ? max(0.1, min(0.9, ratio)) : currentRatio
            return .split(
                id: id,
                axis: axis,
                ratio: updatedRatio,
                first: first.settingRatio(ratio, for: splitID),
                second: second.settingRatio(ratio, for: splitID)
            )
        }
    }
}
