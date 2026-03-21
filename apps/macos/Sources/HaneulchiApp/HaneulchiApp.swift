import SwiftUI

@main
struct HaneulchiApp: App {
    var body: some Scene {
        WindowGroup {
            AppShellView(model: AppShellModel.liveDefault())
        }
    }
}
