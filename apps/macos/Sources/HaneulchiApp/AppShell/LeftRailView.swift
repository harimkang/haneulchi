import SwiftUI

struct LeftRailView: View {
    let items: [AppShellChromeState.RailItem]
    let activeRoute: Route
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .center, spacing: HaneulchiMetrics.Spacing.xs) {
            ForEach(items) { item in
                RailItemButton(
                    item: item,
                    isActive: item.route == activeRoute,
                    onAction: onAction
                )
            }

            Spacer()
        }
        .padding(.vertical, HaneulchiMetrics.Spacing.sm)
        .frame(minWidth: HaneulchiMetrics.Shell.railWidth, maxWidth: HaneulchiMetrics.Shell.railWidth, maxHeight: .infinity)
        .background(HaneulchiChrome.Surface.recess)
    }
}

// MARK: - RailItemButton

private struct RailItemButton: View {
    let item: AppShellChromeState.RailItem
    let isActive: Bool
    let onAction: (AppShellAction) -> Void

    @State private var isHovered = false

    var body: some View {
        Button {
            onAction(.selectRoute(item.route))
        } label: {
            ZStack(alignment: .leading) {
                // Active left-side accent line (2px)
                if isActive {
                    Rectangle()
                        .fill(HaneulchiChrome.Gradient.primaryEnd)
                        .frame(width: 2)
                        .frame(maxHeight: HaneulchiMetrics.Icon.large)
                        .clipShape(RoundedRectangle(cornerRadius: 1))
                }

                VStack(spacing: HaneulchiMetrics.Spacing.xxs) {
                    Image(systemName: item.route.symbolName)
                        .font(.system(size: HaneulchiMetrics.Icon.standard, weight: isActive ? .semibold : .regular))
                        .foregroundStyle(
                            isActive
                                ? HaneulchiChrome.Gradient.primaryEnd
                                : HaneulchiChrome.Label.muted
                        )
                        .frame(width: HaneulchiMetrics.Icon.standard, height: HaneulchiMetrics.Icon.standard)

                    if let badgeText = item.badgeText {
                        Text(badgeText)
                            .font(HaneulchiTypography.compactMeta)
                            .foregroundStyle(HaneulchiChrome.Label.primary)
                            .padding(.horizontal, HaneulchiMetrics.Spacing.xxs)
                            .padding(.vertical, 2)
                            .background(HaneulchiChrome.Surface.raised)
                            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.leading, 2) // offset for accent line space
            }
            .frame(width: HaneulchiMetrics.Shell.railWidth, height: HaneulchiMetrics.Shell.railWidth)
            .background(
                isActive
                    ? HaneulchiChrome.Surface.base.opacity(0.6)
                    : (isHovered ? HaneulchiChrome.Surface.base.opacity(0.4) : Color.clear)
            )
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.hoverShift), value: isHovered)
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.pressedSelection), value: isActive)
        }
        .buttonStyle(.plain)
        .onHover { isHovered = $0 }
    }
}
