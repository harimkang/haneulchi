import AppKit
import SwiftUI

struct QuickPreviewView: View {
    let workspaceState: ProjectFocusWorkspaceState
    let onEdit: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Quick Preview")

            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
                if let selectedFilePath = workspaceState.selectedFilePath {
                    Text(selectedFilePath)
                        .font(HaneulchiTypography.compactMeta)
                        .tracking(HaneulchiTypography.Tracking.metaModerate)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .lineLimit(1)
                }

                Group {
                    switch workspaceState.previewMode {
                    case .empty:
                        Text("Select a file to preview.")
                            .font(HaneulchiTypography.body)
                            .foregroundStyle(HaneulchiChrome.Label.secondary)
                    case .image:
                        if
                            let path = workspaceState.selectedFilePath,
                            let image = NSImage(contentsOfFile: path)
                        {
                            Image(nsImage: image)
                                .resizable()
                                .scaledToFit()
                        } else {
                            Text("Image preview unavailable.")
                                .font(HaneulchiTypography.body)
                                .foregroundStyle(HaneulchiChrome.Label.secondary)
                        }
                    case .text, .markdown, .json, .yaml:
                        ScrollView {
                            Text(workspaceState.previewText ?? "")
                                .font(HaneulchiTypography.body)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                if workspaceState.previewMode != .image, workspaceState.selectedFilePath != nil {
                    Button("Quick Edit", action: onEdit)
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                }
            }
            .padding(HaneulchiMetrics.Padding.card)
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }
}
