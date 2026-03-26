@testable import HaneulchiApp
import Testing

@Test("chrome actions expose consistent iconography and accessibility labels")
func chromeActionsExposeStableIconsAndLabels() {
    #expect(HaneulchiChromeAction.commandPalette.symbolName == "magnifyingglass")
    #expect(HaneulchiChromeAction.commandPalette.accessibilityLabel == "Open Command Palette")

    #expect(HaneulchiChromeAction.notifications.symbolName == "bell")
    #expect(HaneulchiChromeAction.notifications.accessibilityLabel == "Open Notifications")

    #expect(HaneulchiChromeAction.splitHorizontal.symbolName == "rectangle.split.2x1")
    #expect(HaneulchiChromeAction.splitHorizontal.accessibilityLabel == "Split Horizontally")

    #expect(HaneulchiChromeAction.resolve.symbolName == "checkmark")
    #expect(HaneulchiChromeAction.snooze.symbolName == "bell.slash")
    #expect(HaneulchiChromeAction.refresh.symbolName == "arrow.clockwise")
    #expect(HaneulchiChromeAction.refresh.accessibilityLabel == "Refresh Snapshot")
    #expect(HaneulchiChromeAction.reconcile.symbolName == "arrow.triangle.2.circlepath")
    #expect(HaneulchiChromeAction.reconcile.accessibilityLabel == "Reconcile Automation")
    #expect(HaneulchiChromeAction.reload.symbolName == "arrow.clockwise.circle")
    #expect(HaneulchiChromeAction.reload.accessibilityLabel == "Reload Workflow")
}
