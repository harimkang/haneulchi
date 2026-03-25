import SwiftUI

struct AdapterWatchSummaryView: View {
    let session: AppShellSnapshot.SessionSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Adapter Watch")
                .font(HaneulchiTypography.heading(18))

            Text([session.providerID, session.modelID].compactMap(\.self).joined(separator: " · "))
                .font(HaneulchiTypography.caption)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            if let commentary = session.latestCommentary {
                Text(commentary)
                    .font(HaneulchiTypography.body)
            }

            if let activeWindowTitle = session.activeWindowTitle {
                Text("window: \(activeWindowTitle)")
                    .font(HaneulchiTypography.caption)
            }

            Text("dispatch: \(session.dispatchReason ?? session.dispatchState.rawValue)")
                .font(HaneulchiTypography.caption)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(HaneulchiChrome.Colors.surfaceMuted)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
    }
}
