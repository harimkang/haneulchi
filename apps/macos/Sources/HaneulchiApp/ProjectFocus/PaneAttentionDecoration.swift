import SwiftUI

struct PaneAttentionDecoration: ViewModifier {
    let hasAttention: Bool
    let hasUnread: Bool

    func body(content: Content) -> some View {
        content.overlay(
            RoundedRectangle(cornerRadius: 18)
                .strokeBorder(
                    hasAttention
                        ? HaneulchiChrome.Colors.warning.opacity(0.55)
                        : (hasUnread ? HaneulchiChrome.Colors.unread.opacity(0.45) : .clear),
                    lineWidth: hasAttention || hasUnread ? 2 : 0
                )
        )
    }
}

extension View {
    func paneAttentionDecoration(hasAttention: Bool, hasUnread: Bool) -> some View {
        modifier(PaneAttentionDecoration(hasAttention: hasAttention, hasUnread: hasUnread))
    }
}
