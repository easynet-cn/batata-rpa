pub mod executor;
pub mod runtime;
pub mod variable;

use crate::automation::AutomationError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum EngineError {
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Variable error: {0}")]
    VariableError(String),

    #[error("Automation error: {0}")]
    AutomationError(#[from] AutomationError),
}

pub type EngineResult<T> = Result<T, EngineError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub id: String,
    pub timestamp: String,
    pub level: LogLevel,
    pub node_id: Option<String>,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl ExecutionLog {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: LogLevel::Info,
            node_id: None,
            message: message.into(),
            details: None,
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: LogLevel::Warn,
            node_id: None,
            message: message.into(),
            details: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: LogLevel::Error,
            node_id: None,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}
