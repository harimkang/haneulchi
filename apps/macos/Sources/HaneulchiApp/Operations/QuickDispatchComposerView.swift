import SwiftUI

struct QuickDispatchComposerView: View {
    @State var viewModel: QuickDispatchComposerViewModel
    let onSend: (String, String) -> Void
    let onClose: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Dispatch")
                .font(HaneulchiTypography.heading(20))

            Picker("Target", selection: Binding(
                get: { viewModel.selectedTargetID ?? "" },
                set: { viewModel.selectTarget(id: $0) }
            )) {
                ForEach(viewModel.targets) { target in
                    Text(target.title).tag(target.id)
                }
            }

            TextEditor(text: $viewModel.messageText)
                .frame(minHeight: 100)
                .padding(8)
                .background(HaneulchiChrome.Colors.surfaceMuted)
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))

            if let reason = viewModel.sendDisabledReason, !viewModel.sendEnabled {
                Text(reason)
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.warning)
            }

            HStack {
                Button("Close") { onClose() }
                Spacer()
                Button(viewModel.selectedTarget?.isNewSession == true ? "Open Session" : "Send") {
                    if let targetID = viewModel.selectedTargetID {
                        onSend(targetID, viewModel.messageText)
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(!viewModel.sendEnabled)
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .background(HaneulchiChrome.Colors.surfaceRaised)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }
}
