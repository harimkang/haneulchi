use std::path::PathBuf;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let socket = value_after(&args, "--socket")
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path);
    let requests = value_after(&args, "--requests")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(8);

    hc_control_plane::reset_task_board_for_tests();
    let sample = hc_control_plane::ControlPlaneState::sample();
    hc_control_plane::reset_shared_control_plane_snapshot_for_tests(sample.snapshot().clone());

    let server = hc_api::server::ApiServer::bind(&socket).expect("bind uds server");
    server.serve_requests(requests).expect("serve requests");
}

fn default_socket_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Haneulchi")
        .join("run")
        .join("control.sock")
}

fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}
