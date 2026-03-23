import SwiftUI

struct ControlTowerPlaceholderView: View {
    let descriptor: RouteDestinationDescriptor
    let snapshot: AppShellSnapshot

    var body: some View {
        ControlTowerOpsStripView(model: AutomationPanelViewModel(snapshot: snapshot))
    }
}
