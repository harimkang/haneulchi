import Testing
@testable import HaneulchiApp

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
}
