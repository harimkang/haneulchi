import SwiftUI

@main
struct HaneulchiApp: App {
    @StateObject private var shellModel = AppShellModel.liveDefault()

    var body: some Scene {
        WindowGroup {
            AppShellView(model: shellModel)
        }
        .commands {
            AppShellCommands(model: shellModel)
        }
    }
}
