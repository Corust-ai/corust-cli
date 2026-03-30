use agent_client_protocol::{
    Client, ContentBlock, ReadTextFileRequest, ReadTextFileResponse, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, Result, SelectedPermissionOutcome,
    SessionNotification, SessionUpdate, WriteTextFileRequest, WriteTextFileResponse,
};

use crate::event::{Event, PermissionResponse};

/// ACP Client implementation.
///
/// Receives notifications/requests from the agent server and forwards them
/// as [`Event`]s for the UI layer to consume and render.
/// ACP types are passed through directly — no flattening.
pub struct CliClient {
    event_tx: futures::channel::mpsc::UnboundedSender<Event>,
}

impl CliClient {
    pub fn new(event_tx: futures::channel::mpsc::UnboundedSender<Event>) -> Self {
        Self { event_tx }
    }
}

#[async_trait::async_trait(?Send)]
impl Client for CliClient {
    async fn session_notification(&self, notification: SessionNotification) -> Result<()> {
        let event = match notification.update {
            SessionUpdate::AgentMessageChunk(chunk) => {
                if let ContentBlock::Text(text) = chunk.content {
                    Some(Event::AgentText(text.text))
                } else {
                    None
                }
            }
            SessionUpdate::AgentThoughtChunk(chunk) => {
                if let ContentBlock::Text(text) = chunk.content {
                    Some(Event::AgentThought(text.text))
                } else {
                    None
                }
            }
            SessionUpdate::ToolCall(tool_call) => Some(Event::ToolCallStarted(tool_call)),
            SessionUpdate::ToolCallUpdate(update) => Some(Event::ToolCallUpdated(update)),
            SessionUpdate::UsageUpdate(usage) => Some(Event::UsageUpdate(usage)),
            _ => None,
        };

        if let Some(event) = event {
            let _ = self.event_tx.unbounded_send(event);
        }
        Ok(())
    }

    async fn request_permission(
        &self,
        request: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse> {
        let (tx, rx) = futures::channel::oneshot::channel();

        let _ = self.event_tx.unbounded_send(Event::PermissionRequest {
            session_id: request.session_id.clone(),
            tool_call: request.tool_call.clone(),
            options: request.options.clone(),
            respond: tx,
        });

        match rx.await {
            Ok(PermissionResponse::Selected(idx)) => {
                if let Some(opt) = request.options.get(idx) {
                    Ok(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                            opt.option_id.clone(),
                        )),
                    ))
                } else {
                    Ok(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ))
                }
            }
            _ => Ok(RequestPermissionResponse::new(
                RequestPermissionOutcome::Cancelled,
            )),
        }
    }

    async fn read_text_file(&self, args: ReadTextFileRequest) -> Result<ReadTextFileResponse> {
        let content = std::fs::read_to_string(&args.path).unwrap_or_default();
        Ok(ReadTextFileResponse::new(content))
    }

    async fn write_text_file(&self, args: WriteTextFileRequest) -> Result<WriteTextFileResponse> {
        if let Some(parent) = args.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&args.path, &args.content)
            .map_err(|_| agent_client_protocol::Error::internal_error())?;
        Ok(WriteTextFileResponse::new())
    }
}
