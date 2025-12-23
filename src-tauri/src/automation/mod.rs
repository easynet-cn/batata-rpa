pub mod desktop;
pub mod file;
pub mod highlight;
pub mod web;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum AutomationError {
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Operation timeout: {0}")]
    Timeout(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

pub type AutomationResult<T> = Result<T, AutomationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClickType {
    Single,
    Double,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputMethod {
    Type,
    Set,
}
