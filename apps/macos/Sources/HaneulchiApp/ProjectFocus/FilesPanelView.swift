import SwiftUI

struct FilesPanelView: View {
    enum IndexState: Equatable, Sendable {
        case noProjectSelected
        case loading
        case indexingFailed
        case loaded
    }

    struct Presentation: Equatable, Sendable {
        let showsSearchField: Bool
        let entries: [ProjectFileIndex.Entry]
        let emptyStateMessage: String?
        let emptyStateDetail: String?
    }

    @Binding var workspaceState: ProjectFocusWorkspaceState
    var indexState: IndexState = .loaded
    var columnWidth: CGFloat = HaneulchiMetrics.Panel.explorerColumnWidth

    var body: some View {
        let presentation = Self.presentation(
            workspaceState: workspaceState,
            indexState: indexState,
        )

        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Files")

            if presentation.showsSearchField {
                TextField("Quick Search", text: $workspaceState.searchQuery)
                    .textFieldStyle(.roundedBorder)
                    .font(HaneulchiTypography.bodySmall)
                    .padding(.horizontal, HaneulchiMetrics.Padding.compact)
                    .padding(.vertical, HaneulchiMetrics.Spacing.xs)
            }

            if let emptyStateMessage = presentation.emptyStateMessage {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                    Text(emptyStateMessage)
                        .font(HaneulchiTypography.bodySmall)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)

                    if let emptyStateDetail = presentation.emptyStateDetail {
                        Text(emptyStateDetail)
                            .font(HaneulchiTypography.compactMeta)
                            .tracking(HaneulchiTypography.Tracking.metaModerate)
                            .foregroundStyle(HaneulchiChrome.Label.muted)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .padding(HaneulchiMetrics.Padding.compact)
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 0) {
                        ForEach(presentation.entries) { entry in
                            HaneulchiTableRow(
                                isSelected: workspaceState.selectedFilePath == entry.absolutePath,
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
                    .padding(.bottom, HaneulchiMetrics.Spacing.xs)
                    .frame(maxWidth: .infinity, alignment: .topLeading)
                }
                .frame(maxHeight: .infinity, alignment: .topLeading)
            }
        }
        .frame(
            minWidth: columnWidth,
            maxWidth: columnWidth,
            maxHeight: .infinity,
            alignment: .topLeading,
        )
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }

    nonisolated static func presentation(
        workspaceState: ProjectFocusWorkspaceState,
        indexState: IndexState,
    ) -> Presentation {
        switch indexState {
        case .noProjectSelected:
            return Presentation(
                showsSearchField: false,
                entries: [],
                emptyStateMessage: "Select a project to browse files.",
                emptyStateDetail: "Choose a project to load the explorer.",
            )
        case .loading:
            return Presentation(
                showsSearchField: false,
                entries: [],
                emptyStateMessage: "Indexing project files…",
                emptyStateDetail: "The explorer will populate when indexing finishes.",
            )
        case .indexingFailed:
            return Presentation(
                showsSearchField: false,
                entries: [],
                emptyStateMessage: "File indexing failed.",
                emptyStateDetail: "Try reopening the project or retrying the explorer.",
            )
        case .loaded:
            break
        }

        if workspaceState.fileEntries.isEmpty {
            return Presentation(
                showsSearchField: false,
                entries: [],
                emptyStateMessage: "No files in this project.",
                emptyStateDetail: "Add files to this project to populate the explorer.",
            )
        }

        if workspaceState.filteredEntries.isEmpty {
            return Presentation(
                showsSearchField: true,
                entries: [],
                emptyStateMessage: #"No files match "\#(workspaceState.searchQuery)"."#,
                emptyStateDetail: "Clear or change the search query.",
            )
        }

        return Presentation(
            showsSearchField: true,
            entries: workspaceState.filteredEntries,
            emptyStateMessage: nil,
            emptyStateDetail: nil,
        )
    }
}
