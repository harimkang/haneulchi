import Foundation

enum TerminalTranscriptFixtureError: Error, Equatable {
    case missing(String)
}

enum TerminalTranscriptFixtures {
    static var availableNames: [String] {
        GeneratedTerminalTranscriptFixtures.fixtures.keys.sorted()
    }

    static func load(named name: String) throws -> String {
        guard let transcript = GeneratedTerminalTranscriptFixtures.fixtures[name] else {
            throw TerminalTranscriptFixtureError.missing(name)
        }

        return transcript
    }
}
