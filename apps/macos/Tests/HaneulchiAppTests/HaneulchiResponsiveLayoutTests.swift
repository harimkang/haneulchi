@testable import HaneulchiAppUI
import Testing

@Test("shared viewport classes classify the documented breakpoint thresholds")
func viewportClassifiesDocumentedThresholds() {
    #expect(HaneulchiViewportClass.forWidth(0) == .compact)
    #expect(HaneulchiViewportClass.forWidth(959) == .compact)
    #expect(HaneulchiViewportClass.forWidth(960) == .medium)
    #expect(HaneulchiViewportClass.forWidth(1239) == .medium)
    #expect(HaneulchiViewportClass.forWidth(1240) == .wide)
    #expect(HaneulchiViewportClass.forWidth(1519) == .wide)
    #expect(HaneulchiViewportClass.forWidth(1520) == .expanded)
}

@Test("shared route policy follows the viewport class without route-local thresholds")
func routeLayoutPolicyFollowsViewportClass() {
    let compact = HaneulchiViewportContext(width: 959).routeLayoutPolicy
    let medium = HaneulchiViewportContext(width: 960).routeLayoutPolicy
    let wide = HaneulchiViewportContext(width: 1240).routeLayoutPolicy
    let expanded = HaneulchiViewportContext(width: 1520).routeLayoutPolicy

    #expect(compact.showsSessionColumn == false)
    #expect(compact.showsCompactSessionContext == true)
    #expect(compact.showsExplorerColumn == false)
    #expect(compact.showsSupportingColumn == false)
    #expect(compact.stacksSupportingPanels == true)

    #expect(medium.showsSessionColumn == true)
    #expect(medium.showsCompactSessionContext == false)
    #expect(medium.showsExplorerColumn == false)
    #expect(medium.showsSupportingColumn == false)
    #expect(medium.stacksSupportingPanels == true)

    #expect(wide.showsSessionColumn == true)
    #expect(wide.showsCompactSessionContext == false)
    #expect(wide.showsExplorerColumn == false)
    #expect(wide.showsSupportingColumn == true)
    #expect(wide.stacksSupportingPanels == false)

    #expect(expanded.showsSessionColumn == true)
    #expect(expanded.showsCompactSessionContext == false)
    #expect(expanded.showsExplorerColumn == true)
    #expect(expanded.showsSupportingColumn == true)
    #expect(expanded.stacksSupportingPanels == false)
}

@Test("control tower route policy resolves shared project matrix density and lower-stage stacking")
func controlTowerRoutePolicyUsesSharedViewportClasses() {
    let compact = ControlTowerResponsiveLayout(viewportClass: .compact)
    let medium = ControlTowerResponsiveLayout(viewportClass: .medium)
    let wide = ControlTowerResponsiveLayout(viewportClass: .wide)
    let expanded = ControlTowerResponsiveLayout(viewportClass: .expanded)

    #expect(compact.projectGridColumnCount == 1)
    #expect(compact.stacksLowerStage == true)
    #expect(compact.usesDenseProjectGrid == false)

    #expect(medium.projectGridColumnCount == 2)
    #expect(medium.stacksLowerStage == true)
    #expect(medium.usesDenseProjectGrid == false)

    #expect(wide.projectGridColumnCount == 2)
    #expect(wide.stacksLowerStage == false)
    #expect(wide.usesDenseProjectGrid == false)

    #expect(expanded.projectGridColumnCount == 3)
    #expect(expanded.stacksLowerStage == false)
    #expect(expanded.usesDenseProjectGrid == true)
}

@Test("welcome launcher route policy stacks only in compact widths")
func welcomeRoutePolicyUsesSharedViewportClasses() {
    let compact = WelcomeReadinessResponsiveLayout(viewportClass: .compact)
    let medium = WelcomeReadinessResponsiveLayout(viewportClass: .medium)
    let wide = WelcomeReadinessResponsiveLayout(viewportClass: .wide)

    #expect(compact.usesSplitLauncher == false)
    #expect(medium.usesSplitLauncher == true)
    #expect(wide.usesSplitLauncher == true)
}

@Test("welcome readiness falls back to local width when shell viewport context is unavailable")
func welcomeReadinessUsesLocalViewportFallback() {
    let local = WelcomeReadinessView.resolvedViewportContext(
        shellViewportContext: .init(width: 0),
        localWidth: 480,
    )
    let bootstrap = WelcomeReadinessView.resolvedViewportContext(
        shellViewportContext: .init(width: 0),
        localWidth: 0,
    )
    let shell = WelcomeReadinessView.resolvedViewportContext(
        shellViewportContext: .init(width: 959),
        localWidth: 1520,
    )

    #expect(local.viewportClass == .compact)
    #expect(bootstrap.viewportClass == .medium)
    #expect(shell.viewportClass == .compact)
}

@Test("worktree inventory falls back to local width when shell viewport context is unavailable")
func worktreeInventoryUsesLocalViewportFallback() {
    let local = WorktreeInventoryView.resolvedViewportContext(
        shellViewportContext: .init(width: 0),
        localWidth: 480,
    )
    let bootstrap = WorktreeInventoryView.resolvedViewportContext(
        shellViewportContext: .init(width: 0),
        localWidth: 0,
    )
    let shell = WorktreeInventoryView.resolvedViewportContext(
        shellViewportContext: .init(width: 960),
        localWidth: 480,
    )

    #expect(local.viewportClass == .compact)
    #expect(bootstrap.viewportClass == .medium)
    #expect(shell.viewportClass == .medium)
}

@Test("worktree inventory rows stack action groups on compact widths and stay inline otherwise")
func worktreeInventoryRowActionsUseResponsiveLayout() {
    let compact = WorktreeInventoryRowPresentation(viewportClass: .compact)
    let medium = WorktreeInventoryRowPresentation(viewportClass: .medium)
    let wide = WorktreeInventoryRowPresentation(viewportClass: .wide)

    #expect(compact.actionLayout == .stacked)
    #expect(medium.actionLayout == .inline)
    #expect(wide.actionLayout == .inline)
}

@Test("shared modal policy derives from shared modal tokens and clamps to them")
func modalWidthPolicyUsesSharedTokensAndClamps() {
    let compact = HaneulchiViewportContext(width: 0).modalWidthPolicy
    let wide = HaneulchiViewportContext(width: 1240).modalWidthPolicy
    let compactTokens = HaneulchiMetrics.Modal.compact
    let wideTokens = HaneulchiMetrics.Modal.wide

    #expect(compact.minimumWidth == compactTokens.minimumWidth)
    #expect(compact.idealWidth == compactTokens.idealWidth)
    #expect(compact.maximumWidth == compactTokens.maximumWidth)
    #expect(compact.clampedWidth(compactTokens.minimumWidth - 1) == compactTokens.minimumWidth)
    #expect(compact.clampedWidth(compactTokens.idealWidth) == compactTokens.idealWidth)
    #expect(compact.clampedWidth(compactTokens.maximumWidth + 1) == compactTokens.maximumWidth)

    #expect(wide.minimumWidth == wideTokens.minimumWidth)
    #expect(wide.idealWidth == wideTokens.idealWidth)
    #expect(wide.maximumWidth == wideTokens.maximumWidth)
    #expect(wide.clampedWidth(wideTokens.minimumWidth - 1) == wideTokens.minimumWidth)
    #expect(wide.clampedWidth(wideTokens.idealWidth) == wideTokens.idealWidth)
    #expect(wide.clampedWidth(wideTokens.maximumWidth + 1) == wideTokens.maximumWidth)
}

@Test("shared modal policy resolves a viewport-safe frame width from the preferred width")
func modalWidthPolicyResolvesViewportSafeFrameWidth() {
    let compact = HaneulchiViewportContext(width: 959).modalWidthPolicy
    let expanded = HaneulchiViewportContext(width: 1520).modalWidthPolicy

    #expect(compact.resolvedWidth(availableWidth: 480) == 480)
    #expect(compact.resolvedWidth(availableWidth: 640) == HaneulchiMetrics.Modal.compact.idealWidth)
    #expect(compact.resolvedWidth(preferredWidth: 760, availableWidth: 900) ==
        HaneulchiMetrics.Modal.compact.maximumWidth)

    #expect(expanded.resolvedWidth(availableWidth: 0) == HaneulchiMetrics.Modal.expanded.idealWidth)
    #expect(expanded.resolvedWidth(availableWidth: 720) == 720)
    #expect(expanded.resolvedWidth(preferredWidth: 840, availableWidth: 900) ==
        HaneulchiMetrics.Modal.expanded.maximumWidth)
}

@Test("shared drawer policies expose notification and context width roles with viewport clamps")
func drawerWidthPoliciesUseSharedRolesAndViewportClamps() {
    let compact = HaneulchiViewportContext(width: 959)
    let expanded = HaneulchiViewportContext(width: 1520)
    let notification = compact.drawerWidthPolicy(for: .notification)
    let context = expanded.drawerWidthPolicy(for: .context)

    #expect(notification.minimumWidth == HaneulchiMetrics.Panel.inspectorMin)
    #expect(notification.idealWidth == HaneulchiMetrics.Panel.inspectorMin)
    #expect(notification.maximumWidth == HaneulchiMetrics.Panel.inspectorMax)
    #expect(notification.resolvedWidth(availableWidth: 280) == 280)
    #expect(notification.resolvedWidth(availableWidth: 420) == notification.idealWidth)

    #expect(context.minimumWidth == 360)
    #expect(context.idealWidth == 420)
    #expect(context.maximumWidth == 520)
    #expect(context.resolvedWidth(availableWidth: 0) == context.idealWidth)
    #expect(context.resolvedWidth(availableWidth: 380) == 380)
    #expect(context.resolvedWidth(preferredWidth: 560, availableWidth: 900) == context.maximumWidth)
}

@Test("shell viewport context derives content width from the shell width once")
func shellViewportContextDerivesContentWidthFromShellWidth() {
    let context = HaneulchiViewportContext(shellWidth: 512)

    #expect(context.width == 464)
    #expect(context.viewportClass == .compact)
}

@Test("root transient viewport keeps full root width while shell viewport subtracts the rail")
func rootTransientViewportPreservesFullRootWidth() {
    let root = HaneulchiViewportContext(rootWidth: HaneulchiMetrics.Responsive.wideWidth)
    let shell = HaneulchiViewportContext(shellWidth: HaneulchiMetrics.Responsive.wideWidth)

    #expect(root.width == HaneulchiMetrics.Responsive.wideWidth)
    #expect(root.viewportClass == .wide)
    #expect(shell.width == HaneulchiMetrics.Responsive.wideWidth - HaneulchiMetrics.Shell.railWidth)
    #expect(shell.viewportClass == .medium)
}

@Test("shell viewport context clamps content width before classifying it")
func shellViewportContextClampsContentWidthBeforeClassifying() {
    let context = HaneulchiViewportContext(shellWidth: 24)

    #expect(context.width == 0)
    #expect(context.viewportClass == .compact)
}

@Test("shell chrome density switches from compact to regular at the wide threshold")
func shellChromeDensityUsesSharedWidthRules() {
    let compact = HaneulchiViewportContext(shellWidth: 959 + HaneulchiMetrics.Shell.railWidth)
    let regular = HaneulchiViewportContext(shellWidth: 1240 + HaneulchiMetrics.Shell.railWidth)

    #expect(compact.shellChromeDensity == .compact)
    #expect(regular.shellChromeDensity == .regular)
}

@Test("compact top-bar chip selection prefers stronger severities and exposes overflow tone")
func compactTopBarChipSelectionPrefersSeverity() {
    let chips: [AppShellChromeState.Chip] = [
        .init(title: "workspace", tone: nil),
        .init(title: "warning", tone: .degraded),
        .init(title: "critical", tone: .failed),
        .init(title: "noise", tone: .unread),
    ]

    let presentation = HaneulchiViewportContext(width: 0)
        .compactTopBarChipPresentation(for: chips, visibleLimit: 2)

    #expect(presentation.visibleChips.map(\.title) == ["critical", "warning"])
    #expect(presentation.overflowChip?.title == "+2")
    #expect(presentation.overflowChip?.tone == .unread)
}

@Test("compact bottom-strip presentation preserves metrics and degrades transient notice safely")
func compactBottomStripPresentationPreservesMetricsFirst() {
    let items: [AppShellChromeState.StripItem] = [
        .init(title: "logs", detail: "clear"),
        .init(title: "problems", detail: "none"),
        .init(title: "terminal", detail: "4 sessions"),
        .init(title: "runtime hint", detail: "ok"),
    ]
    let transientNotice = "Dispatch sent to session alpha and waiting for reconciliation"

    let presentation = HaneulchiViewportContext(width: 0)
        .compactBottomStripPresentation(items: items, transientNotice: transientNotice)

    #expect(presentation.items == items)
    #expect(presentation.transientNotice != transientNotice)
    #expect((presentation.transientNotice?.count ?? 0) < transientNotice.count)
    #expect((presentation.transientNotice?.count ?? 0) > 0)
}
