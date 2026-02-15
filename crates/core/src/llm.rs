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

/// Build the classification prompt for all providers.
fn build_classification_prompt(expenses: &[ParsedExpense], existing_categories: &[String]) -> String {
    let cats = if existing_categories.is_empty() {
        "None yet — suggest appropriate categories.".to_string()
    } else {
        existing_categories.join(", ")
    };

    let mut expense_list = String::new();
    for (i, e) in expenses.iter().enumerate() {
        expense_list.push_str(&format!("{}. {}\n", i + 1, e.title));
    }

    format!(
        "You are an expense classifier. Given these expense titles and a list of known categories, \
         assign each expense to the most appropriate category. If none fit, suggest a new short category name.\n\n\
         Known categories: [{}]\n\n\
         Expenses:\n{}\n\
         Respond with ONLY a JSON array of category strings, one per expense. Example: [\"Groceries\", \"Transport\"]\n\
         Do not include any other text, explanation, or markdown formatting.",
        cats, expense_list
    )
}

/// Parse a JSON array of category strings from an LLM response.
/// Handles responses that may include markdown code fences or extra text.
fn parse_classification_response(response: &str, expected_count: usize) -> Result<Vec<Option<String>>, LlmError> {
    // Strip markdown code fences if present
    let cleaned = response.trim();
    let cleaned = if cleaned.starts_with("```") {
        let inner = cleaned
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        inner
    } else {
        cleaned
    };

    // Find the JSON array in the response
    let start = cleaned.find('[').ok_or_else(|| {
        LlmError::RequestFailed(format!("No JSON array found in response: {}", &response[..response.len().min(200)]))
    })?;
    let end = cleaned.rfind(']').ok_or_else(|| {
        LlmError::RequestFailed("No closing bracket found in response".to_string())
    })?;

    let json_str = &cleaned[start..=end];
    let parsed: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|e| LlmError::RequestFailed(format!("Failed to parse JSON: {}", e)))?;

    if parsed.len() != expected_count {
        return Err(LlmError::RequestFailed(format!(
            "Expected {} categories, got {}",
            expected_count,
            parsed.len()
        )));
    }

    Ok(parsed
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) if !s.is_empty() => Some(s),
            serde_json::Value::Null => None,
            _ => None,
        })
        .collect())
}

// ── OpenAI Provider ──

pub struct OpenAiProvider;

impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn validate(&self, config: &LlmConfig) -> Result<(), LlmError> {
        if config.api_key.is_empty() {
            return Err(LlmError::InvalidApiKey);
        }

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "Hi"}],
                "max_tokens": 1
            }))
            .send()
            .map_err(|e| LlmError::RequestFailed(format!("Connection failed: {}", e)))?;

        match resp.status().as_u16() {
            200 => Ok(()),
            401 | 403 => Err(LlmError::InvalidApiKey),
            status => {
                let body = resp.text().unwrap_or_default();
                Err(LlmError::RequestFailed(format!("HTTP {}: {}", status, body)))
            }
        }
    }

    fn classify_batch(
        &self,
        expenses: &[ParsedExpense],
        existing_categories: &[String],
        config: &LlmConfig,
    ) -> Result<Vec<Option<String>>, LlmError> {
        if expenses.is_empty() {
            return Ok(vec![]);
        }
        if config.api_key.is_empty() {
            return Err(LlmError::InvalidApiKey);
        }

        let prompt = build_classification_prompt(expenses, existing_categories);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.1
            }))
            .send()
            .map_err(|e| LlmError::RequestFailed(format!("Connection failed: {}", e)))?;

        match resp.status().as_u16() {
            401 | 403 => return Err(LlmError::InvalidApiKey),
            s if s >= 400 => {
                let body = resp.text().unwrap_or_default();
                return Err(LlmError::RequestFailed(format!("HTTP {}: {}", s, body)));
            }
            _ => {}
        }

        let body: serde_json::Value = resp
            .json()
            .map_err(|e| LlmError::RequestFailed(format!("Failed to parse response: {}", e)))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::RequestFailed("No content in response".to_string()))?;

        parse_classification_response(content, expenses.len())
    }
}

// ── Anthropic Provider ──

pub struct AnthropicProvider;

impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn validate(&self, config: &LlmConfig) -> Result<(), LlmError> {
        if config.api_key.is_empty() {
            return Err(LlmError::InvalidApiKey);
        }

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-haiku-4-5-20251001",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "Hi"}]
            }))
            .send()
            .map_err(|e| LlmError::RequestFailed(format!("Connection failed: {}", e)))?;

        match resp.status().as_u16() {
            200 => Ok(()),
            401 | 403 => Err(LlmError::InvalidApiKey),
            status => {
                let body = resp.text().unwrap_or_default();
                Err(LlmError::RequestFailed(format!("HTTP {}: {}", status, body)))
            }
        }
    }

    fn classify_batch(
        &self,
        expenses: &[ParsedExpense],
        existing_categories: &[String],
        config: &LlmConfig,
    ) -> Result<Vec<Option<String>>, LlmError> {
        if expenses.is_empty() {
            return Ok(vec![]);
        }
        if config.api_key.is_empty() {
            return Err(LlmError::InvalidApiKey);
        }

        let prompt = build_classification_prompt(expenses, existing_categories);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-haiku-4-5-20251001",
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .map_err(|e| LlmError::RequestFailed(format!("Connection failed: {}", e)))?;

        match resp.status().as_u16() {
            401 | 403 => return Err(LlmError::InvalidApiKey),
            s if s >= 400 => {
                let body = resp.text().unwrap_or_default();
                return Err(LlmError::RequestFailed(format!("HTTP {}: {}", s, body)));
            }
            _ => {}
        }

        let body: serde_json::Value = resp
            .json()
            .map_err(|e| LlmError::RequestFailed(format!("Failed to parse response: {}", e)))?;

        let content = body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| LlmError::RequestFailed("No content in response".to_string()))?;

        parse_classification_response(content, expenses.len())
    }
}

// ── Ollama Provider ──

pub struct OllamaProvider;

impl OllamaProvider {
    fn endpoint(config: &LlmConfig) -> String {
        let base = if config.api_key.is_empty() {
            "http://localhost:11434".to_string()
        } else {
            config.api_key.trim_end_matches('/').to_string()
        };
        base
    }
}

impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn validate(&self, config: &LlmConfig) -> Result<(), LlmError> {
        let base = Self::endpoint(config);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(format!("{}/api/tags", base))
            .send()
            .map_err(|e| LlmError::RequestFailed(format!("Could not connect to Ollama at {}: {}", base, e)))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(LlmError::RequestFailed(format!(
                "Ollama returned HTTP {}",
                resp.status()
            )))
        }
    }

    fn classify_batch(
        &self,
        expenses: &[ParsedExpense],
        existing_categories: &[String],
        config: &LlmConfig,
    ) -> Result<Vec<Option<String>>, LlmError> {
        if expenses.is_empty() {
            return Ok(vec![]);
        }

        let base = Self::endpoint(config);
        let prompt = build_classification_prompt(expenses, existing_categories);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(format!("{}/api/chat", base))
            .json(&serde_json::json!({
                "model": "llama3",
                "messages": [{"role": "user", "content": prompt}],
                "stream": false
            }))
            .send()
            .map_err(|e| LlmError::RequestFailed(format!("Connection failed: {}", e)))?;

        if !resp.status().is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(LlmError::RequestFailed(format!("Ollama error: {}", body)));
        }

        let body: serde_json::Value = resp
            .json()
            .map_err(|e| LlmError::RequestFailed(format!("Failed to parse response: {}", e)))?;

        let content = body["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::RequestFailed("No content in Ollama response".to_string()))?;

        parse_classification_response(content, expenses.len())
    }
}

// ── Factory ──

/// Create the appropriate LLM provider from a provider name string.
pub fn create_provider(provider_name: &str) -> Option<Box<dyn LlmProvider>> {
    match provider_name {
        "openai" => Some(Box::new(OpenAiProvider)),
        "anthropic" => Some(Box::new(AnthropicProvider)),
        "ollama" => Some(Box::new(OllamaProvider)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_expenses() -> Vec<ParsedExpense> {
        vec![
            ParsedExpense {
                title: "LIDL STORE #42".to_string(),
                amount: 45.20,
                date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            },
            ParsedExpense {
                title: "UBER TRIP".to_string(),
                amount: 12.50,
                date: NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
            },
            ParsedExpense {
                title: "NETFLIX SUBSCRIPTION".to_string(),
                amount: 15.99,
                date: NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
            },
        ]
    }

    // ── Factory tests ──

    #[test]
    fn test_create_provider_openai() {
        let p = create_provider("openai");
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "openai");
    }

    #[test]
    fn test_create_provider_anthropic() {
        let p = create_provider("anthropic");
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "anthropic");
    }

    #[test]
    fn test_create_provider_ollama() {
        let p = create_provider("ollama");
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "ollama");
    }

    #[test]
    fn test_create_provider_unknown() {
        assert!(create_provider("grok").is_none());
        assert!(create_provider("").is_none());
        assert!(create_provider("OPENAI").is_none());
    }

    // ── Validation tests (no real API calls) ──

    #[test]
    fn test_openai_validate_empty_key() {
        let provider = OpenAiProvider;
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: "".to_string(),
        };
        let result = provider.validate(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::InvalidApiKey));
    }

    #[test]
    fn test_anthropic_validate_empty_key() {
        let provider = AnthropicProvider;
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            api_key: "".to_string(),
        };
        let result = provider.validate(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::InvalidApiKey));
    }

    #[test]
    fn test_openai_classify_empty_key() {
        let provider = OpenAiProvider;
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: "".to_string(),
        };
        let result = provider.classify_batch(&sample_expenses(), &[], &config);
        assert!(matches!(result.unwrap_err(), LlmError::InvalidApiKey));
    }

    #[test]
    fn test_anthropic_classify_empty_key() {
        let provider = AnthropicProvider;
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            api_key: "".to_string(),
        };
        let result = provider.classify_batch(&sample_expenses(), &[], &config);
        assert!(matches!(result.unwrap_err(), LlmError::InvalidApiKey));
    }

    // ── classify_batch with empty expenses ──

    #[test]
    fn test_openai_classify_batch_empty_expenses() {
        let provider = OpenAiProvider;
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: "sk-test".to_string(),
        };
        let result = provider.classify_batch(&[], &[], &config).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_anthropic_classify_batch_empty_expenses() {
        let provider = AnthropicProvider;
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            api_key: "sk-test".to_string(),
        };
        let result = provider.classify_batch(&[], &[], &config).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_ollama_classify_batch_empty_expenses() {
        let provider = OllamaProvider;
        let config = LlmConfig {
            provider: "ollama".to_string(),
            api_key: "http://localhost:11434".to_string(),
        };
        let result = provider.classify_batch(&[], &[], &config).unwrap();
        assert!(result.is_empty());
    }

    // ── Response parsing tests ──

    #[test]
    fn test_parse_classification_response_valid() {
        let response = r#"["Groceries", "Transport", "Entertainment"]"#;
        let result = parse_classification_response(response, 3).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Some("Groceries".to_string()));
        assert_eq!(result[1], Some("Transport".to_string()));
        assert_eq!(result[2], Some("Entertainment".to_string()));
    }

    #[test]
    fn test_parse_classification_response_with_markdown() {
        let response = "```json\n[\"Groceries\", \"Transport\"]\n```";
        let result = parse_classification_response(response, 2).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Some("Groceries".to_string()));
    }

    #[test]
    fn test_parse_classification_response_with_extra_text() {
        let response = "Here are the categories:\n[\"Food\", \"Rides\"]\nHope this helps!";
        let result = parse_classification_response(response, 2).unwrap();
        assert_eq!(result[0], Some("Food".to_string()));
        assert_eq!(result[1], Some("Rides".to_string()));
    }

    #[test]
    fn test_parse_classification_response_with_nulls() {
        let response = r#"["Groceries", null, "Entertainment"]"#;
        let result = parse_classification_response(response, 3).unwrap();
        assert_eq!(result[0], Some("Groceries".to_string()));
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some("Entertainment".to_string()));
    }

    #[test]
    fn test_parse_malformed_response_no_array() {
        let response = "I don't know what you mean";
        let result = parse_classification_response(response, 3);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::RequestFailed(_)));
    }

    #[test]
    fn test_parse_malformed_response_wrong_count() {
        let response = r#"["Groceries", "Transport"]"#;
        let result = parse_classification_response(response, 3);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, LlmError::RequestFailed(_)));
        assert!(err.to_string().contains("Expected 3"));
    }

    #[test]
    fn test_parse_malformed_response_invalid_json() {
        let response = "[not valid json]";
        let result = parse_classification_response(response, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_classification_response_empty_strings() {
        let response = r#"["Groceries", "", "Entertainment"]"#;
        let result = parse_classification_response(response, 3).unwrap();
        assert_eq!(result[0], Some("Groceries".to_string()));
        assert_eq!(result[1], None); // empty string becomes None
        assert_eq!(result[2], Some("Entertainment".to_string()));
    }

    // ── Prompt building tests ──

    #[test]
    fn test_build_prompt_with_categories() {
        let expenses = sample_expenses();
        let categories = vec!["Groceries".to_string(), "Transport".to_string()];
        let prompt = build_classification_prompt(&expenses, &categories);
        assert!(prompt.contains("Groceries, Transport"));
        assert!(prompt.contains("LIDL STORE #42"));
        assert!(prompt.contains("UBER TRIP"));
        assert!(prompt.contains("NETFLIX SUBSCRIPTION"));
        assert!(prompt.contains("JSON array"));
    }

    #[test]
    fn test_build_prompt_without_categories() {
        let expenses = sample_expenses();
        let prompt = build_classification_prompt(&expenses, &[]);
        assert!(prompt.contains("None yet"));
    }

    // ── Ollama endpoint parsing ──

    #[test]
    fn test_ollama_endpoint_default() {
        let config = LlmConfig {
            provider: "ollama".to_string(),
            api_key: "".to_string(),
        };
        assert_eq!(OllamaProvider::endpoint(&config), "http://localhost:11434");
    }

    #[test]
    fn test_ollama_endpoint_custom() {
        let config = LlmConfig {
            provider: "ollama".to_string(),
            api_key: "http://myserver:11434/".to_string(),
        };
        assert_eq!(OllamaProvider::endpoint(&config), "http://myserver:11434");
    }

    // ── Integration tests (require real API keys) ──

    #[test]
    #[ignore]
    fn test_openai_real_classify() {
        let key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let provider = OpenAiProvider;
        let config = LlmConfig {
            provider: "openai".to_string(),
            api_key: key,
        };
        provider.validate(&config).expect("Validation failed");
        let result = provider
            .classify_batch(&sample_expenses(), &["Groceries".to_string()], &config)
            .expect("Classification failed");
        assert_eq!(result.len(), 3);
        for cat in &result {
            assert!(cat.is_some(), "Expected all expenses to be classified");
        }
    }

    #[test]
    #[ignore]
    fn test_anthropic_real_classify() {
        let key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
        let provider = AnthropicProvider;
        let config = LlmConfig {
            provider: "anthropic".to_string(),
            api_key: key,
        };
        provider.validate(&config).expect("Validation failed");
        let result = provider
            .classify_batch(&sample_expenses(), &["Groceries".to_string()], &config)
            .expect("Classification failed");
        assert_eq!(result.len(), 3);
    }

    #[test]
    #[ignore]
    fn test_ollama_real_classify() {
        let provider = OllamaProvider;
        let config = LlmConfig {
            provider: "ollama".to_string(),
            api_key: "http://localhost:11434".to_string(),
        };
        provider.validate(&config).expect("Ollama not running");
        let result = provider
            .classify_batch(&sample_expenses(), &[], &config)
            .expect("Classification failed");
        assert_eq!(result.len(), 3);
    }
}
