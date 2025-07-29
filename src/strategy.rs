use crate::{RouterError, RouterResult, Target, RouterContext, Condition};
use rand::Rng;
use regex::Regex;
use serde_json::Value;


#[derive(Debug, Clone)]
pub enum Operator {
    // Comparison Operators
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,
    NotIn,
    Regex,
    // Logical Operators
    And,
    Or,
}

impl Operator {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "$eq" => Some(Self::Equal),
            "$ne" => Some(Self::NotEqual),
            "$gt" => Some(Self::GreaterThan),
            "$gte" => Some(Self::GreaterThanOrEqual),
            "$lt" => Some(Self::LessThan),
            "$lte" => Some(Self::LessThanOrEqual),
            "$in" => Some(Self::In),
            "$nin" => Some(Self::NotIn),
            "$regex" => Some(Self::Regex),
            "$and" => Some(Self::And),
            "$or" => Some(Self::Or),
            _ => None,
        }
    }
}

pub struct ConditionalRouter<'a> {
    targets: &'a [Target],
    context: &'a RouterContext,
}

impl<'a> ConditionalRouter<'a> {
    pub fn new(targets: &'a [Target], context: &'a RouterContext) -> Self {
        Self { targets, context }
    }

    pub fn resolve_target(&self, conditions: &[Condition], default_target: Option<&str>) -> RouterResult<&'a Target> {
        if conditions.is_empty() {
            return Err(RouterError::NoConditions);
        }

        for condition in conditions {
            if self.evaluate_query(&condition.query)? {
                return self.find_target(&condition.then_target);
            }
        }

        // If no conditions matched and a default is specified, return the default target
        if let Some(default) = default_target {
            return self.find_target(default);
        }

        Err(RouterError::NoValidTarget)
    }

    fn evaluate_query(&self, query: &Value) -> RouterResult<bool> {
        let query_obj = query.as_object()
            .ok_or_else(|| RouterError::InvalidQuery { 
                message: "Query must be an object".to_string() 
            })?;

        for (key, value) in query_obj {
            if key == "$or" {
                if let Some(conditions) = value.as_array() {
                    let mut any_match = false;
                    for condition in conditions {
                        if self.evaluate_query(condition)? {
                            any_match = true;
                            break;
                        }
                    }
                    if !any_match {
                        return Ok(false);
                    }
                }
                continue;
            }

            if key == "$and" {
                if let Some(conditions) = value.as_array() {
                    for condition in conditions {
                        if !self.evaluate_query(condition)? {
                            return Ok(false);
                        }
                    }
                }
                continue;
            }

            let context_value = self.get_context_value(key);

            if let Some(operator_obj) = value.as_object() {
                if !self.evaluate_operator(operator_obj, &context_value)? {
                    return Ok(false);
                }
            } else if context_value != *value {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn evaluate_operator(&self, operator_obj: &serde_json::Map<String, Value>, context_value: &Value) -> RouterResult<bool> {
        for (op_str, compare_value) in operator_obj {
            let operator = Operator::from_str(op_str)
                .ok_or_else(|| RouterError::UnsupportedOperator { 
                    operator: op_str.clone() 
                })?;

            match operator {
                Operator::Equal => {
                    if context_value != compare_value {
                        return Ok(false);
                    }
                }
                Operator::NotEqual => {
                    if context_value == compare_value {
                        return Ok(false);
                    }
                }
                Operator::GreaterThan => {
                    if !self.compare_numbers(context_value, compare_value, |a, b| a > b)? {
                        return Ok(false);
                    }
                }
                Operator::GreaterThanOrEqual => {
                    if !self.compare_numbers(context_value, compare_value, |a, b| a >= b)? {
                        return Ok(false);
                    }
                }
                Operator::LessThan => {
                    if !self.compare_numbers(context_value, compare_value, |a, b| a < b)? {
                        return Ok(false);
                    }
                }
                Operator::LessThanOrEqual => {
                    if !self.compare_numbers(context_value, compare_value, |a, b| a <= b)? {
                        return Ok(false);
                    }
                }
                Operator::In => {
                    if let Some(array) = compare_value.as_array() {
                        if !array.contains(context_value) {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
                Operator::NotIn => {
                    if let Some(array) = compare_value.as_array() {
                        if array.contains(context_value) {
                            return Ok(false);
                        }
                    }
                }
                Operator::Regex => {
                    if let (Some(pattern), Some(text)) = (compare_value.as_str(), context_value.as_str()) {
                        let regex = Regex::new(pattern)?;
                        if !regex.is_match(text) {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
                _ => {
                    return Err(RouterError::UnsupportedOperator { 
                        operator: op_str.clone() 
                    });
                }
            }
        }
        Ok(true)
    }

    fn compare_numbers<F>(&self, a: &Value, b: &Value, op: F) -> RouterResult<bool>
    where
        F: Fn(f64, f64) -> bool,
    {
        let a_num = a.as_f64()
            .or_else(|| a.as_str().and_then(|s| s.parse().ok()))
            .unwrap_or(0.0);
        let b_num = b.as_f64()
            .or_else(|| b.as_str().and_then(|s| s.parse().ok()))
            .unwrap_or(0.0);
        
        Ok(op(a_num, b_num))
    }

    fn find_target(&self, name: &str) -> RouterResult<&'a Target> {
        self.targets
            .iter()
            .find(|target| target.name == name)
            .ok_or_else(|| RouterError::InvalidTargetName { 
                name: name.to_string() 
            })
    }

    fn get_context_value(&self, key: &str) -> Value {
        let parts: Vec<&str> = key.split('.').collect();
        
        if parts.len() >= 2 {
            match parts[0] {
                "metadata" => {
                    if let Some(value) = self.context.metadata.get(parts[1]) {
                        return Value::String(value.clone());
                    }
                }
                "params" => {
                    if let Some(value) = self.context.params.get(parts[1]) {
                        return value.clone();
                    }
                }
                _ => {}
            }
        }
        
        Value::Null
    }
}

pub struct LoadBalancer;

impl LoadBalancer {
    pub fn select_target(targets: &[Target]) -> RouterResult<&Target> {
        if targets.is_empty() {
            return Err(RouterError::NoTargets);
        }

        // Calculate total weight
        let total_weight: u32 = targets.iter()
            .map(|t| t.weight.unwrap_or(1))
            .sum();

        if total_weight == 0 {
            return Ok(&targets[0]);
        }

        // Generate random number
        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0..total_weight);

        // Select target based on weight
        for target in targets {
            let weight = target.weight.unwrap_or(1);
            if random_weight < weight {
                return Ok(target);
            }
            random_weight -= weight;
        }

        // Fallback to first target (shouldn't happen)
        Ok(&targets[0])
    }
}