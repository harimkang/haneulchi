use std::path::PathBuf;

use hc_ffi::resolve_api_server_socket_path;

#[test]
fn api_server_socket_path_prefers_explicit_argument_then_env_then_default() {
    let explicit = resolve_api_server_socket_path(Some("/tmp/haneulchi-explicit.sock"));
    assert_eq!(explicit, PathBuf::from("/tmp/haneulchi-explicit.sock"));

    unsafe {
        std::env::set_var("HC_CONTROL_SOCKET", "/tmp/haneulchi-env.sock");
    }
    let env_path = resolve_api_server_socket_path(None);
    assert_eq!(env_path, PathBuf::from("/tmp/haneulchi-env.sock"));
    unsafe {
        std::env::remove_var("HC_CONTROL_SOCKET");
    }

    let default_path = resolve_api_server_socket_path(None);
    assert!(default_path.ends_with("Library/Application Support/Haneulchi/run/control.sock"));
}
