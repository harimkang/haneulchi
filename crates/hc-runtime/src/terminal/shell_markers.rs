use crate::terminal::session::ShellIntegrationMetadata;

const MARKER_PREFIX: &[u8] = b"\x1fHC|";

pub fn filter_shell_markers(
    drained: Vec<u8>,
    metadata: &mut ShellIntegrationMetadata,
    remainder: &mut Vec<u8>,
) -> Vec<u8> {
    let mut combined = Vec::with_capacity(remainder.len() + drained.len());
    combined.extend_from_slice(remainder);
    combined.extend_from_slice(&drained);
    remainder.clear();

    let mut visible = Vec::new();
    let mut start = 0usize;

    for index in 0..combined.len() {
        if combined[index] != b'\n' {
            continue;
        }

        let line = &combined[start..=index];
        if !consume_marker(line, metadata) {
            visible.extend_from_slice(line);
        }
        start = index + 1;
    }

    if start < combined.len() {
        remainder.extend_from_slice(&combined[start..]);
    }

    visible
}

fn consume_marker(line: &[u8], metadata: &mut ShellIntegrationMetadata) -> bool {
    if !line.starts_with(MARKER_PREFIX) {
        return false;
    }

    let payload = String::from_utf8_lossy(&line[MARKER_PREFIX.len()..])
        .trim_end_matches(['\r', '\n'])
        .to_string();
    let mut parts = payload.splitn(2, '|');
    let kind = parts.next().unwrap_or_default();
    let value = parts.next().unwrap_or_default();

    match kind {
        "cwd" => metadata.current_directory = Some(value.to_string()),
        "command" => metadata.last_command = Some(value.to_string()),
        "exit" => metadata.last_exit_code = value.parse::<i32>().ok(),
        "branch" => metadata.branch = Some(value.to_string()),
        _ => {}
    }

    true
}
