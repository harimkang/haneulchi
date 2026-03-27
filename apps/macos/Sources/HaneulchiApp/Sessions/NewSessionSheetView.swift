import SwiftUI

enum NewSessionSheetActionLayout: Equatable {
    case inline
    case stacked
}

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

            switch Self.actionLayout(for: measuredModalWidth) {
            case .inline:
                HStack {
                    genericShellButton
                    presetButton
                    isolatedButton
                }
            case .stacked:
                VStack(alignment: .leading, spacing: 12) {
                    genericShellButton
                    presetButton
                    isolatedButton
                }
            }
        }
        .padding(24)
        .frame(
            width: viewportContext.modalWidthPolicy.resolvedWidth(
                preferredWidth: 520,
                availableWidth: measuredModalWidth,
            ),
            alignment: .leading,
        )
    }

    nonisolated static func actionLayout(
        for availableWidth: CGFloat?,
    ) -> NewSessionSheetActionLayout {
        guard let availableWidth else {
            return .inline
        }

        return availableWidth < 440 ? .stacked : .inline
    }

    private var measuredModalWidth: CGFloat? {
        viewportContext.width > 0 ? viewportContext.width : nil
    }

    private var genericShellButton: some View {
        Button("Generic Shell") {
            if let descriptor = try? viewModel.makeGenericDescriptor() {
                onLaunch(descriptor)
            }
        }
        .buttonStyle(.borderedProminent)
    }

    private var presetButton: some View {
        Button("Preset") {
            if let descriptor = try? viewModel.makePresetDescriptor() {
                onLaunch(descriptor)
            }
        }
        .buttonStyle(.bordered)
    }

    private var isolatedButton: some View {
        Button("Isolated") {
            if let descriptor = try? viewModel.makeIsolatedDescriptor() {
                onLaunch(descriptor)
            }
        }
        .buttonStyle(.bordered)
    }
}
