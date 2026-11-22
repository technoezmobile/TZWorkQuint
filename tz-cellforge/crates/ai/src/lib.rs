//! Shared AI provider abstraction, credential security, tool orchestration, and editing proposals
//! for the TZWorkQuint Productivity Suite.

pub mod adapter;
pub mod client;
pub mod credentials;
pub mod gemini;
pub mod mock;
pub mod openai;
pub mod orchestrator;
pub mod proposal;
pub mod security;
pub mod tools;
pub mod types;

#[cfg(test)]
mod tests;

pub use adapter::{
    RangeCellData, WorkbookRangeAdapter, MAX_PROPOSAL_CELLS, MAX_READ_CELLS, MAX_SELECTION_CELLS,
};
pub use client::{create_client, AiError, LlmClient};
pub use credentials::{default_store_path, load_connections, save_connections, CredentialError};
pub use orchestrator::{AgentOrchestrator, AssistantMessage, ScopedContext};
pub use proposal::{CellDiff, EditProposal, ProposalStatus};
pub use security::{is_loopback, sanitize_log, validate_transport_url, wrap_untrusted_content, SecurityError};
pub use tools::get_spreadsheet_tools;
pub use types::{
    CancelHandle, ChatDelta, ChatMessage, ConnectionConfig, ConnectionTestResult, MessageRole,
    ModelInfo, ProviderType, ToolCall, ToolDefinition, ToolSupport,
};
