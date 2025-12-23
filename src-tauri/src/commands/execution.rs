use crate::engine::executor::{Executor, Workflow};
use crate::engine::runtime::{DebugMode, RuntimeState};
use crate::engine::variable::VariableValue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugModeParam {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "step")]
    StepByStep,
    #[serde(rename = "breakpoint")]
    Breakpoint,
}

impl From<DebugModeParam> for DebugMode {
    fn from(param: DebugModeParam) -> Self {
        match param {
            DebugModeParam::None => DebugMode::None,
            DebugModeParam::StepByStep => DebugMode::StepByStep,
            DebugModeParam::Breakpoint => DebugMode::Breakpoint,
        }
    }
}

pub struct ExecutorState {
    pub executors: RwLock<HashMap<String, Arc<Executor>>>,
}

impl ExecutorState {
    pub fn new() -> Self {
        Self {
            executors: RwLock::new(HashMap::new()),
        }
    }
}

#[command]
pub async fn execute_workflow(
    workflow: Workflow,
    state: State<'_, ExecutorState>,
) -> Result<String, String> {
    log::info!("Executing workflow: {}", workflow.name);

    let workflow_id = workflow.id.clone();
    let executor = Arc::new(Executor::new(workflow));

    {
        let mut executors = state.executors.write().await;
        executors.insert(workflow_id.clone(), executor.clone());
    }

    // Execute in background
    let executor_clone = executor.clone();
    tokio::spawn(async move {
        if let Err(e) = executor_clone.execute().await {
            log::error!("Workflow execution failed: {}", e);
        }
    });

    Ok(workflow_id)
}

#[command]
pub async fn get_execution_state(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<RuntimeState, String> {
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        Ok(executor.get_state().await)
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn pause_execution(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Pausing execution: {}", workflow_id);
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        executor.pause().await;
        Ok(())
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn resume_execution(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Resuming execution: {}", workflow_id);
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        executor.resume().await;
        Ok(())
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn stop_execution(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Stopping execution: {}", workflow_id);
    let mut executors = state.executors.write().await;
    executors.remove(&workflow_id);
    Ok(())
}

// Debug commands

#[command]
pub async fn execute_workflow_debug(
    workflow: Workflow,
    debug_mode: DebugModeParam,
    state: State<'_, ExecutorState>,
) -> Result<String, String> {
    log::info!("Executing workflow in debug mode: {} ({:?})", workflow.name, debug_mode);

    let workflow_id = workflow.id.clone();
    let executor = Arc::new(Executor::new(workflow));

    {
        let mut executors = state.executors.write().await;
        executors.insert(workflow_id.clone(), executor.clone());
    }

    // Execute in background
    let executor_clone = executor.clone();
    let mode: DebugMode = debug_mode.into();
    tokio::spawn(async move {
        if let Err(e) = executor_clone.execute_debug(mode).await {
            log::error!("Workflow debug execution failed: {}", e);
        }
    });

    Ok(workflow_id)
}

#[command]
pub async fn step_execution(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Stepping execution: {}", workflow_id);
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        executor.step().await;
        Ok(())
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn add_breakpoint(
    workflow_id: String,
    node_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Adding breakpoint at node {} for workflow {}", node_id, workflow_id);
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        executor.add_breakpoint(node_id).await;
        Ok(())
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn remove_breakpoint(
    workflow_id: String,
    node_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Removing breakpoint at node {} for workflow {}", node_id, workflow_id);
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        executor.remove_breakpoint(&node_id).await;
        Ok(())
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn clear_breakpoints(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<(), String> {
    log::info!("Clearing all breakpoints for workflow {}", workflow_id);
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        executor.clear_breakpoints().await;
        Ok(())
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn get_breakpoints(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<HashSet<String>, String> {
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        Ok(executor.get_breakpoints().await)
    } else {
        Err("Execution not found".to_string())
    }
}

#[command]
pub async fn get_variables(
    workflow_id: String,
    state: State<'_, ExecutorState>,
) -> Result<HashMap<String, VariableValue>, String> {
    let executors = state.executors.read().await;
    if let Some(executor) = executors.get(&workflow_id) {
        Ok(executor.get_variables().await)
    } else {
        Err("Execution not found".to_string())
    }
}
