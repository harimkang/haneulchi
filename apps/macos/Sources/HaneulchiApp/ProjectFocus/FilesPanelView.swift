import SwiftUI

struct FilesPanelView: View {
    @Binding var workspaceState: ProjectFocusWorkspaceState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Files")

            TextField("Quick Search", text: $workspaceState.searchQuery)
                .textFieldStyle(.roundedBorder)
                .font(HaneulchiTypography.bodySmall)
                .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                .padding(.vertical, HaneulchiMetrics.Spacing.xs)

            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(workspaceState.filteredEntries) { entry in
                        HaneulchiTableRow(
                            isSelected: workspaceState.selectedFilePath == entry.absolutePath
                        ) {
                            Button {
                                workspaceState.openFile(entry.absolutePath)
                            } label: {
                                HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                                    Text(entry.relativePath)
                                        .font(HaneulchiTypography.body)
                                        .foregroundStyle(HaneulchiChrome.Label.primary)
                                        .lineLimit(1)
                                    Spacer()
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
                .padding(.horizontal, HaneulchiMetrics.Spacing.xxs)
            }
        }
        .frame(
            minWidth: HaneulchiMetrics.Panel.explorerMin,
            maxWidth: HaneulchiMetrics.Panel.explorerMax,
            maxHeight: .infinity,
            alignment: .topLeading
        )
        .background(HaneulchiChrome.Surface.base)
    }
}
