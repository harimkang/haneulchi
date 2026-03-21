import SwiftUI

struct ControlTowerPlaceholderView: View {
    let descriptor: RouteDestinationDescriptor

    var body: some View {
        RoutePlaceholderCard(descriptor: descriptor)
    }
}
