@testable import HaneulchiApp
import Testing

@Test("typography families prefer SF variants for display, body, labels, and mono content")
func typographyFamiliesPreferSFVariants() {
    #expect(HaneulchiTypography.Family.display == "SF Pro Display")
    #expect(HaneulchiTypography.Family.text == "SF Pro Text")
    #expect(HaneulchiTypography.Family.label == "SF Compact Text")
    #expect(HaneulchiTypography.Family.mono == "SF Mono")
}

@Test(
    "typography prefers named SF faces when installed and falls back deterministically when they are missing",
)
func typographyResolvesNamedFacesWithFallbacks() {
    #expect(
        HaneulchiTypography.source(
            for: .display,
            isAvailable: { $0 == HaneulchiTypography.Family.display },
        ) == .custom(HaneulchiTypography.Family.display),
    )
    #expect(
        HaneulchiTypography.source(
            for: .label,
            isAvailable: { $0 == HaneulchiTypography.Family.text },
        ) == .custom(HaneulchiTypography.Family.text),
    )
    #expect(
        HaneulchiTypography.source(
            for: .label,
            isAvailable: { _ in false },
        ) == .system(.standard),
    )
    #expect(
        HaneulchiTypography.source(
            for: .mono,
            isAvailable: { _ in false },
        ) == .system(.monospaced),
    )
}
