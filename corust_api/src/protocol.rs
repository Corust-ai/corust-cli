//! Protocol v1: the minimal submit + event-stream contract.
//!
//! # Scope
//! This module defines the smallest protocol surface needed to drive an agent turn:
//! - Input is submitted via [`Op`].
//! - Progress is observed by repeatedly polling [`Event`] via `ConversationImpl::next_event()`.
//! - Side effects that require user consent are modeled as approval requests in the event stream,
//!   and are resumed by submitting the corresponding approval [`Op`].
//!
//! # Stability
//! v1 is intended to be **stable**. Once frozen, changes must be deliberate and reviewed, because
//! adapters and CLI consumers depend on its serialized form.
//!
//! # Wire schema policy
//! v1 is designed to be easy to consume across process boundaries:
//! - Use explicit `type` tags and stable `snake_case` naming via serde attributes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// =============================================================================
// Submission types
// =============================================================================

/// Submission operations for the protocol v1 minimal loop.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Op {
    /// Abort current task. The server is expected to emit `EventMsg::TurnAborted`.
    Interrupt,

    /// User input to start/continue a turn.
    UserInput {
        /// User input items.
        items: Vec<UserInput>,
    },

    /// Approval response for a previously emitted `ExecApprovalRequest`.
    ExecApproval {
        /// The approval request identifier to respond to.
        id: String,
        /// User decision.
        decision: ReviewDecision,
    },

    /// Approval response for a previously emitted `ApplyPatchApprovalRequest`.
    PatchApproval {
        /// The approval request identifier to respond to.
        id: String,
        /// User decision.
        decision: ReviewDecision,
    },

    /// Request the list of available skills for one or more working directories.
    ListSkills {
        /// Absolute working directories to scan for repo skills.
        #[serde(default)]
        cwds: Vec<PathBuf>,
        /// Reserved for compatibility; currently ignored.
        #[serde(default)]
        force_reload: bool,
    },

    /// Override turn context for subsequent turns.
    ///
    /// Model/provider changes take effect on the next `UserInput` submission.
    OverrideTurnContext {
        /// Model identifier override (e.g., "claude-3-5-sonnet-20241022").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model_id: Option<String>,
        /// Provider override (e.g., "anthropic", "openai").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        provider: Option<String>,
    },
}

/// User input items.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserInput {
    Text {
        text: String,
    },
    Image {
        image_url: String,
    },
    /// Explicitly selected skill by name and absolute `SKILL.md` path.
    Skill {
        name: String,
        /// Absolute path to the selected `SKILL.md`.
        path: PathBuf,
    },
}

/// User decision in response to an approval request.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    ApprovedOnce,
    ApprovedForSession,
    Denied,
    Abort,
}

// =============================================================================
// Event types
// =============================================================================

/// Event queue entry. Each event is correlated with a submission `id`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub msg: EventMsg,
}

/// Response events from the agent, protocol v1 minimal loop.
///
/// Contract:
/// - For simpler codegen and adapters, "not available" values generally use empty strings or
///   empty collections rather than `Option<T>`.
/// - Some fields use `Option<T>` where absence has distinct semantics (e.g., `process_id`).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventMsg {
    Error(ErrorEvent),
    Warning(WarningEvent),

    TaskStarted(TaskStartedEvent),
    TaskComplete(TaskCompleteEvent),

    AgentMessage(AgentMessageEvent),
    UserMessage(UserMessageEvent),
    AgentMessageDelta(AgentMessageDeltaEvent),

    AgentReasoning(AgentReasoningEvent),
    AgentReasoningDelta(AgentReasoningDeltaEvent),

    ExecApprovalRequest(ExecApprovalRequestEvent),
    ExecCommandBegin(ExecCommandBeginEvent),
    ExecCommandOutputDelta(ExecCommandOutputDeltaEvent),
    TerminalInteraction(TerminalInteractionEvent),
    ExecCommandEnd(ExecCommandEndEvent),

    ApplyPatchApprovalRequest(ApplyPatchApprovalRequestEvent),
    PatchApplyBegin(PatchApplyBeginEvent),
    PatchApplyEnd(PatchApplyEndEvent),

    ReadFileBegin(ReadFileBeginEvent),
    ReadFileEnd(ReadFileEndEvent),

    WebSearchBegin(WebSearchBeginEvent),
    WebSearchEnd(WebSearchEndEvent),

    CritiqueBegin(CritiqueBeginEvent),
    CritiqueEnd(CritiqueEndEvent),

    /// Response payload for `Op::ListSkills`.
    ListSkillsResponse(ListSkillsResponseEvent),

    TurnAborted(TurnAbortedEvent),
}

// =============================================================================
// Event payloads
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ErrorEvent {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct WarningEvent {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskStartedEvent {}

/// Aggregated token usage for a completed task.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskCompleteEvent {
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AgentMessageEvent {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct UserMessageEvent {
    pub message: String,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SkillMetadata {
    /// Skill display name from frontmatter.
    pub name: String,
    /// Skill description from frontmatter.
    pub description: String,
    /// Absolute path to the discovered `SKILL.md`.
    pub path: PathBuf,
    /// Whether the skill is currently enabled for model-driven use.
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SkillErrorInfo {
    /// Absolute path that failed during skill discovery or loading.
    pub path: PathBuf,
    /// Human-readable discovery/load failure.
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SkillsListEntry {
    /// Absolute working directory used for this discovery pass.
    pub cwd: PathBuf,
    /// Skills discovered for this working directory.
    pub skills: Vec<SkillMetadata>,
    /// Load errors encountered while scanning this working directory.
    pub errors: Vec<SkillErrorInfo>,
}

/// Response payload emitted for `Op::ListSkills`; contains one entry per requested cwd.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ListSkillsResponseEvent {
    pub skills: Vec<SkillsListEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AgentMessageDeltaEvent {
    pub delta: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AgentReasoningEvent {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AgentReasoningDeltaEvent {
    pub delta: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExecApprovalRequestEvent {
    pub call_id: String,
    pub turn_id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecCommandSource {
    Agent,
    UserShell,
    UnifiedExecStartup,
    UnifiedExecInteraction,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExecCommandBeginEvent {
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<String>,
    pub turn_id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub source: ExecCommandSource,
    pub interaction_input: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecOutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExecCommandOutputDeltaEvent {
    pub call_id: String,
    pub stream: ExecOutputStream,
    /// Raw output bytes (base64-encoded on the wire).
    #[serde(with = "base64_bytes")]
    pub chunk: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TerminalInteractionEvent {
    pub call_id: String,
    pub process_id: String,
    pub stdin: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExecCommandEndEvent {
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<String>,
    pub turn_id: String,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub source: ExecCommandSource,
    pub interaction_input: String,
    pub stdout: String,
    pub stderr: String,
    pub aggregated_output: String,
    pub exit_code: i32,
    pub duration: Duration,
    pub formatted_output: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ApplyPatchApprovalRequestEvent {
    pub call_id: String,
    pub turn_id: String,
    pub changes: HashMap<PathBuf, FileChange>,
    pub reason: String,
    pub grant_root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileChange {
    Add { content: String },
    Delete { content: String },
    Update {
        unified_diff: String,
        old_content: String,
        new_content: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PatchApplyBeginEvent {
    pub call_id: String,
    pub turn_id: String,
    pub auto_approved: bool,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PatchApplyEndEvent {
    pub call_id: String,
    pub turn_id: String,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    /// The changes that were applied.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub changes: HashMap<PathBuf, FileChange>,
}

/// Maximum content size for ReadFileEndEvent (100KB).
pub const READ_FILE_CONTENT_LIMIT: usize = 100_000;

/// Emitted when a file read operation starts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReadFileBeginEvent {
    /// Tool call ID for correlation.
    pub call_id: String,
    /// Path to the file being read.
    pub path: PathBuf,
}

/// Emitted when a file read operation completes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReadFileEndEvent {
    /// Tool call ID for correlation.
    pub call_id: String,
    /// Path to the file that was read.
    pub path: PathBuf,
    /// Number of lines read.
    pub num_lines: u32,
    /// File content (truncated if too large).
    pub content: String,
    /// Whether content was truncated due to size limit.
    pub truncated: bool,
}

/// Emitted when a web search operation starts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct WebSearchBeginEvent {
    /// Tool call ID for correlation.
    pub call_id: String,
    /// Tool name (e.g., "docs_api", "crates_api").
    pub provider: String,
    /// Human-readable query description.
    pub query: String,
}

/// Emitted when a web search operation completes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct WebSearchEndEvent {
    /// Tool call ID for correlation.
    pub call_id: String,
    /// Full tool result (JSON).
    pub result: serde_json::Value,
    /// Whether the search succeeded.
    pub success: bool,
}

/// Emitted when critique of a tool result starts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CritiqueBeginEvent {
    /// Tool call ID for correlation.
    pub call_id: String,
    /// Name of the tool being critiqued.
    pub tool_name: String,
    /// The intention extracted from tool args.
    pub intention: String,
}

/// Emitted when critique of a tool result completes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CritiqueEndEvent {
    /// Tool call ID for correlation.
    pub call_id: String,
    /// Feedback from the critique model.
    pub feedback: String,
    /// Whether the critique succeeded (false if critique model failed).
    pub success: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TurnAbortReason {
    Interrupted,
    Replaced,
    ReviewEnded,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TurnAbortedEvent {
    pub reason: TurnAbortReason,
}

// =============================================================================
// base64 serde helper (replaces serde_with dependency)
// =============================================================================

mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    use serde::de;

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        use base64::Engine;
        let s = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(de::Error::custom)
    }
}
