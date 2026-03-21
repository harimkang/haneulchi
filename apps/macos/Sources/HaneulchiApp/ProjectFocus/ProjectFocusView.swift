import SwiftUI

struct ProjectFocusView: View {
    struct Model: Equatable, Sendable {
        let deck: TerminalDeckView.Model

        static let demo = Self(deck: .demo)
    }

    let model: Model

    var body: some View {
        TerminalDeckView(model: model.deck)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .padding(24)
    }
}
