//! The async trait that the core engine must implement.
//!
//! The CLI only depends on this trait — it never touches the concrete `Conversation` struct
//! from the private core crate. This allows the CLI to be compiled and tested independently
//! (e.g., with a mock implementation).
//!
//! # Protocol contract (v1)
//!
//! - `submit` accepts a protocol [`Op`](crate::protocol::Op) and returns a submission id
//!   used to correlate subsequent [`Event`](crate::protocol::Event)s.
//! - `next_event` returns the next queued `Event` in FIFO order and awaits until one is available.
//! - Turn completion is signaled via `EventMsg::TaskComplete` or `EventMsg::TurnAborted`.
//! - `snapshot_history` captures the current chat history as opaque JSON for session persistence.
//!   The CLI stores and restores this blob without inspecting its contents.

use crate::error::ApiError;
use crate::protocol::{Event, Op};

/// The core conversation interface.
///
/// The CLI programs against this trait. The private `corust_core` crate provides the
/// real implementation; tests and external contributors can use a mock.
#[async_trait::async_trait]
pub trait ConversationImpl: Send + Sync {
    /// Submit an operation (user input, approval, interrupt, etc.).
    ///
    /// Returns a submission ID that correlates with subsequent events.
    async fn submit(&self, op: Op) -> Result<String, ApiError>;

    /// Await the next event from the agent.
    ///
    /// This call blocks until an event is available or the conversation ends.
    async fn next_event(&self) -> Result<Event, ApiError>;

    /// Capture the current chat history as opaque JSON.
    ///
    /// The returned value can be stored to disk and later passed back to
    /// a factory method to resume the conversation. The CLI must treat this
    /// as an opaque blob — its internal structure is owned by the core.
    fn snapshot_history(&self) -> serde_json::Value;
}
