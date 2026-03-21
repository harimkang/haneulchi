import SwiftUI

struct AppShellView: View {
    @State private var selectedRoute: Route? = .projectFocus

    var body: some View {
        NavigationSplitView {
            List(Route.allCases, selection: $selectedRoute) { route in
                Label(route.title, systemImage: route.symbolName)
                    .tag(route)
            }
            .navigationTitle("Haneulchi")
        } detail: {
            VStack(alignment: .leading, spacing: 16) {
                Text(selectedRoute?.title ?? "No Selection")
                    .font(.largeTitle)
                    .bold()
                Text("Initial shell scaffold aligned to Sprint 1 foundation.")
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .padding(24)
        }
    }
}

#Preview {
    AppShellView()
}
