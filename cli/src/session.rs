use std::path::PathBuf;

use agent_client_protocol::{
    Agent, ContentBlock, InitializeRequest, NewSessionRequest, PromptRequest, ProtocolVersion,
    SessionId, SessionModeState, StopReason, TextContent,
};

use crate::connection::Connection;
use crate::error::CliError;

#[allow(dead_code)] // Fields used by TUI (not yet implemented)
/// Metadata captured during session initialization.
pub struct SessionInfo {
    pub session_id: SessionId,
    pub agent_name: Option<String>,
    pub modes: Option<SessionModeState>,
}

/// Manages the lifecycle of an ACP session.
pub struct Session {
    session_id: SessionId,
}

impl Session {
    /// Initialize the ACP connection and create a new session.
    /// Returns both the Session handle and metadata for the UI.
    pub async fn start(conn: &Connection, cwd: PathBuf) -> Result<(Self, SessionInfo), CliError> {
        let init_response = conn
            .agent
            .initialize(InitializeRequest::new(ProtocolVersion::V1))
            .await?;

        tracing::debug!("initialized: {:?}", init_response);

        let agent_name = init_response
            .agent_info
            .as_ref()
            .map(|info| info.name.clone());

        // Authenticate if required.
        if !init_response.auth_methods.is_empty() {
            let method = &init_response.auth_methods[0];
            conn.agent
                .authenticate(agent_client_protocol::AuthenticateRequest::new(
                    method.id().clone(),
                ))
                .await?;
        }

        let session_response = conn.agent.new_session(NewSessionRequest::new(cwd)).await?;

        let info = SessionInfo {
            session_id: session_response.session_id.clone(),
            agent_name,
            modes: session_response.modes.clone(),
        };

        let session = Self {
            session_id: session_response.session_id,
        };

        Ok((session, info))
    }

    /// Send a user prompt and wait for the turn to complete.
    pub async fn prompt(&self, conn: &Connection, text: &str) -> Result<StopReason, CliError> {
        let request = PromptRequest::new(
            self.session_id.clone(),
            vec![ContentBlock::Text(TextContent::new(text))],
        );

        let response = conn.agent.prompt(request).await?;
        Ok(response.stop_reason)
    }

    #[allow(dead_code)] // Used by TUI (not yet implemented)
    pub fn id(&self) -> &SessionId {
        &self.session_id
    }
}
