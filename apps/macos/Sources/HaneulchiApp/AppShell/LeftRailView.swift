import SwiftUI

struct LeftRailView: View {
    let items: [AppShellChromeState.RailItem]
    let activeRoute: Route
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(items) { item in
                Button {
                    onAction(.selectRoute(item.route))
                } label: {
                    HStack(spacing: 10) {
                        Image(systemName: item.route.symbolName)
                            .frame(width: 18)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(item.title)
                                .font(HaneulchiTypography.label(12))
                            if let shortcutLabel = item.shortcutLabel {
                                Text(shortcutLabel)
                                    .font(HaneulchiTypography.caption)
                                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                            }
                        }

                        Spacer()

                        if let badgeText = item.badgeText {
                            Text(badgeText)
                                .font(HaneulchiTypography.label(11))
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(HaneulchiChrome.Colors.surfaceRaised)
                                .clipShape(Capsule())
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(item.route == activeRoute ? HaneulchiChrome.Colors.surfaceRaised : Color.clear)
                    .clipShape(RoundedRectangle(cornerRadius: 14))
                }
                .buttonStyle(.plain)
            }

            Spacer()
        }
        .padding(16)
        .frame(minWidth: 220, maxWidth: 220, maxHeight: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceMuted)
    }
}
