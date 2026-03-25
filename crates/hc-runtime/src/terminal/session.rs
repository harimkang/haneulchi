use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, native_pty_system};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::terminal::geometry::TerminalGeometry;
use crate::terminal::shell_markers::filter_shell_markers;

#[derive(Debug, Error)]
pub enum TerminalSessionError {
    #[error("pty operation failed: {0}")]
    Pty(#[from] anyhow::Error),
    #[error("io operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("session did not exit within {0:?}")]
    WaitTimedOut(Duration),
    #[error("reader thread panicked")]
    ReaderThreadPanicked,
    #[error("session not found: {0}")]
    SessionNotFound(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TerminalLaunchConfig {
    pub program: String,
    pub args: Vec<String>,
    pub current_directory: Option<PathBuf>,
    /// Runtime-only env vars injected at launch. Excluded from serialization —
    /// secrets must be re-injected from Keychain when restoring.
    #[serde(skip)]
    pub environment: BTreeMap<String, String>,
}

impl TerminalLaunchConfig {
    pub fn shell(current_directory: Option<PathBuf>) -> Self {
        Self {
            program: "/bin/zsh".to_string(),
            args: default_shell_bootstrap_args("/bin/zsh"),
            current_directory,
            environment: BTreeMap::new(),
        }
    }

    pub fn command<S, I, A>(program: S, args: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = A>,
        A: Into<String>,
    {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            current_directory: None,
            environment: BTreeMap::new(),
        }
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    fn to_command_builder(&self) -> CommandBuilder {
        let mut builder = CommandBuilder::new(&self.program);
        builder.args(self.args.iter().map(String::as_str));

        if let Some(current_directory) = &self.current_directory {
            builder.cwd(current_directory);
        }

        for (key, value) in &self.environment {
            builder.env(key, value);
        }

        builder
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShellIntegrationMetadata {
    pub current_directory: Option<String>,
    pub last_command: Option<String>,
    pub last_exit_code: Option<i32>,
    pub branch: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TerminalRestorePoint {
    pub launch: TerminalLaunchConfig,
    pub geometry: TerminalGeometry,
}

impl TerminalRestorePoint {
    pub fn new(launch: TerminalLaunchConfig, geometry: TerminalGeometry) -> Self {
        Self { launch, geometry }
    }
}

pub struct TerminalSession {
    launch: TerminalLaunchConfig,
    geometry: TerminalGeometry,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output: Arc<Mutex<Vec<u8>>>,
    shell_metadata: Arc<Mutex<ShellIntegrationMetadata>>,
    marker_remainder: Arc<Mutex<Vec<u8>>>,
    reader_thread: Option<JoinHandle<io::Result<()>>>,
    exit_status: Option<ExitStatus>,
}

impl TerminalSession {
    pub fn spawn(
        launch: TerminalLaunchConfig,
        geometry: TerminalGeometry,
    ) -> Result<Self, TerminalSessionError> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(geometry.to_pty_size())?;
        let portable_pty::PtyPair { master, slave } = pair;
        let child = slave.spawn_command(launch.to_command_builder())?;
        let reader = master.try_clone_reader()?;
        let writer = master.take_writer()?;
        let output = Arc::new(Mutex::new(Vec::new()));
        let shell_metadata = Arc::new(Mutex::new(ShellIntegrationMetadata::default()));
        let marker_remainder = Arc::new(Mutex::new(Vec::new()));
        let reader_thread = Some(spawn_reader_thread(reader, Arc::clone(&output)));

        Ok(Self {
            launch,
            geometry,
            master,
            writer,
            child,
            output,
            shell_metadata,
            marker_remainder,
            reader_thread,
            exit_status: None,
        })
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<(), TerminalSessionError> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn drain_output(&self) -> Result<Vec<u8>, TerminalSessionError> {
        let mut output = self
            .output
            .lock()
            .map_err(|_| io::Error::other("terminal output buffer lock poisoned"))?;
        let drained = std::mem::take(&mut *output);
        drop(output);

        let mut metadata = self
            .shell_metadata
            .lock()
            .map_err(|_| io::Error::other("terminal metadata lock poisoned"))?;
        let mut remainder = self
            .marker_remainder
            .lock()
            .map_err(|_| io::Error::other("terminal marker remainder lock poisoned"))?;

        Ok(filter_shell_markers(drained, &mut metadata, &mut remainder))
    }

    pub fn wait_and_drain(&mut self, timeout: Duration) -> Result<Vec<u8>, TerminalSessionError> {
        let _ = self.wait_for_exit(timeout)?;
        self.drain_output()
    }

    pub fn resize(&mut self, geometry: TerminalGeometry) -> Result<(), TerminalSessionError> {
        self.master.resize(geometry.to_pty_size())?;
        self.geometry = geometry;
        Ok(())
    }

    pub fn terminate(&mut self) -> Result<(), TerminalSessionError> {
        if self.exit_status.is_none() {
            self.child.kill()?;
            self.exit_status = Some(self.child.wait()?);
        }

        self.finish_reader_thread()?;
        Ok(())
    }

    pub fn geometry(&self) -> TerminalGeometry {
        self.geometry
    }

    pub fn exit_status(&self) -> Option<&ExitStatus> {
        self.exit_status.as_ref()
    }

    pub fn launch(&self) -> &TerminalLaunchConfig {
        &self.launch
    }

    pub fn restore_point(&self) -> TerminalRestorePoint {
        TerminalRestorePoint::new(self.launch.clone(), self.geometry)
    }

    pub fn shell_metadata(&self) -> Result<ShellIntegrationMetadata, TerminalSessionError> {
        let metadata = self
            .shell_metadata
            .lock()
            .map_err(|_| io::Error::other("terminal metadata lock poisoned"))?;
        Ok(metadata.clone())
    }

    fn wait_for_exit(&mut self, timeout: Duration) -> Result<&ExitStatus, TerminalSessionError> {
        if self.exit_status.is_some() {
            return Ok(self.exit_status.as_ref().expect("checked is_some"));
        }

        let deadline = Instant::now() + timeout;

        while Instant::now() <= deadline {
            if let Some(status) = self.child.try_wait()? {
                self.exit_status = Some(status);
                self.finish_reader_thread()?;
                return Ok(self.exit_status.as_ref().expect("set before return"));
            }

            thread::sleep(Duration::from_millis(10));
        }

        Err(TerminalSessionError::WaitTimedOut(timeout))
    }

    fn finish_reader_thread(&mut self) -> Result<(), TerminalSessionError> {
        let Some(reader_thread) = self.reader_thread.take() else {
            return Ok(());
        };

        match reader_thread.join() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(error.into()),
            Err(_) => Err(TerminalSessionError::ReaderThreadPanicked),
        }
    }
}

fn default_shell_bootstrap_args(program: &str) -> Vec<String> {
    let script_name = if program.contains("bash") {
        "haneulchi.bash"
    } else {
        "haneulchi.zsh"
    };

    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root")
        .join("config/shell-integration")
        .join(script_name);

    vec![
        "-lc".to_string(),
        format!("source '{}'; exec {} -i", script_path.display(), program),
    ]
}

fn spawn_reader_thread(
    mut reader: Box<dyn Read + Send>,
    output: Arc<Mutex<Vec<u8>>>,
) -> JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(read) => {
                    let mut collected = output
                        .lock()
                        .map_err(|_| io::Error::other("terminal output buffer lock poisoned"))?;
                    collected.extend_from_slice(&buffer[..read]);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
    })
}
