@testable import HaneulchiApp
import Testing

@Test("live runtime info maps the chosen backend id and transport")
func liveRuntimeInfoMapsBackendAndTransport() throws {
    let descriptor = try CoreBridge.live.runtimeInfo()

    #expect(descriptor.rendererID == "swiftterm")
    #expect(descriptor.transport == "ffi_c_abi")
    #expect(descriptor.demoMode == true)
}
