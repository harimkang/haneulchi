import SwiftUI

struct TaskAutomationSection: View {
    let model: TaskDrawerModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Automation")
                .font(HaneulchiTypography.label(13))
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            if let automationMode = model.automationMode {
                Text("mode: \(automationMode.label)")
                    .font(HaneulchiTypography.caption)
            }
            Text("claim: \(model.claimState.rawValue)")
                .font(HaneulchiTypography.caption)
            if let trackerBindingState = model.trackerBindingState {
                Text("tracker: \(trackerBindingState)")
                    .font(HaneulchiTypography.caption)
            }
            Text("require review: \(model.requireReview ? "yes" : "no")")
                .font(HaneulchiTypography.caption)
            if let maxRuntimeMinutes = model.maxRuntimeMinutes {
                Text("max runtime: \(maxRuntimeMinutes)m")
                    .font(HaneulchiTypography.caption)
            }
            if let unsafeOverridePolicy = model.unsafeOverridePolicy {
                Text("unsafe override: \(unsafeOverridePolicy)")
                    .font(HaneulchiTypography.caption)
            }
            if let blockerReason = model.blockerReason {
                Text("blocker: \(blockerReason)")
                    .font(HaneulchiTypography.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.warning)
            }
        }
    }
}
