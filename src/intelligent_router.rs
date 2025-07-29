use crate::{
    models::{ModelRegistry, TaskRequirements, TaskPriority},
    RouterError, RouterResult, Target,
};
use serde::{Deserialize, Serialize};

/// Intelligent AI model router that automatically selects the best model for each task
#[derive(Debug)]
pub struct IntelligentRouter {
    model_registry: ModelRegistry,
    routing_rules: Vec<RoutingRule>,
    fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub name: String,
    pub conditions: Vec<RoutingCondition>,
    pub target_model: String,
    pub priority: u8, // 1-10, higher = more priority
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingCondition {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackStrategy {
    BestQuality,
    FastestResponse,
    MostCostEffective,
    Balanced,
}

#[derive(Debug, Clone)]
pub struct IntelligentRoutingContext {
    pub task_type: TaskType,
    pub complexity_score: f64,      // 0.0-1.0
    pub urgency: Urgency,
    pub budget_constraint: Option<f64>, // Max cost per 1K tokens
    pub content_analysis: ContentAnalysis,
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Clone)]
pub enum TaskType {
    CodeGeneration,
    ComplexReasoning,
    Analysis,
    CreativeWriting,
    QuickResponse,
    Research,
    Multimodal,
    Translation,
    Summarization,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Urgency {
    Critical,   // Need fastest response
    High,       // Balance speed and quality
    Normal,     // Default routing
    Low,        // Can use slower but better models
}

#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub estimated_tokens: u32,
    pub language: Option<String>,
    pub contains_code: bool,
    pub contains_math: bool,
    pub requires_reasoning: bool,
    pub has_images: bool,
}

#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub preferred_provider: Option<String>,
    pub max_latency_ms: Option<u32>,
    pub quality_over_speed: bool,
    pub cost_conscious: bool,
}

impl IntelligentRouter {
    pub fn new() -> Self {
        let mut router = Self {
            model_registry: ModelRegistry::new(),
            routing_rules: Vec::new(),
            fallback_strategy: FallbackStrategy::Balanced,
        };
        router.load_default_rules();
        router
    }

    /// Load intelligent routing rules optimized for different scenarios
    fn load_default_rules(&mut self) {
        // Rule 1: Ultra-complex theoretical/research tasks -> Claude 4 Opus
        self.routing_rules.push(RoutingRule {
            name: "ultra_complex_theoretical".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "complexity_score".to_string(),
                    operator: "gte".to_string(),
                    value: serde_json::json!(0.8),
                },
                RoutingCondition {
                    field: "task_type".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!("ComplexReasoning"),
                },
            ],
            target_model: "claude-opus-4-20250514".to_string(),
            priority: 10,
        });

        // Rule 2: Complex Rust/Systems Programming -> Claude 4 Sonnet
        self.routing_rules.push(RoutingRule {
            name: "complex_rust_tasks".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "task_type".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!("CodeGeneration"),
                },
                RoutingCondition {
                    field: "complexity_score".to_string(),
                    operator: "gte".to_string(),
                    value: serde_json::json!(0.5),
                },
            ],
            target_model: "claude-sonnet-4-20250514".to_string(),
            priority: 9,
        });

        // Rule 3: Any complex analysis -> Claude 4 Sonnet
        self.routing_rules.push(RoutingRule {
            name: "complex_analysis".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "task_type".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!("Analysis"),
                },
                RoutingCondition {
                    field: "complexity_score".to_string(),
                    operator: "gte".to_string(),
                    value: serde_json::json!(0.4),
                },
            ],
            target_model: "claude-sonnet-4-20250514".to_string(),
            priority: 8,
        });

        // Rule 3: Fast responses -> GPT-4o-mini or Claude Haiku
        self.routing_rules.push(RoutingRule {
            name: "fast_responses".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "urgency".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!("Critical"),
                },
            ],
            target_model: "gpt-4o-mini".to_string(),
            priority: 8,
        });

        // Rule 4: Multimodal tasks -> GPT-4o
        self.routing_rules.push(RoutingRule {
            name: "multimodal_tasks".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "has_images".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!(true),
                },
            ],
            target_model: "gpt-4o".to_string(),
            priority: 9,
        });

        // Rule 5: Cost-conscious tasks -> Claude Haiku or GPT-4o-mini
        self.routing_rules.push(RoutingRule {
            name: "cost_conscious".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "cost_conscious".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!(true),
                },
                RoutingCondition {
                    field: "complexity_score".to_string(),
                    operator: "lt".to_string(),
                    value: serde_json::json!(0.6),
                },
            ],
            target_model: "claude-3-haiku-20240307".to_string(),
            priority: 6,
        });

        // Rule 6: High-quality analysis -> Claude 3.5 Sonnet
        self.routing_rules.push(RoutingRule {
            name: "high_quality_analysis".to_string(),
            conditions: vec![
                RoutingCondition {
                    field: "task_type".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!("Analysis"),
                },
                RoutingCondition {
                    field: "quality_over_speed".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::json!(true),
                },
            ],
            target_model: "claude-3-5-sonnet-20241022".to_string(),
            priority: 8,
        });
    }

    /// Main routing function with intelligent model selection
    pub fn route_intelligent(
        &self,
        context: &IntelligentRoutingContext,
        available_targets: &[Target],
    ) -> RouterResult<String> {
        // Step 1: Try rule-based routing
        if let Some(model_id) = self.apply_routing_rules(context) {
            if self.model_registry.get_model(&model_id).is_some() {
                return Ok(model_id);
            }
        }

        // Step 2: Use intelligent model selection
        let requirements = self.context_to_requirements(context);
        if let Some(model) = self.model_registry.select_best_model(&requirements) {
            return Ok(model.id.clone());
        }

        // Step 3: Fallback strategy
        self.apply_fallback_strategy(context, available_targets)
    }

    fn apply_routing_rules(&self, context: &IntelligentRoutingContext) -> Option<String> {
        let mut matching_rules: Vec<&RoutingRule> = self.routing_rules
            .iter()
            .filter(|rule| self.rule_matches(rule, context))
            .collect();

        // Sort by priority (highest first)
        matching_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        matching_rules.first().map(|rule| rule.target_model.clone())
    }

    fn rule_matches(&self, rule: &RoutingRule, context: &IntelligentRoutingContext) -> bool {
        rule.conditions.iter().all(|condition| {
            self.evaluate_condition(condition, context)
        })
    }

    fn evaluate_condition(&self, condition: &RoutingCondition, context: &IntelligentRoutingContext) -> bool {
        let context_value = self.get_context_field_value(&condition.field, context);
        
        match condition.operator.as_str() {
            "eq" => context_value == condition.value,
            "ne" => context_value != condition.value,
            "gt" => {
                if let (Some(a), Some(b)) = (context_value.as_f64(), condition.value.as_f64()) {
                    a > b
                } else {
                    false
                }
            }
            "gte" => {
                if let (Some(a), Some(b)) = (context_value.as_f64(), condition.value.as_f64()) {
                    a >= b
                } else {
                    false
                }
            }
            "lt" => {
                if let (Some(a), Some(b)) = (context_value.as_f64(), condition.value.as_f64()) {
                    a < b
                } else {
                    false
                }
            }
            "lte" => {
                if let (Some(a), Some(b)) = (context_value.as_f64(), condition.value.as_f64()) {
                    a <= b
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn get_context_field_value(&self, field: &str, context: &IntelligentRoutingContext) -> serde_json::Value {
        match field {
            "task_type" => serde_json::json!(format!("{:?}", context.task_type)),
            "complexity_score" => serde_json::json!(context.complexity_score),
            "urgency" => serde_json::json!(format!("{:?}", context.urgency)),
            "contains_code" => serde_json::json!(context.content_analysis.contains_code),
            "contains_math" => serde_json::json!(context.content_analysis.contains_math),
            "requires_reasoning" => serde_json::json!(context.content_analysis.requires_reasoning),
            "has_images" => serde_json::json!(context.content_analysis.has_images),
            "cost_conscious" => serde_json::json!(context.user_preferences.cost_conscious),
            "quality_over_speed" => serde_json::json!(context.user_preferences.quality_over_speed),
            _ => serde_json::Value::Null,
        }
    }

    fn context_to_requirements(&self, context: &IntelligentRoutingContext) -> TaskRequirements {
        let priority = match context.urgency {
            Urgency::Critical => TaskPriority::Speed,
            Urgency::High => TaskPriority::Balanced,
            Urgency::Normal => {
                if context.user_preferences.quality_over_speed {
                    TaskPriority::Quality
                } else if context.user_preferences.cost_conscious {
                    TaskPriority::Cost
                } else {
                    TaskPriority::Balanced
                }
            }
            Urgency::Low => TaskPriority::Quality,
        };

        let use_cases = match context.task_type {
            TaskType::CodeGeneration => vec!["code_generation".to_string(), "complex_reasoning".to_string()],
            TaskType::ComplexReasoning => vec!["complex_reasoning".to_string(), "analysis".to_string()],
            TaskType::Analysis => vec!["analysis".to_string(), "research".to_string()],
            TaskType::CreativeWriting => vec!["creative_writing".to_string()],
            TaskType::QuickResponse => vec!["quick_responses".to_string(), "simple_tasks".to_string()],
            TaskType::Research => vec!["research".to_string(), "analysis".to_string()],
            TaskType::Multimodal => vec!["multimodal".to_string()],
            _ => vec!["general_tasks".to_string()],
        };

        TaskRequirements {
            priority,
            min_context_length: if context.content_analysis.estimated_tokens > 50000 { 100000 } else { 8000 },
            min_output_tokens: if context.complexity_score > 0.7 { 4000 } else { 1000 },
            needs_vision: context.content_analysis.has_images,
            needs_function_calling: matches!(context.task_type, TaskType::CodeGeneration | TaskType::Analysis),
            preferred_provider: context.user_preferences.preferred_provider.clone(),
            use_cases,
        }
    }

    fn apply_fallback_strategy(&self, _context: &IntelligentRoutingContext, _targets: &[Target]) -> RouterResult<String> {
        let models = match self.fallback_strategy {
            FallbackStrategy::BestQuality => {
                // Return the highest reasoning score models
                let mut models = self.model_registry.get_all_models();
                models.sort_by(|a, b| b.capabilities.reasoning_score.cmp(&a.capabilities.reasoning_score));
                models
            }
            FallbackStrategy::FastestResponse => {
                // Return the fastest models
                let mut models = self.model_registry.get_all_models();
                models.sort_by(|a, b| b.capabilities.speed_score.cmp(&a.capabilities.speed_score));
                models
            }
            FallbackStrategy::MostCostEffective => {
                // Return the most cost-effective models
                let mut models = self.model_registry.get_all_models();
                models.sort_by(|a, b| b.capabilities.cost_efficiency.cmp(&a.capabilities.cost_efficiency));
                models
            }
            FallbackStrategy::Balanced => {
                // Return balanced models (good across all metrics)
                let mut models = self.model_registry.get_all_models();
                models.sort_by(|a, b| {
                    let score_a = (a.capabilities.reasoning_score + a.capabilities.speed_score + a.capabilities.cost_efficiency) as f64 / 3.0;
                    let score_b = (b.capabilities.reasoning_score + b.capabilities.speed_score + b.capabilities.cost_efficiency) as f64 / 3.0;
                    score_b.partial_cmp(&score_a).unwrap()
                });
                models
            }
        };

        if let Some(model) = models.first() {
            Ok(model.id.clone())
        } else {
            Err(RouterError::NoValidTarget)
        }
    }

    /// Analyze content to determine task type and complexity
    pub fn analyze_content(&self, content: &str) -> IntelligentRoutingContext {
        let content_lower = content.to_lowercase();
        
        // Detect task type (more aggressive detection)
        let task_type = if content_lower.contains("theoretical") || content_lower.contains("quantum") || 
                          content_lower.contains("mathematical") || content_lower.contains("formal proof") ||
                          content_lower.contains("computational complexity") || content_lower.contains("cryptographic") ||
                          content_lower.contains("distributed consensus") || content_lower.contains("algorithm") {
            TaskType::ComplexReasoning
        } else if content_lower.contains("rust") || content_lower.contains("cargo") || content_lower.contains("fn ") ||
                  content_lower.contains("implement") || content_lower.contains("concurrent") ||
                  content_lower.contains("data structure") || content_lower.contains("performance") {
            TaskType::CodeGeneration
        } else if content_lower.contains("analyze") || content_lower.contains("explain") || content_lower.contains("compare") ||
                  content_lower.contains("framework") || content_lower.contains("comprehensive") {
            TaskType::Analysis
        } else if content_lower.contains("write") || content_lower.contains("create") || content_lower.contains("story") {
            TaskType::CreativeWriting
        } else if content_lower.contains("research") || content_lower.contains("find") || content_lower.contains("investigate") {
            TaskType::Research
        } else if content.len() < 50 {
            TaskType::QuickResponse
        } else {
            // Default to Analysis for substantial content
            TaskType::Analysis
        };

        // Calculate complexity score
        let complexity_score = self.calculate_complexity_score(content);

        // Analyze content
        let content_analysis = ContentAnalysis {
            estimated_tokens: (content.len() / 4) as u32, // Rough estimate
            language: Some("en".to_string()), // Default to English
            contains_code: content.contains("fn ") || content.contains("impl ") || content.contains("struct "),
            contains_math: content.contains("=") || content.contains("+") || content.contains("*"),
            requires_reasoning: content_lower.contains("why") || content_lower.contains("how") || content_lower.contains("explain"),
            has_images: false, // Would need to check for image attachments
        };

        IntelligentRoutingContext {
            task_type,
            complexity_score,
            urgency: Urgency::Normal,
            budget_constraint: None,
            content_analysis,
            user_preferences: UserPreferences {
                preferred_provider: None,
                max_latency_ms: None,
                quality_over_speed: complexity_score > 0.7,
                cost_conscious: false,
            },
        }
    }

    fn calculate_complexity_score(&self, content: &str) -> f64 {
        let mut score: f64 = 0.0;
        let content_lower = content.to_lowercase();
        
        // Base complexity for any substantial content
        if content.len() > 100 {
            score += 0.3; // Start with higher base score
        }
        if content.len() > 500 {
            score += 0.2;
        }
        if content.len() > 1000 {
            score += 0.2;
        }

        // High-complexity technical keywords (more aggressive scoring)
        let ultra_complex_keywords = [
            "quantum", "cryptographic", "distributed", "consensus", "theoretical",
            "mathematical", "formal proof", "computational complexity", "algorithm",
            "enterprise", "concurrent", "lock-free", "high-performance", "scalable",
            "optimization", "architecture", "sophisticated", "advanced", "framework",
            "comprehensive", "analysis", "research", "investigate", "implement"
        ];

        for keyword in &ultra_complex_keywords {
            if content_lower.contains(keyword) {
                score += 0.15; // Higher weight for complex keywords
            }
        }

        // Programming/technical indicators
        let technical_keywords = [
            "rust", "algorithm", "data structure", "performance", "memory",
            "thread", "async", "concurrent", "parallel", "optimization",
            "benchmark", "profiling", "debugging", "testing", "deployment"
        ];

        for keyword in &technical_keywords {
            if content_lower.contains(keyword) {
                score += 0.12;
            }
        }

        // Academic/research indicators
        let academic_keywords = [
            "analyze", "analysis", "research", "study", "investigate", "examine",
            "compare", "evaluate", "assess", "theoretical", "empirical", "methodology",
            "framework", "model", "hypothesis", "conclusion", "implications"
        ];

        for keyword in &academic_keywords {
            if content_lower.contains(keyword) {
                score += 0.1;
            }
        }

        // Code complexity indicators
        if content.contains("fn ") || content.contains("impl ") || content.contains("struct ") {
            score += 0.25;
        }

        // Reasoning and explanation indicators
        if content_lower.contains("why") || content_lower.contains("how") || content_lower.contains("explain") {
            score += 0.15;
        }

        // Multiple technical domains (interdisciplinary complexity)
        let domains = [
            "computer science", "mathematics", "physics", "engineering",
            "cryptography", "security", "networking", "databases", "ai", "ml"
        ];
        let domain_count = domains.iter().filter(|&domain| content_lower.contains(domain)).count();
        if domain_count > 1 {
            score += 0.2 * domain_count as f64;
        }

        // Bias toward complexity for any non-trivial request
        if score > 0.2 {
            score += 0.3; // Add complexity bias
        }

        score.min(1.0) // Cap at 1.0
    }

    /// Get recommended models for your complex tasks
    pub fn get_recommended_models_for_complex_tasks(&self) -> Vec<String> {
        let complex_models = self.model_registry.get_complex_task_models();
        complex_models.iter().map(|m| m.id.clone()).collect()
    }
}

impl Default for IntelligentRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intelligent_router_creation() {
        let router = IntelligentRouter::new();
        assert!(!router.routing_rules.is_empty());
    }

    #[test]
    fn test_content_analysis() {
        let router = IntelligentRouter::new();
        
        // Test Rust code detection
        let rust_content = "fn main() { println!('Hello, world!'); }";
        let context = router.analyze_content(rust_content);
        assert!(matches!(context.task_type, TaskType::CodeGeneration));
        assert!(context.content_analysis.contains_code);

        // Test analysis task detection
        let analysis_content = "Please analyze the performance implications of this approach";
        let context = router.analyze_content(analysis_content);
        assert!(matches!(context.task_type, TaskType::Analysis));
    }

    #[test]
    fn test_complexity_scoring() {
        let router = IntelligentRouter::new();
        
        let simple_content = "Hello";
        let simple_context = router.analyze_content(simple_content);
        assert!(simple_context.complexity_score < 0.3);

        let complex_content = "Implement a sophisticated concurrent algorithm for optimizing enterprise-scale performance with advanced memory management";
        let complex_context = router.analyze_content(complex_content);
        assert!(complex_context.complexity_score > 0.7);
    }

    #[test]
    fn test_model_recommendations() {
        let router = IntelligentRouter::new();
        let recommended = router.get_recommended_models_for_complex_tasks();
        
        assert!(!recommended.is_empty());
        assert!(recommended.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(recommended.contains(&"gpt-4o".to_string()));
    }
}