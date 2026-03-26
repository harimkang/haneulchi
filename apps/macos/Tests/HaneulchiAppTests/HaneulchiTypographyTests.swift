@testable import HaneulchiApp
import Testing

@Test("typography families prefer SF variants for display, body, labels, and mono content")
func typographyFamiliesPreferSFVariants() {
    #expect(HaneulchiTypography.Family.display == "SF Pro Display")
    #expect(HaneulchiTypography.Family.text == "SF Pro Text")
    #expect(HaneulchiTypography.Family.label == "SF Compact Text")
    #expect(HaneulchiTypography.Family.mono == "SF Mono")
}
