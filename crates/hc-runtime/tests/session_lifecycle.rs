use hc_runtime::terminal::geometry::TerminalGeometry;
use hc_runtime::terminal::session::{ShellIntegrationMetadata, TerminalLaunchConfig, TerminalSession};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[test]
fn session_spawns_and_drains_shell_output() {
    let mut session = TerminalSession::spawn(
        TerminalLaunchConfig::command("/bin/sh", ["-lc", "printf 'ready\\n'"]),
        TerminalGeometry::new(80, 24),
    )
    .unwrap();

    let drained = session
        .wait_and_drain(std::time::Duration::from_secs(2))
        .unwrap();

    assert!(String::from_utf8_lossy(&drained).contains("ready"));
}

#[test]
fn session_round_trips_written_input() {
    let mut session = TerminalSession::spawn(
        TerminalLaunchConfig::command("/bin/sh", ["-lc", "cat"]),
        TerminalGeometry::new(80, 24),
    )
    .unwrap();

    session.write_input(b"ping\n").unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut captured = Vec::new();

    while Instant::now() < deadline {
        let drained = session.drain_output().unwrap();
        if !drained.is_empty() {
            captured.extend_from_slice(&drained);
        }

        if String::from_utf8_lossy(&captured).contains("ping") {
            break;
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    session.terminate().unwrap();

    assert!(String::from_utf8_lossy(&captured).contains("ping"));
}

#[test]
fn session_updates_geometry_after_resize() {
    let mut session = TerminalSession::spawn(
        TerminalLaunchConfig::command("/bin/sh", ["-lc", "sleep 5"]),
        TerminalGeometry::new(80, 24),
    )
    .unwrap();

    session.resize(TerminalGeometry::new(120, 40)).unwrap();
    session.terminate().unwrap();

    assert_eq!(session.geometry(), TerminalGeometry::new(120, 40));
}

#[test]
fn session_terminate_stops_long_running_process() {
    let mut session = TerminalSession::spawn(
        TerminalLaunchConfig::command("/bin/sh", ["-lc", "sleep 5"]),
        TerminalGeometry::new(80, 24),
    )
    .unwrap();

    session.terminate().unwrap();

    assert!(session.exit_status().is_some());
}

#[test]
fn shell_launch_bootstrap_sources_integration_script() {
    let launch = TerminalLaunchConfig::shell(Some(PathBuf::from("/tmp/demo")));

    assert_eq!(launch.program, "/bin/zsh");
    assert!(!launch.args.is_empty());
    assert!(launch.args.join(" ").contains("haneulchi.zsh"));
}

#[test]
fn session_strips_shell_markers_and_tracks_shell_metadata() {
    let mut session = TerminalSession::spawn(
        TerminalLaunchConfig::command(
            "/bin/sh",
            [
                "-lc",
                "printf '\\037HC|cwd|/tmp/demo\\n\\037HC|command|npm test\\n\\037HC|exit|17\\nvisible\\n'",
            ],
        ),
        TerminalGeometry::new(80, 24),
    )
    .unwrap();

    let drained = session
        .wait_and_drain(std::time::Duration::from_secs(2))
        .unwrap();
    let metadata = session.shell_metadata().unwrap();

    assert!(String::from_utf8_lossy(&drained).contains("visible"));
    assert_eq!(
        metadata,
        ShellIntegrationMetadata {
            current_directory: Some("/tmp/demo".to_string()),
            last_command: Some("npm test".to_string()),
            last_exit_code: Some(17),
            branch: None,
        }
    );
}
