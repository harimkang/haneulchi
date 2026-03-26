import SwiftUI

enum HaneulchiSignalAccent: Equatable {
    case neutral
    case success
    case reviewReady
    case warning
    case error
    case manual

    static func from(_ state: HaneulchiStatusBadge.State) -> Self {
        switch state {
        case .active:
            .success
        case .reviewReady:
            .reviewReady
        case .waitingInput, .retryDue, .degraded:
            .warning
        case .manualTakeover:
            .manual
        case .blocked:
            .error
        case .idle, .done:
            .neutral
        }
    }

    var tint: Color {
        switch self {
        case .neutral:
            HaneulchiChrome.Label.secondary
        case .success:
            HaneulchiChrome.State.success
        case .reviewReady:
            HaneulchiChrome.Gradient.primaryEnd
        case .warning:
            HaneulchiChrome.State.warning
        case .error:
            HaneulchiChrome.State.error
        case .manual:
            HaneulchiChrome.Gradient.primaryStart
        }
    }

    var line: Color {
        switch self {
        case .neutral:
            HaneulchiChrome.Surface.raised
        case .success:
            HaneulchiChrome.State.successSolid
        case .reviewReady:
            HaneulchiChrome.Gradient.primaryEnd
        case .warning:
            HaneulchiChrome.State.warningSolid
        case .error:
            HaneulchiChrome.State.errorSolid
        case .manual:
            HaneulchiChrome.Gradient.primaryStart.opacity(0.7)
        }
    }
}

struct HaneulchiMonolithMetric: Identifiable, Equatable {
    let id: String
    let label: String
    let value: String
    let accent: HaneulchiSignalAccent

    static func defaultOrder(
        cadence: String,
        lastTick: String,
        nextTick: String,
        reconcile: String,
        slots: String,
        workflow: String,
        tracker: String,
        queue: String,
        paused: String,
    ) -> [Self] {
        [
            .init(id: "cadence", label: "cadence", value: cadence, accent: .neutral),
            .init(id: "last_tick", label: "last tick", value: lastTick, accent: .neutral),
            .init(id: "next_tick", label: "next tick", value: nextTick, accent: .neutral),
            .init(id: "reconcile", label: "reconcile", value: reconcile, accent: .neutral),
            .init(id: "slots", label: "slots", value: slots, accent: .success),
            .init(id: "workflow", label: "workflow", value: workflow, accent: .reviewReady),
            .init(id: "tracker", label: "tracker", value: tracker, accent: .warning),
            .init(id: "queue", label: "queue", value: queue, accent: .warning),
            .init(id: "paused", label: "paused", value: paused, accent: .neutral),
        ]
    }
}

struct HaneulchiHeaderDeck<Trailing: View>: View {
    let title: String
    let subtitle: String?
    let horizontalPadding: CGFloat
    @ViewBuilder let trailing: Trailing

    init(
        title: String,
        subtitle: String? = nil,
        horizontalPadding: CGFloat = HaneulchiMetrics.Padding.card,
        @ViewBuilder trailing: () -> Trailing,
    ) {
        self.title = title
        self.subtitle = subtitle
        self.horizontalPadding = horizontalPadding
        self.trailing = trailing()
    }

    var body: some View {
        HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.lg) {
            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xs) {
                Text(title)
                    .font(HaneulchiTypography.display)
                    .tracking(HaneulchiTypography.Tracking.displayTight)
                    .foregroundStyle(HaneulchiChrome.Label.primary)

                if let subtitle, !subtitle.isEmpty {
                    Text(subtitle)
                        .font(HaneulchiTypography.deckSubtitle)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }

            Spacer(minLength: HaneulchiMetrics.Spacing.lg)

            trailing
        }
        .padding(.horizontal, horizontalPadding)
        .padding(.vertical, HaneulchiMetrics.Spacing.sm)
        .background(HaneulchiChrome.Surface.foundation)
    }
}

struct HaneulchiMonolithStrip<Trailing: View>: View {
    let metrics: [HaneulchiMonolithMetric]
    @ViewBuilder let trailing: Trailing

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.md) {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: HaneulchiMetrics.Spacing.lg) {
                        ForEach(metrics) { metric in
                            metricCell(metric)
                        }
                    }
                    .padding(.vertical, 1)
                }

                trailing
                    .fixedSize(horizontal: true, vertical: false)
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            if metrics.count > 6 {
                Divider()
                    .overlay(HaneulchiChrome.Stroke.ghost)
            }
        }
        .padding(.horizontal, HaneulchiMetrics.Padding.card)
        .padding(.vertical, HaneulchiMetrics.Spacing.sm)
        .frame(minHeight: HaneulchiMetrics.Operations.opsStripMinHeight)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large))
    }

    private func metricCell(_ metric: HaneulchiMonolithMetric) -> some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
            Text(metric.label)
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.metaModerate)
                .foregroundStyle(HaneulchiChrome.Label.muted)

            Text(metric.value)
                .font(HaneulchiTypography.opsValue)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundStyle(metric.accent.tint)
        }
        .frame(minWidth: 76, alignment: .leading)
    }
}

struct HaneulchiOpsRailPanel<Content: View>: View {
    let title: String
    let count: Int?
    @ViewBuilder let content: Content

    init(
        title: String,
        count: Int? = nil,
        @ViewBuilder content: () -> Content,
    ) {
        self.title = title
        self.count = count
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.sm) {
            HStack(alignment: .firstTextBaseline, spacing: HaneulchiMetrics.Spacing.xs) {
                Text(title)
                    .font(HaneulchiTypography.sectionHeading)
                    .foregroundStyle(HaneulchiChrome.Label.primary)

                if let count {
                    Text("\(count)")
                        .font(HaneulchiTypography.compactMeta)
                        .tracking(HaneulchiTypography.Tracking.metaModerate)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                }

                Spacer()
            }
            content
        }
        .padding(HaneulchiMetrics.Padding.card)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large))
    }
}

struct HaneulchiSignalRow<Trailing: View>: View {
    let accent: HaneulchiSignalAccent
    let eyebrow: String?
    let title: String
    let summary: String?
    let meta: String?
    @ViewBuilder let trailing: Trailing

    init(
        accent: HaneulchiSignalAccent,
        eyebrow: String? = nil,
        title: String,
        summary: String? = nil,
        meta: String? = nil,
        @ViewBuilder trailing: () -> Trailing,
    ) {
        self.accent = accent
        self.eyebrow = eyebrow
        self.title = title
        self.summary = summary
        self.meta = meta
        self.trailing = trailing()
    }

    var body: some View {
        HStack(alignment: .top, spacing: HaneulchiMetrics.Spacing.sm) {
            RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                .fill(accent.line)
                .frame(width: 4)

            VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
                if let eyebrow, !eyebrow.isEmpty {
                    Text(eyebrow)
                        .font(HaneulchiTypography.compactMeta)
                        .tracking(HaneulchiTypography.Tracking.labelWide)
                        .foregroundStyle(accent.tint)
                }

                Text(title)
                    .font(HaneulchiTypography.sectionHeading)
                    .foregroundStyle(HaneulchiChrome.Label.primary)

                if let summary, !summary.isEmpty {
                    Text(summary)
                        .font(HaneulchiTypography.bodySmall)
                        .foregroundStyle(HaneulchiChrome.Label.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }

                if let meta, !meta.isEmpty {
                    Text(meta)
                        .font(HaneulchiTypography.compactMeta)
                        .tracking(HaneulchiTypography.Tracking.metaModerate)
                        .foregroundStyle(HaneulchiChrome.Label.muted)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            trailing
        }
        .padding(HaneulchiMetrics.Padding.card)
        .frame(minHeight: HaneulchiMetrics.Operations.signalRowMinHeight)
        .background(HaneulchiChrome.Surface.raised)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }
}
