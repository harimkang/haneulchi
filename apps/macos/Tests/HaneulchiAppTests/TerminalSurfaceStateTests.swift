import Foundation
@testable import HaneulchiApp
import Testing

@Test("bootstrap maps nil fixture to empty")
func bootstrapMapsNilFixtureToEmpty() {
    let controller = TerminalTranscriptController(
        runtimeInfoProvider: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: true,
            )
        },
    )

    let state = controller.bootstrap(fixtureName: nil)

    #expect(state.kind == .empty)
}

@Test("bootstrap maps missing fixture to degraded")
func bootstrapMapsMissingFixtureToDegraded() {
    let controller = TerminalTranscriptController(
        runtimeInfoProvider: {
            TerminalBackendDescriptor(
                rendererID: "swiftterm",
                transport: "ffi_c_abi",
                demoMode: true,
            )
        },
        fixtureLoader: { _ in
            throw TerminalTranscriptFixtureError.missing("missing.ansi")
        },
    )

    let state = controller.bootstrap(fixtureName: "missing.ansi")

    #expect(state.kind == .degraded)
}

@Test("bootstrap maps bridge failure to failed")
func bootstrapMapsBridgeFailureToFailed() {
    let controller = TerminalTranscriptController(
        runtimeInfoProvider: {
            throw CoreBridgeError.invalidRuntimeInfo
        },
    )

    let state = controller.bootstrap(fixtureName: "hello-world.ansi")

    #expect(state.kind == .failed)
}

@Test("generated fixture catalog resolves demo transcript content")
func generatedFixtureCatalogResolvesDemoTranscript() throws {
    #expect(TerminalTranscriptFixtures.availableNames == [
        "alternate-screen.ansi",
        "hello-world.ansi",
    ])
    let transcript = try TerminalTranscriptFixtures.load(named: "hello-world.ansi")

    #expect(transcript.contains("Haneulchi says hello"))
}

@Test("live surface restore failure maps to an operator-visible failed state")
func liveSurfaceRestoreFailureIsVisible() {
    let state = TerminalSurfaceState
        .live(.init(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false))
        .resolvedFailure("Hosted terminal could not start.")

    #expect(state.kind == .failed)
    #expect(state.message == "Hosted terminal could not start.")
}
