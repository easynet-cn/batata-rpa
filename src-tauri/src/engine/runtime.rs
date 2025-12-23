use super::variable::{VariableScope, VariableStore, VariableValue};
use super::{ExecutionLog, ExecutionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugMode {
    /// Normal execution without debugging
    None,
    /// Step-by-step execution, pause after each node
    StepByStep,
    /// Run until breakpoint
    Breakpoint,
}

impl Default for DebugMode {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugState {
    pub mode: DebugMode,
    pub breakpoints: HashSet<String>,
    pub step_pending: bool,
    pub paused_at_node: Option<String>,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            mode: DebugMode::None,
            breakpoints: HashSet::new(),
            step_pending: false,
            paused_at_node: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub current_node_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub logs: Vec<ExecutionLog>,
    pub error: Option<String>,
    pub debug: DebugState,
}

impl RuntimeState {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            status: ExecutionStatus::Idle,
            current_node_id: None,
            start_time: None,
            end_time: None,
            logs: Vec::new(),
            error: None,
            debug: DebugState::default(),
        }
    }
}

pub struct Runtime {
    state: Arc<RwLock<RuntimeState>>,
    variables: Arc<RwLock<VariableStore>>,
}

impl Runtime {
    pub fn new(workflow_id: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(RuntimeState::new(workflow_id))),
            variables: Arc::new(RwLock::new(VariableStore::new())),
        }
    }

    pub async fn start(&self) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Running;
        state.start_time = Some(chrono::Utc::now().to_rfc3339());
        state.logs.push(ExecutionLog::info("Workflow execution started"));
    }

    pub async fn start_debug(&self, mode: DebugMode) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Running;
        state.start_time = Some(chrono::Utc::now().to_rfc3339());
        state.debug.mode = mode;
        state.debug.step_pending = mode == DebugMode::StepByStep;
        state
            .logs
            .push(ExecutionLog::info(format!("Debug execution started (mode: {:?})", mode)));
    }

    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Paused;
        state.logs.push(ExecutionLog::info("Workflow execution paused"));
    }

    pub async fn resume(&self) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Running;
        state.debug.step_pending = false;
        state.debug.paused_at_node = None;
        state.logs.push(ExecutionLog::info("Workflow execution resumed"));
    }

    pub async fn step(&self) {
        let mut state = self.state.write().await;
        if state.status == ExecutionStatus::Paused {
            state.status = ExecutionStatus::Running;
            state.debug.step_pending = true;
            state.debug.paused_at_node = None;
            state.logs.push(ExecutionLog::info("Step execution"));
        }
    }

    pub async fn complete(&self) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Completed;
        state.end_time = Some(chrono::Utc::now().to_rfc3339());
        state.debug.mode = DebugMode::None;
        state.logs.push(ExecutionLog::info("Workflow execution completed"));
    }

    pub async fn fail(&self, error: String) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Failed;
        state.end_time = Some(chrono::Utc::now().to_rfc3339());
        state.error = Some(error.clone());
        state.debug.mode = DebugMode::None;
        state
            .logs
            .push(ExecutionLog::error(format!("Workflow execution failed: {}", error)));
    }

    pub async fn set_current_node(&self, node_id: Option<String>) {
        let mut state = self.state.write().await;
        state.current_node_id = node_id;
    }

    pub async fn add_log(&self, log: ExecutionLog) {
        let mut state = self.state.write().await;
        state.logs.push(log);
    }

    pub async fn get_state(&self) -> RuntimeState {
        self.state.read().await.clone()
    }

    pub async fn get_status(&self) -> ExecutionStatus {
        self.state.read().await.status.clone()
    }

    // Debug methods

    pub async fn add_breakpoint(&self, node_id: String) {
        let mut state = self.state.write().await;
        state.debug.breakpoints.insert(node_id.clone());
        state
            .logs
            .push(ExecutionLog::info(format!("Breakpoint added: {}", node_id)));
    }

    pub async fn remove_breakpoint(&self, node_id: &str) {
        let mut state = self.state.write().await;
        state.debug.breakpoints.remove(node_id);
        state
            .logs
            .push(ExecutionLog::info(format!("Breakpoint removed: {}", node_id)));
    }

    pub async fn clear_breakpoints(&self) {
        let mut state = self.state.write().await;
        state.debug.breakpoints.clear();
        state.logs.push(ExecutionLog::info("All breakpoints cleared"));
    }

    pub async fn has_breakpoint(&self, node_id: &str) -> bool {
        let state = self.state.read().await;
        state.debug.breakpoints.contains(node_id)
    }

    pub async fn get_breakpoints(&self) -> HashSet<String> {
        self.state.read().await.debug.breakpoints.clone()
    }

    pub async fn get_debug_mode(&self) -> DebugMode {
        self.state.read().await.debug.mode
    }

    pub async fn set_debug_mode(&self, mode: DebugMode) {
        let mut state = self.state.write().await;
        state.debug.mode = mode;
    }

    /// Check if execution should pause at the current node
    /// Returns true if we should pause (breakpoint hit or step mode)
    pub async fn should_pause_at_node(&self, node_id: &str) -> bool {
        let state = self.state.read().await;

        match state.debug.mode {
            DebugMode::None => false,
            DebugMode::StepByStep => {
                // In step mode, always pause after executing a node
                true
            }
            DebugMode::Breakpoint => {
                // In breakpoint mode, only pause if this node has a breakpoint
                state.debug.breakpoints.contains(node_id)
            }
        }
    }

    /// Pause execution at a node (for debugging)
    pub async fn pause_at_node(&self, node_id: &str) {
        let mut state = self.state.write().await;
        state.status = ExecutionStatus::Paused;
        state.debug.paused_at_node = Some(node_id.to_string());
        state.logs.push(
            ExecutionLog::info(format!("Paused at node: {}", node_id))
                .with_node(node_id),
        );
    }

    /// Wait for step/resume command when in step mode
    pub async fn wait_for_step(&self) {
        loop {
            let state = self.state.read().await;

            // If not paused, continue
            if state.status != ExecutionStatus::Paused {
                return;
            }

            // If step is pending (user clicked step), we can proceed
            if state.debug.step_pending {
                return;
            }

            drop(state);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    /// Get all variables for debugging
    pub async fn get_all_variables(&self) -> std::collections::HashMap<String, VariableValue> {
        let vars = self.variables.read().await;
        vars.get_all()
    }

    // Variable methods

    pub async fn set_variable(&self, name: impl Into<String>, value: VariableValue) {
        let mut vars = self.variables.write().await;
        vars.set(name, value, VariableScope::Global);
    }

    pub async fn get_variable(&self, name: &str) -> Option<VariableValue> {
        let vars = self.variables.read().await;
        vars.get(name).cloned()
    }

    pub async fn interpolate(&self, text: &str) -> String {
        let vars = self.variables.read().await;
        vars.interpolate(text)
    }
}
