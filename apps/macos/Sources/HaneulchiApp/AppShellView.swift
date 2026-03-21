import SwiftUI

struct AppShellView: View {
    @State private var selectedRoute: Route? = .projectFocus
    @State private var projectFocusModel = AppShellView.bootstrapProjectFocusModel()

    var body: some View {
        NavigationSplitView {
            List(Route.allCases, selection: $selectedRoute) { route in
                Label(route.title, systemImage: route.symbolName)
                    .tag(route)
            }
            .navigationTitle("Haneulchi")
        } detail: {
            detailView
        }
    }

    @ViewBuilder
    private var detailView: some View {
        switch selectedRoute {
        case .projectFocus:
            ProjectFocusView(model: projectFocusModel)
        case .controlTower, .taskBoard, .review, .attention:
            placeholderDetail
        case nil:
            placeholderDetail
        }
    }

    private var placeholderDetail: some View {
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

    private static func bootstrapProjectFocusModel(
        restoreStore: TerminalSessionRestoreStore = .liveDefault
    ) -> ProjectFocusView.Model {
        (try? ProjectFocusView.Model.bootstrap(restoreStore: restoreStore)) ?? .demo
    }
}

#Preview {
    AppShellView()
}
