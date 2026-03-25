use hc_runtime::terminal::geometry::TerminalGeometry;
use hc_runtime::terminal::runtime::TerminalRuntime;
use hc_runtime::terminal::session::{TerminalLaunchConfig, TerminalRestorePoint};

#[test]
fn geometry_clamps_zero_values() {
    let geometry = TerminalGeometry::new(0, 0);

    assert_eq!(geometry.cols(), 1);
    assert_eq!(geometry.rows(), 1);
}

#[test]
fn restore_point_keeps_launch_config_and_geometry() {
    let config = TerminalLaunchConfig::shell(None);
    let restore = TerminalRestorePoint::new(config.clone(), TerminalGeometry::new(120, 40));

    assert_eq!(restore.geometry.cols(), 120);
    assert_eq!(restore.launch.program, config.program);
}

#[test]
fn restore_bundle_round_trips_to_json() {
    let restore = TerminalRestorePoint::new(
        TerminalLaunchConfig::command("/bin/sh", ["-lc", "printf 'restored\\n'"]),
        TerminalGeometry::new(100, 30),
    );

    let encoded = serde_json::to_string(&restore).unwrap();
    let decoded: TerminalRestorePoint = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded.geometry, restore.geometry);
    assert_eq!(decoded.launch, restore.launch);
}

#[test]
fn cold_restore_replays_launch_descriptor_with_latest_geometry() {
    let original = TerminalRestorePoint::new(
        TerminalLaunchConfig::command("/bin/sh", ["-lc", "printf 'restored\\n'"]),
        TerminalGeometry::new(100, 30),
    );
    let mut runtime = TerminalRuntime::default();
    let restored = runtime.restore(original).unwrap();

    let drained = restored
        .wait_and_drain(std::time::Duration::from_secs(2))
        .unwrap();

    assert!(String::from_utf8_lossy(&drained).contains("restored"));
    assert_eq!(restored.geometry(), TerminalGeometry::new(100, 30));
}

#[test]
fn restore_point_environment_is_excluded_from_serialization() {
    // Build a launch config that carries a secret env var.
    let config = TerminalLaunchConfig::shell(None)
        .with_env("SECRET_KEY", "secret_value");

    let restore = TerminalRestorePoint::new(config, TerminalGeometry::new(80, 24));
    let json = serde_json::to_string(&restore).expect("serialize restore point");

    // The #[serde(skip)] annotation on TerminalLaunchConfig::environment must
    // prevent the secret value from appearing in the serialized form.
    assert!(
        !json.contains("secret_value"),
        "serialized restore point must not contain secret env values; json={json:?}"
    );
    assert!(
        !json.contains("SECRET_KEY"),
        "serialized restore point must not contain env key names; json={json:?}"
    );
}
