import SwiftUI

struct ProjectFocusView: View {
    struct Model: Equatable, Sendable {
        let deck: TerminalDeckView.Model

        static let demo = Self(deck: .demo)
        static let runtimeDemo = Self(deck: .runtimeDemo)

        static func restored(_ bundle: TerminalRestoreBundle) -> Self {
            Self(deck: .restored(bundle))
        }

        static func bootstrap(
            selectedProjectRoot: String? = nil,
            restoreStore: TerminalSessionRestoreStore
        ) throws -> Self {
            if let selectedProjectRoot {
                return .restored(.genericShell(at: selectedProjectRoot))
            }

            let bundles = try restoreStore.load()
            if let bundle = bundles.first {
                return .restored(bundle)
            }

            return .demo
        }
    }

    let model: Model
    let snapshot: AppShellSnapshot?
    let onAction: (AppShellAction) -> Void

    init(
        model: Model,
        snapshot: AppShellSnapshot? = nil,
        onAction: @escaping (AppShellAction) -> Void = { _ in }
    ) {
        self.model = model
        self.snapshot = snapshot
        self.onAction = onAction
    }

    var body: some View {
        HStack(spacing: 0) {
            if let snapshot, !snapshot.sessions.isEmpty {
                SessionStackView(
                    rows: SessionStackView.rows(from: snapshot),
                    onAction: onAction
                )
            }

            TerminalDeckView(model: model.deck)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
    }
}
