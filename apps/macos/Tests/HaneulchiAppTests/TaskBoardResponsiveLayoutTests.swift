@testable import HaneulchiAppUI
import Testing

@Test(
    "task board uses stacked compact lanes, a two-lane medium grid, and a full board rhythm from wide upward",
)
func taskBoardPresentationModesFollowViewportClasses() {
    let compact = TaskBoardPresentationLayout(viewportClass: .compact)
    let medium = TaskBoardPresentationLayout(viewportClass: .medium)
    let wide = TaskBoardPresentationLayout(viewportClass: .wide)
    let expanded = TaskBoardPresentationLayout(viewportClass: .expanded)

    #expect(compact.mode == .stacked)
    #expect(compact.rows.count == TaskBoardColumnID.allCases.count)
    #expect(compact.rows.allSatisfy { $0.columns.count == 1 })

    #expect(medium.mode == .twoLaneGrid)
    #expect(medium.rows.count == 3)
    #expect(medium.rows.allSatisfy { $0.columns.count == 2 })

    #expect(wide.mode == .fullBoard)
    #expect(wide.rows.count == 1)
    #expect(wide.rows.first?.columns == TaskBoardColumnID.allCases)

    #expect(expanded.mode == .fullBoard)
    #expect(expanded.rows.count == 1)
    #expect(expanded.rows.first?.columns == TaskBoardColumnID.allCases)
}

@Test("task board layout preserves all canonical columns and drop targets in every viewport class")
func taskBoardPresentationPreservesDropTargetCoverage() {
    for viewportClass in HaneulchiViewportClass.allCases {
        let layout = TaskBoardPresentationLayout(viewportClass: viewportClass)

        #expect(layout.presentedColumns == TaskBoardColumnID.allCases)
        #expect(layout.dropTargetColumns == TaskBoardColumnID.allCases)

        let flattenedTargets = layout.rows.flatMap(\.columns)
        #expect(flattenedTargets == TaskBoardColumnID.allCases)
    }
}

@Test("task board requires vertical overflow access when compact and medium rows stack downward")
func taskBoardPresentationRequiresVerticalOverflowAccessOnNarrowWidths() {
    let compact = TaskBoardPresentationLayout(viewportClass: .compact)
    let medium = TaskBoardPresentationLayout(viewportClass: .medium)
    let wide = TaskBoardPresentationLayout(viewportClass: .wide)

    #expect(compact.requiresVerticalOverflowScroll == true)
    #expect(medium.requiresVerticalOverflowScroll == true)
    #expect(wide.requiresVerticalOverflowScroll == false)
}
