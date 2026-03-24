import SwiftUI

struct NotificationDrawerView: View {
    let items: [NotificationDrawerModel.Item]
    let onAction: (AppShellAction) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ForEach(items) { item in
                Button {
                    onAction(item.action)
                } label: {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text(item.title)
                                .font(.headline)
                            Spacer()
                            Text(item.stateLabel)
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                        }
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
