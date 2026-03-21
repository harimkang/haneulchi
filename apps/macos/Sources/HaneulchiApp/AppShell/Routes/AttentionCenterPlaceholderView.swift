import SwiftUI

struct AttentionCenterPlaceholderView: View {
    let descriptor: RouteDestinationDescriptor

    var body: some View {
        RoutePlaceholderCard(descriptor: descriptor)
    }
}
