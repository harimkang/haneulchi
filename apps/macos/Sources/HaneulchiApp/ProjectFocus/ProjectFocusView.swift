import SwiftUI

struct ProjectFocusView: View {
    struct Model: Equatable, Sendable {
        let deck: TerminalDeckView.Model

        static let demo = Self(deck: .demo)
        static let runtimeDemo = Self(deck: .runtimeDemo)

        static func restored(_ bundle: TerminalRestoreBundle) -> Self {
            Self(deck: .restored(bundle))
        }

        static func bootstrap(restoreStore: TerminalSessionRestoreStore) throws -> Self {
            let bundles = try restoreStore.load()
            guard let bundle = bundles.first else {
                return .demo
            }

            return .restored(bundle)
        }
    }

    let model: Model

    var body: some View {
        TerminalDeckView(model: model.deck)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .padding(24)
    }
}
