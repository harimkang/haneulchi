import Testing
@testable import HaneulchiApp

@Test("preferences store persists the last active route and defaults to project focus")
func preferencesStorePersistsLastActiveRoute() throws {
    let store = AppShellPreferencesStore.inMemory

    #expect(try store.load().lastActiveRoute == .projectFocus)

    try store.save(.init(lastActiveRoute: .controlTower))

    #expect(try store.load().lastActiveRoute == .controlTower)
}
