use crate::{RouterError, RouterResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardrailType {
    Input,
    Output,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardrailAction {
    Block,
    Warn,
    Modify,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub name: String,
    pub enabled: bool,
    pub guardrail_type: GuardrailType,
    pub action: GuardrailAction,
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct GuardrailResult {
    pub passed: bool,
    pub action: GuardrailAction,
    pub message: Option<String>,
    pub modified_content: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub trait Guardrail: Send + Sync {
    fn name(&self) -> &str;
    fn check_input(&self, content: &str, metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult>;
    fn check_output(&self, content: &str, metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult>;
}

// Built-in Guardrails

pub struct ContentModerationGuardrail {
    config: GuardrailConfig,
    blocked_patterns: Vec<Regex>,
}

impl ContentModerationGuardrail {
    pub fn new(config: GuardrailConfig) -> RouterResult<Self> {
        let patterns = config.settings.get("blocked_patterns")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap_or(""))
            .map(|pattern| Regex::new(pattern))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            config,
            blocked_patterns: patterns,
        })
    }
}

impl Guardrail for ContentModerationGuardrail {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn check_input(&self, content: &str, _metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult> {
        self.check_content(content)
    }

    fn check_output(&self, content: &str, _metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult> {
        self.check_content(content)
    }
}

impl ContentModerationGuardrail {
    fn check_content(&self, content: &str) -> RouterResult<GuardrailResult> {
        for pattern in &self.blocked_patterns {
            if pattern.is_match(content) {
                return Ok(GuardrailResult {
                    passed: false,
                    action: self.config.action.clone(),
                    message: Some(format!("Content blocked by pattern: {}", pattern.as_str())),
                    modified_content: None,
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(GuardrailResult {
            passed: true,
            action: self.config.action.clone(),
            message: None,
            modified_content: None,
            metadata: HashMap::new(),
        })
    }
}

pub struct PIIDetectionGuardrail {
    config: GuardrailConfig,
    email_regex: Regex,
    phone_regex: Regex,
    ssn_regex: Regex,
    credit_card_regex: Regex,
}

impl PIIDetectionGuardrail {
    pub fn new(config: GuardrailConfig) -> RouterResult<Self> {
        Ok(Self {
            config,
            email_regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?,
            phone_regex: Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b")?,
            ssn_regex: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")?,
            credit_card_regex: Regex::new(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b")?,
        })
    }

    fn detect_pii(&self, content: &str) -> Vec<String> {
        let mut detected = Vec::new();

        if self.email_regex.is_match(content) {
            detected.push("email".to_string());
        }
        if self.phone_regex.is_match(content) {
            detected.push("phone".to_string());
        }
        if self.ssn_regex.is_match(content) {
            detected.push("ssn".to_string());
        }
        if self.credit_card_regex.is_match(content) {
            detected.push("credit_card".to_string());
        }

        detected
    }

    fn redact_pii(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        result = self.email_regex.replace_all(&result, "[EMAIL_REDACTED]").to_string();
        result = self.phone_regex.replace_all(&result, "[PHONE_REDACTED]").to_string();
        result = self.ssn_regex.replace_all(&result, "[SSN_REDACTED]").to_string();
        result = self.credit_card_regex.replace_all(&result, "[CARD_REDACTED]").to_string();
        
        result
    }
}

impl Guardrail for PIIDetectionGuardrail {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn check_input(&self, content: &str, _metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult> {
        self.check_content(content)
    }

    fn check_output(&self, content: &str, _metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult> {
        self.check_content(content)
    }
}

impl PIIDetectionGuardrail {
    fn check_content(&self, content: &str) -> RouterResult<GuardrailResult> {
        let detected_pii = self.detect_pii(content);
        
        if detected_pii.is_empty() {
            Ok(GuardrailResult {
                passed: true,
                action: self.config.action.clone(),
                message: None,
                modified_content: None,
                metadata: HashMap::new(),
            })
        } else {
            let mut metadata = HashMap::new();
            metadata.insert("detected_pii".to_string(), serde_json::json!(detected_pii));

            let (passed, modified_content) = match self.config.action {
                GuardrailAction::Block => (false, None),
                GuardrailAction::Modify => (true, Some(self.redact_pii(content))),
                _ => (false, None),
            };

            Ok(GuardrailResult {
                passed,
                action: self.config.action.clone(),
                message: Some(format!("PII detected: {}", detected_pii.join(", "))),
                modified_content,
                metadata,
            })
        }
    }
}

pub struct TokenLimitGuardrail {
    config: GuardrailConfig,
    max_tokens: usize,
}

impl TokenLimitGuardrail {
    pub fn new(config: GuardrailConfig) -> RouterResult<Self> {
        let max_tokens = config.settings.get("max_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(4000) as usize;

        Ok(Self {
            config,
            max_tokens,
        })
    }

    fn estimate_tokens(&self, content: &str) -> usize {
        // Simple token estimation: ~4 characters per token
        // In a real implementation, you'd use a proper tokenizer
        content.len() / 4
    }
}

impl Guardrail for TokenLimitGuardrail {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn check_input(&self, content: &str, _metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult> {
        let token_count = self.estimate_tokens(content);
        
        if token_count > self.max_tokens {
            Ok(GuardrailResult {
                passed: false,
                action: self.config.action.clone(),
                message: Some(format!("Token limit exceeded: {} > {}", token_count, self.max_tokens)),
                modified_content: None,
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("token_count".to_string(), serde_json::json!(token_count));
                    map.insert("max_tokens".to_string(), serde_json::json!(self.max_tokens));
                    map
                },
            })
        } else {
            Ok(GuardrailResult {
                passed: true,
                action: self.config.action.clone(),
                message: None,
                modified_content: None,
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("token_count".to_string(), serde_json::json!(token_count));
                    map
                },
            })
        }
    }

    fn check_output(&self, content: &str, metadata: &HashMap<String, String>) -> RouterResult<GuardrailResult> {
        self.check_input(content, metadata)
    }
}

pub struct GuardrailEngine {
    guardrails: Vec<Box<dyn Guardrail>>,
}

impl GuardrailEngine {
    pub fn new() -> Self {
        Self {
            guardrails: Vec::new(),
        }
    }

    pub fn add_guardrail(&mut self, guardrail: Box<dyn Guardrail>) {
        self.guardrails.push(guardrail);
    }

    pub fn check_input(&self, content: &str, metadata: &HashMap<String, String>) -> RouterResult<Vec<GuardrailResult>> {
        let mut results = Vec::new();
        
        for guardrail in &self.guardrails {
            let result = guardrail.check_input(content, metadata)?;
            results.push(result);
        }
        
        Ok(results)
    }

    pub fn check_output(&self, content: &str, metadata: &HashMap<String, String>) -> RouterResult<Vec<GuardrailResult>> {
        let mut results = Vec::new();
        
        for guardrail in &self.guardrails {
            let result = guardrail.check_output(content, metadata)?;
            results.push(result);
        }
        
        Ok(results)
    }

    pub fn should_block(&self, results: &[GuardrailResult]) -> bool {
        results.iter().any(|r| !r.passed && matches!(r.action, GuardrailAction::Block))
    }

    pub fn get_modified_content(&self, results: &[GuardrailResult], original: &str) -> String {
        for result in results {
            if let Some(modified) = &result.modified_content {
                return modified.clone();
            }
        }
        original.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_moderation_guardrail() {
        let config = GuardrailConfig {
            name: "content_moderation".to_string(),
            enabled: true,
            guardrail_type: GuardrailType::Both,
            action: GuardrailAction::Block,
            settings: {
                let mut map = HashMap::new();
                map.insert("blocked_patterns".to_string(), json!(["badword", "spam"]));
                map
            },
        };

        let guardrail = ContentModerationGuardrail::new(config).unwrap();
        
        // Test blocked content
        let result = guardrail.check_input("This contains badword", &HashMap::new()).unwrap();
        assert!(!result.passed);
        assert!(result.message.is_some());

        // Test allowed content
        let result = guardrail.check_input("This is clean content", &HashMap::new()).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_pii_detection_guardrail() {
        let config = GuardrailConfig {
            name: "pii_detection".to_string(),
            enabled: true,
            guardrail_type: GuardrailType::Both,
            action: GuardrailAction::Modify,
            settings: HashMap::new(),
        };

        let guardrail = PIIDetectionGuardrail::new(config).unwrap();
        
        // Test PII detection and redaction
        let content = "Contact me at john@example.com or call 555-123-4567";
        let result = guardrail.check_input(content, &HashMap::new()).unwrap();
        
        assert!(result.passed); // Should pass with modification
        assert!(result.modified_content.is_some());
        let modified = result.modified_content.unwrap();
        assert!(modified.contains("[EMAIL_REDACTED]"));
        assert!(modified.contains("[PHONE_REDACTED]"));
    }

    #[test]
    fn test_token_limit_guardrail() {
        let config = GuardrailConfig {
            name: "token_limit".to_string(),
            enabled: true,
            guardrail_type: GuardrailType::Input,
            action: GuardrailAction::Block,
            settings: {
                let mut map = HashMap::new();
                map.insert("max_tokens".to_string(), json!(10));
                map
            },
        };

        let guardrail = TokenLimitGuardrail::new(config).unwrap();
        
        // Test content that exceeds token limit (40+ chars = 10+ tokens)
        let long_content = "This is a very long piece of content that should exceed the token limit";
        let result = guardrail.check_input(long_content, &HashMap::new()).unwrap();
        assert!(!result.passed);

        // Test content within limit
        let short_content = "Short text";
        let result = guardrail.check_input(short_content, &HashMap::new()).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_guardrail_engine() {
        let mut engine = GuardrailEngine::new();
        
        // Add PII detection guardrail
        let pii_config = GuardrailConfig {
            name: "pii".to_string(),
            enabled: true,
            guardrail_type: GuardrailType::Both,
            action: GuardrailAction::Block,
            settings: HashMap::new(),
        };
        engine.add_guardrail(Box::new(PIIDetectionGuardrail::new(pii_config).unwrap()));

        // Test content with PII
        let content = "My email is test@example.com";
        let results = engine.check_input(content, &HashMap::new()).unwrap();
        
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(engine.should_block(&results));
    }
}