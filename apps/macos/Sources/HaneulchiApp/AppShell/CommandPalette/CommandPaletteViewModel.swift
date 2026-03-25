import Foundation

@MainActor
final class CommandPaletteViewModel: ObservableObject {
    let catalog: CommandPaletteCatalog

    @Published var query = "" {
        didSet {
            normalizeSelection()
        }
    }

    @Published private var selectedItemID: String?

    init(catalog: CommandPaletteCatalog) {
        self.catalog = catalog
        selectedItemID = catalog.sections.first?.items.first?.id
    }

    var filteredSections: [CommandPaletteSection] {
        let normalizedQuery = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !normalizedQuery.isEmpty else {
            return catalog.sections.filter { !$0.items.isEmpty }
        }

        return catalog.sections.compactMap { section in
            let items = section.items.filter { item in
                item.title.lowercased().contains(normalizedQuery)
                    || (item.subtitle?.lowercased().contains(normalizedQuery) ?? false)
                    || item.tokens.contains(where: { $0.contains(normalizedQuery) })
            }

            guard !items.isEmpty else {
                return nil
            }

            return CommandPaletteSection(kind: section.kind, items: items)
        }
    }

    var selection: CommandPaletteItem? {
        let items = flattenedItems
        guard !items.isEmpty else {
            return nil
        }

        if let selectedItemID, let item = items.first(where: { $0.id == selectedItemID }) {
            return item
        }

        return items.first
    }

    func select(_ item: CommandPaletteItem) {
        selectedItemID = item.id
    }

    func moveSelectionDown() {
        moveSelection(offset: 1)
    }

    func moveSelectionUp() {
        moveSelection(offset: -1)
    }

    private var flattenedItems: [CommandPaletteItem] {
        filteredSections.flatMap(\.items)
    }

    private func moveSelection(offset: Int) {
        let items = flattenedItems
        guard !items.isEmpty else {
            selectedItemID = nil
            return
        }

        guard let currentID = selection?.id,
              let index = items.firstIndex(where: { $0.id == currentID })
        else {
            selectedItemID = items.first?.id
            return
        }

        let nextIndex = (index + offset + items.count) % items.count
        selectedItemID = items[nextIndex].id
    }

    private func normalizeSelection() {
        let items = flattenedItems
        guard !items.isEmpty else {
            selectedItemID = nil
            return
        }

        if let selectedItemID, items.contains(where: { $0.id == selectedItemID }) {
            return
        }

        selectedItemID = items.first?.id
    }
}
