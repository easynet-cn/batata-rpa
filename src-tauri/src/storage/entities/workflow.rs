use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "workflows")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// JSON serialized nodes
    pub nodes: String,
    /// JSON serialized edges
    pub edges: String,
    /// JSON serialized variables
    pub variables: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Workflow info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Model> for WorkflowInfo {
    fn from(model: Model) -> Self {
        // Parse nodes to count them
        let node_count = serde_json::from_str::<Vec<serde_json::Value>>(&model.nodes)
            .map(|nodes| nodes.len())
            .unwrap_or(0);

        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            node_count,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
