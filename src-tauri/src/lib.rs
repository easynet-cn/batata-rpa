pub mod automation;
pub mod commands;
pub mod element;
pub mod engine;
pub mod recorder;
pub mod storage;

use commands::execution::ExecutorState;
use commands::recorder::RecorderState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ExecutorState::new())
        .manage(RecorderState::new())
        .setup(|_app| {
            // Initialize database on startup
            tauri::async_runtime::block_on(async {
                if let Err(e) = storage::init_database().await {
                    log::error!("Failed to initialize database: {}", e);
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
