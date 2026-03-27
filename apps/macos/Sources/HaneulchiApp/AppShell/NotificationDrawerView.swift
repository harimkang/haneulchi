import SwiftUI

struct NotificationDrawerView: View {
    @Environment(\.viewportContext) private var viewportContext
    let items: [NotificationDrawerModel.Item]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        ScrollView(showsIndicators: true) {
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                ForEach(items) { item in
                    Button {
                        onAction(item.action)
                    } label: {
                        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                            HStack(
                                alignment: .firstTextBaseline,
                                spacing: HaneulchiMetrics.Spacing.xs,
                            ) {
                                Text(item.title)
                                    .font(HaneulchiTypography.sectionHeading)
                                    .foregroundStyle(HaneulchiChrome.Label.primary)
                                    .multilineTextAlignment(.leading)
                                Spacer(minLength: HaneulchiMetrics.Spacing.xs)
                                Text(item.stateLabel.uppercased())
                                    .font(HaneulchiTypography.compactMeta)
                                    .tracking(HaneulchiTypography.Tracking.labelWide)
                                    .foregroundStyle(HaneulchiChrome.Label.muted)
                            }

                            Text(item.summary)
                                .font(HaneulchiTypography.bodySmall)
                                .foregroundStyle(HaneulchiChrome.Label.secondary)
                                .fixedSize(horizontal: false, vertical: true)
                        }
                        .padding(HaneulchiMetrics.Padding.card)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(HaneulchiChrome.Surface.raised)
                        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(HaneulchiMetrics.Padding.card)
        }
        .frame(width: drawerWidth, alignment: .topLeading)
        .frame(maxHeight: 360, alignment: .topLeading)
        .background(HaneulchiChrome.Surface.base.opacity(0.98))
        .overlay(
            RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large)
                .strokeBorder(HaneulchiChrome.Stroke.ghost, lineWidth: 1),
        )
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large))
        .shadow(color: Color.black.opacity(0.24), radius: 18, x: 0, y: 8)
    }

    private var drawerWidth: CGFloat {
        viewportContext.drawerWidthPolicy(for: .notification).resolvedWidth(
            availableWidth: viewportContext.width > 0
                ? max(0, viewportContext.width - HaneulchiChrome.Spacing.screenPadding)
                : nil,
        )
    }
}
