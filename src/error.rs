use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("No targets configured")]
    NoTargets,
    
    #[error("No conditions passed in conditional router")]
    NoConditions,
    
    #[error("Conditional router did not resolve to any valid target")]
    NoValidTarget,
    
    #[error("Invalid target name found in router: {name}")]
    InvalidTargetName { name: String },
    
    #[error("Unsupported operator used in router: {operator}")]
    UnsupportedOperator { operator: String },
    
    #[error("Invalid query format: {message}")]
    InvalidQuery { message: String },
    
    #[error("Regex compilation failed: {source}")]
    RegexError {
        #[from]
        source: regex::Error,
    },
    
    #[error("JSON parsing failed: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
}

pub type RouterResult<T> = Result<T, RouterError>;