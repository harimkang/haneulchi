use hc_runtime::terminal::backend::{TerminalBackendDescriptor, TerminalCapabilities};

#[test]
fn recommended_backend_descriptor_matches_terminal_contract() {
    let descriptor = TerminalBackendDescriptor::recommended();

    assert_eq!(descriptor.id, "swiftterm");
    assert_eq!(descriptor.display_name, "SwiftTerm");
    assert_eq!(
        descriptor.capabilities,
        TerminalCapabilities {
            alt_screen: true,
            resize: true,
            unicode_input: true,
            hyperlinks: true,
            webview_based: false,
        }
    );
}
