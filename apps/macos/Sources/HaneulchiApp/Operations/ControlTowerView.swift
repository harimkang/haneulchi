import SwiftUI

struct ControlTowerView: View {
    let model: ControlTowerViewModel
    let onAction: (AppShellAction) -> Void
    @Environment(\.viewportContext) private var viewportContext
    private let layout = HaneulchiOperationalLayoutMetrics.standard

    private var responsiveLayout: ControlTowerResponsiveLayout {
        .init(viewportClass: viewportContext.viewportClass)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: layout.sectionSpacing) {
                HaneulchiHeaderDeck(
                    title: "Control Tower",
                    subtitle: "Scan control-plane health and multi-project activity without leaving the operator surface.",
                    horizontalPadding: layout.headerInnerPadding,
                ) {
                    controlTowerActions
                }

                ControlTowerOpsStripView(
                    model: model.opsModel,
                    onRefresh: { onAction(.refreshShellSnapshot) },
                    onReconcile: { onAction(.reconcileAutomation) },
                    onReload: { onAction(.reloadWorkflow) },
                )

                VStack(alignment: .leading, spacing: layout.sectionSpacing) {
                    projectMatrixHeader

                    ControlTowerProjectCardGrid(cards: model.projectCards) { _ in
                        onAction(.selectRoute(.projectFocus))
                    }

                    if responsiveLayout.stacksLowerStage {
                        VStack(alignment: .leading, spacing: layout.columnSpacing) {
                            attentionQueueSummary
                            recentArtifactsTable
                        }
                    } else {
                        HStack(alignment: .top, spacing: layout.columnSpacing) {
                            attentionQueueSummary
                            recentArtifactsTable
                        }
                    }
                }
            }
            .padding(.horizontal, layout.screenPadding)
            .padding(.vertical, layout.sectionSpacing)
            .padding(.bottom, layout.sectionSpacing)
        }
        .background(HaneulchiChrome.Surface.foundation)
    }

    private var controlTowerActions: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                resyncButton
                newSessionButton
            }

            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                resyncButton
                newSessionButton
            }
        }
    }

    private var projectMatrixHeader: some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .firstTextBaseline, spacing: HaneulchiMetrics.Spacing.sm) {
                projectMatrixTitle
                Spacer(minLength: HaneulchiMetrics.Spacing.sm)
                quickDispatchButton
            }

            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                projectMatrixTitle
                quickDispatchButton
            }
        }
    }

    private var projectMatrixTitle: some View {
        HStack(alignment: .firstTextBaseline, spacing: HaneulchiMetrics.Spacing.xs) {
            Text("Project Matrix")
                .font(HaneulchiTypography.sectionHeading)
                .foregroundStyle(HaneulchiChrome.Label.primary)
            Text("\(model.projectCards.count)")
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.metaModerate)
                .foregroundStyle(HaneulchiChrome.Label.muted)
        }
    }

    private var resyncButton: some View {
        Button {
            onAction(.refreshShellSnapshot)
        } label: {
            Label("Resync", systemImage: HaneulchiChromeAction.refresh.symbolName)
        }
        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
    }

    private var newSessionButton: some View {
        Button("New Session") {
            onAction(.presentNewSessionSheet)
        }
        .buttonStyle(HaneulchiButtonStyle(variant: .primary))
    }

    private var quickDispatchButton: some View {
        Button {
            onAction(.presentQuickDispatch(.controlTower))
        } label: {
            Label(
                "Quick Dispatch",
                systemImage: HaneulchiChromeAction.dispatch.symbolName,
            )
        }
        .buttonStyle(HaneulchiButtonStyle(variant: .tertiary))
    }

    private var attentionQueueSummary: some View {
        AttentionQueueSummaryView(items: model.attentionItems) { item in
            if let sessionID = item.targetSessionID {
                onAction(.jumpToSession(sessionID))
            } else {
                onAction(.selectRoute(item.targetRoute))
            }
        }
    }

    private var recentArtifactsTable: some View {
        RecentArtifactsTableView(items: model.recentArtifacts) { item in
            onAction(.selectRoute(item.targetRoute))
        }
    }
}
