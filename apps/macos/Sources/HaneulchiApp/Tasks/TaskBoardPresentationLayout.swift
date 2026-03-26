import Foundation

enum TaskBoardPresentationMode: Equatable, Sendable {
    case stacked
    case twoLaneGrid
    case fullBoard
}

struct TaskBoardPresentationRow: Equatable, Sendable, Identifiable {
    let columns: [TaskBoardColumnID]

    var id: String {
        columns.map(\.rawValue).joined(separator: "-")
    }
}

struct TaskBoardPresentationLayout: Equatable, Sendable {
    let mode: TaskBoardPresentationMode
    let rows: [TaskBoardPresentationRow]

    init(viewportClass: HaneulchiViewportClass) {
        switch viewportClass {
        case .compact:
            mode = .stacked
            rows = TaskBoardColumnID.allCases.map { TaskBoardPresentationRow(columns: [$0]) }
        case .medium:
            mode = .twoLaneGrid
            rows = stride(from: 0, to: TaskBoardColumnID.allCases.count, by: 2).map { index in
                let columns = Array(TaskBoardColumnID.allCases[index ..< min(
                    index + 2,
                    TaskBoardColumnID.allCases.count,
                )])
                return TaskBoardPresentationRow(columns: columns)
            }
        case .wide, .expanded:
            mode = .fullBoard
            rows = [TaskBoardPresentationRow(columns: TaskBoardColumnID.allCases)]
        }
    }

    var presentedColumns: [TaskBoardColumnID] {
        rows.flatMap(\.columns)
    }

    var dropTargetColumns: [TaskBoardColumnID] {
        presentedColumns
    }
}
