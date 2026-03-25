import SwiftUI

// MARK: - Glass Panel
// Floating surfaces: command palette, modals, context menus
struct GlassPanel: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(HaneulchiChrome.Surface.floating(0.84))
            .background(.ultraThinMaterial)
            .clipShape(RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.large))
            .shadow(color: .black.opacity(0.35), radius: 20, x: 0, y: 9)
    }
}

// MARK: - Ambient Shadow
// Subtle drop shadow for raised/floating surfaces
struct AmbientShadow: ViewModifier {
    var radius: CGFloat = 20
    var opacity: Double = 0.35
    func body(content: Content) -> some View {
        content
            .shadow(color: .black.opacity(opacity), radius: radius, x: 0, y: 9)
    }
}

// MARK: - Ghost Focus Ring
// Focus indicator using ghost stroke + primary glow
struct GhostFocusRing: ViewModifier {
    var isActive: Bool
    func body(content: Content) -> some View {
        content
            .overlay(
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.medium)
                    .strokeBorder(
                        isActive
                            ? HaneulchiChrome.Gradient.primaryStart.opacity(0.7)
                            : HaneulchiChrome.Stroke.ghost,
                        lineWidth: isActive ? 1.5 : 1
                    )
            )
    }
}

// MARK: - Hover Spotlight
// Subtle background brightening on hover
struct HoverSpotlight: ViewModifier {
    var isHovered: Bool
    var baseColor: Color = HaneulchiChrome.Surface.base
    var highlightColor: Color = HaneulchiChrome.Surface.raised
    func body(content: Content) -> some View {
        content
            .background(isHovered ? highlightColor : baseColor)
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.hoverShift), value: isHovered)
    }
}

// MARK: - Tonal Selection
// Selected state: raised surface + ghost focus stroke
struct TonalSelection: ViewModifier {
    var isSelected: Bool
    func body(content: Content) -> some View {
        content
            .background(isSelected ? HaneulchiChrome.Surface.raised : Color.clear)
            .overlay(
                RoundedRectangle(cornerRadius: HaneulchiMetrics.Radius.small)
                    .strokeBorder(
                        isSelected ? HaneulchiChrome.Stroke.ghost : Color.clear,
                        lineWidth: 1
                    )
            )
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.pressedSelection), value: isSelected)
    }
}

// MARK: - Pressed Scale
// Subtle scale-down feedback on press (max 1.01 per spec)
struct PressedScale: ViewModifier {
    var isPressed: Bool
    func body(content: Content) -> some View {
        content
            .scaleEffect(isPressed ? 0.98 : 1.0)
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.pressedSelection), value: isPressed)
    }
}

// MARK: - Floating Surface
// Panel raise: used for overlays entering/exiting
struct FloatingSurface: ViewModifier {
    var isVisible: Bool
    func body(content: Content) -> some View {
        content
            .opacity(isVisible ? 1 : 0)
            .scaleEffect(isVisible ? 1 : 0.97)
            .animation(.easeInOut(duration: HaneulchiMetrics.Motion.panelRaise), value: isVisible)
    }
}

// MARK: - View Extensions
extension View {
    func glassPanel() -> some View {
        modifier(GlassPanel())
    }

    func ambientShadow(radius: CGFloat = 20, opacity: Double = 0.35) -> some View {
        modifier(AmbientShadow(radius: radius, opacity: opacity))
    }

    func ghostFocusRing(isActive: Bool) -> some View {
        modifier(GhostFocusRing(isActive: isActive))
    }

    func hoverSpotlight(
        isHovered: Bool,
        base: Color = HaneulchiChrome.Surface.base,
        highlight: Color = HaneulchiChrome.Surface.raised
    ) -> some View {
        modifier(HoverSpotlight(isHovered: isHovered, baseColor: base, highlightColor: highlight))
    }

    func tonalSelection(isSelected: Bool) -> some View {
        modifier(TonalSelection(isSelected: isSelected))
    }

    func pressedScale(isPressed: Bool) -> some View {
        modifier(PressedScale(isPressed: isPressed))
    }

    func floatingSurface(isVisible: Bool) -> some View {
        modifier(FloatingSurface(isVisible: isVisible))
    }
}
