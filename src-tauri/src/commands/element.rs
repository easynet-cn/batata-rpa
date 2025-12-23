use crate::element::{ElementLibrary, UIElement};
use crate::storage::{self, entities::element_library};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, QueryOrder};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLibraryInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub element_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[command]
pub async fn capture_element(x: i32, y: i32) -> Result<UIElement, String> {
    log::info!("Capturing element at ({}, {})", x, y);

    #[cfg(target_os = "macos")]
    {
        use crate::automation::desktop::macos::MacOSAutomation;
        use crate::automation::desktop::DesktopAutomation;

        let automation = MacOSAutomation::new();
        automation
            .capture_element(x, y)
            .await
            .map_err(|e| format!("Failed to capture element: {}", e))
    }

    #[cfg(target_os = "windows")]
    {
        use crate::automation::desktop::windows::WindowsAutomation;
        use crate::automation::desktop::DesktopAutomation;

        let automation = WindowsAutomation::new();
        automation
            .capture_element(x, y)
            .await
            .map_err(|e| format!("Failed to capture element: {}", e))
    }

    #[cfg(target_os = "linux")]
    {
        Err("Element capture not implemented for Linux yet".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Element capture not supported on this platform".to_string())
    }
}

#[command]
pub async fn start_element_capture() -> Result<(), String> {
    log::info!("Starting element capture mode");
    // TODO: Enter element capture mode (global hotkey, overlay window)
    Ok(())
}

#[command]
pub async fn stop_element_capture() -> Result<(), String> {
    log::info!("Stopping element capture mode");
    // TODO: Exit element capture mode
    Ok(())
}

#[command]
pub async fn save_element_library(library: ElementLibrary) -> Result<String, String> {
    log::info!("Saving element library: {}", library.name);

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    // Serialize elements to JSON
    let elements_json = serde_json::to_string(&library.elements)
        .map_err(|e| format!("Failed to serialize elements: {}", e))?;

    // Check if library exists
    let existing = element_library::Entity::find_by_id(&library.id)
        .one(&db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if existing.is_some() {
        // Update existing library
        let active_model = element_library::ActiveModel {
            id: Set(library.id.clone()),
            name: Set(library.name),
            description: Set(library.description),
            elements: Set(elements_json),
            created_at: Set(existing.unwrap().created_at),
            updated_at: Set(now),
        };

        active_model
            .update(&db)
            .await
            .map_err(|e| format!("Failed to update library: {}", e))?;
    } else {
        // Insert new library
        let active_model = element_library::ActiveModel {
            id: Set(library.id.clone()),
            name: Set(library.name),
            description: Set(library.description),
            elements: Set(elements_json),
            created_at: Set(now.clone()),
            updated_at: Set(now),
        };

        active_model
            .insert(&db)
            .await
            .map_err(|e| format!("Failed to save library: {}", e))?;
    }

    log::info!("Element library saved: {}", library.id);
    Ok(library.id)
}

#[command]
pub async fn load_element_library(id: String) -> Result<ElementLibrary, String> {
    log::info!("Loading element library: {}", id);

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let model = element_library::Entity::find_by_id(&id)
        .one(&db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Library not found".to_string())?;

    // Deserialize elements
    let elements = serde_json::from_str(&model.elements)
        .map_err(|e| format!("Failed to parse elements: {}", e))?;

    Ok(ElementLibrary {
        id: model.id,
        name: model.name,
        description: model.description,
        elements,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

#[command]
pub async fn list_element_libraries() -> Result<Vec<ElementLibraryInfo>, String> {
    log::info!("Listing element libraries");

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let models = element_library::Entity::find()
        .order_by_desc(element_library::Column::UpdatedAt)
        .all(&db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let libraries: Vec<ElementLibraryInfo> = models
        .into_iter()
        .map(|m| {
            let element_count = serde_json::from_str::<Vec<serde_json::Value>>(&m.elements)
                .map(|elements| elements.len())
                .unwrap_or(0);

            ElementLibraryInfo {
                id: m.id,
                name: m.name,
                description: m.description,
                element_count,
                created_at: m.created_at,
                updated_at: m.updated_at,
            }
        })
        .collect();

    log::info!("Found {} element libraries", libraries.len());
    Ok(libraries)
}

#[command]
pub async fn delete_element_library(id: String) -> Result<(), String> {
    log::info!("Deleting element library: {}", id);

    let db = storage::get_connection()
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    element_library::Entity::delete_by_id(&id)
        .exec(&db)
        .await
        .map_err(|e| format!("Failed to delete library: {}", e))?;

    log::info!("Element library deleted: {}", id);
    Ok(())
}

#[command]
pub async fn highlight_element(element: UIElement) -> Result<(), String> {
    log::info!("Highlighting element: {}", element.name);

    use crate::automation::highlight;

    highlight::highlight_element(element.bounds, None)
        .await
        .map_err(|e| format!("Failed to highlight element: {}", e))
}
