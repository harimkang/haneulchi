import SwiftUI

struct ReviewQueuePlaceholderView: View {
    let descriptor: RouteDestinationDescriptor

    var body: some View {
        RoutePlaceholderCard(descriptor: descriptor)
    }
}
