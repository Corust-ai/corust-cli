use std::process::Stdio;

use agent_client_protocol::ClientSideConnection;
use tokio::process::{Child, Command};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::client::CliClient;
use crate::error::CliError;

/// Manages the connection to the corust-agent-acp server process.
pub struct Connection {
    pub agent: ClientSideConnection,
    child: Child,
}

impl Connection {
    /// Spawn the ACP server process and establish a client-side connection.
    pub async fn spawn(client: CliClient, server_bin: Option<&str>) -> Result<Self, CliError> {
        let bin = resolve_server_bin(server_bin)?;

        let mut child = Command::new(&bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| CliError::ServerSpawn(bin.clone(), e))?;

        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| CliError::ServerIo("failed to capture server stdin".into()))?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| CliError::ServerIo("failed to capture server stdout".into()))?;

        let outgoing = child_stdin.compat_write();
        let incoming = child_stdout.compat();

        let (conn, io_task) = ClientSideConnection::new(client, outgoing, incoming, |fut| {
            tokio::task::spawn_local(fut);
        });

        tokio::task::spawn_local(async move {
            if let Err(e) = io_task.await {
                tracing::error!("ACP I/O task error: {e}");
            }
        });

        Ok(Self { agent: conn, child })
    }

    /// Gracefully shut down the server process.
    pub async fn shutdown(mut self) {
        let _ = self.child.kill().await;
    }
}

/// Resolve the path to the corust-agent-acp binary.
fn resolve_server_bin(explicit: Option<&str>) -> Result<String, CliError> {
    // 1. Explicit override
    if let Some(bin) = explicit {
        return Ok(bin.to_string());
    }

    // 2. Environment variable
    if let Ok(bin) = std::env::var("CORUST_ACP_BIN") {
        return Ok(bin);
    }

    // 3. Same directory as current executable
    if let Ok(exe) = std::env::current_exe() {
        let sibling = exe.with_file_name("corust-agent-acp");
        if sibling.exists() {
            return Ok(sibling.to_string_lossy().into());
        }
    }

    // 4. Fall back to PATH
    Ok("corust-agent-acp".to_string())
}
