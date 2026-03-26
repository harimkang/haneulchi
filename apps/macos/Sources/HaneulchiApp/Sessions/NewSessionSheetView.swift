import SwiftUI

struct NewSessionSheetView: View {
    @Environment(\.viewportContext) private var viewportContext
    @ObservedObject var viewModel: NewSessionSheetViewModel
    let onLaunch: (SessionLaunchDescriptor) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("New Session")
                .font(.title2.weight(.semibold))

            if let workflowSummary = viewModel.workflowSummary {
                VStack(alignment: .leading, spacing: 4) {
                    Text(workflowSummary.name)
                        .font(.headline)
                    Text(
                        "Strategy: \(workflowSummary.strategy) · Base root: \(workflowSummary.baseRoot)",
                    )
                    .font(.caption)
                    .foregroundStyle(.secondary)
                }
            }

            HStack {
                Button("Generic Shell") {
                    if let descriptor = try? viewModel.makeGenericDescriptor() {
                        onLaunch(descriptor)
                    }
                }
                .buttonStyle(.borderedProminent)

                Button("Preset") {
                    if let descriptor = try? viewModel.makePresetDescriptor() {
                        onLaunch(descriptor)
                    }
                }
                .buttonStyle(.bordered)

                Button("Isolated") {
                    if let descriptor = try? viewModel.makeIsolatedDescriptor() {
                        onLaunch(descriptor)
                    }
                }
                .buttonStyle(.bordered)
            }
        }
        .padding(24)
        .frame(
            width: viewportContext.modalWidthPolicy.resolvedWidth(
                preferredWidth: 520,
                availableWidth: viewportContext.width,
            ),
            alignment: .leading,
        )
    }
}
