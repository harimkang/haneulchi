import SwiftUI

struct WorktreeInventoryView: View {
    let viewModel: WorktreeInventoryViewModel
    let onAction: (AppShellAction) -> Void
    let onClose: () -> Void
    @Environment(\.viewportContext) private var viewportContext

    private var summaryCardColumns: [GridItem] {
        let count = switch viewportContext.viewportClass {
        case .compact:
            1
        case .medium:
            2
        case .wide:
            3
        case .expanded:
            5
        }

        return Array(repeating: GridItem(.flexible(), spacing: 12), count: count)
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            content
        }
        .frame(minHeight: 400)
    }

    private var header: some View {
        ViewThatFits(in: .horizontal) {
            HStack {
                Text("Worktree Inventory")
                    .font(.title2.weight(.semibold))
                Spacer()
                closeButton
            }

            VStack(alignment: .leading, spacing: 8) {
                Text("Worktree Inventory")
                    .font(.title2.weight(.semibold))
                closeButton
            }
        }
        .padding()
    }

    private var content: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                summaryCards
                groupSections
            }
            .padding()
        }
    }

    private var summaryCards: some View {
        LazyVGrid(columns: summaryCardColumns, spacing: 12) {
            SummaryCardView(label: "Total", count: viewModel.summaryCard.total, color: .primary)
            SummaryCardView(label: "In Use", count: viewModel.summaryCard.inUse, color: .blue)
            SummaryCardView(
                label: "Recoverable",
                count: viewModel.summaryCard.recoverable,
                color: .orange,
            )
            SummaryCardView(
                label: "Safe to Delete",
                count: viewModel.summaryCard.safeToDelete,
                color: .green,
            )
            SummaryCardView(label: "Stale", count: viewModel.summaryCard.stale, color: .secondary)
        }
    }

    private var closeButton: some View {
        Button {
            onClose()
        } label: {
            Image(systemName: "xmark.circle.fill")
                .foregroundStyle(.secondary)
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var groupSections: some View {
        if viewModel.summaryCard.total == 0 {
            ContentUnavailableView("No worktrees found", systemImage: "tray")
                .frame(maxWidth: .infinity)
        } else {
            if !viewModel.inUseRows.isEmpty {
                InventoryGroupSection(
                    title: "In Use",
                    rows: viewModel.inUseRows,
                    onAction: onAction,
                )
            }
            if !viewModel.recoverableRows.isEmpty {
                InventoryGroupSection(
                    title: "Recoverable",
                    rows: viewModel.recoverableRows,
                    onAction: onAction,
                )
            }
            if !viewModel.safeToDeleteRows.isEmpty {
                InventoryGroupSection(
                    title: "Safe to Delete",
                    rows: viewModel.safeToDeleteRows,
                    onAction: onAction,
                )
            }
            if !viewModel.staleRows.isEmpty {
                InventoryGroupSection(title: "Stale", rows: viewModel.staleRows, onAction: onAction)
            }
        }
    }
}

private struct SummaryCardView: View {
    let label: String
    let count: Int
    let color: Color

    var body: some View {
        VStack(spacing: 4) {
            Text("\(count)")
                .font(.title.bold())
                .foregroundStyle(color)
            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(12)
        .background(.background.secondary, in: RoundedRectangle(cornerRadius: 8))
    }
}

private struct InventoryGroupSection: View {
    let title: String
    let rows: [WorktreeInventoryViewModel.Row]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
            Text(title)
                .font(HaneulchiTypography.sectionHeading)
                .foregroundStyle(HaneulchiChrome.Label.muted)
                .padding(.horizontal, HaneulchiMetrics.Padding.card)
                .padding(.vertical, HaneulchiMetrics.Spacing.xs)
                .background(HaneulchiChrome.Surface.recess)

            VStack(alignment: .leading, spacing: 0) {
                ForEach(rows) { row in
                    WorktreeInventoryRowView(row: row, onAction: onAction)
                }
            }
            .background(HaneulchiChrome.Surface.base)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
        }
    }
}
