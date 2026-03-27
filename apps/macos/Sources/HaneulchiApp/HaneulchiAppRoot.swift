import SwiftUI

public struct HaneulchiAppRoot: App {
    @StateObject private var shellModel = AppShellModel.liveDefault(coreBridge: .live)

    public init() {}

    public var body: some Scene {
        WindowGroup {
            AppShellView(model: shellModel)
                .frame(minWidth: 960, minHeight: 640)
                .task {
                    await shellModel.startLocalControlServerIfNeeded()
                }
        }
        .defaultSize(width: 1360, height: 880)
        .windowResizability(.contentMinSize)
        .commands {
            AppShellCommands(model: shellModel)
        }
    }
}
