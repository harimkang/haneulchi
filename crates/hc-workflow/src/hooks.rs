use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::bootstrap::HookPhaseResult;
use crate::contract::{HookDefinition, HookPhase};

const STDIO_CAPTURE_LIMIT: usize = 32 * 1024;

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
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = run_command_with_timeout(command, hook.timeout_sec);
    match output {
        Ok(CommandRun::Completed(output)) => HookPhaseResult {
            phase: phase_name(phase).to_string(),
            command_path: Some(command_path.display().to_string()),
            exit_code: output.status.code(),
            stdout: truncate_capture(&String::from_utf8_lossy(&output.stdout)),
            stderr: truncate_capture(&String::from_utf8_lossy(&output.stderr)),
            succeeded: output.status.success(),
        },
        Ok(CommandRun::TimedOut(output)) => HookPhaseResult {
            phase: phase_name(phase).to_string(),
            command_path: Some(command_path.display().to_string()),
            exit_code: output.status.code(),
            stdout: truncate_capture(&String::from_utf8_lossy(&output.stdout)),
            stderr: truncate_capture(&format!(
                "hook timed out after {}s{}{}",
                hook.timeout_sec,
                if output.stderr.is_empty() { "" } else { ": " },
                String::from_utf8_lossy(&output.stderr)
            )),
            succeeded: false,
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

enum CommandRun {
    Completed(std::process::Output),
    TimedOut(std::process::Output),
}

fn run_command_with_timeout(mut command: Command, timeout_sec: u64) -> Result<CommandRun, String> {
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let started_at = Instant::now();
    let timeout = Duration::from_secs(timeout_sec);

    loop {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return child
                .wait_with_output()
                .map(CommandRun::Completed)
                .map_err(|error| error.to_string());
        }

        if started_at.elapsed() >= timeout {
            child.kill().map_err(|error| error.to_string())?;
            return child
                .wait_with_output()
                .map(CommandRun::TimedOut)
                .map_err(|error| error.to_string());
        }

        thread::sleep(Duration::from_millis(10));
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
