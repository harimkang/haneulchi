use serde::Serialize;
use std::{env, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadinessStatus {
    Ready,
    Warning,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessCheck {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub status: ReadinessStatus,
}

impl ReadinessCheck {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        detail: impl Into<String>,
        status: ReadinessStatus,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: detail.into(),
            status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessSummary {
    pub ready: usize,
    pub warning: usize,
    pub missing: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessSnapshot {
    pub checks: Vec<ReadinessCheck>,
    pub summary: ReadinessSummary,
}

pub fn summarize_checks(checks: &[ReadinessCheck]) -> ReadinessSummary {
    checks.iter().fold(
        ReadinessSummary {
            ready: 0,
            warning: 0,
            missing: 0,
            total: 0,
        },
        |mut summary, check| {
            match check.status {
                ReadinessStatus::Ready => summary.ready += 1,
                ReadinessStatus::Warning => summary.warning += 1,
                ReadinessStatus::Missing => summary.missing += 1,
            }
            summary.total += 1;
            summary
        },
    )
}

pub fn collect_readiness_checks() -> Vec<ReadinessCheck> {
    let mut checks = vec![
        detect_shell(),
        detect_command("git"),
        detect_command("node"),
        detect_command("npm"),
    ];

    checks.extend(["claude", "codex", "gemini"].map(detect_agent_command));
    checks.push(ReadinessCheck::new(
        "signing",
        "Signing identity",
        "Developer ID certificate detection is not configured in this foundation build",
        ReadinessStatus::Missing,
    ));
    checks.push(ReadinessCheck::new(
        "generic-shell",
        "Generic Shell fallback",
        "Always available so terminal work can continue when agent adapters are missing",
        ReadinessStatus::Ready,
    ));

    checks
}

pub fn collect_readiness_snapshot() -> ReadinessSnapshot {
    let checks = collect_readiness_checks();
    let summary = summarize_checks(&checks);

    ReadinessSnapshot { checks, summary }
}

pub fn detect_command(command: &str) -> ReadinessCheck {
    let status = if find_on_path(command).is_some() {
        ReadinessStatus::Ready
    } else {
        ReadinessStatus::Missing
    };

    let detail = match status {
        ReadinessStatus::Ready => format!("{command} found on PATH"),
        ReadinessStatus::Missing => format!("{command} not found on PATH"),
        ReadinessStatus::Warning => unreachable!("detect_command only returns ready or missing"),
    };

    ReadinessCheck::new(format!("command:{command}"), command, detail, status)
}

fn detect_agent_command(command: &str) -> ReadinessCheck {
    let mut check = detect_command(command);
    check.id = format!("agent:{command}");
    check.label = format!("{command} CLI");
    if check.status == ReadinessStatus::Missing {
        check.status = ReadinessStatus::Warning;
        check.detail =
            format!("{command} CLI preset visible; raw generic shell fallback remains available");
    }
    check
}

fn detect_shell() -> ReadinessCheck {
    match env::var("SHELL") {
        Ok(shell) if !shell.trim().is_empty() => ReadinessCheck::new(
            "shell",
            "Login shell",
            format!(
                "{} detected",
                shell.rsplit('/').next().unwrap_or(shell.as_str())
            ),
            ReadinessStatus::Ready,
        ),
        _ => ReadinessCheck::new(
            "shell",
            "Login shell",
            "SHELL is not set; terminal sessions can still start with system fallback",
            ReadinessStatus::Warning,
        ),
    }
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(command))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::{detect_command, summarize_checks, ReadinessCheck, ReadinessStatus};

    #[test]
    fn summarizes_ready_warning_and_missing_checks() {
        let summary = summarize_checks(&[
            ReadinessCheck::new("shell", "Login shell", "zsh", ReadinessStatus::Ready),
            ReadinessCheck::new(
                "agent",
                "Codex CLI",
                "not configured",
                ReadinessStatus::Warning,
            ),
            ReadinessCheck::new(
                "signing",
                "Signing identity",
                "missing",
                ReadinessStatus::Missing,
            ),
        ]);

        assert_eq!(summary.ready, 1);
        assert_eq!(summary.warning, 1);
        assert_eq!(summary.missing, 1);
        assert_eq!(summary.total, 3);
    }

    #[test]
    fn detects_commands_from_path_without_panicking() {
        let detected = detect_command("sh");

        assert!(matches!(
            detected.status,
            ReadinessStatus::Ready | ReadinessStatus::Missing
        ));
        assert_eq!(detected.id, "command:sh");
    }
}
