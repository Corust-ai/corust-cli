#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("failed to spawn server `{0}`: {1}")]
    ServerSpawn(String, std::io::Error),

    #[error("server I/O error: {0}")]
    ServerIo(String),

    #[error("ACP protocol error: {0}")]
    Protocol(#[from] agent_client_protocol::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
