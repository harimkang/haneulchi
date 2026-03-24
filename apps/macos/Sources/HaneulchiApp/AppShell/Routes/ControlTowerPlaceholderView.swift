import SwiftUI

struct ControlTowerPlaceholderView: View {
    let snapshot: AppShellSnapshot
    let onAction: (AppShellAction) -> Void

    var body: some View {
        ControlTowerView(
            model: ControlTowerViewModel(snapshot: snapshot),
            onAction: onAction
        )
    }
}
