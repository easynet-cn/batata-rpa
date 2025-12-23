use crate::engine::executor::Workflow;
use crate::storage::{self, entities::workflow};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, QueryOrder, ColumnTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tauri::command;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[command]
pub async fn save_workflow(workflow: Workflow) -> Result<String, String> {
    log::info!("Saving workflow: {}", workflow.name);

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    // Serialize nodes, edges to JSON
    let nodes_json = serde_json::to_string(&workflow.nodes)
        .map_err(|e| format!("Failed to serialize nodes: {}", e))?;
    let edges_json = serde_json::to_string(&workflow.edges)
        .map_err(|e| format!("Failed to serialize edges: {}", e))?;

    // Check if workflow exists
    let existing = workflow::Entity::find_by_id(&workflow.id)
        .one(&db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if existing.is_some() {
        // Update existing workflow
        let active_model = workflow::ActiveModel {
            id: Set(workflow.id.clone()),
            name: Set(workflow.name),
            description: Set(None),
            nodes: Set(nodes_json),
            edges: Set(edges_json),
            variables: Set("[]".to_string()),
            created_at: Set(existing.unwrap().created_at),
            updated_at: Set(now),
        };

        active_model
            .update(&db)
            .await
            .map_err(|e| format!("Failed to update workflow: {}", e))?;
    } else {
        // Insert new workflow
        let active_model = workflow::ActiveModel {
            id: Set(workflow.id.clone()),
            name: Set(workflow.name),
            description: Set(None),
            nodes: Set(nodes_json),
            edges: Set(edges_json),
            variables: Set("[]".to_string()),
            created_at: Set(now.clone()),
            updated_at: Set(now),
        };

        active_model
            .insert(&db)
            .await
            .map_err(|e| format!("Failed to save workflow: {}", e))?;
    }

    log::info!("Workflow saved successfully: {}", workflow.id);
    Ok(workflow.id)
}

#[command]
pub async fn load_workflow(id: String) -> Result<Workflow, String> {
    log::info!("Loading workflow: {}", id);

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let model = workflow::Entity::find_by_id(&id)
        .one(&db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Workflow not found".to_string())?;

    // Deserialize nodes and edges
    let nodes = serde_json::from_str(&model.nodes)
        .map_err(|e| format!("Failed to parse nodes: {}", e))?;
    let edges = serde_json::from_str(&model.edges)
        .map_err(|e| format!("Failed to parse edges: {}", e))?;

    Ok(Workflow {
        id: model.id,
        name: model.name,
        nodes,
        edges,
    })
}

#[command]
pub async fn list_workflows() -> Result<Vec<WorkflowInfo>, String> {
    log::info!("Listing workflows");

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let models = workflow::Entity::find()
        .order_by_desc(workflow::Column::UpdatedAt)
        .all(&db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let workflows: Vec<WorkflowInfo> = models
        .into_iter()
        .map(|m| {
            let node_count = serde_json::from_str::<Vec<serde_json::Value>>(&m.nodes)
                .map(|nodes| nodes.len())
                .unwrap_or(0);

            WorkflowInfo {
                id: m.id,
                name: m.name,
                description: m.description,
                node_count,
                created_at: m.created_at,
                updated_at: m.updated_at,
            }
        })
        .collect();

    log::info!("Found {} workflows", workflows.len());
    Ok(workflows)
}

#[command]
pub async fn delete_workflow(id: String) -> Result<(), String> {
    log::info!("Deleting workflow: {}", id);

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    workflow::Entity::delete_by_id(&id)
        .exec(&db)
        .await
        .map_err(|e| format!("Failed to delete workflow: {}", e))?;

    log::info!("Workflow deleted: {}", id);
    Ok(())
}

#[command]
pub async fn export_workflow(id: String, path: String) -> Result<(), String> {
    log::info!("Exporting workflow {} to {}", id, path);

    // Load workflow from database
    let workflow = load_workflow(id).await?;

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&workflow)
        .map_err(|e| format!("Failed to serialize workflow: {}", e))?;

    // Write to file
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    log::info!("Workflow exported to: {}", path);
    Ok(())
}

#[command]
pub async fn import_workflow(path: String) -> Result<Workflow, String> {
    log::info!("Importing workflow from {}", path);

    // Read file
    let json = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse workflow
    let mut workflow: Workflow = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;

    // Generate new ID to avoid conflicts
    workflow.id = uuid::Uuid::new_v4().to_string();

    // Save to database
    save_workflow(workflow.clone()).await?;

    log::info!("Workflow imported: {}", workflow.id);
    Ok(workflow)
}
