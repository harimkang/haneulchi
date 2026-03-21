import SwiftUI

struct TerminalDeckView: View {
    struct Model: Equatable, Sendable {
        let surfaces: [TerminalSurfaceConfiguration]
        let showsSplitControls: Bool

        static let demo = Self(
            surfaces: [.projectFocusDemo],
            showsSplitControls: false
        )
    }

    let model: Model

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Project Focus")
                .font(.largeTitle)
                .bold()

            Text("Hosted transcript replay is fixed to a single surface in MVP2-008.")
                .foregroundStyle(.secondary)

            ForEach(model.surfaces) { surface in
                TerminalSurfaceView(configuration: surface)
            }
        }
    }
}
