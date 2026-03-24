import SwiftUI

struct AttentionCenterPlaceholderView: View {
    let snapshot: AppShellSnapshot
    let onAction: (AppShellAction) -> Void

    var body: some View {
        AttentionCenterView(
            viewModel: AttentionCenterViewModel(
                snapshot: snapshot,
                openTarget: onAction,
                resolveAttention: { onAction(.resolveAttention($0)) },
                dismissAttention: { onAction(.dismissAttention($0)) },
                snoozeAttention: { onAction(.snoozeAttention($0)) }
            )
        )
    }
}
