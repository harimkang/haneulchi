import SwiftUI

struct CommandPaletteOverlay: View {
    @ObservedObject var viewModel: CommandPaletteViewModel
    let onExecute: (AppShellAction) -> Void
    let onClose: () -> Void
    @FocusState private var isSearchFocused: Bool

    var body: some View {
        ZStack(alignment: .top) {
            HaneulchiChrome.Surface.scrim
                .ignoresSafeArea()
                .onTapGesture(perform: onClose)

            VStack(spacing: 0) {
                TextField(
                    "Search commands, files, sessions, tasks, inventory",
                    text: $viewModel.query,
                )
                .textFieldStyle(.plain)
                .font(HaneulchiTypography.body)
                .foregroundStyle(HaneulchiChrome.Label.primary)
                .padding(HaneulchiMetrics.Spacing.md)
                .background(HaneulchiChrome.Surface.base)
                .focused($isSearchFocused)
                .onSubmit(executeSelection)

                ScrollView {
                    LazyVStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                        ForEach(viewModel.filteredSections) { section in
                            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                                HaneulchiSectionHeader(title: section.kind.rawValue.capitalized)

                                ForEach(section.items) { item in
                                    Button {
                                        viewModel.select(item)
                                        executeSelection()
                                    } label: {
                                        HaneulchiTableRow(isSelected: viewModel.selection?
                                            .id == item.id)
                                        {
                                            HStack {
                                                VStack(alignment: .leading, spacing: 2) {
                                                    Text(item.title)
                                                        .font(HaneulchiTypography.body)
                                                        .foregroundStyle(HaneulchiChrome.Label
                                                            .primary)
                                                    if let subtitle = item.subtitle {
                                                        Text(subtitle)
                                                            .font(HaneulchiTypography
                                                                .compactMeta)
                                                            .foregroundStyle(HaneulchiChrome
                                                                .Label
                                                                .muted)
                                                    }
                                                }
                                                Spacer()
                                            }
                                        }
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                    }
                    .padding(HaneulchiMetrics.Spacing.md)
                }
            }
            .frame(
                minWidth: HaneulchiMetrics.Panel.commandPaletteMin,
                maxWidth: HaneulchiMetrics.Panel.commandPaletteMax,
                maxHeight: 520,
            )
            .glassPanel()
            .ambientShadow()
            .floatingSurface(isVisible: true)
            .padding(.top, HaneulchiMetrics.Spacing.xxl)
        }
        .onAppear {
            isSearchFocused = true
        }
        .onMoveCommand { direction in
            switch direction {
            case .down:
                viewModel.moveSelectionDown()
            case .up:
                viewModel.moveSelectionUp()
            default:
                break
            }
        }
        .onExitCommand(perform: onClose)
    }

    private func executeSelection() {
        guard let action = viewModel.selection?.action else {
            return
        }

        onExecute(action)
    }
}
