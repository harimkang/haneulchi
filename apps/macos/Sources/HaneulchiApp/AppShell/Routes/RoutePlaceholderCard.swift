import SwiftUI

struct RoutePlaceholderCard: View {
    let descriptor: RouteDestinationDescriptor

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(descriptor.title)
                .font(HaneulchiTypography.heading(24).weight(.bold))

            Text(descriptor.summary)
                .font(HaneulchiTypography.body)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            Text(descriptor.nextActionTitle)
                .font(HaneulchiTypography.label(12))
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(HaneulchiChrome.Colors.surfaceRaised)
                .clipShape(Capsule())
        }
        .padding(HaneulchiChrome.Spacing.screenPadding)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceBase)
    }
}
