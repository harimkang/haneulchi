/// Sprint 5 optional-tool compatibility smoke tests.
///
/// Tests skip cleanly when a binary is absent so that CI environments without
/// every optional tool installed do not produce false failures.

// ── helpers ───────────────────────────────────────────────────────────────────

fn find_binary(name: &str) -> Option<std::path::PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|dir| dir.join(name))
        .find(|path| path.exists())
}

// ── required-tool compatibility ───────────────────────────────────────────────

#[test]
fn compat_git_version_matches_expected_minimum() {
    let output = std::process::Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to run git --version");
    assert!(output.status.success(), "git --version must exit 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Typical output: "git version 2.43.0"
    let version_str = stdout.trim();
    let major: u32 = version_str
        .split_whitespace()
        .nth(2) // "2.43.0"
        .and_then(|v| v.split('.').next())
        .and_then(|m| m.parse().ok())
        .expect("could not parse git major version from output: {version_str}");
    assert!(
        major >= 2,
        "expected git major version >= 2, got {major} (full: {version_str})"
    );
}

#[test]
fn compat_shell_zsh_can_execute_simple_script() {
    let output = std::process::Command::new("/bin/zsh")
        .args(["-c", "echo ok"])
        .output()
        .expect("failed to run /bin/zsh");
    assert!(output.status.success(), "/bin/zsh -c 'echo ok' must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ok"),
        "expected 'ok' in zsh output; got: {stdout:?}"
    );
}

// ── optional-tool compatibility (skip when absent) ────────────────────────────

#[test]
fn compat_optional_tool_yazi_skipped_when_absent() {
    let Some(binary) = find_binary("yazi") else {
        eprintln!("SKIP: yazi not installed");
        return;
    };
    let status = std::process::Command::new(binary)
        .arg("--version")
        .status()
        .expect("spawn yazi");
    assert!(status.success(), "yazi --version failed");
}

#[test]
fn compat_optional_tool_lazygit_skipped_when_absent() {
    let Some(binary) = find_binary("lazygit") else {
        eprintln!("SKIP: lazygit not installed");
        return;
    };
    let status = std::process::Command::new(binary)
        .arg("--version")
        .status()
        .expect("spawn lazygit");
    assert!(status.success(), "lazygit --version failed");
}

#[test]
fn compat_optional_tool_nvim_skipped_when_absent() {
    let binary = find_binary("nvim").or_else(|| find_binary("vim"));
    let Some(binary) = binary else {
        eprintln!("SKIP: nvim/vim not installed");
        return;
    };
    let status = std::process::Command::new(binary)
        .arg("--version")
        .status()
        .expect("spawn nvim/vim");
    assert!(status.success(), "nvim/vim --version failed");
}
