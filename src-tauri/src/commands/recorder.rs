use crate::recorder::{convert_to_workflow, Recorder, RecordingSession, RecordingState};
use crate::engine::executor::Workflow;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub struct RecorderState {
    recorder: Arc<RwLock<Recorder>>,
}

impl RecorderState {
    pub fn new() -> Self {
        Self {
            recorder: Arc::new(RwLock::new(Recorder::new())),
        }
    }
}

#[derive(serde::Serialize)]
pub struct RecordingStatus {
    state: RecordingState,
    action_count: usize,
    duration_ms: u64,
}

#[tauri::command]
pub async fn start_recording(
    name: Option<String>,
    state: State<'_, RecorderState>,
) -> Result<(), String> {
    let recorder = state.recorder.read().await;
    recorder.start(name).await
}

#[tauri::command]
pub async fn pause_recording(state: State<'_, RecorderState>) -> Result<(), String> {
    let recorder = state.recorder.read().await;
    recorder.pause().await
}

#[tauri::command]
pub async fn resume_recording(state: State<'_, RecorderState>) -> Result<(), String> {
    let recorder = state.recorder.read().await;
    recorder.resume().await
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, RecorderState>,
) -> Result<RecordingSession, String> {
    let recorder = state.recorder.read().await;
    recorder.stop().await
}

#[tauri::command]
pub async fn get_recording_state(
    state: State<'_, RecorderState>,
) -> Result<RecordingStatus, String> {
    let recorder = state.recorder.read().await;
    let session = recorder.get_session().await;

    let duration_ms = if let Some(started) = &session.started_at {
        if let Ok(started_time) = chrono::DateTime::parse_from_rfc3339(started) {
            let now = chrono::Utc::now();
            (now.signed_duration_since(started_time.with_timezone(&chrono::Utc)))
                .num_milliseconds()
                .max(0) as u64
        } else {
            0
        }
    } else {
        0
    };

    Ok(RecordingStatus {
        state: session.state,
        action_count: session.actions.len(),
        duration_ms,
    })
}

#[tauri::command]
pub async fn get_recording_session(
    state: State<'_, RecorderState>,
) -> Result<RecordingSession, String> {
    let recorder = state.recorder.read().await;
    Ok(recorder.get_session().await)
}

#[tauri::command]
pub async fn convert_recording_to_workflow(
    state: State<'_, RecorderState>,
) -> Result<Workflow, String> {
    let recorder = state.recorder.read().await;
    let session = recorder.get_session().await;

    if session.actions.is_empty() {
        return Err("No actions recorded".to_string());
    }

    Ok(convert_to_workflow(&session))
}

#[tauri::command]
pub async fn clear_recording(state: State<'_, RecorderState>) -> Result<(), String> {
    let mut recorder = state.recorder.write().await;
    *recorder = Recorder::new();
    Ok(())
}
