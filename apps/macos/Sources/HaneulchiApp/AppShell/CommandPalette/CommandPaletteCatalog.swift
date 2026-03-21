import Foundation

struct CommandPaletteCatalog: Equatable, Sendable {
    let sections: [CommandPaletteSection]

    static func build(
        snapshot: AppShellSnapshot,
        files: [ProjectFileIndex.Entry],
        tasks: [TaskSearchProjectionStore.Row],
        inventory: [InventorySearchProjectionStore.Row]
    ) -> Self {
        let commandItems = buildCommandItems(snapshot: snapshot)
        let fileItems = files.map {
            CommandPaletteItem(
                id: "file-\($0.absolutePath)",
                section: .files,
                title: $0.relativePath,
                subtitle: $0.absolutePath,
                tokens: [$0.relativePath.lowercased(), $0.absolutePath.lowercased()],
                action: .queueFileSelection($0.absolutePath)
            )
        }
        let sessionItems = snapshot.sessions.map {
            CommandPaletteItem(
                id: "session-\($0.sessionID)",
                section: .sessions,
                title: $0.title,
                subtitle: $0.currentDirectory,
                tokens: [$0.title.lowercased(), ($0.currentDirectory ?? "").lowercased(), $0.sessionID.lowercased()],
                action: .jumpToSession($0.sessionID)
            )
        }
        let taskItems = tasks.map {
            CommandPaletteItem(
                id: "task-\($0.taskID)",
                section: .tasks,
                title: $0.title,
                subtitle: "\($0.state.rawValue) · \($0.automationMode.rawValue)",
                tokens: [$0.title.lowercased(), $0.taskID.lowercased(), $0.projectID.lowercased()],
                action: .selectRoute(.taskBoard)
            )
        }
        let inventoryItems = inventory.map {
            CommandPaletteItem(
                id: "inventory-\($0.itemID)",
                section: .inventory,
                title: $0.title,
                subtitle: $0.rootPath,
                tokens: [$0.title.lowercased(), $0.rootPath.lowercased(), $0.kind.rawValue],
                action: .selectRoute(.projectFocus)
            )
        }

        return Self(sections: [
            .init(kind: .commands, items: commandItems),
            .init(kind: .files, items: fileItems),
            .init(kind: .sessions, items: sessionItems),
            .init(kind: .tasks, items: taskItems),
            .init(kind: .inventory, items: inventoryItems),
        ])
    }

    private static func buildCommandItems(snapshot: AppShellSnapshot) -> [CommandPaletteItem] {
        var items = Route.primaryCases.map { route in
            CommandPaletteItem(
                id: "command-route-\(route.rawValue)",
                section: .commands,
                title: route.title,
                subtitle: route.shortcutLabel,
                tokens: [route.title.lowercased(), route.rawValue],
                action: .selectRoute(route)
            )
        }

        items.append(
            .init(
                id: "command-settings",
                section: .commands,
                title: "Settings",
                subtitle: "Cmd+,",
                tokens: ["settings"],
                action: .openSettings
            )
        )

        items.append(
            .init(
                id: "command-create-task-draft",
                section: .commands,
                title: "Create Task Draft",
                subtitle: "Task Board",
                tokens: ["create task", "task draft", "task board"],
                action: .createTaskDraft("New Task")
            )
        )

        if !snapshot.attention.isEmpty {
            items.append(
                .init(
                    id: "command-latest-unread",
                    section: .commands,
                    title: "Latest Unread",
                    subtitle: "Cmd+Shift+U",
                    tokens: ["latest unread", "attention", "unread"],
                    action: .jumpToLatestUnread
                )
            )
        }

        return items
    }
}
