import SwiftUI

struct TaskBoardView: View {
    let summary: String
    @StateObject private var viewModel: TaskBoardViewModel

    init(
        summary: String = "Task board projection is loaded from Rust-owned task data.",
        viewModel: TaskBoardViewModel = TaskBoardViewModel()
    ) {
        self.summary = summary
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.panelGap) {
            VStack(alignment: .leading, spacing: 8) {
                Text("Task Board")
                    .font(HaneulchiTypography.heading(28))
                Text(summary)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            projectFilterBar

            if let errorMessage = viewModel.errorMessage {
                Text(errorMessage)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Colors.warning)
                    .padding(16)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(HaneulchiChrome.Colors.surfaceMuted)
                    .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
            }

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(alignment: .top, spacing: 18) {
                    ForEach(viewModel.columns) { column in
                        columnView(column)
                    }
                }
                .padding(.bottom, 8)
            }
        }
        .padding(HaneulchiChrome.Spacing.screenPadding)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .task {
            do {
                try viewModel.reload()
            } catch {
                viewModel.present(error: error)
            }
        }
    }

    private var projectFilterBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 10) {
                filterButton(title: "All Projects", projectID: nil, taskCount: totalTaskCount)
                ForEach(viewModel.projectOptions, id: \.projectID) { option in
                    filterButton(title: option.title, projectID: option.projectID, taskCount: option.taskCount)
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
            HStack(spacing: 8) {
                Text(title)
                    .font(HaneulchiTypography.label(12))
                Text("\(taskCount)")
                    .font(HaneulchiTypography.label(11))
                    .foregroundStyle(isSelected ? HaneulchiChrome.Colors.appBackground : HaneulchiChrome.Colors.mutedText)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(isSelected ? HaneulchiChrome.Colors.surfaceBase : HaneulchiChrome.Colors.surfaceRaised)
                    .clipShape(Capsule())
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(isSelected ? HaneulchiChrome.Colors.accent : HaneulchiChrome.Colors.surfaceMuted)
            .foregroundStyle(isSelected ? HaneulchiChrome.Colors.appBackground : .primary)
            .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }

    private func columnView(_ column: TaskBoardViewModel.ColumnModel) -> some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text(column.title)
                    .font(HaneulchiTypography.label(14))
                Spacer()
                Text("\(column.taskCount)")
                    .font(HaneulchiTypography.label(12))
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            if column.tasks.isEmpty {
                Text("Drop a task here or keep the column empty until the next action is clear.")
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    .padding(.top, 8)
            } else {
                VStack(spacing: 12) {
                    ForEach(column.tasks) { task in
                        TaskCardView(task: task)
                            .draggable(task.id)
                    }
                }
            }
        }
        .padding(18)
        .frame(width: 280, alignment: .topLeading)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceMuted)
        .clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
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

    private var totalTaskCount: Int {
        viewModel.columns.reduce(0) { $0 + $1.taskCount }
    }
}
