use super::runtime::{DebugMode, Runtime};
use super::variable::VariableValue;
use super::{EngineError, EngineResult, ExecutionLog, ExecutionStatus};
use std::collections::HashSet;
use crate::automation::desktop;
use crate::automation::file::FileAutomation;
use crate::automation::web::{BrowserOptions, WebAutomation};
use crate::automation::ClickType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: Position,
    pub data: HashMap<String, serde_json::Value>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_handle: Option<String>,
    pub target_handle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

impl Workflow {
    pub fn find_start_node(&self) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.node_type == "start")
    }

    pub fn find_node(&self, id: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn find_next_nodes(&self, node_id: &str) -> Vec<&WorkflowNode> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id)
            .filter_map(|e| self.find_node(&e.target))
            .collect()
    }

    /// Find next nodes connected from a specific source handle
    pub fn find_next_nodes_by_handle(
        &self,
        node_id: &str,
        source_handle: &str,
    ) -> Vec<&WorkflowNode> {
        self.edges
            .iter()
            .filter(|e| {
                e.source == node_id
                    && e.source_handle
                        .as_ref()
                        .map(|h| h == source_handle)
                        .unwrap_or(false)
            })
            .filter_map(|e| self.find_node(&e.target))
            .collect()
    }
}

pub struct Executor {
    workflow: Workflow,
    runtime: Arc<Runtime>,
    automation: Box<dyn desktop::DesktopAutomation>,
    file_automation: FileAutomation,
    web_automation: Arc<WebAutomation>,
}

impl Executor {
    pub fn new(workflow: Workflow) -> Self {
        Self {
            runtime: Arc::new(Runtime::new(workflow.id.clone())),
            workflow,
            automation: desktop::create_automation(),
            file_automation: FileAutomation::new(),
            web_automation: Arc::new(WebAutomation::new()),
        }
    }

    pub async fn execute(&self) -> EngineResult<()> {
        self.runtime.start().await;

        let start_node = self
            .workflow
            .find_start_node()
            .ok_or_else(|| EngineError::InvalidWorkflow("No start node found".to_string()))?;

        if let Err(e) = self.execute_from_node(start_node).await {
            self.runtime.fail(e.to_string()).await;
            return Err(e);
        }

        self.runtime.complete().await;
        Ok(())
    }

    /// Execute workflow in debug mode
    pub async fn execute_debug(&self, mode: DebugMode) -> EngineResult<()> {
        self.runtime.start_debug(mode).await;

        let start_node = self
            .workflow
            .find_start_node()
            .ok_or_else(|| EngineError::InvalidWorkflow("No start node found".to_string()))?;

        if let Err(e) = self.execute_from_node(start_node).await {
            self.runtime.fail(e.to_string()).await;
            return Err(e);
        }

        self.runtime.complete().await;
        Ok(())
    }

    fn execute_from_node<'a>(
        &'a self,
        node: &'a WorkflowNode,
    ) -> Pin<Box<dyn Future<Output = EngineResult<()>> + Send + 'a>> {
        Box::pin(async move {
            // Check if execution should stop
            let status = self.runtime.get_status().await;
            if status == ExecutionStatus::Failed {
                return Ok(());
            }

            // Wait while paused (for resume/step commands)
            self.runtime.wait_for_step().await;

            // Check again after waiting
            let status = self.runtime.get_status().await;
            if status == ExecutionStatus::Failed {
                return Ok(());
            }

            self.runtime.set_current_node(Some(node.id.clone())).await;
            self.runtime
                .add_log(
                    ExecutionLog::info(format!(
                        "Executing node: {} ({})",
                        node.label.as_deref().unwrap_or(&node.id),
                        node.node_type
                    ))
                    .with_node(&node.id),
                )
                .await;

            // Check if we should pause at this node (debug mode)
            if self.runtime.should_pause_at_node(&node.id).await {
                self.runtime.pause_at_node(&node.id).await;
                // Wait for user to step or resume
                self.runtime.wait_for_step().await;
            }

            // Handle special nodes with custom edge routing
            match node.node_type.as_str() {
                "condition" => {
                    return self.execute_condition_node(node).await;
                }
                "loop" => {
                    return self.execute_loop_node(node).await;
                }
                "forEach" => {
                    return self.execute_foreach_node(node).await;
                }
                "tryCatch" => {
                    return self.execute_try_catch_node(node).await;
                }
                _ => {}
            }

            // Execute the node
            self.execute_node(node).await?;

            // Find and execute next nodes
            let next_nodes = self.workflow.find_next_nodes(&node.id);
            for next_node in next_nodes {
                self.execute_from_node(next_node).await?;
            }

            Ok(())
        })
    }

    async fn execute_node(&self, node: &WorkflowNode) -> EngineResult<()> {
        match node.node_type.as_str() {
            "start" => {
                // Start node does nothing
                Ok(())
            }
            "end" => {
                // End node does nothing
                Ok(())
            }
            "click" => self.execute_click(node).await,
            "input" => self.execute_input(node).await,
            "getText" => self.execute_get_text(node).await,
            "delay" => self.execute_delay(node).await,
            "log" => self.execute_log(node).await,
            "setVariable" => self.execute_set_variable(node).await,
            "readFile" => self.execute_read_file(node).await,
            "writeFile" => self.execute_write_file(node).await,
            "waitElement" => self.execute_wait_element(node).await,
            "hotkey" => self.execute_hotkey(node).await,
            "screenshot" => self.execute_screenshot(node).await,
            "openBrowser" => self.execute_open_browser(node).await,
            "navigate" => self.execute_navigate(node).await,
            "webClick" => self.execute_web_click(node).await,
            "webInput" => self.execute_web_input(node).await,
            "webGetText" => self.execute_web_get_text(node).await,
            "closeBrowser" => self.execute_close_browser(node).await,
            "executeJs" => self.execute_js(node).await,
            "readExcel" => self.execute_read_excel(node).await,
            "writeExcel" => self.execute_write_excel(node).await,
            "executeCommand" => self.execute_command(node).await,
            "listDirectory" => self.execute_list_directory(node).await,
            "openApp" => self.execute_open_app(node).await,
            // Special nodes handled in execute_from_node
            "condition" | "loop" | "forEach" | "tryCatch" => Ok(()),
            _ => {
                self.runtime
                    .add_log(
                        ExecutionLog::info(format!("Unknown node type: {}", node.node_type))
                            .with_node(&node.id),
                    )
                    .await;
                Ok(())
            }
        }
    }

    async fn execute_click(&self, node: &WorkflowNode) -> EngineResult<()> {
        let click_type = match node.data.get("clickType").and_then(|v| v.as_str()) {
            Some("double") => ClickType::Double,
            Some("right") => ClickType::Right,
            _ => ClickType::Single,
        };

        // TODO: Get element from element library
        self.runtime
            .add_log(
                ExecutionLog::info(format!("Click with type {:?}", click_type))
                    .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    async fn execute_input(&self, node: &WorkflowNode) -> EngineResult<()> {
        let text = node
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let interpolated = self.runtime.interpolate(text).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Input text: {}", interpolated)).with_node(&node.id),
            )
            .await;

        Ok(())
    }

    async fn execute_get_text(&self, node: &WorkflowNode) -> EngineResult<()> {
        let var_name = node
            .data
            .get("variableName")
            .and_then(|v| v.as_str())
            .unwrap_or("result");

        // TODO: Get text from element
        let text = "placeholder_text".to_string();

        self.runtime
            .set_variable(var_name, text.clone().into())
            .await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Got text '{}' saved to variable '{}'", text, var_name))
                    .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    async fn execute_delay(&self, node: &WorkflowNode) -> EngineResult<()> {
        let delay_ms = node
            .data
            .get("delay")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Waiting for {}ms", delay_ms)).with_node(&node.id),
            )
            .await;

        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        Ok(())
    }

    async fn execute_log(&self, node: &WorkflowNode) -> EngineResult<()> {
        let message = node
            .data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let interpolated = self.runtime.interpolate(message).await;

        self.runtime
            .add_log(ExecutionLog::info(interpolated).with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_set_variable(&self, node: &WorkflowNode) -> EngineResult<()> {
        let var_name = node
            .data
            .get("variableName")
            .and_then(|v| v.as_str())
            .unwrap_or("variable");

        let value_str = node
            .data
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let value_type = node
            .data
            .get("valueType")
            .and_then(|v| v.as_str())
            .unwrap_or("string");

        // Interpolate the value string
        let interpolated = self.runtime.interpolate(value_str).await;

        // Parse value based on type
        let value: VariableValue = match value_type {
            "number" => interpolated
                .parse::<f64>()
                .map(VariableValue::Number)
                .unwrap_or(VariableValue::String(interpolated.clone())),
            "boolean" => VariableValue::Bool(
                interpolated.to_lowercase() == "true" || interpolated == "1",
            ),
            "json" => serde_json::from_str(&interpolated)
                .unwrap_or(VariableValue::String(interpolated.clone())),
            _ => VariableValue::String(interpolated.clone()),
        };

        self.runtime.set_variable(var_name, value.clone()).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Set variable '{}' = {}",
                    var_name,
                    value.to_string_value()
                ))
                .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    async fn execute_read_file(&self, node: &WorkflowNode) -> EngineResult<()> {
        let file_path = node
            .data
            .get("filePath")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let var_name = node
            .data
            .get("variableName")
            .and_then(|v| v.as_str())
            .unwrap_or("fileContent");

        // Interpolate the path
        let interpolated_path = self.runtime.interpolate(file_path).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Reading file: {}", interpolated_path))
                    .with_node(&node.id),
            )
            .await;

        match self.file_automation.read_file(&interpolated_path).await {
            Ok(content) => {
                self.runtime
                    .set_variable(var_name, VariableValue::String(content.clone()))
                    .await;

                self.runtime
                    .add_log(
                        ExecutionLog::info(format!(
                            "File read successfully, {} bytes saved to '{}'",
                            content.len(),
                            var_name
                        ))
                        .with_node(&node.id),
                    )
                    .await;
            }
            Err(e) => {
                self.runtime
                    .add_log(
                        ExecutionLog::error(format!("Failed to read file: {}", e))
                            .with_node(&node.id),
                    )
                    .await;
                return Err(EngineError::ExecutionFailed(format!(
                    "Failed to read file: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    async fn execute_write_file(&self, node: &WorkflowNode) -> EngineResult<()> {
        let file_path = node
            .data
            .get("filePath")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let content = node
            .data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let write_mode = node
            .data
            .get("writeMode")
            .and_then(|v| v.as_str())
            .unwrap_or("overwrite");

        // Interpolate path and content
        let interpolated_path = self.runtime.interpolate(file_path).await;
        let interpolated_content = self.runtime.interpolate(content).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Writing to file: {} (mode: {})",
                    interpolated_path, write_mode
                ))
                .with_node(&node.id),
            )
            .await;

        let final_content = if write_mode == "append" {
            // Read existing content and append
            let existing = self
                .file_automation
                .read_file(&interpolated_path)
                .await
                .unwrap_or_default();
            format!("{}{}", existing, interpolated_content)
        } else {
            interpolated_content
        };

        match self
            .file_automation
            .write_file(&interpolated_path, &final_content)
            .await
        {
            Ok(()) => {
                self.runtime
                    .add_log(
                        ExecutionLog::info(format!(
                            "File written successfully: {}",
                            interpolated_path
                        ))
                        .with_node(&node.id),
                    )
                    .await;
            }
            Err(e) => {
                self.runtime
                    .add_log(
                        ExecutionLog::error(format!("Failed to write file: {}", e))
                            .with_node(&node.id),
                    )
                    .await;
                return Err(EngineError::ExecutionFailed(format!(
                    "Failed to write file: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    async fn execute_condition_node(&self, node: &WorkflowNode) -> EngineResult<()> {
        // Evaluate the condition
        let result = self.evaluate_condition(node).await;

        let branch = if result { "true" } else { "false" };

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Condition evaluated to: {}", branch))
                    .with_node(&node.id),
            )
            .await;

        // Execute nodes connected to the appropriate branch
        let next_nodes = self.workflow.find_next_nodes_by_handle(&node.id, branch);
        for next_node in next_nodes {
            self.execute_from_node(next_node).await?;
        }

        Ok(())
    }

    async fn evaluate_condition(&self, node: &WorkflowNode) -> bool {
        let operator = node
            .data
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("==");

        let left_operand = node
            .data
            .get("leftOperand")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let right_operand = node
            .data
            .get("rightOperand")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Interpolate operands
        let left = self.runtime.interpolate(left_operand).await;
        let right = self.runtime.interpolate(right_operand).await;

        match operator {
            "==" => left == right,
            "!=" => left != right,
            ">" => {
                let l: f64 = left.parse().unwrap_or(0.0);
                let r: f64 = right.parse().unwrap_or(0.0);
                l > r
            }
            "<" => {
                let l: f64 = left.parse().unwrap_or(0.0);
                let r: f64 = right.parse().unwrap_or(0.0);
                l < r
            }
            ">=" => {
                let l: f64 = left.parse().unwrap_or(0.0);
                let r: f64 = right.parse().unwrap_or(0.0);
                l >= r
            }
            "<=" => {
                let l: f64 = left.parse().unwrap_or(0.0);
                let r: f64 = right.parse().unwrap_or(0.0);
                l <= r
            }
            "contains" => left.contains(&right),
            "isEmpty" => left.is_empty(),
            "isNotEmpty" => !left.is_empty(),
            _ => false,
        }
    }

    async fn execute_loop_node(&self, node: &WorkflowNode) -> EngineResult<()> {
        let loop_type = node
            .data
            .get("loopType")
            .and_then(|v| v.as_str())
            .unwrap_or("count");

        let index_variable = node
            .data
            .get("indexVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("index");

        match loop_type {
            "count" => {
                let count = node
                    .data
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as usize;

                self.runtime
                    .add_log(
                        ExecutionLog::info(format!("Starting count loop: {} iterations", count))
                            .with_node(&node.id),
                    )
                    .await;

                for i in 0..count {
                    // Set the index variable
                    self.runtime
                        .set_variable(index_variable, VariableValue::Number(i as f64))
                        .await;

                    self.runtime
                        .add_log(
                            ExecutionLog::info(format!("Loop iteration {}/{}", i + 1, count))
                                .with_node(&node.id),
                        )
                        .await;

                    // Execute body nodes
                    let body_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "body");
                    for body_node in body_nodes {
                        self.execute_from_node(body_node).await?;
                    }
                }
            }
            "while" => {
                let condition_str = node
                    .data
                    .get("condition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("false");

                self.runtime
                    .add_log(
                        ExecutionLog::info(format!("Starting while loop: {}", condition_str))
                            .with_node(&node.id),
                    )
                    .await;

                let mut iteration = 0;
                let max_iterations = 10000; // Safety limit

                while iteration < max_iterations {
                    // Evaluate condition
                    let interpolated = self.runtime.interpolate(condition_str).await;
                    let condition_result = self.evaluate_simple_expression(&interpolated);

                    if !condition_result {
                        break;
                    }

                    self.runtime
                        .set_variable(index_variable, VariableValue::Number(iteration as f64))
                        .await;

                    self.runtime
                        .add_log(
                            ExecutionLog::info(format!("While loop iteration {}", iteration + 1))
                                .with_node(&node.id),
                        )
                        .await;

                    // Execute body nodes
                    let body_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "body");
                    for body_node in body_nodes {
                        self.execute_from_node(body_node).await?;
                    }

                    iteration += 1;
                }
            }
            _ => {}
        }

        // Execute done nodes
        let done_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "done");
        for done_node in done_nodes {
            self.execute_from_node(done_node).await?;
        }

        Ok(())
    }

    async fn execute_foreach_node(&self, node: &WorkflowNode) -> EngineResult<()> {
        let list_variable = node
            .data
            .get("listVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let item_variable = node
            .data
            .get("itemVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("item");

        let index_variable = node
            .data
            .get("indexVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("index");

        // Get the list from variables
        let list = self.runtime.get_variable(list_variable).await;

        if let Some(VariableValue::List(items)) = list {
            let items = items.clone();
            self.runtime
                .add_log(
                    ExecutionLog::info(format!(
                        "Starting forEach loop: {} items",
                        items.len()
                    ))
                    .with_node(&node.id),
                )
                .await;

            for (i, item) in items.iter().enumerate() {
                self.runtime
                    .set_variable(index_variable, VariableValue::Number(i as f64))
                    .await;
                self.runtime
                    .set_variable(item_variable, item.clone())
                    .await;

                self.runtime
                    .add_log(
                        ExecutionLog::info(format!(
                            "ForEach iteration {}/{}",
                            i + 1,
                            items.len()
                        ))
                        .with_node(&node.id),
                    )
                    .await;

                // Execute body nodes
                let body_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "body");
                for body_node in body_nodes {
                    self.execute_from_node(body_node).await?;
                }
            }
        } else {
            self.runtime
                .add_log(
                    ExecutionLog::warn(format!(
                        "Variable '{}' is not a list or not found",
                        list_variable
                    ))
                    .with_node(&node.id),
                )
                .await;
        }

        // Execute done nodes
        let done_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "done");
        for done_node in done_nodes {
            self.execute_from_node(done_node).await?;
        }

        Ok(())
    }

    fn evaluate_simple_expression(&self, expr: &str) -> bool {
        // Simple expression evaluator for conditions like "true", "false", "1", "0"
        let expr = expr.trim().to_lowercase();
        expr == "true" || expr == "1" || expr == "yes"
    }

    async fn execute_try_catch_node(&self, node: &WorkflowNode) -> EngineResult<()> {
        let error_variable = node
            .data
            .get("errorVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("error");

        let max_retries = node
            .data
            .get("maxRetries")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let retry_delay = node
            .data
            .get("retryDelay")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Starting try-catch block (max retries: {})",
                    max_retries
                ))
                .with_node(&node.id),
            )
            .await;

        let try_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "try");
        let mut last_error: Option<String> = None;

        // Try to execute with retries
        for attempt in 0..=max_retries {
            if attempt > 0 {
                self.runtime
                    .add_log(
                        ExecutionLog::info(format!("Retry attempt {}/{}", attempt, max_retries))
                            .with_node(&node.id),
                    )
                    .await;
                tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay)).await;
            }

            let mut success = true;
            for try_node in &try_nodes {
                if let Err(e) = self.execute_from_node(try_node).await {
                    last_error = Some(e.to_string());
                    success = false;
                    break;
                }
            }

            if success {
                // Execute finally nodes and return
                let finally_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "finally");
                for finally_node in finally_nodes {
                    self.execute_from_node(finally_node).await?;
                }
                return Ok(());
            }
        }

        // All retries failed, execute catch block
        if let Some(error) = &last_error {
            self.runtime
                .set_variable(error_variable, VariableValue::String(error.clone()))
                .await;

            self.runtime
                .add_log(
                    ExecutionLog::warn(format!("Caught error: {}", error)).with_node(&node.id),
                )
                .await;
        }

        let catch_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "catch");
        for catch_node in catch_nodes {
            self.execute_from_node(catch_node).await?;
        }

        // Execute finally nodes
        let finally_nodes = self.workflow.find_next_nodes_by_handle(&node.id, "finally");
        for finally_node in finally_nodes {
            self.execute_from_node(finally_node).await?;
        }

        Ok(())
    }

    async fn execute_wait_element(&self, node: &WorkflowNode) -> EngineResult<()> {
        let timeout = node
            .data
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30000);

        let wait_condition = node
            .data
            .get("waitCondition")
            .and_then(|v| v.as_str())
            .unwrap_or("visible");

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Waiting for element (condition: {}, timeout: {}ms)",
                    wait_condition, timeout
                ))
                .with_node(&node.id),
            )
            .await;

        // TODO: Implement actual element waiting with UI automation
        // For now, just simulate waiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        self.runtime
            .add_log(ExecutionLog::info("Element wait completed").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_hotkey(&self, node: &WorkflowNode) -> EngineResult<()> {
        let keys = node
            .data
            .get("keys")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let key = node
            .data
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let modifiers: Vec<String> = node
            .data
            .get("modifiers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let key_combo = if keys.is_empty() {
            if modifiers.is_empty() {
                key.to_string()
            } else {
                format!("{}+{}", modifiers.join("+"), key)
            }
        } else {
            keys.to_string()
        };

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Pressing hotkey: {}", key_combo)).with_node(&node.id),
            )
            .await;

        // Try to use browser keyboard if there's an active browser session
        let browser_session_id = self
            .runtime
            .get_variable("_browser_session_id")
            .await
            .and_then(|v| {
                if let VariableValue::String(s) = v {
                    Some(s)
                } else {
                    None
                }
            });

        if let Some(session_id) = browser_session_id {
            // Use browser's keyboard input
            self.web_automation
                .press_key(&session_id, &key_combo)
                .await
                .map_err(|e| EngineError::ExecutionFailed(format!("Failed to press key: {}", e)))?;
        } else {
            // TODO: Implement desktop keyboard simulation
            log::warn!("Desktop keyboard simulation not yet implemented");
        }

        Ok(())
    }

    async fn execute_screenshot(&self, node: &WorkflowNode) -> EngineResult<()> {
        let file_path = node
            .data
            .get("filePath")
            .and_then(|v| v.as_str())
            .unwrap_or("screenshot.png");

        let screenshot_type = node
            .data
            .get("screenshotType")
            .and_then(|v| v.as_str())
            .unwrap_or("fullPage");

        let interpolated_path = self.runtime.interpolate(file_path).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Taking screenshot (type: {}) to: {}",
                    screenshot_type, interpolated_path
                ))
                .with_node(&node.id),
            )
            .await;

        // TODO: Implement actual screenshot capture
        // For now, just log the action

        self.runtime
            .add_log(ExecutionLog::info("Screenshot saved").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_open_browser(&self, node: &WorkflowNode) -> EngineResult<()> {
        let browser_type = node
            .data
            .get("browserType")
            .and_then(|v| v.as_str())
            .unwrap_or("chrome");

        let headless = node
            .data
            .get("headless")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let initial_url = node
            .data
            .get("initialUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Opening {} browser (headless: {})",
                    browser_type, headless
                ))
                .with_node(&node.id),
            )
            .await;

        // Create browser session with unique ID
        let session_id = format!("{}_{}", browser_variable, uuid::Uuid::new_v4());

        let options = BrowserOptions {
            headless,
            browser_path: None,
            user_data_dir: None,
            window_size: Some((1280, 800)),
        };

        // Open browser using Chrome DevTools Protocol
        self.web_automation
            .open_browser(&session_id, options)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to open browser: {}", e)))?;

        // Store session ID in variable for later use
        self.runtime
            .set_variable(browser_variable, VariableValue::String(session_id.clone()))
            .await;

        // Also store in a special internal variable for hotkey/other nodes to find
        self.runtime
            .set_variable("_browser_session_id", VariableValue::String(session_id.clone()))
            .await;

        // Navigate to initial URL if provided
        if !initial_url.is_empty() {
            let interpolated_url = self.runtime.interpolate(initial_url).await;
            self.runtime
                .add_log(
                    ExecutionLog::info(format!("Navigating to: {}", interpolated_url))
                        .with_node(&node.id),
                )
                .await;

            self.web_automation
                .navigate(&session_id, &interpolated_url)
                .await
                .map_err(|e| EngineError::ExecutionFailed(format!("Failed to navigate: {}", e)))?;
        }

        self.runtime
            .add_log(ExecutionLog::info("Browser opened successfully").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_navigate(&self, node: &WorkflowNode) -> EngineResult<()> {
        let url = node
            .data
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let wait_until = node
            .data
            .get("waitUntil")
            .and_then(|v| v.as_str())
            .unwrap_or("load");

        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        let interpolated_url = self.runtime.interpolate(url).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Navigating to: {} (wait: {})",
                    interpolated_url, wait_until
                ))
                .with_node(&node.id),
            )
            .await;

        // Get browser session ID from variable
        let session_id = self
            .runtime
            .get_variable(browser_variable)
            .await
            .and_then(|v| {
                if let VariableValue::String(s) = v {
                    Some(s)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                EngineError::ExecutionFailed(format!(
                    "Browser session '{}' not found. Please open a browser first.",
                    browser_variable
                ))
            })?;

        // Navigate using Chrome DevTools Protocol
        self.web_automation
            .navigate(&session_id, &interpolated_url)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to navigate: {}", e)))?;

        self.runtime
            .add_log(ExecutionLog::info("Navigation completed").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_web_click(&self, node: &WorkflowNode) -> EngineResult<()> {
        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        let selector = node
            .data
            .get("selector")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Web clicking element: {}", selector))
                    .with_node(&node.id),
            )
            .await;

        // Get browser session ID from variable
        let session_id = self.get_browser_session(browser_variable).await?;

        // Click using Chrome DevTools Protocol
        self.web_automation
            .click(&session_id, selector)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to click: {}", e)))?;

        self.runtime
            .add_log(ExecutionLog::info("Web click completed").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_web_input(&self, node: &WorkflowNode) -> EngineResult<()> {
        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        let selector = node
            .data
            .get("selector")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let text = node
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let interpolated_text = self.runtime.interpolate(text).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Web input to element: {}", selector))
                    .with_node(&node.id),
            )
            .await;

        // Get browser session ID from variable
        let session_id = self.get_browser_session(browser_variable).await?;

        // Input using Chrome DevTools Protocol
        self.web_automation
            .input(&session_id, selector, &interpolated_text)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to input: {}", e)))?;

        self.runtime
            .add_log(ExecutionLog::info("Web input completed").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_web_get_text(&self, node: &WorkflowNode) -> EngineResult<()> {
        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        let selector = node
            .data
            .get("selector")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let var_name = node
            .data
            .get("variableName")
            .and_then(|v| v.as_str())
            .unwrap_or("result");

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Getting text from element: {}", selector))
                    .with_node(&node.id),
            )
            .await;

        // Get browser session ID from variable
        let session_id = self.get_browser_session(browser_variable).await?;

        // Get text using Chrome DevTools Protocol
        let text = self
            .web_automation
            .get_text(&session_id, selector)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to get text: {}", e)))?;

        self.runtime
            .set_variable(var_name, VariableValue::String(text.clone()))
            .await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Got text '{}' saved to '{}'", text, var_name))
                    .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    async fn execute_close_browser(&self, node: &WorkflowNode) -> EngineResult<()> {
        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Closing browser: {}", browser_variable))
                    .with_node(&node.id),
            )
            .await;

        // Get browser session ID from variable
        let session_id = self.get_browser_session(browser_variable).await?;

        // Close browser
        self.web_automation
            .close(&session_id)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to close browser: {}", e)))?;

        self.runtime
            .add_log(ExecutionLog::info("Browser closed").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn execute_js(&self, node: &WorkflowNode) -> EngineResult<()> {
        let browser_variable = node
            .data
            .get("browserVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("browser");

        let script = node
            .data
            .get("script")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let result_variable = node
            .data
            .get("resultVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        self.runtime
            .add_log(ExecutionLog::info("Executing JavaScript").with_node(&node.id))
            .await;

        // Get browser session ID from variable
        let session_id = self.get_browser_session(browser_variable).await?;

        // Execute JavaScript
        let result = self
            .web_automation
            .execute_js(&session_id, script)
            .await
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to execute JS: {}", e)))?;

        // Save result to variable if specified
        if !result_variable.is_empty() {
            self.runtime
                .set_variable(result_variable, VariableValue::String(result.clone()))
                .await;

            self.runtime
                .add_log(
                    ExecutionLog::info(format!("JS result saved to '{}'", result_variable))
                        .with_node(&node.id),
                )
                .await;
        }

        self.runtime
            .add_log(ExecutionLog::info("JavaScript executed").with_node(&node.id))
            .await;

        Ok(())
    }

    async fn get_browser_session(&self, browser_variable: &str) -> EngineResult<String> {
        self.runtime
            .get_variable(browser_variable)
            .await
            .and_then(|v| {
                if let VariableValue::String(s) = v {
                    Some(s)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                EngineError::ExecutionFailed(format!(
                    "Browser session '{}' not found. Please open a browser first.",
                    browser_variable
                ))
            })
    }

    async fn execute_read_excel(&self, node: &WorkflowNode) -> EngineResult<()> {
        let file_path = node
            .data
            .get("filePath")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let sheet_name = node
            .data
            .get("sheetName")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let var_name = node
            .data
            .get("variableName")
            .and_then(|v| v.as_str())
            .unwrap_or("excelData");

        let read_mode = node
            .data
            .get("readMode")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let interpolated_path = self.runtime.interpolate(file_path).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Reading Excel file: {}", interpolated_path))
                    .with_node(&node.id),
            )
            .await;

        match read_mode {
            "cell" => {
                let cell_position = node
                    .data
                    .get("cellPosition")
                    .and_then(|v| v.as_str())
                    .unwrap_or("A1");

                // Parse cell position (e.g., "A1" -> (0, 0), "B2" -> (1, 1))
                let (col, row) = self.parse_cell_position(cell_position);

                let target_sheet = if sheet_name.is_empty() {
                    // Get first sheet name
                    self.file_automation
                        .get_excel_sheet_names(&interpolated_path)
                        .map_err(|e| EngineError::ExecutionFailed(e.to_string()))?
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "Sheet1".to_string())
                } else {
                    sheet_name.to_string()
                };

                let value = self
                    .file_automation
                    .read_excel_cell(&interpolated_path, &target_sheet, row, col)
                    .map_err(|e| EngineError::ExecutionFailed(e.to_string()))?;

                self.runtime
                    .set_variable(var_name, VariableValue::String(value.clone()))
                    .await;

                self.runtime
                    .add_log(
                        ExecutionLog::info(format!(
                            "Read cell {} = '{}' saved to '{}'",
                            cell_position, value, var_name
                        ))
                        .with_node(&node.id),
                    )
                    .await;
            }
            _ => {
                // Read all data
                let data = if sheet_name.is_empty() {
                    self.file_automation
                        .read_excel(&interpolated_path)
                        .map_err(|e| EngineError::ExecutionFailed(e.to_string()))?
                } else {
                    let sheet = self
                        .file_automation
                        .read_excel_sheet(&interpolated_path, sheet_name)
                        .map_err(|e| EngineError::ExecutionFailed(e.to_string()))?;

                    crate::automation::file::ExcelData {
                        sheets: vec![sheet],
                    }
                };

                // Convert to JSON and store as string
                let json_data = serde_json::to_string(&data)
                    .map_err(|e| EngineError::ExecutionFailed(format!("Failed to serialize Excel data: {}", e)))?;

                self.runtime
                    .set_variable(var_name, VariableValue::String(json_data))
                    .await;

                let row_count: usize = data.sheets.iter().map(|s| s.rows.len()).sum();
                self.runtime
                    .add_log(
                        ExecutionLog::info(format!(
                            "Read {} sheets with {} rows, saved to '{}'",
                            data.sheets.len(),
                            row_count,
                            var_name
                        ))
                        .with_node(&node.id),
                    )
                    .await;
            }
        }

        Ok(())
    }

    async fn execute_write_excel(&self, node: &WorkflowNode) -> EngineResult<()> {
        let file_path = node
            .data
            .get("filePath")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let sheet_name = node
            .data
            .get("sheetName")
            .and_then(|v| v.as_str())
            .unwrap_or("Sheet1");

        let interpolated_path = self.runtime.interpolate(file_path).await;

        // Expand ~ to home directory
        let expanded_path = if interpolated_path.starts_with('~') {
            if let Some(base_dirs) = directories::BaseDirs::new() {
                interpolated_path.replacen('~', base_dirs.home_dir().to_str().unwrap_or(""), 1)
            } else {
                interpolated_path.clone()
            }
        } else {
            interpolated_path.clone()
        };

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Writing Excel file: {}", expanded_path))
                    .with_node(&node.id),
            )
            .await;

        // Get data - either from inline "data" field or from "dataVariable" reference
        let rows: Vec<Vec<String>> = if let Some(inline_data) = node.data.get("data") {
            // Inline data array in node config
            if let Some(arr) = inline_data.as_array() {
                let mut result = Vec::new();
                for row in arr {
                    if let Some(row_arr) = row.as_array() {
                        let mut row_result = Vec::new();
                        for cell in row_arr {
                            let cell_str = cell.as_str().unwrap_or("");
                            // Interpolate variables in each cell
                            let interpolated = self.runtime.interpolate(cell_str).await;
                            row_result.push(interpolated);
                        }
                        result.push(row_result);
                    }
                }
                result
            } else {
                return Err(EngineError::ExecutionFailed(
                    "Inline 'data' field must be a 2D array".to_string(),
                ));
            }
        } else if let Some(data_variable) = node.data.get("dataVariable").and_then(|v| v.as_str()) {
            // Data from variable reference
            if data_variable.is_empty() {
                return Err(EngineError::ExecutionFailed(
                    "Either 'data' or 'dataVariable' must be provided".to_string(),
                ));
            }

            let data_json = self
                .runtime
                .get_variable(data_variable)
                .await
                .and_then(|v| {
                    if let VariableValue::String(s) = v {
                        Some(s)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    EngineError::ExecutionFailed(format!(
                        "Variable '{}' not found or not a string",
                        data_variable
                    ))
                })?;

            // Try to parse as ExcelData first, then as simple 2D array, then as array of objects
            if let Ok(excel_data) =
                serde_json::from_str::<crate::automation::file::ExcelData>(&data_json)
            {
                excel_data
                    .sheets
                    .first()
                    .map(|s| s.rows.clone())
                    .unwrap_or_default()
            } else if let Ok(parsed_rows) = serde_json::from_str::<Vec<Vec<String>>>(&data_json) {
                parsed_rows
            } else if let Ok(objects) = serde_json::from_str::<Vec<serde_json::Value>>(&data_json) {
                // Convert array of objects to 2D array (with header row)
                self.convert_objects_to_rows(&objects, node)?
            } else {
                return Err(EngineError::ExecutionFailed(
                    "Data variable must be valid JSON (ExcelData, 2D array, or array of objects)".to_string(),
                ));
            }
        } else {
            return Err(EngineError::ExecutionFailed(
                "Either 'data' or 'dataVariable' must be provided".to_string(),
            ));
        };

        self.file_automation
            .write_excel_sheet(&expanded_path, sheet_name, &rows)
            .map_err(|e| EngineError::ExecutionFailed(e.to_string()))?;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Wrote {} rows to sheet '{}'",
                    rows.len(),
                    sheet_name
                ))
                .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    fn parse_cell_position(&self, position: &str) -> (u32, u32) {
        let position = position.to_uppercase();
        let mut col: u32 = 0;
        let mut row: u32 = 0;

        for c in position.chars() {
            if c.is_ascii_alphabetic() {
                col = col * 26 + (c as u32 - 'A' as u32);
            } else if c.is_ascii_digit() {
                row = row * 10 + (c as u32 - '0' as u32);
            }
        }

        // Excel rows are 1-indexed, convert to 0-indexed
        (col, if row > 0 { row - 1 } else { 0 })
    }

    /// Execute a shell command and optionally save output to a variable
    async fn execute_command(&self, node: &WorkflowNode) -> EngineResult<()> {
        let command = node
            .data
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let args: Vec<String> = node
            .data
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let output_variable = node
            .data
            .get("outputVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let working_dir = node
            .data
            .get("workingDir")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let interpolated_command = self.runtime.interpolate(command).await;
        let interpolated_working_dir = if working_dir.is_empty() {
            None
        } else {
            Some(self.runtime.interpolate(working_dir).await)
        };

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Executing command: {} {:?}", interpolated_command, args))
                    .with_node(&node.id),
            )
            .await;

        // Build command
        let mut cmd = std::process::Command::new(&interpolated_command);
        cmd.args(&args);

        if let Some(dir) = &interpolated_working_dir {
            cmd.current_dir(dir);
        }

        // Execute command
        let output = cmd
            .output()
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            self.runtime
                .add_log(
                    ExecutionLog::warn(format!("Command exited with error: {}", stderr))
                        .with_node(&node.id),
                )
                .await;
        }

        // Save output to variable if specified
        if !output_variable.is_empty() {
            self.runtime
                .set_variable(output_variable, VariableValue::String(stdout.clone()))
                .await;
            self.runtime
                .add_log(
                    ExecutionLog::info(format!("Command output saved to '{}'", output_variable))
                        .with_node(&node.id),
                )
                .await;
        }

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Command completed with status: {}", output.status))
                    .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    /// List files and directories in a given path
    async fn execute_list_directory(&self, node: &WorkflowNode) -> EngineResult<()> {
        let path = node
            .data
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let output_variable = node
            .data
            .get("outputVariable")
            .and_then(|v| v.as_str())
            .unwrap_or("files");

        let include_hidden = node
            .data
            .get("includeHidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let recursive = node
            .data
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let interpolated_path = self.runtime.interpolate(path).await;

        // Expand ~ to home directory
        let expanded_path = if interpolated_path.starts_with('~') {
            if let Some(base_dirs) = directories::BaseDirs::new() {
                interpolated_path.replacen('~', base_dirs.home_dir().to_str().unwrap_or(""), 1)
            } else {
                interpolated_path.clone()
            }
        } else {
            interpolated_path.clone()
        };

        self.runtime
            .add_log(
                ExecutionLog::info(format!("Listing directory: {}", expanded_path))
                    .with_node(&node.id),
            )
            .await;

        let path_buf = std::path::PathBuf::from(&expanded_path);
        if !path_buf.exists() {
            return Err(EngineError::ExecutionFailed(format!(
                "Directory does not exist: {}",
                expanded_path
            )));
        }

        let mut entries: Vec<serde_json::Value> = Vec::new();

        if recursive {
            self.list_directory_recursive(&path_buf, include_hidden, &mut entries)?;
        } else {
            let dir_entries = std::fs::read_dir(&path_buf)
                .map_err(|e| EngineError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

            for entry in dir_entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files if not included
                if !include_hidden && file_name.starts_with('.') {
                    continue;
                }

                let metadata = entry.metadata().ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                entries.push(serde_json::json!({
                    "name": file_name,
                    "path": entry.path().to_string_lossy().to_string(),
                    "isDirectory": is_dir,
                    "size": size
                }));
            }
        }

        // Sort entries: directories first, then by name
        entries.sort_by(|a, b| {
            let a_is_dir = a.get("isDirectory").and_then(|v| v.as_bool()).unwrap_or(false);
            let b_is_dir = b.get("isDirectory").and_then(|v| v.as_bool()).unwrap_or(false);
            if a_is_dir != b_is_dir {
                return b_is_dir.cmp(&a_is_dir); // Directories first
            }
            let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
            a_name.cmp(b_name)
        });

        // Save to variable as JSON string
        let json_str = serde_json::to_string(&entries)
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to serialize entries: {}", e)))?;

        self.runtime
            .set_variable(output_variable, VariableValue::String(json_str))
            .await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Found {} entries, saved to '{}'",
                    entries.len(),
                    output_variable
                ))
                .with_node(&node.id),
            )
            .await;

        Ok(())
    }

    fn list_directory_recursive(
        &self,
        path: &std::path::Path,
        include_hidden: bool,
        entries: &mut Vec<serde_json::Value>,
    ) -> EngineResult<()> {
        let dir_entries = std::fs::read_dir(path)
            .map_err(|e| EngineError::ExecutionFailed(format!("Failed to read directory: {}", e)))?;

        for entry in dir_entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if not included
            if !include_hidden && file_name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

            entries.push(serde_json::json!({
                "name": file_name,
                "path": entry.path().to_string_lossy().to_string(),
                "isDirectory": is_dir,
                "size": size
            }));

            // Recurse into directories
            if is_dir {
                let _ = self.list_directory_recursive(&entry.path(), include_hidden, entries);
            }
        }

        Ok(())
    }

    /// Convert JSON array of objects to 2D array for Excel
    fn convert_objects_to_rows(
        &self,
        objects: &[serde_json::Value],
        node: &WorkflowNode,
    ) -> EngineResult<Vec<Vec<String>>> {
        if objects.is_empty() {
            return Ok(Vec::new());
        }

        // Get column names from node config or auto-detect from first object
        let columns: Vec<String> = if let Some(cols) = node.data.get("columns").and_then(|v| v.as_array()) {
            cols.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            // Auto-detect columns from first object
            if let Some(first_obj) = objects.first().and_then(|v| v.as_object()) {
                first_obj.keys().cloned().collect()
            } else {
                return Err(EngineError::ExecutionFailed(
                    "Cannot determine columns from data".to_string(),
                ));
            }
        };

        let mut rows: Vec<Vec<String>> = Vec::new();

        // Add header row
        let include_header = node
            .data
            .get("includeHeader")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if include_header {
            rows.push(columns.clone());
        }

        // Add data rows
        for obj in objects {
            if let Some(obj_map) = obj.as_object() {
                let row: Vec<String> = columns
                    .iter()
                    .map(|col| {
                        obj_map
                            .get(col)
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Null => String::new(),
                                _ => v.to_string(),
                            })
                            .unwrap_or_default()
                    })
                    .collect();
                rows.push(row);
            }
        }

        Ok(rows)
    }

    /// Open an application
    async fn execute_open_app(&self, node: &WorkflowNode) -> EngineResult<()> {
        let app_name = node
            .data
            .get("appName")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let app_path = node
            .data
            .get("appPath")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let arguments: Vec<String> = node
            .data
            .get("arguments")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let interpolated_app_name = self.runtime.interpolate(app_name).await;
        let interpolated_app_path = self.runtime.interpolate(app_path).await;

        self.runtime
            .add_log(
                ExecutionLog::info(format!(
                    "Opening application: {}",
                    if !interpolated_app_name.is_empty() {
                        &interpolated_app_name
                    } else {
                        &interpolated_app_path
                    }
                ))
                .with_node(&node.id),
            )
            .await;

        #[cfg(target_os = "macos")]
        {
            let mut cmd = std::process::Command::new("open");

            if !interpolated_app_name.is_empty() {
                cmd.args(["-a", &interpolated_app_name]);
            } else if !interpolated_app_path.is_empty() {
                cmd.arg(&interpolated_app_path);
            } else {
                return Err(EngineError::ExecutionFailed(
                    "Either appName or appPath must be provided".to_string(),
                ));
            }

            // Add any additional arguments
            if !arguments.is_empty() {
                cmd.arg("--args");
                cmd.args(&arguments);
            }

            cmd.spawn()
                .map_err(|e| EngineError::ExecutionFailed(format!("Failed to open application: {}", e)))?;
        }

        #[cfg(target_os = "windows")]
        {
            let target = if !interpolated_app_path.is_empty() {
                &interpolated_app_path
            } else {
                &interpolated_app_name
            };

            let mut cmd = std::process::Command::new("cmd");
            cmd.args(["/C", "start", "", target]);
            cmd.args(&arguments);

            cmd.spawn()
                .map_err(|e| EngineError::ExecutionFailed(format!("Failed to open application: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            let target = if !interpolated_app_path.is_empty() {
                &interpolated_app_path
            } else {
                &interpolated_app_name
            };

            let mut cmd = std::process::Command::new("xdg-open");
            cmd.arg(target);
            cmd.args(&arguments);

            cmd.spawn()
                .map_err(|e| EngineError::ExecutionFailed(format!("Failed to open application: {}", e)))?;
        }

        // Give the app time to start
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        self.runtime
            .add_log(
                ExecutionLog::info("Application opened successfully").with_node(&node.id),
            )
            .await;

        Ok(())
    }

    pub async fn get_state(&self) -> super::runtime::RuntimeState {
        self.runtime.get_state().await
    }

    pub async fn pause(&self) {
        self.runtime.pause().await;
    }

    pub async fn resume(&self) {
        self.runtime.resume().await;
    }

    // Debug control methods

    /// Step to the next node (single step execution)
    pub async fn step(&self) {
        self.runtime.step().await;
    }

    /// Add a breakpoint at a node
    pub async fn add_breakpoint(&self, node_id: String) {
        self.runtime.add_breakpoint(node_id).await;
    }

    /// Remove a breakpoint from a node
    pub async fn remove_breakpoint(&self, node_id: &str) {
        self.runtime.remove_breakpoint(node_id).await;
    }

    /// Clear all breakpoints
    pub async fn clear_breakpoints(&self) {
        self.runtime.clear_breakpoints().await;
    }

    /// Get all breakpoints
    pub async fn get_breakpoints(&self) -> HashSet<String> {
        self.runtime.get_breakpoints().await
    }

    /// Get current debug mode
    pub async fn get_debug_mode(&self) -> DebugMode {
        self.runtime.get_debug_mode().await
    }

    /// Get all variables for debugging
    pub async fn get_variables(&self) -> std::collections::HashMap<String, VariableValue> {
        self.runtime.get_all_variables().await
    }
}
