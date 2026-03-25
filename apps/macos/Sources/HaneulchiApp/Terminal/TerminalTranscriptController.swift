import Foundation

struct TerminalTranscriptController {
    typealias RuntimeInfoProvider = @Sendable () throws -> TerminalBackendDescriptor
    typealias FixtureLoader = @Sendable (String) throws -> String

    let runtimeInfoProvider: RuntimeInfoProvider
    let fixtureLoader: FixtureLoader

    init(
        runtimeInfoProvider: @escaping RuntimeInfoProvider = CoreBridge.live.runtimeInfo,
        fixtureLoader: @escaping FixtureLoader = TerminalTranscriptFixtures.load,
    ) {
        self.runtimeInfoProvider = runtimeInfoProvider
        self.fixtureLoader = fixtureLoader
    }

    func bootstrap(fixtureName: String?) -> TerminalSurfaceState {
        let backend: TerminalBackendDescriptor

        do {
            backend = try runtimeInfoProvider()
        } catch {
            return .failed(message: "Terminal bridge bootstrap failed.")
        }

        guard let fixtureName else {
            return .empty(backend)
        }

        do {
            let transcript = try fixtureLoader(fixtureName)
            return .ready(
                TerminalReplay(
                    backend: backend,
                    transcript: transcript,
                ),
            )
        } catch TerminalTranscriptFixtureError.missing {
            return .degraded(
                backend,
                message: "Transcript fixture is missing: \(fixtureName)",
            )
        } catch {
            return .degraded(
                backend,
                message: "Transcript fixture failed to load: \(fixtureName)",
            )
        }
    }

    func bootstrapLive() -> TerminalSurfaceState {
        do {
            return try .live(runtimeInfoProvider())
        } catch {
            return .failed(message: "Terminal bridge bootstrap failed.")
        }
    }
}
