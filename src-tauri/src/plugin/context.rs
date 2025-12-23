//! Plugin execution context
//!
//! Provides APIs for plugins to interact with the RPA runtime

use crate::engine::{ExecutionLog, runtime::Runtime};
use crate::engine::variable::VariableValue;
use std::sync::Arc;
use mlua::{UserData, UserDataMethods, Lua, Result as LuaResult, Value, MultiValue};

/// Context passed to plugin node handlers
/// Provides safe access to runtime functionality
pub struct PluginContext {
    runtime: Arc<Runtime>,
    node_id: String,
    node_data: serde_json::Value,
}

impl PluginContext {
    pub fn new(runtime: Arc<Runtime>, node_id: String, node_data: serde_json::Value) -> Self {
        Self {
            runtime,
            node_id,
            node_data,
        }
    }

    /// Get a value from node data
    pub fn get_data(&self, key: &str) -> Option<serde_json::Value> {
        self.node_data.get(key).cloned()
    }

    /// Get a variable value
    pub async fn get_variable(&self, name: &str) -> Option<String> {
        self.runtime.get_variable(name).await.map(|v| match v {
            VariableValue::String(s) => s,
            VariableValue::Number(n) => n.to_string(),
            VariableValue::Bool(b) => b.to_string(),
            VariableValue::List(l) => serde_json::to_string(&l).unwrap_or_default(),
            VariableValue::Dict(d) => serde_json::to_string(&d).unwrap_or_default(),
            VariableValue::Null => String::new(),
        })
    }

    /// Set a variable value
    pub async fn set_variable(&self, name: &str, value: &str) {
        self.runtime
            .set_variable(name, VariableValue::String(value.to_string()))
            .await;
    }

    /// Interpolate variables in a string
    pub async fn interpolate(&self, text: &str) -> String {
        self.runtime.interpolate(text).await
    }

    /// Add a log message
    pub async fn log(&self, level: &str, message: &str) {
        let log = match level {
            "error" => ExecutionLog::error(message.to_string()),
            "warn" => ExecutionLog::warn(message.to_string()),
            _ => ExecutionLog::info(message.to_string()),
        };
        self.runtime.add_log(log.with_node(&self.node_id)).await;
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

/// Lua userdata wrapper for synchronous context access
pub struct LuaContext {
    pub runtime: Arc<Runtime>,
    pub node_id: String,
    pub node_data: serde_json::Value,
}

impl UserData for LuaContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Get data from node configuration
        methods.add_method("get_data", |lua, this, key: String| {
            let value = this.node_data.get(&key);
            match value {
                Some(v) => json_to_lua(lua, v),
                None => Ok(Value::Nil),
            }
        });

        // Get all node data
        methods.add_method("get_all_data", |lua, this, ()| {
            json_to_lua(lua, &this.node_data)
        });

        // Get variable (blocking)
        methods.add_method("get_variable", |_lua, this, name: String| {
            let runtime = this.runtime.clone();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.get_variable(&name).await
                })
            });
            Ok(result.map(|v| match v {
                VariableValue::String(s) => s,
                VariableValue::Number(n) => n.to_string(),
                VariableValue::Bool(b) => b.to_string(),
                VariableValue::List(l) => serde_json::to_string(&l).unwrap_or_default(),
                VariableValue::Dict(d) => serde_json::to_string(&d).unwrap_or_default(),
                VariableValue::Null => String::new(),
            }))
        });

        // Set variable (blocking)
        methods.add_method("set_variable", |_lua, this, (name, value): (String, String)| {
            let runtime = this.runtime.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.set_variable(&name, VariableValue::String(value)).await;
                })
            });
            Ok(())
        });

        // Set variable with different types
        methods.add_method("set_number", |_lua, this, (name, value): (String, f64)| {
            let runtime = this.runtime.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.set_variable(&name, VariableValue::Number(value)).await;
                })
            });
            Ok(())
        });

        methods.add_method("set_boolean", |_lua, this, (name, value): (String, bool)| {
            let runtime = this.runtime.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.set_variable(&name, VariableValue::Bool(value)).await;
                })
            });
            Ok(())
        });

        // Interpolate variables in string
        methods.add_method("interpolate", |_lua, this, text: String| {
            let runtime = this.runtime.clone();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.interpolate(&text).await
                })
            });
            Ok(result)
        });

        // Log messages
        methods.add_method("log", |_lua, this, (level, message): (String, String)| {
            let runtime = this.runtime.clone();
            let node_id = this.node_id.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let log = match level.as_str() {
                        "error" => ExecutionLog::error(message),
                        "warn" => ExecutionLog::warn(message),
                        _ => ExecutionLog::info(message),
                    };
                    runtime.add_log(log.with_node(&node_id)).await;
                })
            });
            Ok(())
        });

        // Shorthand log methods
        methods.add_method("info", |_lua, this, message: String| {
            let runtime = this.runtime.clone();
            let node_id = this.node_id.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.add_log(ExecutionLog::info(message).with_node(&node_id)).await;
                })
            });
            Ok(())
        });

        methods.add_method("warn", |_lua, this, message: String| {
            let runtime = this.runtime.clone();
            let node_id = this.node_id.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.add_log(ExecutionLog::warn(message).with_node(&node_id)).await;
                })
            });
            Ok(())
        });

        methods.add_method("error", |_lua, this, message: String| {
            let runtime = this.runtime.clone();
            let node_id = this.node_id.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    runtime.add_log(ExecutionLog::error(message).with_node(&node_id)).await;
                })
            });
            Ok(())
        });

        // Execute shell command
        methods.add_method("execute_command", |_lua, _this, (cmd, args): (String, Vec<String>)| {
            let output = std::process::Command::new(&cmd)
                .args(&args)
                .output()
                .map_err(|e| mlua::Error::external(e))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let success = output.status.success();

            Ok((stdout, stderr, success))
        });

        // Read file
        methods.add_method("read_file", |_lua, _this, path: String| {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| mlua::Error::external(e))?;
            Ok(content)
        });

        // Write file
        methods.add_method("write_file", |_lua, _this, (path, content): (String, String)| {
            std::fs::write(&path, &content)
                .map_err(|e| mlua::Error::external(e))?;
            Ok(())
        });

        // Sleep (milliseconds)
        methods.add_method("sleep", |_lua, _this, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        });

        // Get current timestamp
        methods.add_method("now", |_lua, _this, ()| {
            Ok(chrono::Utc::now().to_rfc3339())
        });

        // Get node ID
        methods.add_method("node_id", |_lua, this, ()| {
            Ok(this.node_id.clone())
        });
    }
}

/// Convert serde_json::Value to Lua Value
fn json_to_lua(lua: &Lua, value: &serde_json::Value) -> LuaResult<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                table.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

/// Convert Lua Value to serde_json::Value
pub fn lua_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Nil => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(b),
        Value::Integer(i) => serde_json::Value::Number(i.into()),
        Value::Number(n) => serde_json::Number::from_f64(n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.to_str().map(|s| s.to_string()).unwrap_or_default()),
        Value::Table(t) => {
            // Check if it's an array (sequential integer keys starting from 1)
            let mut is_array = true;
            let mut max_key = 0i64;

            for pair in t.clone().pairs::<Value, Value>() {
                if let Ok((k, _)) = pair {
                    if let Value::Integer(i) = k {
                        if i > 0 {
                            max_key = max_key.max(i);
                            continue;
                        }
                    }
                    is_array = false;
                    break;
                }
            }

            if is_array && max_key > 0 {
                let mut arr = Vec::new();
                for i in 1..=max_key {
                    if let Ok(v) = t.get::<Value>(i) {
                        arr.push(lua_to_json(v));
                    }
                }
                serde_json::Value::Array(arr)
            } else {
                let mut obj = serde_json::Map::new();
                for pair in t.pairs::<String, Value>() {
                    if let Ok((k, v)) = pair {
                        obj.insert(k, lua_to_json(v));
                    }
                }
                serde_json::Value::Object(obj)
            }
        }
        _ => serde_json::Value::Null,
    }
}
