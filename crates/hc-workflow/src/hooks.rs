use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::bootstrap::HookPhaseResult;
use crate::contract::{HookDefinition, HookPhase};

const STDIO_CAPTURE_LIMIT: usize = 120;

pub(crate) fn run_hook(
    phase: HookPhase,
    hook: &HookDefinition,
    command_path: &Path,
    session_cwd: &Path,
    env: &BTreeMap<String, String>,
) -> HookPhaseResult {
    let mut command = Command::new(command_path);
    command.current_dir(session_cwd);
    command.args(&hook.args);
    command.envs(env);

    let output = command.output();
    match output {
        Ok(output) => HookPhaseResult {
            phase: phase_name(phase).to_string(),
            command_path: Some(command_path.display().to_string()),
            exit_code: output.status.code(),
            stdout: truncate_capture(&String::from_utf8_lossy(&output.stdout)),
            stderr: truncate_capture(&String::from_utf8_lossy(&output.stderr)),
            succeeded: output.status.success(),
        },
        Err(error) => HookPhaseResult {
            phase: phase_name(phase).to_string(),
            command_path: Some(command_path.display().to_string()),
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
            succeeded: false,
        },
    }
}

pub(crate) fn truncate_capture(value: &str) -> String {
    if value.len() <= STDIO_CAPTURE_LIMIT {
        return value.to_string();
    }

    let prefix = value.chars().take(STDIO_CAPTURE_LIMIT).collect::<String>();
    format!("{prefix}[truncated]")
}

pub(crate) fn phase_name(phase: HookPhase) -> &'static str {
    match phase {
        HookPhase::AfterCreate => "after_create",
        HookPhase::BeforeRun => "before_run",
        HookPhase::AfterRun => "after_run",
    }
}
