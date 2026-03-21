use std::collections::BTreeMap;

use serde::Serialize;

use crate::terminal::geometry::TerminalGeometry;
use crate::terminal::session::{
    TerminalLaunchConfig, TerminalRestorePoint, TerminalSession, TerminalSessionError,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalSessionSnapshot {
    pub session_id: String,
    pub launch: TerminalLaunchConfig,
    pub geometry: TerminalGeometry,
    pub running: bool,
    pub exit_code: Option<u32>,
}

#[derive(Default)]
pub struct TerminalRuntime {
    next_id: u64,
    sessions: BTreeMap<String, TerminalSession>,
}

impl TerminalRuntime {
    pub fn spawn(
        &mut self,
        launch: TerminalLaunchConfig,
        geometry: TerminalGeometry,
    ) -> Result<String, TerminalSessionError> {
        let session = TerminalSession::spawn(launch, geometry)?;
        self.next_id += 1;
        let session_id = format!("session-{:04}", self.next_id);
        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub fn restore(
        &mut self,
        restore: TerminalRestorePoint,
    ) -> Result<&mut TerminalSession, TerminalSessionError> {
        let session_id = self.spawn(restore.launch, restore.geometry)?;
        self.session_mut(&session_id)
    }

    pub fn drain_output(&mut self, session_id: &str) -> Result<Vec<u8>, TerminalSessionError> {
        self.session_mut(session_id)?.drain_output()
    }

    pub fn write_input(
        &mut self,
        session_id: &str,
        data: &[u8],
    ) -> Result<(), TerminalSessionError> {
        self.session_mut(session_id)?.write_input(data)
    }

    pub fn resize(
        &mut self,
        session_id: &str,
        geometry: TerminalGeometry,
    ) -> Result<(), TerminalSessionError> {
        self.session_mut(session_id)?.resize(geometry)
    }

    pub fn terminate(&mut self, session_id: &str) -> Result<(), TerminalSessionError> {
        self.session_mut(session_id)?.terminate()
    }

    pub fn snapshot(
        &self,
        session_id: &str,
    ) -> Result<TerminalSessionSnapshot, TerminalSessionError> {
        let session = self.session(session_id)?;
        let exit_code = session.exit_status().map(|status| status.exit_code());

        Ok(TerminalSessionSnapshot {
            session_id: session_id.to_string(),
            launch: session.launch().clone(),
            geometry: session.geometry(),
            running: exit_code.is_none(),
            exit_code,
        })
    }

    pub fn session(&self, session_id: &str) -> Result<&TerminalSession, TerminalSessionError> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| TerminalSessionError::SessionNotFound(session_id.to_string()))
    }

    pub fn session_mut(
        &mut self,
        session_id: &str,
    ) -> Result<&mut TerminalSession, TerminalSessionError> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| TerminalSessionError::SessionNotFound(session_id.to_string()))
    }
}
