import Foundation

struct PresetRegistry: Equatable, Sendable {
    enum InstallState: String, Equatable, Sendable {
        case installed
        case missing
    }

    struct Preset: Codable, Equatable, Sendable, Identifiable {
        let id: String
        let title: String
        let binary: String
        let defaultArgs: [String]
        let capabilityFlags: [String]
        let requiresShellIntegration: Bool
        let installState: InstallState

        enum CodingKeys: String, CodingKey {
            case id
            case title
            case binary
            case defaultArgs = "default_args"
            case capabilityFlags = "capability_flags"
            case requiresShellIntegration = "requires_shell_integration"
        }

        init(
            id: String,
            title: String,
            binary: String,
            defaultArgs: [String],
            capabilityFlags: [String],
            requiresShellIntegration: Bool,
            installState: InstallState,
        ) {
            self.id = id
            self.title = title
            self.binary = binary
            self.defaultArgs = defaultArgs
            self.capabilityFlags = capabilityFlags
            self.requiresShellIntegration = requiresShellIntegration
            self.installState = installState
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            id = try container.decode(String.self, forKey: .id)
            title = try container.decode(String.self, forKey: .title)
            binary = try container.decode(String.self, forKey: .binary)
            defaultArgs = try container.decode([String].self, forKey: .defaultArgs)
            capabilityFlags = try container.decode([String].self, forKey: .capabilityFlags)
            requiresShellIntegration = try container.decode(
                Bool.self,
                forKey: .requiresShellIntegration,
            )
            installState = .missing
        }
    }

    let presets: [Preset]

    static func loadDefault(
        commandResolver: @escaping (String) -> Bool = { command in
            ProcessInfo.processInfo.environment["PATH"]?.split(separator: ":")
                .contains(where: { path in
                    FileManager.default
                        .isExecutableFile(atPath: URL(fileURLWithPath: String(path))
                            .appendingPathComponent(command).path)
                }) == true
        },
    ) throws -> Self {
        let data = try Data(contentsOf: defaultPresetURL)
        let rawPresets = try JSONDecoder().decode([Preset].self, from: data)
        return Self(
            presets: rawPresets.map { preset in
                Preset(
                    id: preset.id,
                    title: preset.title,
                    binary: preset.binary,
                    defaultArgs: preset.defaultArgs,
                    capabilityFlags: preset.capabilityFlags,
                    requiresShellIntegration: preset.requiresShellIntegration,
                    installState: commandResolver(preset.binary) ? .installed : .missing,
                )
            },
        )
    }

    func preset(id: String?) -> Preset? {
        guard let id else {
            return nil
        }
        return presets.first(where: { $0.id == id })
    }

    private static var defaultPresetURL: URL {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0 ..< 6 {
            url.deleteLastPathComponent()
        }
        return url
            .appendingPathComponent("config/presets/default-presets.json")
    }
}
