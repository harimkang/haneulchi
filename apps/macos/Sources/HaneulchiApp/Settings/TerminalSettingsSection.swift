import SwiftUI

struct TerminalSettingsSection: View {
    let row: SettingsStatusViewModel.TerminalSettingsRow?

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Terminal")
                .font(.title3.weight(.semibold))

            if let row {
                VStack(alignment: .leading, spacing: 6) {
                    labeledRow(label: "Shell", value: row.shell)
                    labeledRow(label: "Default columns", value: "\(row.defaultCols)")
                    labeledRow(label: "Default rows", value: "\(row.defaultRows)")
                    labeledRow(label: "Scrollback lines", value: "\(row.scrollbackLines)")
                    labeledRow(
                        label: "Font",
                        value: row.fontName.isEmpty ? "System default" : row.fontName,
                    )
                    labeledRow(
                        label: "Theme",
                        value: row.theme.isEmpty ? "System default" : row.theme,
                    )
                    labeledRow(
                        label: "Cursor style",
                        value: row.cursorStyle.isEmpty ? "Block (default)" : row.cursorStyle,
                    )
                }
            } else {
                Text("No terminal settings configured.")
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }

    private func labeledRow(label: String, value: String) -> some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(label)
                    .font(.headline)
                    .frame(minWidth: 120, alignment: .leading)
                Text(value)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }

            VStack(alignment: .leading, spacing: 2) {
                Text(label)
                    .font(.headline)
                Text(value)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
    }
}
