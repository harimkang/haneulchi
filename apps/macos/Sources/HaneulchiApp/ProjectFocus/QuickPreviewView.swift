import AppKit
import SwiftUI

struct QuickPreviewView: View {
    let workspaceState: ProjectFocusWorkspaceState
    let onEdit: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Preview")
                .font(.headline)

            if let selectedFilePath = workspaceState.selectedFilePath {
                Text(selectedFilePath)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Group {
                switch workspaceState.previewMode {
                case .empty:
                    Text("Select a file to preview.")
                        .foregroundStyle(.secondary)
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
                            .foregroundStyle(.secondary)
                    }
                case .text, .markdown, .json, .yaml:
                    ScrollView {
                        Text(workspaceState.previewText ?? "")
                            .font(HaneulchiTypography.body)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            if workspaceState.previewMode != .image, workspaceState.selectedFilePath != nil {
                Button("Quick Edit", action: onEdit)
                    .buttonStyle(.bordered)
            }
        }
        .padding(16)
        .background(HaneulchiChrome.Colors.surfaceBase)
    }
}
