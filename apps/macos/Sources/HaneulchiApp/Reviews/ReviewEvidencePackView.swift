import SwiftUI

struct ReviewEvidencePackView: View {
    let model: ReviewEvidencePackModel

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
            HaneulchiSectionHeader(title: "Evidence Pack")

            // Summary
            HaneulchiPanel {
                Text(model.summary)
                    .font(HaneulchiTypography.body)
                    .foregroundStyle(HaneulchiChrome.Label.primary)
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            // Primary facts
            if !model.primaryFacts.isEmpty {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                    ForEach(model.primaryFacts, id: \.label) { fact in
                        HaneulchiTableRow {
                            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                                Text(fact.label)
                                    .font(HaneulchiTypography.compactMeta)
                                    .tracking(HaneulchiTypography.Tracking.metaModerate)
                                    .foregroundStyle(HaneulchiChrome.Label.muted)
                                    .frame(width: 80, alignment: .leading)
                                Text(fact.value)
                                    .font(HaneulchiTypography.bodySmall)
                                    .foregroundStyle(HaneulchiChrome.Label.primary)
                                    .lineLimit(1)
                            }
                        }
                    }
                }
                .background(HaneulchiChrome.Surface.base)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }

            // Touched files
            if !model.touchedFiles.isEmpty {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                    HaneulchiSectionHeader(
                        title: "Touched Files",
                        count: model.touchedFiles.count,
                    )
                    ForEach(model.touchedFiles, id: \.self) { file in
                        HaneulchiTableRow {
                            Text(file)
                                .font(HaneulchiTypography.compactMeta)
                                .tracking(HaneulchiTypography.Tracking.metaModerate)
                                .foregroundStyle(HaneulchiChrome.Label.secondary)
                                .lineLimit(1)
                                .truncationMode(.middle)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
                .background(HaneulchiChrome.Surface.base)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }

            // Test results tile row
            HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                if let ciRunURL = model.ciRunURL {
                    HaneulchiMetricTile(
                        label: "CI",
                        value: ciRunURL,
                        state: ciTileState(for: model),
                    )
                }
                if let prURL = model.prURL {
                    HaneulchiMetricTile(
                        label: "PR",
                        value: prURL,
                        state: .active,
                    )
                }
                if let manifestPath = model.manifestPath {
                    HaneulchiMetricTile(
                        label: "Manifest",
                        value: manifestPath,
                        state: .active,
                    )
                }
            }

            // Warning rows
            if !model.warningRows.isEmpty {
                VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                    ForEach(model.warningRows, id: \.self) { warning in
                        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
                            Image(systemName: "exclamationmark.triangle.fill")
                                .font(.system(size: HaneulchiMetrics.Icon.small))
                                .foregroundStyle(HaneulchiChrome.State.error)
                            Text(warning)
                                .font(HaneulchiTypography.bodySmall)
                                .foregroundStyle(HaneulchiChrome.State.error)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .padding(.horizontal, HaneulchiMetrics.Padding.card)
                        .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                    }
                }
                .background(HaneulchiChrome.State.errorSolid.opacity(0.10))
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            }
        }
    }

    // MARK: - Helpers

    /// Derives the semantic badge state for the CI run tile from available evidence signals.
    /// Returns `.blocked` when evidence is degraded or the tests summary indicates failure,
    /// and `.active` otherwise (URL present, no failure signal detected).
    private func ciTileState(for model: ReviewEvidencePackModel) -> HaneulchiStatusBadge.State {
        if model.hasDegradedEvidence { return .blocked }
        if let tests = model.testsSummary {
            let lower = tests.lowercased()
            if lower.contains("fail") || lower.contains("error") { return .blocked }
        }
        return .active
    }
}
