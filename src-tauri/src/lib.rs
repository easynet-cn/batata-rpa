pub mod automation;
pub mod commands;
pub mod element;
pub mod engine;
pub mod plugin;
pub mod recorder;
pub mod storage;

use commands::execution::ExecutorState;
use commands::recorder::RecorderState;
use plugin::{PluginRegistry, PluginLoader};
use std::sync::Arc;
use tauri::Manager;

/// Global plugin state
pub struct PluginState {
    pub registry: Arc<PluginRegistry>,
}

impl PluginState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let plugin_state = PluginState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ExecutorState::new())
        .manage(RecorderState::new())
        .manage(plugin_state)
        .setup(|app| {
            // Initialize database on startup
            tauri::async_runtime::block_on(async {
                if let Err(e) = storage::init_database().await {
                    log::error!("Failed to initialize database: {}", e);
                }
            });

            // Load plugins from plugins directory
            let plugins_dir = app
                .path()
                .app_data_dir()
                .map(|p: std::path::PathBuf| p.join("plugins"))
                .unwrap_or_else(|_| std::path::PathBuf::from("plugins"));

            // Create plugins directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
                log::warn!("Failed to create plugins directory: {}", e);
            }

            // Load plugins using the plugin_state we created earlier
            let registry = Arc::new(PluginRegistry::new());
            let registry_clone = registry.clone();
            tauri::async_runtime::block_on(async move {
                let loader = PluginLoader::new(registry_clone);
                match loader.load_directory(&plugins_dir).await {
                    Ok(plugins) => {
                        log::info!("Loaded {} plugins from {:?}", plugins.len(), plugins_dir);
                    }
                    Err(e) => {
                        log::error!("Failed to load plugins: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Workflow commands
            commands::save_workflow,
            commands::load_workflow,
            commands::list_workflows,
            commands::delete_workflow,
            commands::export_workflow,
            commands::import_workflow,
            // Element commands
            commands::capture_element,
            commands::start_element_capture,
            commands::stop_element_capture,
            commands::save_element_library,
            commands::load_element_library,
            commands::list_element_libraries,
            commands::delete_element_library,
            commands::highlight_element,
            // Execution commands
            commands::execute_workflow,
            commands::get_execution_state,
            commands::pause_execution,
            commands::resume_execution,
            commands::stop_execution,
            // Debug commands
            commands::execute_workflow_debug,
            commands::step_execution,
            commands::add_breakpoint,
            commands::remove_breakpoint,
            commands::clear_breakpoints,
            commands::get_breakpoints,
            commands::get_variables,
            // Recorder commands
            commands::start_recording,
            commands::pause_recording,
            commands::resume_recording,
            commands::stop_recording,
            commands::get_recording_state,
            commands::get_recording_session,
            commands::convert_recording_to_workflow,
            commands::clear_recording,
            // Settings commands
            commands::get_db_config,
            commands::set_db_config,
            commands::test_db_connection,
            commands::apply_db_config,
            commands::get_supported_db_types,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
