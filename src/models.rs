use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Latest AI model definitions with capabilities and routing intelligence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDefinition {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub capabilities: ModelCapabilities,
    pub pricing: ModelPricing,
    pub limits: ModelLimits,
    pub performance: ModelPerformance,
    pub use_cases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub reasoning_score: u8,        // 1-10 scale
    pub speed_score: u8,           // 1-10 scale  
    pub cost_efficiency: u8,       // 1-10 scale
    pub context_length: u32,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub supports_streaming: bool,
    pub supports_json_mode: bool,
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_cost_per_1k: f64,    // USD per 1K tokens
    pub output_cost_per_1k: f64,   // USD per 1K tokens
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimits {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub requests_per_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub avg_latency_ms: u32,
    pub throughput_tokens_per_sec: u32,
    pub reliability_score: u8,     // 1-10 scale
}

/// Model registry with all supported models
#[derive(Debug)]
pub struct ModelRegistry {
    models: HashMap<String, ModelDefinition>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.load_default_models();
        registry
    }

    /// Load all the latest and most powerful models
    fn load_default_models(&mut self) {
        // ===== CLAUDE 4 MODELS (Latest Anthropic - May 2025) =====
        
        // Claude 4 Opus - The most capable model for ultra-complex tasks
        self.add_model(ModelDefinition {
            id: "claude-opus-4-20250514".to_string(),
            provider: "anthropic".to_string(),
            display_name: "Claude 4 Opus".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 10,
                speed_score: 7,
                cost_efficiency: 5,
                context_length: 500000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 16384,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 20.0,
                output_cost_per_1k: 100.0,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 1000,
                tokens_per_minute: 100000,
                requests_per_day: 100000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 1500,
                throughput_tokens_per_sec: 90,
                reliability_score: 10,
            },
            use_cases: vec![
                "ultra_complex_reasoning".to_string(),
                "theoretical_analysis".to_string(),
                "research".to_string(),
                "expert_tasks".to_string(),
                "mathematical_proofs".to_string(),
            ],
        });

        // Claude 4 Sonnet - Balanced performance for complex tasks
        self.add_model(ModelDefinition {
            id: "claude-sonnet-4-20250514".to_string(),
            provider: "anthropic".to_string(),
            display_name: "Claude 4 Sonnet".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 10,
                speed_score: 9,
                cost_efficiency: 7,
                context_length: 500000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 16384,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 5.0,
                output_cost_per_1k: 25.0,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 2000,
                tokens_per_minute: 150000,
                requests_per_day: 200000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 900,
                throughput_tokens_per_sec: 120,
                reliability_score: 10,
            },
            use_cases: vec![
                "complex_reasoning".to_string(),
                "code_generation".to_string(),
                "analysis".to_string(),
                "creative_writing".to_string(),
                "research".to_string(),
                "enterprise_tasks".to_string(),
            ],
        });

        // Claude 3 Opus - Most capable for complex tasks
        self.add_model(ModelDefinition {
            id: "claude-3-opus-20240229".to_string(),
            provider: "anthropic".to_string(),
            display_name: "Claude 3 Opus".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 10,
                speed_score: 6,
                cost_efficiency: 4,
                context_length: 200000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 4096,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 15.0,
                output_cost_per_1k: 75.0,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 1000,
                tokens_per_minute: 80000,
                requests_per_day: 100000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 1800,
                throughput_tokens_per_sec: 65,
                reliability_score: 10,
            },
            use_cases: vec![
                "complex_reasoning".to_string(),
                "research".to_string(),
                "analysis".to_string(),
                "expert_tasks".to_string(),
            ],
        });

        // Claude 3 Haiku - Fast and efficient
        self.add_model(ModelDefinition {
            id: "claude-3-haiku-20240307".to_string(),
            provider: "anthropic".to_string(),
            display_name: "Claude 3 Haiku".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 7,
                speed_score: 10,
                cost_efficiency: 10,
                context_length: 200000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 4096,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 0.25,
                output_cost_per_1k: 1.25,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 2000,
                tokens_per_minute: 100000,
                requests_per_day: 200000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 400,
                throughput_tokens_per_sec: 120,
                reliability_score: 9,
            },
            use_cases: vec![
                "simple_tasks".to_string(),
                "quick_responses".to_string(),
                "high_volume".to_string(),
            ],
        });

        // ===== GPT-4 MODELS (Latest OpenAI) =====

        // GPT-4o - Most capable OpenAI model
        self.add_model(ModelDefinition {
            id: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            display_name: "GPT-4o".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 9,
                speed_score: 8,
                cost_efficiency: 6,
                context_length: 128000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 16384,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 2.5,
                output_cost_per_1k: 10.0,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 5000,
                tokens_per_minute: 800000,
                requests_per_day: 500000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 800,
                throughput_tokens_per_sec: 100,
                reliability_score: 9,
            },
            use_cases: vec![
                "complex_reasoning".to_string(),
                "code_generation".to_string(),
                "multimodal".to_string(),
                "analysis".to_string(),
            ],
        });

        // GPT-4o-mini - Fast and cost-effective
        self.add_model(ModelDefinition {
            id: "gpt-4o-mini".to_string(),
            provider: "openai".to_string(),
            display_name: "GPT-4o Mini".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 8,
                speed_score: 10,
                cost_efficiency: 10,
                context_length: 128000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 16384,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 0.15,
                output_cost_per_1k: 0.6,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 10000,
                tokens_per_minute: 2000000,
                requests_per_day: 1000000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 300,
                throughput_tokens_per_sec: 150,
                reliability_score: 9,
            },
            use_cases: vec![
                "general_tasks".to_string(),
                "high_volume".to_string(),
                "cost_optimization".to_string(),
                "quick_responses".to_string(),
            ],
        });

        // GPT-4 Turbo - Balanced performance
        self.add_model(ModelDefinition {
            id: "gpt-4-turbo".to_string(),
            provider: "openai".to_string(),
            display_name: "GPT-4 Turbo".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 9,
                speed_score: 7,
                cost_efficiency: 7,
                context_length: 128000,
                supports_vision: true,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: true,
                max_output_tokens: 4096,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 10.0,
                output_cost_per_1k: 30.0,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 5000,
                tokens_per_minute: 800000,
                requests_per_day: 500000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 1000,
                throughput_tokens_per_sec: 90,
                reliability_score: 9,
            },
            use_cases: vec![
                "complex_reasoning".to_string(),
                "analysis".to_string(),
                "code_generation".to_string(),
            ],
        });

        // GPT-4 - Original GPT-4
        self.add_model(ModelDefinition {
            id: "gpt-4".to_string(),
            provider: "openai".to_string(),
            display_name: "GPT-4".to_string(),
            capabilities: ModelCapabilities {
                reasoning_score: 9,
                speed_score: 5,
                cost_efficiency: 5,
                context_length: 8192,
                supports_vision: false,
                supports_function_calling: true,
                supports_streaming: true,
                supports_json_mode: false,
                max_output_tokens: 4096,
            },
            pricing: ModelPricing {
                input_cost_per_1k: 30.0,
                output_cost_per_1k: 60.0,
                currency: "USD".to_string(),
            },
            limits: ModelLimits {
                requests_per_minute: 3000,
                tokens_per_minute: 150000,
                requests_per_day: 200000,
            },
            performance: ModelPerformance {
                avg_latency_ms: 1500,
                throughput_tokens_per_sec: 70,
                reliability_score: 8,
            },
            use_cases: vec![
                "complex_reasoning".to_string(),
                "analysis".to_string(),
            ],
        });
    }

    pub fn add_model(&mut self, model: ModelDefinition) {
        self.models.insert(model.id.clone(), model);
    }

    pub fn get_model(&self, id: &str) -> Option<&ModelDefinition> {
        self.models.get(id)
    }

    pub fn get_all_models(&self) -> Vec<&ModelDefinition> {
        self.models.values().collect()
    }

    pub fn get_models_by_provider(&self, provider: &str) -> Vec<&ModelDefinition> {
        self.models
            .values()
            .filter(|model| model.provider == provider)
            .collect()
    }

    /// Intelligent model selection based on task requirements
    pub fn select_best_model(&self, requirements: &TaskRequirements) -> Option<&ModelDefinition> {
        let mut candidates: Vec<&ModelDefinition> = self.models.values().collect();

        // Filter by provider if specified
        if let Some(provider) = &requirements.preferred_provider {
            candidates.retain(|model| &model.provider == provider);
        }

        // Filter by capabilities
        candidates.retain(|model| {
            model.capabilities.context_length >= requirements.min_context_length
                && model.capabilities.max_output_tokens >= requirements.min_output_tokens
                && (!requirements.needs_vision || model.capabilities.supports_vision)
                && (!requirements.needs_function_calling || model.capabilities.supports_function_calling)
        });

        if candidates.is_empty() {
            return None;
        }

        // Score models based on requirements
        let mut scored_models: Vec<(&ModelDefinition, f64)> = candidates
            .into_iter()
            .map(|model| {
                let score = self.calculate_model_score(model, requirements);
                (model, score)
            })
            .collect();

        // Sort by score (highest first)
        scored_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scored_models.first().map(|(model, _)| *model)
    }

    fn calculate_model_score(&self, model: &ModelDefinition, requirements: &TaskRequirements) -> f64 {
        let mut score = 0.0;

        // Weight factors based on requirements
        match requirements.priority {
            TaskPriority::Speed => {
                score += model.capabilities.speed_score as f64 * 0.4;
                score += model.capabilities.cost_efficiency as f64 * 0.3;
                score += model.capabilities.reasoning_score as f64 * 0.3;
            }
            TaskPriority::Quality => {
                score += model.capabilities.reasoning_score as f64 * 0.5;
                score += model.capabilities.speed_score as f64 * 0.2;
                score += model.capabilities.cost_efficiency as f64 * 0.3;
            }
            TaskPriority::Cost => {
                score += model.capabilities.cost_efficiency as f64 * 0.5;
                score += model.capabilities.speed_score as f64 * 0.3;
                score += model.capabilities.reasoning_score as f64 * 0.2;
            }
            TaskPriority::Balanced => {
                score += model.capabilities.reasoning_score as f64 * 0.35;
                score += model.capabilities.speed_score as f64 * 0.35;
                score += model.capabilities.cost_efficiency as f64 * 0.3;
            }
        }

        // Bonus for matching use cases
        for use_case in &requirements.use_cases {
            if model.use_cases.contains(use_case) {
                score += 1.0;
            }
        }

        score
    }

    /// Get recommended models for complex tasks (like yours!)
    pub fn get_complex_task_models(&self) -> Vec<&ModelDefinition> {
        let requirements = TaskRequirements {
            priority: TaskPriority::Quality,
            min_context_length: 100000,
            min_output_tokens: 4000,
            needs_vision: false,
            needs_function_calling: true,
            preferred_provider: None,
            use_cases: vec!["complex_reasoning".to_string(), "code_generation".to_string()],
        };

        let mut models: Vec<&ModelDefinition> = self.models.values()
            .filter(|model| {
                model.capabilities.reasoning_score >= 8
                    && model.capabilities.context_length >= requirements.min_context_length
                    && model.capabilities.max_output_tokens >= requirements.min_output_tokens
            })
            .collect();

        // Sort by reasoning capability (best first)
        models.sort_by(|a, b| b.capabilities.reasoning_score.cmp(&a.capabilities.reasoning_score));
        models
    }
}

#[derive(Debug, Clone)]
pub struct TaskRequirements {
    pub priority: TaskPriority,
    pub min_context_length: u32,
    pub min_output_tokens: u32,
    pub needs_vision: bool,
    pub needs_function_calling: bool,
    pub preferred_provider: Option<String>,
    pub use_cases: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TaskPriority {
    Speed,
    Quality,
    Cost,
    Balanced,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = ModelRegistry::new();
        assert!(!registry.models.is_empty());
        
        // Check that we have Claude 3.5 Sonnet
        assert!(registry.get_model("claude-3-5-sonnet-20241022").is_some());
        
        // Check that we have GPT-4o
        assert!(registry.get_model("gpt-4o").is_some());
    }

    #[test]
    fn test_complex_task_models() {
        let registry = ModelRegistry::new();
        let complex_models = registry.get_complex_task_models();
        
        assert!(!complex_models.is_empty());
        
        // Should include Claude 3.5 Sonnet and GPT-4o for complex tasks
        let model_ids: Vec<&str> = complex_models.iter().map(|m| m.id.as_str()).collect();
        assert!(model_ids.contains(&"claude-3-5-sonnet-20241022"));
        assert!(model_ids.contains(&"gpt-4o"));
    }

    #[test]
    fn test_model_selection() {
        let registry = ModelRegistry::new();
        
        let requirements = TaskRequirements {
            priority: TaskPriority::Quality,
            min_context_length: 100000,
            min_output_tokens: 4000,
            needs_vision: false,
            needs_function_calling: true,
            preferred_provider: Some("anthropic".to_string()),
            use_cases: vec!["complex_reasoning".to_string()],
        };

        let selected = registry.select_best_model(&requirements);
        assert!(selected.is_some());
        
        let model = selected.unwrap();
        assert_eq!(model.provider, "anthropic");
        assert!(model.capabilities.reasoning_score >= 8);
    }
}