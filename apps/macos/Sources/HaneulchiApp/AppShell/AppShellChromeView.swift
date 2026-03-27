import SwiftUI

struct AppShellChromeView<Content: View>: View {
    let chrome: AppShellChromeState
    let destination: Route
    let onAction: (AppShellAction) -> Void
    @ViewBuilder let content: Content
    @State private var viewportContext = HaneulchiViewportContext(width: 0)

    var body: some View {
        VStack(spacing: 0) {
            TopAppBarView(chrome: chrome, onAction: onAction)

            HStack(spacing: 0) {
                LeftRailView(
                    items: chrome.leftRailItems,
                    activeRoute: destination,
                    onAction: onAction,
                )

                content
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)

            BottomStatusStripView(
                items: chrome.bottomStripItems,
                transientNotice: chrome.transientNotice,
                onAction: onAction,
            )
        }
        .environment(\.viewportContext, viewportContext)
        .onGeometryChange(for: CGFloat.self) { geometry in
            geometry.size.width
        } action: { _, shellWidth in
            let resolvedContext = HaneulchiViewportContext(shellWidth: shellWidth)
            if resolvedContext != viewportContext {
                viewportContext = resolvedContext
            }
        }
        .background(HaneulchiChrome.Colors.appBackground)
    }
}
