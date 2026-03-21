import Foundation

struct TerminalBackendDescriptor: Decodable, Equatable, Sendable {
    let rendererID: String
    let transport: String
    let demoMode: Bool

    enum CodingKeys: String, CodingKey {
        case rendererID = "renderer_id"
        case transport
        case demoMode = "demo_mode"
    }
}
