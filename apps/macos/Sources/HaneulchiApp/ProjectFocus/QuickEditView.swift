import SwiftUI

struct QuickEditView: View {
    @Binding var workspaceState: ProjectFocusWorkspaceState

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Edit")
                .font(.headline)

            TextEditor(text: $workspaceState.editingText)
                .font(HaneulchiTypography.body)
                .frame(minHeight: 220)

            HStack {
                Button("Save") {
                    try? workspaceState.saveQuickEdit()
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding(16)
        .background(HaneulchiChrome.Colors.surfaceBase)
    }
}
