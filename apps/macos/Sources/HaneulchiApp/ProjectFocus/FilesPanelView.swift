import SwiftUI

struct FilesPanelView: View {
    @Binding var workspaceState: ProjectFocusWorkspaceState

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Files")
                .font(.headline)

            TextField("Quick Search", text: $workspaceState.searchQuery)
                .textFieldStyle(.roundedBorder)

            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(workspaceState.filteredEntries) { entry in
                        Button(entry.relativePath) {
                            workspaceState.openFile(entry.absolutePath)
                        }
                        .buttonStyle(.plain)
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
        }
        .padding(16)
        .frame(width: 260, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.secondaryPanel)
    }
}
