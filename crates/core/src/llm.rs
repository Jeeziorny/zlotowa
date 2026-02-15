use crate::models::ParsedExpense;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("LLM not configured")]
    NotConfigured,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Request failed: {0}")]
    RequestFailed(String),
}

/// Configuration for an LLM provider.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
}

/// An LLM provider that can classify expenses.
pub trait LlmProvider: Send + Sync {
    /// Provider name (e.g. "openai", "anthropic", "ollama").
    fn name(&self) -> &str;

    /// Validate that the API key / connection works.
    fn validate(&self, config: &LlmConfig) -> Result<(), LlmError>;

    /// Classify a batch of expenses, returning suggested category for each.
    fn classify_batch(
        &self,
        expenses: &[ParsedExpense],
        existing_categories: &[String],
        config: &LlmConfig,
    ) -> Result<Vec<Option<String>>, LlmError>;
}
