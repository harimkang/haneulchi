import SwiftUI

struct TaskBoardView: View {
    let summary: String
    @StateObject private var viewModel: TaskBoardViewModel
    @Environment(\.viewportContext) private var viewportContext
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    private var presentationLayout: TaskBoardPresentationLayout {
        .init(viewportClass: viewportContext.viewportClass)
    }

    init(
        summary: String = "Task board projection is loaded from Rust-owned task data.",
        viewModel: TaskBoardViewModel = TaskBoardViewModel(),
    ) {
        self.summary = summary
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        Group {
            if presentationLayout.requiresVerticalOverflowScroll {
                ScrollView(showsIndicators: true) {
                    routeContent
                }
            } else {
                routeContent
            }
        }
        .background(HaneulchiChrome.Surface.foundation)
        .task {
            do {
                try viewModel.reload()
            } catch {
                viewModel.present(error: error)
            }
        }
    }

    private var routeContent: some View {
        VStack(alignment: .leading, spacing: layout.sectionSpacing) {
            HaneulchiHeaderDeck(
                title: "Task Board",
                subtitle: summary,
                horizontalPadding: layout.headerInnerPadding,
            ) {
                EmptyView()
            }

            projectFilterBar

            if let errorMessage = viewModel.errorMessage {
                Text(errorMessage)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.State.warning)
                    .padding(HaneulchiMetrics.Spacing.md)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(HaneulchiChrome.Surface.recess)
                    .clipShape(RoundedRectangle(
                        cornerRadius: HaneulchiMetrics.Radius.medium,
                        style: .continuous,
                    ))
            }

            boardContent
        }
        .padding(.horizontal, layout.screenPadding)
        .padding(.vertical, layout.sectionSpacing)
        .frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private var projectFilterBar: some View {
        ScrollView(.horizontal, showsIndicators: true) {
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                filterButton(title: "All Projects", projectID: nil, taskCount: totalTaskCount)
                ForEach(viewModel.projectOptions, id: \.projectID) { option in
                    filterButton(
                        title: option.title,
                        projectID: option.projectID,
                        taskCount: option.taskCount,
                    )
                }
            }
        }
    }

    private func filterButton(title: String, projectID: String?, taskCount: Int) -> some View {
        let isSelected = viewModel.selectedProjectID == projectID

        return Button {
            do {
                try viewModel.selectProject(projectID)
            } catch {
                viewModel.present(error: error)
            }
        } label: {
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                Text(title)
                    .font(HaneulchiTypography.systemLabel)
                    .foregroundStyle(isSelected ? HaneulchiChrome.Label.primary : HaneulchiChrome
                        .Label.secondary)
                Text("\(taskCount)")
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(isSelected ? HaneulchiChrome.Surface
                        .foundation : HaneulchiChrome.Label.muted)
                    .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
                    .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                    .background(isSelected ? HaneulchiChrome.Surface.base : HaneulchiChrome.Surface
                        .raised)
                    .clipShape(Capsule())
            }
            .padding(.horizontal, HaneulchiMetrics.Spacing.md)
            .padding(.vertical, HaneulchiMetrics.Spacing.sm)
            .background(filterButtonBackground(isSelected: isSelected))
            .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var boardContent: some View {
        switch presentationLayout.mode {
        case .fullBoard:
            ScrollView(.horizontal, showsIndicators: true) {
                HStack(alignment: .top, spacing: layout.columnSpacing) {
                    ForEach(presentationLayout.rows.first?.columns ?? [], id: \.self) { columnID in
                        if let column = columnModel(for: columnID) {
                            columnView(column)
                        }
                    }
                }
                .padding(.bottom, HaneulchiMetrics.Spacing.xs)
            }
        case .stacked, .twoLaneGrid:
            VStack(alignment: .leading, spacing: layout.columnSpacing) {
                ForEach(presentationLayout.rows) { row in
                    boardRow(row)
                }
            }
        }
    }

    private func boardRow(_ row: TaskBoardPresentationRow) -> some View {
        HStack(alignment: .top, spacing: layout.columnSpacing) {
            ForEach(row.columns, id: \.self) { columnID in
                if let column = columnModel(for: columnID) {
                    columnView(column)
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                }
            }
        }
    }

    private func columnView(_ column: TaskBoardViewModel.ColumnModel) -> some View {
        let isDone = column.column == .done

        return VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            HStack {
                Text(column.title.uppercased())
                    .font(HaneulchiTypography.systemLabel)
                    .tracking(HaneulchiTypography.Tracking.labelWide)
                    .foregroundStyle(isDone ? HaneulchiChrome.Label.muted : HaneulchiChrome.Label
                        .muted)
                Spacer()
                Text("\(column.taskCount)")
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(isDone ? HaneulchiChrome.Label.muted : HaneulchiChrome.Label
                        .muted)
            }

            if column.tasks.isEmpty {
                Text("Drop a task here or keep the column empty until the next action is clear.")
                    .font(HaneulchiTypography.compactMeta)
                    .foregroundStyle(HaneulchiChrome.Label.muted)
                    .padding(.top, HaneulchiMetrics.Spacing.xs)
            } else {
                VStack(spacing: HaneulchiMetrics.Spacing.sm) {
                    ForEach(column.tasks) { task in
                        TaskCardView(task: task)
                            .draggable(task.id)
                    }
                }
            }
        }
        .padding(HaneulchiMetrics.Spacing.md)
        .frame(
            minWidth: presentationLayout.mode == .fullBoard
                ? HaneulchiMetrics.Panel.boardColumnMin
                : nil,
            alignment: .topLeading,
        )
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(columnBackground(for: column.column))
        .clipShape(RoundedRectangle(
            cornerRadius: HaneulchiMetrics.Radius.large,
            style: .continuous,
        ))
        .dropDestination(for: String.self) { items, _ in
            guard let taskID = items.first else {
                return false
            }
            do {
                try viewModel.moveTask(taskID: taskID, to: column.column)
                return true
            } catch {
                viewModel.present(error: error)
                return false
            }
        }
    }

    @ViewBuilder
    private func columnBackground(for column: TaskBoardColumnID) -> some View {
        switch column {
        case .running:
            HaneulchiChrome.Surface.base
        case .review:
            HaneulchiChrome.Surface.base
                .overlay(
                    HaneulchiChrome.Gradient.primaryEnd.opacity(0.06),
                )
        case .blocked:
            HaneulchiChrome.Surface.base
                .overlay(
                    HaneulchiChrome.State.errorSolid.opacity(0.08),
                )
        case .inbox, .ready, .done:
            HaneulchiChrome.Surface.recess
        }
    }

    @ViewBuilder
    private func filterButtonBackground(isSelected: Bool) -> some View {
        if isSelected {
            HaneulchiChrome.Gradient.primaryLinear
        } else {
            HaneulchiChrome.Surface.recess
        }
    }

    private var totalTaskCount: Int {
        viewModel.columns.reduce(0) { $0 + $1.taskCount }
    }

    private func columnModel(for columnID: TaskBoardColumnID) -> TaskBoardViewModel.ColumnModel? {
        viewModel.columns.first { $0.column == columnID }
    }
}
