use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<VariableValue>),
    Dict(HashMap<String, VariableValue>),
}

impl VariableValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            VariableValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            VariableValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            VariableValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_string_value(&self) -> String {
        match self {
            VariableValue::Null => "null".to_string(),
            VariableValue::Bool(b) => b.to_string(),
            VariableValue::Number(n) => n.to_string(),
            VariableValue::String(s) => s.clone(),
            VariableValue::List(l) => serde_json::to_string(l).unwrap_or_default(),
            VariableValue::Dict(d) => serde_json::to_string(d).unwrap_or_default(),
        }
    }
}

impl From<&str> for VariableValue {
    fn from(s: &str) -> Self {
        VariableValue::String(s.to_string())
    }
}

impl From<String> for VariableValue {
    fn from(s: String) -> Self {
        VariableValue::String(s)
    }
}

impl From<i32> for VariableValue {
    fn from(n: i32) -> Self {
        VariableValue::Number(n as f64)
    }
}

impl From<f64> for VariableValue {
    fn from(n: f64) -> Self {
        VariableValue::Number(n)
    }
}

impl From<bool> for VariableValue {
    fn from(b: bool) -> Self {
        VariableValue::Bool(b)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: VariableValue,
    pub scope: VariableScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableScope {
    Global,
    Local,
}

#[derive(Debug, Default)]
pub struct VariableStore {
    variables: HashMap<String, Variable>,
}

impl VariableStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: impl Into<String>, value: VariableValue, scope: VariableScope) {
        let name = name.into();
        self.variables.insert(
            name.clone(),
            Variable {
                name,
                value,
                scope,
            },
        );
    }

    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name).map(|v| &v.value)
    }

    pub fn remove(&mut self, name: &str) -> Option<Variable> {
        self.variables.remove(name)
    }

    pub fn clear_local(&mut self) {
        self.variables
            .retain(|_, v| v.scope == VariableScope::Global);
    }

    pub fn all(&self) -> &HashMap<String, Variable> {
        &self.variables
    }

    /// Get all variables as a simple name -> value map (for debugging)
    pub fn get_all(&self) -> HashMap<String, VariableValue> {
        self.variables
            .iter()
            .map(|(name, var)| (name.clone(), var.value.clone()))
            .collect()
    }

    pub fn interpolate(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (name, var) in &self.variables {
            let placeholder = format!("${{{}}}", name);
            result = result.replace(&placeholder, &var.value.to_string_value());
        }
        result
    }
}
