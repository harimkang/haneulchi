#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalCapabilities {
    pub alt_screen: bool,
    pub resize: bool,
    pub unicode_input: bool,
    pub hyperlinks: bool,
    pub webview_based: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalBackendDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub capabilities: TerminalCapabilities,
}

impl TerminalBackendDescriptor {
    pub const fn recommended() -> Self {
        Self {
            id: "swiftterm",
            display_name: "SwiftTerm",
            capabilities: TerminalCapabilities {
                alt_screen: true,
                resize: true,
                unicode_input: true,
                hyperlinks: true,
                webview_based: false,
            },
        }
    }
}
