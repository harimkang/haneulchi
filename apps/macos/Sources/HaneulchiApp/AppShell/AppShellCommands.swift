import SwiftUI

struct AppShellCommands: Commands {
    let model: AppShellModel

    var body: some Commands {
        CommandMenu("Navigate") {
            ForEach(Route.primaryCases) { route in
                Button(route.title) {
                    Task {
                        await model.perform(.selectRoute(route))
                    }
                }
                .keyboardShortcut(
                    route.keyboardShortcut.keyEquivalent,
                    modifiers: route.keyboardShortcut.eventModifiers,
                )
            }

            Button("Command Palette") {
                Task {
                    await model.perform(.toggleCommandPalette)
                }
            }
            .keyboardShortcut("P", modifiers: [.command, .shift])

            Button("New Session") {
                Task {
                    await model.perform(.presentNewSessionSheet)
                }
            }
            .keyboardShortcut("N", modifiers: [.command])

            Button("Workflow Contract") {
                Task {
                    await model.perform(.presentWorkflowDrawer)
                }
            }
            .keyboardShortcut("R", modifiers: [.command, .shift])

            Button("Settings") {
                Task {
                    await model.perform(.openSettings)
                }
            }
            .keyboardShortcut(",", modifiers: [.command])

            Button("Latest Unread") {
                Task {
                    await model.perform(.jumpToLatestUnread)
                }
            }
            .keyboardShortcut(
                Route.latestUnreadShortcut.keyEquivalent,
                modifiers: Route.latestUnreadShortcut.eventModifiers,
            )

            Button("Show Inventory") {
                Task {
                    await model.perform(.presentInventory)
                }
            }
            .keyboardShortcut("i", modifiers: [.command, .shift])
        }
    }
}

private extension RouteShortcut {
    var keyEquivalent: KeyEquivalent {
        KeyEquivalent(Character(key))
    }

    var eventModifiers: EventModifiers {
        var modifiers: EventModifiers = []

        if self.modifiers.contains(.command) {
            modifiers.insert(.command)
        }

        if self.modifiers.contains(.shift) {
            modifiers.insert(.shift)
        }

        return modifiers
    }
}
