import SwiftUI

struct CommandPaletteOverlay: View {
    @ObservedObject var viewModel: CommandPaletteViewModel
    let onExecute: (AppShellAction) -> Void
    let onClose: () -> Void
    @FocusState private var isSearchFocused: Bool

    var body: some View {
        ZStack(alignment: .top) {
            Color.black.opacity(0.24)
                .ignoresSafeArea()
                .onTapGesture(perform: onClose)

            VStack(spacing: 0) {
                TextField("Search commands, files, sessions, tasks, inventory", text: $viewModel.query)
                    .textFieldStyle(.plain)
                    .padding(16)
                    .focused($isSearchFocused)
                    .onSubmit(executeSelection)

                Divider()
                    .overlay(HaneulchiChrome.Colors.surfaceMuted)

                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 12) {
                        ForEach(viewModel.filteredSections) { section in
                            VStack(alignment: .leading, spacing: 8) {
                                Text(section.kind.rawValue.capitalized)
                                    .font(HaneulchiTypography.label(11))
                                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)

                                ForEach(section.items) { item in
                                    Button {
                                        viewModel.select(item)
                                        executeSelection()
                                    } label: {
                                        HStack {
                                            VStack(alignment: .leading, spacing: 2) {
                                                Text(item.title)
                                                    .font(HaneulchiTypography.body)
                                                if let subtitle = item.subtitle {
                                                    Text(subtitle)
                                                        .font(HaneulchiTypography.caption)
                                                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                                                }
                                            }
                                            Spacer()
                                        }
                                        .padding(10)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .background(viewModel.selection?.id == item.id ? HaneulchiChrome.Colors.surfaceRaised : Color.clear)
                                        .clipShape(RoundedRectangle(cornerRadius: 12))
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                    }
                    .padding(16)
                }
            }
            .frame(minWidth: 720, maxWidth: 720, maxHeight: 520)
            .background(HaneulchiChrome.Colors.surfaceBase)
            .clipShape(RoundedRectangle(cornerRadius: 20))
            .padding(.top, 48)
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
