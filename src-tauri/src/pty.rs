use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    sync::atomic::{AtomicU64, Ordering},
    sync::mpsc,
    thread,
    time::Duration,
};

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyCommandCapture {
    pub output: String,
    pub exit_code: u32,
    pub exit_success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalPtySession {
    pub id: String,
    pub title: String,
    pub command: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalPtyOutputEvent {
    pub session_id: String,
    pub seq: u64,
    pub chunk: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalPtySnapshot {
    pub total: usize,
    pub sessions: Vec<TerminalPtySession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnTerminalPtyRequest {
    pub title: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTerminalPtyRequest {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

struct LivePtySession {
    snapshot: TerminalPtySession,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

#[derive(Default)]
pub struct TerminalPtyManager {
    sessions: BTreeMap<String, LivePtySession>,
}

impl TerminalPtyManager {
    pub fn spawn_session_with_output_sink<F>(
        &mut self,
        title: &str,
        command: &str,
        args: &[&str],
        cols: u16,
        rows: u16,
        output_sink: F,
    ) -> Result<TerminalPtySession, String>
    where
        F: Fn(TerminalPtyOutputEvent) + Send + 'static,
    {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| error.to_string())?;

        let mut command_builder = CommandBuilder::new(command);
        for arg in args {
            command_builder.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(command_builder)
            .map_err(|error| error.to_string())?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| error.to_string())?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| error.to_string())?;

        let snapshot = TerminalPtySession {
            id: next_session_id(),
            title: title.to_string(),
            command: command.to_string(),
            cols,
            rows,
        };
        let session_id = snapshot.id.clone();

        thread::spawn(move || {
            let mut seq = 1_u64;
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        output_sink(TerminalPtyOutputEvent {
                            session_id: session_id.clone(),
                            seq,
                            chunk: String::from_utf8_lossy(&buffer[..read]).to_string(),
                        });
                        seq += 1;
                    }
                    Err(_) => break,
                }
            }
        });

        self.sessions.insert(
            snapshot.id.clone(),
            LivePtySession {
                snapshot: snapshot.clone(),
                master: pair.master,
                writer,
                child,
            },
        );

        Ok(snapshot)
    }

    pub fn write_session_input(&mut self, id: &str, input: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| format!("terminal session {id} was not found"))?;

        session
            .writer
            .write_all(input.as_bytes())
            .and_then(|_| session.writer.flush())
            .map_err(|error| error.to_string())
    }

    pub fn spawn_from_request_with_output_sink<F>(
        &mut self,
        request: SpawnTerminalPtyRequest,
        output_sink: F,
    ) -> Result<TerminalPtySession, String>
    where
        F: Fn(TerminalPtyOutputEvent) + Send + 'static,
    {
        let args = request.args.iter().map(String::as_str).collect::<Vec<_>>();
        self.spawn_session_with_output_sink(
            &request.title,
            &request.command,
            &args,
            request.cols,
            request.rows,
            output_sink,
        )
    }

    pub fn snapshot(&self) -> TerminalPtySnapshot {
        TerminalPtySnapshot {
            total: self.sessions.len(),
            sessions: self
                .sessions
                .values()
                .map(|session| session.snapshot.clone())
                .collect(),
        }
    }

    pub fn resize_session(&mut self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(id)
            .ok_or_else(|| format!("terminal session {id} was not found"))?;

        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| error.to_string())?;
        session.snapshot.cols = cols;
        session.snapshot.rows = rows;

        Ok(())
    }

    pub fn close_session(&mut self, id: &str) -> Result<(), String> {
        let mut session = self
            .sessions
            .remove(id)
            .ok_or_else(|| format!("terminal session {id} was not found"))?;

        session.child.kill().map_err(|error| error.to_string())
    }
}

pub fn capture_command_once(
    command: &str,
    args: &[&str],
    cols: u16,
    rows: u16,
) -> Result<PtyCommandCapture, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| error.to_string())?;

    let mut command_builder = CommandBuilder::new(command);
    for arg in args {
        command_builder.arg(arg);
    }

    let mut child = pair
        .slave
        .spawn_command(command_builder)
        .map_err(|error| error.to_string())?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| error.to_string())?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut output = String::new();
        let result = reader.read_to_string(&mut output).map(|_| output);
        let _ = tx.send(result);
    });

    let writer = pair
        .master
        .take_writer()
        .map_err(|error| error.to_string())?;
    drop(writer);

    if cfg!(target_os = "macos") {
        thread::sleep(Duration::from_millis(20));
    }

    let status = child.wait().map_err(|error| error.to_string())?;
    drop(pair.master);

    let output = rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

    Ok(PtyCommandCapture {
        output,
        exit_code: status.exit_code(),
        exit_success: status.success(),
    })
}

pub fn capture_from_request(
    request: CaptureTerminalPtyRequest,
) -> Result<PtyCommandCapture, String> {
    let args = request.args.iter().map(String::as_str).collect::<Vec<_>>();
    capture_command_once(&request.command, &args, request.cols, request.rows)
}

fn next_session_id() -> String {
    let value = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    format!("pty_{value}")
}

#[cfg(test)]
mod tests {
    use super::{capture_command_once, TerminalPtyManager};
    use std::{sync::mpsc, time::Duration};

    #[test]
    fn captures_output_from_a_real_pty_command() {
        let capture = capture_command_once("sh", &["-lc", "printf haneulchi"], 80, 24)
            .expect("pty command should run");

        assert!(capture.exit_success);
        assert!(capture.output.contains("haneulchi"));
    }

    #[test]
    fn tracks_spawned_sessions_in_the_manager_snapshot() {
        let mut manager = TerminalPtyManager::default();
        let session = manager
            .spawn_session_with_output_sink(
                "test-shell",
                "sh",
                &["-lc", "printf ready"],
                100,
                30,
                |_| {},
            )
            .expect("session should spawn");

        let snapshot = manager.snapshot();

        assert_eq!(snapshot.total, 1);
        assert_eq!(snapshot.sessions[0].id, session.id);
        assert_eq!(snapshot.sessions[0].title, "test-shell");
        assert_eq!(snapshot.sessions[0].cols, 100);
        assert_eq!(snapshot.sessions[0].rows, 30);
    }

    #[test]
    fn emits_ordered_output_events_from_spawned_session() {
        let mut manager = TerminalPtyManager::default();
        let (tx, rx) = mpsc::channel();
        let session = manager
            .spawn_session_with_output_sink(
                "emit-test",
                "sh",
                &["-lc", "printf streamed-output"],
                80,
                24,
                move |event| {
                    let _ = tx.send(event);
                },
            )
            .expect("session should spawn");

        let event = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("output event should be emitted");

        assert_eq!(event.session_id, session.id);
        assert_eq!(event.seq, 1);
        assert!(event.chunk.contains("streamed-output"));
    }

    #[test]
    fn returns_error_when_writing_to_missing_session() {
        let mut manager = TerminalPtyManager::default();

        let error = manager
            .write_session_input("pty_missing", "hello")
            .expect_err("missing session should fail");

        assert!(error.contains("pty_missing"));
    }
}
