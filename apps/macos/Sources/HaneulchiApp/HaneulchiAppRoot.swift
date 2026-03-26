import SwiftUI

public struct HaneulchiAppRoot: App {
    @StateObject private var shellModel = AppShellModel.liveDefault(coreBridge: .live)

    public init() {}

    public var body: some Scene {
        WindowGroup {
            AppShellView(model: shellModel)
                .task {
                    await shellModel.startLocalControlServerIfNeeded()
                }
        }
        .commands {
            AppShellCommands(model: shellModel)
        }
    }
}
