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

    /// Classify a batch of expenses, returning suggested category and confidence for each.
    fn classify_batch(
        &self,
        expenses: &[ParsedExpense],
        existing_categories: &[String],
        config: &LlmConfig,
    ) -> Result<Vec<Option<LlmClassification>>, LlmError>;
}

/// Classification result from LLM including confidence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmClassification {
    pub category: String,
    pub confidence: f64,
}

/// Build the classification prompt for all providers.
fn build_classification_prompt(expenses: &[ParsedExpense], existing_categories: &[String]) -> String {
    let category_instruction = if existing_categories.is_empty() {
        "No existing categories yet — suggest appropriate short category names.".to_string()
    } else {
        format!(
            "Known categories: [{}]\n\
             Choose from the known categories whenever possible. Only invent a new category if the expense genuinely doesn't fit ANY existing one.",
            existing_categories.join(", ")
        )
    };

    let mut expense_list = String::new();
    for (i, e) in expenses.iter().enumerate() {
        expense_list.push_str(&format!("{}. {} — {:.2}\n", i + 1, e.title, e.amount));
    }

    format!(
        "You are an expense classifier. Assign each expense to the most appropriate category.\n\n\
         {}\n\n\
         Expenses:\n{}\n\
         For each expense, respond with a JSON array of objects. Each object must have:\n\
         - \"id\": the expense number (1-based)\n\
         - \"category\": the assigned category string\n\
         - \"confidence\": \"high\", \"medium\", or \"low\"\n\n\
         Use \"low\" confidence if the title is too vague or cryptic to classify reliably \
         (e.g. reference numbers, generic payment descriptions).\n\n\
         Example: [{{\"id\": 1, \"category\": \"Groceries\", \"confidence\": \"high\"}}]\n\
         Respond with ONLY the JSON array. No other text.",
        category_instruction, expense_list
    )
}

fn confidence_str_to_f64(s: &str) -> f64 {
    match s.to_lowercase().as_str() {
        "high" => 0.9,
        "medium" => 0.6,
        "low" => 0.3,
        _ => 0.5,
    }
}

/// Extract JSON array from LLM response, handling markdown fences and extra text.
fn extract_json_array(response: &str) -> Result<Vec<serde_json::Value>, LlmError> {
    let cleaned = response.trim();
    let cleaned = if cleaned.starts_with("```") {
        cleaned
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        cleaned
    };

    let start = cleaned.find('[').ok_or_else(|| {
        LlmError::RequestFailed(format!("No JSON array found in response: {}", &response[..response.len().min(200)]))
    })?;
    let end = cleaned.rfind(']').ok_or_else(|| {
        LlmError::RequestFailed("No closing bracket found in response".to_string())
    })?;

    let json_str = &cleaned[start..=end];
    serde_json::from_str(json_str)
        .map_err(|e| LlmError::RequestFailed(format!("Failed to parse JSON: {}", e)))
}

/// Parse keyed response format: [{"id": 1, "category": "...", "confidence": "high"}, ...]
/// Results are indexed by ID (1-based) into a Vec of size expected_count.
/// Missing IDs result in None — partial results are accepted.
fn parse_classification_response(response: &str, expected_count: usize) -> Result<Vec<Option<LlmClassification>>, LlmError> {
    let parsed = extract_json_array(response)?;

    let mut results: Vec<Option<LlmClassification>> = vec![None; expected_count];

    for item in &parsed {
        // Support both keyed format (new) and flat string format (legacy)
        if let Some(obj) = item.as_object() {
            let id = obj.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let category = obj.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let confidence_str = obj.get("confidence").and_then(|v| v.as_str()).unwrap_or("medium");

            if id >= 1 && id <= expected_count && !category.is_empty() {
                results[id - 1] = Some(LlmClassification {
                    category,
                    confidence: confidence_str_to_f64(confidence_str),
                });
            }
        } else if let Some(s) = item.as_str() {
            // Legacy flat string fallback — assign in order to first empty slot
            if !s.is_empty() {
                if let Some(slot) = results.iter_mut().find(|r| r.is_none()) {
                    *slot = Some(LlmClassification {
                        category: s.to_string(),
                        confidence: 0.6,
                    });
                }
            }
        }
    }

    Ok(results)
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
    ) -> Result<Vec<Option<LlmClassification>>, LlmError> {
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
    ) -> Result<Vec<Option<LlmClassification>>, LlmError> {
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
                "temperature": 0.1,
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
    ) -> Result<Vec<Option<LlmClassification>>, LlmError> {
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
                "stream": false,
                "options": {"temperature": 0.1}
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
    fn test_parse_keyed_response() {
        let response = r#"[{"id": 1, "category": "Groceries", "confidence": "high"}, {"id": 2, "category": "Transport", "confidence": "medium"}, {"id": 3, "category": "Entertainment", "confidence": "low"}]"#;
        let result = parse_classification_response(response, 3).unwrap();
        assert_eq!(result.len(), 3);
        let r0 = result[0].as_ref().unwrap();
        assert_eq!(r0.category, "Groceries");
        assert!((r0.confidence - 0.9).abs() < 0.01);
        let r1 = result[1].as_ref().unwrap();
        assert_eq!(r1.category, "Transport");
        assert!((r1.confidence - 0.6).abs() < 0.01);
        let r2 = result[2].as_ref().unwrap();
        assert_eq!(r2.category, "Entertainment");
        assert!((r2.confidence - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_parse_keyed_response_partial() {
        // Only 2 out of 3 returned — missing ID 2 should be None
        let response = r#"[{"id": 1, "category": "Groceries", "confidence": "high"}, {"id": 3, "category": "Fun", "confidence": "medium"}]"#;
        let result = parse_classification_response(response, 3).unwrap();
        assert!(result[0].is_some());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
    }

    #[test]
    fn test_parse_legacy_string_response() {
        let response = r#"["Groceries", "Transport"]"#;
        let result = parse_classification_response(response, 2).unwrap();
        assert_eq!(result[0].as_ref().unwrap().category, "Groceries");
        assert_eq!(result[1].as_ref().unwrap().category, "Transport");
    }

    #[test]
    fn test_parse_response_with_markdown() {
        let response = "```json\n[{\"id\": 1, \"category\": \"Food\", \"confidence\": \"high\"}]\n```";
        let result = parse_classification_response(response, 1).unwrap();
        assert_eq!(result[0].as_ref().unwrap().category, "Food");
    }

    #[test]
    fn test_parse_response_with_extra_text() {
        let response = "Here are the results:\n[{\"id\": 1, \"category\": \"Food\", \"confidence\": \"high\"}, {\"id\": 2, \"category\": \"Rides\", \"confidence\": \"medium\"}]\nDone!";
        let result = parse_classification_response(response, 2).unwrap();
        assert_eq!(result[0].as_ref().unwrap().category, "Food");
        assert_eq!(result[1].as_ref().unwrap().category, "Rides");
    }

    #[test]
    fn test_parse_malformed_response_no_array() {
        let response = "I don't know what you mean";
        let result = parse_classification_response(response, 3);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::RequestFailed(_)));
    }

    #[test]
    fn test_parse_malformed_response_invalid_json() {
        let response = "[not valid json]";
        let result = parse_classification_response(response, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_keyed_response_out_of_range_id() {
        // ID 5 is out of range for expected_count=2 — should be ignored
        let response = r#"[{"id": 1, "category": "A", "confidence": "high"}, {"id": 5, "category": "B", "confidence": "high"}]"#;
        let result = parse_classification_response(response, 2).unwrap();
        assert!(result[0].is_some());
        assert!(result[1].is_none());
    }

    // ── Prompt building tests ──

    #[test]
    fn test_build_prompt_with_categories() {
        let expenses = sample_expenses();
        let categories = vec!["Groceries".to_string(), "Transport".to_string()];
        let prompt = build_classification_prompt(&expenses, &categories);
        assert!(prompt.contains("Groceries, Transport"));
        assert!(prompt.contains("LIDL STORE #42"));
        assert!(prompt.contains("45.20")); // amount included
        assert!(prompt.contains("UBER TRIP"));
        assert!(prompt.contains("confidence"));
        assert!(prompt.contains("\"id\""));
    }

    #[test]
    fn test_build_prompt_without_categories() {
        let expenses = sample_expenses();
        let prompt = build_classification_prompt(&expenses, &[]);
        assert!(prompt.contains("No existing categories"));
    }

    #[test]
    fn test_build_prompt_constrains_invention() {
        let expenses = sample_expenses();
        let categories = vec!["Food".to_string()];
        let prompt = build_classification_prompt(&expenses, &categories);
        assert!(prompt.contains("Only invent a new category"));
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
