use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "element_libraries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// JSON serialized elements
    pub elements: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Element library info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLibraryInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub element_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Model> for ElementLibraryInfo {
    fn from(model: Model) -> Self {
        // Parse elements to count them
        let element_count = serde_json::from_str::<Vec<serde_json::Value>>(&model.elements)
            .map(|elements| elements.len())
            .unwrap_or(0);

        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            element_count,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
