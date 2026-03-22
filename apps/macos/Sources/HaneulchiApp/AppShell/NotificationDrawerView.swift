import SwiftUI

struct NotificationDrawerView: View {
    let items: [Item]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ForEach(items) { item in
                Button {
                    onAction(item.action)
                } label: {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(item.title)
                            .font(.headline)
                        Text(item.summary)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .buttonStyle(.plain)
            }
        }
        .padding(16)
        .frame(minWidth: 320, alignment: .topLeading)
    }
}

extension NotificationDrawerView {
    struct Item: Equatable, Identifiable {
        let id: String
        let title: String
        let summary: String
        let action: AppShellAction
    }

    static func items(from snapshot: AppShellSnapshot) -> [Item] {
        snapshot.attention.map { attention in
            Item(
                id: attention.attentionID,
                title: attention.headline,
                summary: attention.summary ?? attention.headline,
                action: attention.targetSessionID.map(AppShellAction.jumpToSession) ?? .selectRoute(attention.targetRoute)
            )
        }
    }
}
