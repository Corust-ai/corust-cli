//! Public API types and traits for Corust.
//!
//! This crate defines the **interface contract** between the CLI (public) and
//! the core engine (private). It contains only data types and trait definitions —
//! no implementation logic.
//!
//! # Modules
//!
//! - [`protocol`] — Wire types for the submit/event-stream protocol (v1).
//! - [`conversation`] — The async trait that the core must implement.
//! - [`error`] — Error types shared across the boundary.

pub mod conversation;
pub mod error;
pub mod protocol;

pub use conversation::ConversationImpl;
pub use error::ApiError;
pub use protocol::{Event, EventMsg, Op, ReviewDecision, UserInput};
