import AppKit
import SwiftUI

struct QuickPreviewView: View {
    let workspaceState: ProjectFocusWorkspaceState
    var layoutStyle: ProjectFocusSupportingPanelLayoutStyle = .regular
    let onEdit: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HaneulchiSectionHeader(title: "Quick Preview")

            VStack(alignment: .leading, spacing: contentSpacing) {
                if let selectedFilePath = workspaceState.selectedFilePath {
                    Text(selectedFilePath)
                        .font(HaneulchiTypography.compactMeta)
                        .tracking(HaneulchiTypography.Tracking.metaModerate)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                        .lineLimit(layoutStyle == .compact ? 2 : 1)
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
                                .font(previewFont)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)

                if workspaceState.previewMode != .image, workspaceState.selectedFilePath != nil {
                    Button("Quick Edit", action: onEdit)
                        .buttonStyle(HaneulchiButtonStyle(variant: .secondary))
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .padding(contentPadding)
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }

    private var contentSpacing: CGFloat {
        layoutStyle == .compact ? HaneulchiMetrics.Spacing.xs : HaneulchiMetrics.Spacing.sm
    }

    private var contentPadding: CGFloat {
        layoutStyle == .compact ? HaneulchiMetrics.Padding.compact : HaneulchiMetrics.Padding.card
    }

    private var previewFont: Font {
        layoutStyle == .compact ? HaneulchiTypography.bodySmall : HaneulchiTypography.body
    }
}
