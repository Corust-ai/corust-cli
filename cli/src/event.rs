use agent_client_protocol::{
    PermissionOption, SessionId, SessionModeState, ToolCall, ToolCallUpdate,
};
use futures::channel::oneshot;

#[allow(dead_code)] // Variants/fields used by TUI (not yet implemented)
/// Internal events flowing from the ACP client to the UI layer.
///
/// These carry ACP types directly so the TUI/REPL can render
/// structured data (diffs, tool content, locations) without loss.
pub enum Event {
    // --- Agent output ---
    /// Streamed text chunk from the agent (markdown).
    AgentText(String),

    /// Streamed reasoning/thought chunk from the agent.
    AgentThought(String),

    // --- Tool calls (structured, not flattened) ---
    /// A new tool call has been initiated. Contains the full ACP ToolCall
    /// with id, title, kind, status, content (diffs/text/terminal), locations.
    ToolCallStarted(ToolCall),

    /// An existing tool call was updated (status change, new content, etc.).
    ToolCallUpdated(ToolCallUpdate),

    // --- Permission (blocking) ---
    /// The agent is requesting permission from the user.
    /// The UI must present options and send a response back through the oneshot.
    PermissionRequest {
        session_id: SessionId,
        tool_call: ToolCallUpdate,
        options: Vec<PermissionOption>,
        respond: oneshot::Sender<PermissionResponse>,
    },

    // --- System / meta ---
    /// Session successfully started. Contains metadata for the status bar.
    SessionStarted {
        session_id: SessionId,
        agent_name: Option<String>,
        modes: Option<SessionModeState>,
    },

    /// An ACP protocol error occurred (API error, rate limit, etc.).
    Error(String),
}

/// The user's response to a permission request.
pub enum PermissionResponse {
    /// User selected an option by index.
    Selected(usize),
    /// User cancelled.
    Cancelled,
}
