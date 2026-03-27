import SwiftUI

enum HaneulchiPreviewGalleryContexts {
    static let projectFocusDesktopViewportContext = HaneulchiViewportContext(
        width: HaneulchiMetrics.Responsive.expandedWidth,
    )
}

#Preview("App Shell / Welcome") {
    AppShellView(
        model: HaneulchiPreviewFixtures.shellModel(
            route: .projectFocus,
            entrySurface: .welcome(.firstRun),
        ),
        projectFolderPicker: HaneulchiPreviewFixtures.projectFolderPicker(),
        demoWorkspaceScaffold: HaneulchiPreviewFixtures.demoWorkspaceScaffold(),
    )
}

#Preview("App Shell / Shell") {
    AppShellView(
        model: HaneulchiPreviewFixtures.shellModel(route: .projectFocus),
        projectFolderPicker: HaneulchiPreviewFixtures.projectFolderPicker(),
        demoWorkspaceScaffold: HaneulchiPreviewFixtures.demoWorkspaceScaffold(),
    )
}

#Preview("App Shell / Command Palette") {
    AppShellView(
        model: HaneulchiPreviewFixtures.shellModel(
            route: .projectFocus,
            showCommandPalette: true,
        ),
        projectFolderPicker: HaneulchiPreviewFixtures.projectFolderPicker(),
        demoWorkspaceScaffold: HaneulchiPreviewFixtures.demoWorkspaceScaffold(),
    )
}

#Preview("App Shell / Notifications") {
    AppShellView(
        model: HaneulchiPreviewFixtures.shellModel(
            route: .projectFocus,
            showNotifications: true,
        ),
        projectFolderPicker: HaneulchiPreviewFixtures.projectFolderPicker(),
        demoWorkspaceScaffold: HaneulchiPreviewFixtures.demoWorkspaceScaffold(),
    )
}

#Preview("Project Focus") {
    ProjectFocusView(
        model: .runtimeDemo,
        snapshot: HaneulchiPreviewFixtures.snapshot(activeRoute: .projectFocus),
        onAction: { _ in },
    )
    .environment(
        \.viewportContext,
        HaneulchiPreviewGalleryContexts.projectFocusDesktopViewportContext,
    )
    .frame(width: 1600, height: 900)
}

#Preview("Control Tower") {
    ControlTowerView(
        model: ControlTowerViewModel(
            snapshot: HaneulchiPreviewFixtures.snapshot(activeRoute: .controlTower),
        ),
        onAction: { _ in },
    )
}

#Preview("Task Board") {
    TaskBoardView(
        summary: "Task board preview using shared preview fixtures.",
        viewModel: HaneulchiPreviewFixtures.taskBoardViewModel(),
    )
}

#Preview("Review Queue") {
    ReviewQueueView(
        summary: "Review queue preview using shared preview fixtures.",
        viewModel: HaneulchiPreviewFixtures.reviewQueueViewModel(),
    )
}

#Preview("Attention Center") {
    AttentionCenterView(
        viewModel: HaneulchiPreviewFixtures.attentionViewModel(),
    )
}

#Preview("Settings") {
    SettingsView(
        viewModel: HaneulchiPreviewFixtures.settingsViewModel(),
        onAction: { _ in },
    )
}

#Preview("Quick Dispatch") {
    QuickDispatchComposerView(
        viewModel: HaneulchiPreviewFixtures.quickDispatchViewModel(),
        onSend: { _, _ in },
        onClose: {},
    )
    .frame(width: 520)
    .padding(32)
    .background(HaneulchiChrome.Surface.foundation)
}

#Preview("Task Context Drawer") {
    TaskContextDrawerView(
        model: HaneulchiPreviewFixtures.taskDrawerModel(),
        onPrimaryAction: { _ in },
        onQuickDispatch: {},
    )
}

#Preview("Workflow Drawer") {
    WorkflowDrawerView(
        status: HaneulchiPreviewFixtures.workflowStatus,
        onReload: {},
    )
}

#Preview("New Session Sheet") {
    NewSessionSheetView(
        viewModel: HaneulchiPreviewFixtures.newSessionSheetViewModel(),
        onLaunch: { _ in },
    )
}

#Preview("Worktree Inventory") {
    WorktreeInventoryView(
        viewModel: HaneulchiPreviewFixtures.inventoryViewModel(),
        onAction: { _ in },
        onClose: {},
    )
}

#Preview("Command Palette Overlay") {
    CommandPaletteOverlay(
        viewModel: HaneulchiPreviewFixtures.commandPaletteViewModel(),
        onExecute: { _ in },
        onClose: {},
    )
}

#Preview("Notification Drawer") {
    NotificationDrawerView(
        items: HaneulchiPreviewFixtures.notificationItems,
        onAction: { _ in },
    )
}
