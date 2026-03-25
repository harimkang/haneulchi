import SwiftUI

// MARK: - HaneulchiButtonStyle

struct HaneulchiButtonStyle: ButtonStyle {
    enum Variant {
        case primary
        case secondary
        case tertiary
        case danger
    }

    var variant: Variant = .primary

    func makeBody(configuration: Configuration) -> some View {
        HaneulchiButtonBody(configuration: configuration, variant: variant)
    }
}

private struct HaneulchiButtonBody: View {
    let configuration: ButtonStyleConfiguration
    let variant: HaneulchiButtonStyle.Variant

    @State private var isHovered = false

    var body: some View {
        configuration.label
            .font(HaneulchiTypography.systemLabel)
            .tracking(HaneulchiTypography.Tracking.labelWide)
            .foregroundColor(labelColor)
            .padding(.horizontal, HaneulchiMetrics.Spacing.md)
            .padding(.vertical, HaneulchiMetrics.Spacing.xs)
            .frame(minHeight: HaneulchiMetrics.Target.compact)
            .background(backgroundView)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            .pressedScale(isPressed: configuration.isPressed)
            .onHover { isHovered = $0 }
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.hoverShift), value: isHovered)
    }

    @ViewBuilder
    private var backgroundView: some View {
        switch variant {
        case .primary:
            HaneulchiChrome.Gradient.primaryLinear
        case .secondary:
            isHovered
                ? HaneulchiChrome.Surface.raised
                : HaneulchiChrome.Surface.base
        case .tertiary:
            Color.clear
        case .danger:
            isHovered
                ? HaneulchiChrome.State.errorSolid.opacity(0.25)
                : HaneulchiChrome.State.errorSolid.opacity(0.12)
        }
    }

    private var labelColor: Color {
        switch variant {
        case .primary:
            HaneulchiChrome.Surface.foundation
        case .secondary:
            HaneulchiChrome.Label.primary
        case .tertiary:
            isHovered
                ? HaneulchiChrome.Gradient.primaryEnd
                : HaneulchiChrome.Label.secondary
        case .danger:
            HaneulchiChrome.State.error
        }
    }
}

struct HaneulchiIconButton: View {
    enum Tone {
        case secondary
        case tertiary
    }

    let action: HaneulchiChromeAction
    let tone: Tone
    var size: CGFloat = HaneulchiMetrics.Target.compact
    let onTrigger: () -> Void

    @State private var isHovered = false

    var body: some View {
        Button(action: onTrigger) {
            Image(systemName: action.symbolName)
                .font(.system(size: HaneulchiMetrics.Icon.standard, weight: .medium))
                .foregroundStyle(foregroundColor)
                .frame(width: size, height: size)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
        }
        .buttonStyle(.plain)
        .help(action.accessibilityLabel)
        .accessibilityLabel(action.accessibilityLabel)
        .onHover { isHovered = $0 }
        .animation(.easeInOut(duration: HaneulchiMetrics.Motion.hoverShift), value: isHovered)
    }

    private var foregroundColor: Color {
        switch tone {
        case .secondary:
            HaneulchiChrome.Label.secondary
        case .tertiary:
            isHovered ? HaneulchiChrome.Label.primary : HaneulchiChrome.Label.muted
        }
    }

    private var backgroundColor: Color {
        switch tone {
        case .secondary:
            isHovered ? HaneulchiChrome.Surface.raised : HaneulchiChrome.Surface.base
        case .tertiary:
            isHovered ? HaneulchiChrome.Surface.base.opacity(0.55) : Color.clear
        }
    }
}

// MARK: - HaneulchiStatusBadge

struct HaneulchiStatusBadge: View {
    enum State {
        case active
        case reviewReady
        case waitingInput
        case retryDue
        case manualTakeover
        case degraded
        case blocked
        case idle
        case done
    }

    var state: State
    var label: String

    var body: some View {
        Text(label)
            .font(HaneulchiTypography.compactMeta)
            .tracking(HaneulchiTypography.Tracking.labelWide)
            .foregroundColor(textColor)
            .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
            .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
            .background(fillColor)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
    }

    private var fillColor: Color {
        switch state {
        case .active:
            HaneulchiChrome.State.successSolid
        case .reviewReady:
            HaneulchiChrome.Gradient.primaryEnd
        case .waitingInput:
            HaneulchiChrome.State.warningSolid
        case .retryDue:
            HaneulchiChrome.State.warningSolid.opacity(0.7)
        case .manualTakeover:
            HaneulchiChrome.Gradient.primaryStart.opacity(0.18)
        case .degraded:
            HaneulchiChrome.State.warningSolid.opacity(0.18)
        case .blocked:
            HaneulchiChrome.State.errorSolid
        case .idle:
            HaneulchiChrome.Surface.base
        case .done:
            HaneulchiChrome.Surface.base.opacity(0.6)
        }
    }

    private var textColor: Color {
        switch state {
        case .active:
            HaneulchiChrome.State.success
        case .reviewReady:
            HaneulchiChrome.Label.primary
        case .waitingInput:
            HaneulchiChrome.State.warning
        case .retryDue:
            HaneulchiChrome.State.warning
        case .manualTakeover:
            HaneulchiChrome.Gradient.primaryStart
        case .degraded:
            HaneulchiChrome.State.warning
        case .blocked:
            HaneulchiChrome.State.error
        case .idle:
            HaneulchiChrome.Label.muted
        case .done:
            HaneulchiChrome.Label.muted
        }
    }
}

// MARK: - HaneulchiPanel

struct HaneulchiPanel<Content: View>: View {
    @ViewBuilder var content: Content

    var body: some View {
        content
            .padding(HaneulchiMetrics.Padding.card)
            .background(HaneulchiChrome.Surface.base)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }
}

// MARK: - HaneulchiCard

struct HaneulchiCard<Content: View>: View {
    var isSelected: Bool = false
    @ViewBuilder var content: Content

    @State private var isHovered = false

    var body: some View {
        content
            .padding(HaneulchiMetrics.Padding.card)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
            .overlay(
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium)
                    .strokeBorder(
                        isSelected ? HaneulchiChrome.Stroke.ghost : Color.clear,
                        lineWidth: 1,
                    ),
            )
            .onHover { isHovered = $0 }
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.hoverShift), value: isHovered)
            .animation(
                .easeInOut(duration: HaneulchiMetrics.Motion.pressedSelection),
                value: isSelected,
            )
    }

    private var backgroundColor: Color {
        if isSelected {
            HaneulchiChrome.Surface.raised
        } else if isHovered {
            HaneulchiChrome.Surface.raised
        } else {
            HaneulchiChrome.Surface.base
        }
    }
}

// MARK: - HaneulchiSectionHeader

struct HaneulchiSectionHeader: View {
    var title: String
    var count: Int?

    var body: some View {
        HStack(spacing: HaneulchiMetrics.Spacing.xs) {
            Text(title)
                .font(HaneulchiTypography.sectionHeading)
                .foregroundColor(HaneulchiChrome.Label.primary)

            if let count {
                Text("\(count)")
                    .font(HaneulchiTypography.compactMeta)
                    .tracking(HaneulchiTypography.Tracking.metaModerate)
                    .foregroundColor(HaneulchiChrome.Label.muted)
                    .padding(.horizontal, HaneulchiMetrics.Spacing.xs)
                    .padding(.vertical, HaneulchiMetrics.Spacing.xxs)
                    .background(HaneulchiChrome.Surface.recess)
                    .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.pill))
            }

            Spacer()
        }
        .padding(.horizontal, HaneulchiMetrics.Padding.card)
        .padding(.vertical, HaneulchiMetrics.Spacing.xs)
        .background(HaneulchiChrome.Surface.recess)
    }
}

// MARK: - HaneulchiTableRow

struct HaneulchiTableRow<Content: View>: View {
    var isSelected: Bool = false
    @ViewBuilder var content: Content

    @State private var isHovered = false

    var body: some View {
        content
            .frame(minHeight: HaneulchiMetrics.Target.row)
            .padding(.horizontal, HaneulchiMetrics.Padding.card)
            .background(backgroundColor)
            .overlay(
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                    .strokeBorder(
                        isSelected ? HaneulchiChrome.Stroke.ghost : Color.clear,
                        lineWidth: 1,
                    ),
            )
            .onHover { isHovered = $0 }
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.hoverShift), value: isHovered)
            .animation(
                .easeInOut(duration: HaneulchiMetrics.Motion.pressedSelection),
                value: isSelected,
            )
    }

    private var backgroundColor: Color {
        if isSelected {
            HaneulchiChrome.Surface.raised
        } else if isHovered {
            HaneulchiChrome.Surface.raised
        } else {
            Color.clear
        }
    }
}

// MARK: - HaneulchiMetricTile

struct HaneulchiMetricTile: View {
    var label: String
    var value: String
    var state: HaneulchiStatusBadge.State = .idle

    var body: some View {
        VStack(alignment: .leading, spacing: HaneulchiMetrics.Spacing.xxs) {
            Text(label)
                .font(HaneulchiTypography.compactMeta)
                .tracking(HaneulchiTypography.Tracking.metaModerate)
                .foregroundColor(HaneulchiChrome.Label.muted)

            Text(value)
                .font(HaneulchiTypography.systemLabel)
                .tracking(HaneulchiTypography.Tracking.labelWide)
                .foregroundColor(valueColor)
        }
        .padding(.horizontal, HaneulchiMetrics.Padding.compact)
        .padding(.vertical, HaneulchiMetrics.Spacing.xs)
        .background(HaneulchiChrome.Surface.base)
        .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium))
    }

    private var valueColor: Color {
        switch state {
        case .active:
            HaneulchiChrome.State.success
        case .reviewReady:
            HaneulchiChrome.Gradient.primaryEnd
        case .waitingInput:
            HaneulchiChrome.State.warning
        case .retryDue:
            HaneulchiChrome.State.warning
        case .manualTakeover:
            HaneulchiChrome.Gradient.primaryStart
        case .degraded:
            HaneulchiChrome.State.warning
        case .blocked:
            HaneulchiChrome.State.error
        case .idle:
            HaneulchiChrome.Label.secondary
        case .done:
            HaneulchiChrome.Label.muted
        }
    }
}
