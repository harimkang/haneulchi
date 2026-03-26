@testable import HaneulchiAppUI
import Testing

@Test("live runtime info maps the chosen backend id and transport")
func liveRuntimeInfoMapsBackendAndTransport() throws {
    let descriptor = try CoreBridge.live.runtimeInfo()

    if descriptor.demoMode {
        #expect(descriptor.rendererID == "preview")
        #expect(descriptor.transport == "preview")
    } else {
        #expect(descriptor.rendererID == "swiftterm")
        #expect(descriptor.transport == "ffi_c_abi")
    }
}
