import Foundation
import Testing
@testable import HaneulchiApp

@Test("spawn response can decode a session identifier before fetching a snapshot")
func spawnResponseDecodesSessionIdentifier() throws {
    let payload = Data(#"{"session_id":"session-0001"}"#.utf8)

    #expect(try decodeSpawnSessionID(from: payload) == "session-0001")
}

@Test("session snapshots can decode when launch geometry is omitted and top-level geometry is present")
func sessionSnapshotDecodesWithoutLaunchGeometry() throws {
    let payload = Data(
        #"""
        {
          "session_id": "session-0001",
          "launch": {
            "program": "/bin/zsh",
            "args": [],
            "current_directory": null
          },
          "geometry": {
            "cols": 80,
            "rows": 24
          },
          "running": true,
          "exit_code": null
        }
        """#.utf8
    )

    let snapshot = try JSONDecoder().decode(TerminalSessionSnapshot.self, from: payload)

    #expect(snapshot.sessionID == "session-0001")
    #expect(snapshot.launch.geometry == TerminalGridSize(cols: 80, rows: 24))
    #expect(snapshot.geometry == TerminalGridSize(cols: 80, rows: 24))
    #expect(snapshot.running)
}
