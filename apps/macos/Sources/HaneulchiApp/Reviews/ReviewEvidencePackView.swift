import SwiftUI

struct ReviewEvidencePackView: View {
    let model: ReviewEvidencePackModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Evidence Pack")
                .font(HaneulchiTypography.heading(18))

            Text(model.summary)
                .font(HaneulchiTypography.body)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            if !model.touchedFiles.isEmpty {
                Text("Touched files: \(model.touchedFiles.joined(separator: ", "))")
                    .font(HaneulchiTypography.caption)
            }

            ForEach(model.primaryFacts, id: \.label) { fact in
                Text("\(fact.label): \(fact.value)")
                    .font(HaneulchiTypography.caption)
            }

            if !model.warningRows.isEmpty {
                Text("Warnings: \(model.warningRows.joined(separator: ", "))")
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.warning)
            }

            if let manifestPath = model.manifestPath {
                Text("Manifest: \(manifestPath)")
                    .font(HaneulchiTypography.caption)
            }
            if let ciRunURL = model.ciRunURL {
                Text("CI: \(ciRunURL)")
                    .font(HaneulchiTypography.caption)
            }
            if let prURL = model.prURL {
                Text("PR: \(prURL)")
                    .font(HaneulchiTypography.caption)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(HaneulchiChrome.Colors.surfaceMuted)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }
}
