pub mod automation;
pub mod commands;
pub mod element;
pub mod engine;
pub mod storage;

use commands::execution::ExecutorState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ExecutorState::new())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
