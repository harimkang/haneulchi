import SwiftUI

struct TaskBoardPlaceholderView: View {
    let descriptor: RouteDestinationDescriptor

    var body: some View {
        RoutePlaceholderCard(descriptor: descriptor)
    }
}
