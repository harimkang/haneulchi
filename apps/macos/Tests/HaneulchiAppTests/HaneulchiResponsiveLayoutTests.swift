@testable import HaneulchiAppUI
import Testing

@Test("shared viewport classes classify the documented breakpoint thresholds")
func viewportClassifiesDocumentedThresholds() {
    #expect(HaneulchiViewportClass.forWidth(0) == .compact)
    #expect(HaneulchiViewportClass.forWidth(959) == .compact)
    #expect(HaneulchiViewportClass.forWidth(960) == .medium)
    #expect(HaneulchiViewportClass.forWidth(1239) == .medium)
    #expect(HaneulchiViewportClass.forWidth(1240) == .wide)
    #expect(HaneulchiViewportClass.forWidth(1519) == .wide)
    #expect(HaneulchiViewportClass.forWidth(1520) == .expanded)
}
