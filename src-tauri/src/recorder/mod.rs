use crate::element::UIElement;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordedActionType {
    Click,
    DoubleClick,
    RightClick,
    Input,
    Scroll,
    Hotkey,
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedAction {
    pub id: String,
    pub action_type: RecordedActionType,
    pub timestamp: u64,
    pub element: Option<UIElement>,
    pub position: Option<(i32, i32)>,
    pub data: HashMap<String, serde_json::Value>,
    pub screenshot: Option<Vec<u8>>,
}

impl RecordedAction {
    pub fn new(action_type: RecordedActionType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            element: None,
            position: None,
            data: HashMap::new(),
            screenshot: None,
        }
    }

    pub fn with_element(mut self, element: UIElement) -> Self {
        self.element = Some(element);
        self
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.position = Some((x, y));
        self
    }

    pub fn with_data(mut self, key: &str, value: serde_json::Value) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub name: String,
    pub state: RecordingState,
    pub actions: Vec<RecordedAction>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

impl RecordingSession {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            state: RecordingState::Idle,
            actions: Vec::new(),
            started_at: None,
            ended_at: None,
        }
    }
}

pub struct Recorder {
    session: Arc<RwLock<RecordingSession>>,
    #[cfg(target_os = "macos")]
    event_tap: Arc<RwLock<Option<macos::EventTap>>>,
    #[cfg(target_os = "windows")]
    event_hook: Arc<RwLock<Option<windows::EventHook>>>,
    #[cfg(target_os = "linux")]
    event_monitor: Arc<RwLock<Option<linux::EventMonitor>>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(RecordingSession::new("Untitled".to_string()))),
            #[cfg(target_os = "macos")]
            event_tap: Arc::new(RwLock::new(None)),
            #[cfg(target_os = "windows")]
            event_hook: Arc::new(RwLock::new(None)),
            #[cfg(target_os = "linux")]
            event_monitor: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self, name: Option<String>) -> Result<(), String> {
        let mut session = self.session.write().await;

        if session.state == RecordingState::Recording {
            return Err("Recording already in progress".to_string());
        }

        session.name = name.unwrap_or_else(|| "Recording".to_string());
        session.state = RecordingState::Recording;
        session.started_at = Some(chrono::Utc::now().to_rfc3339());
        session.actions.clear();
        session.ended_at = None;

        drop(session);

        // Start platform-specific event listening
        #[cfg(target_os = "macos")]
        {
            self.start_macos_event_tap().await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.start_windows_event_hook().await?;
        }

        #[cfg(target_os = "linux")]
        {
            self.start_linux_event_monitor().await?;
        }

        log::info!("Recording started");
        Ok(())
    }

    pub async fn pause(&self) -> Result<(), String> {
        let mut session = self.session.write().await;

        if session.state != RecordingState::Recording {
            return Err("Not currently recording".to_string());
        }

        session.state = RecordingState::Paused;

        #[cfg(target_os = "macos")]
        {
            self.pause_macos_event_tap().await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.pause_windows_event_hook().await?;
        }

        #[cfg(target_os = "linux")]
        {
            self.pause_linux_event_monitor().await?;
        }

        log::info!("Recording paused");
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), String> {
        let mut session = self.session.write().await;

        if session.state != RecordingState::Paused {
            return Err("Recording not paused".to_string());
        }

        session.state = RecordingState::Recording;

        #[cfg(target_os = "macos")]
        {
            self.resume_macos_event_tap().await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.resume_windows_event_hook().await?;
        }

        #[cfg(target_os = "linux")]
        {
            self.resume_linux_event_monitor().await?;
        }

        log::info!("Recording resumed");
        Ok(())
    }

    pub async fn stop(&self) -> Result<RecordingSession, String> {
        let mut session = self.session.write().await;

        if session.state == RecordingState::Idle {
            return Err("No recording in progress".to_string());
        }

        session.state = RecordingState::Idle;
        session.ended_at = Some(chrono::Utc::now().to_rfc3339());

        #[cfg(target_os = "macos")]
        {
            self.stop_macos_event_tap().await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.stop_windows_event_hook().await?;
        }

        #[cfg(target_os = "linux")]
        {
            self.stop_linux_event_monitor().await?;
        }

        log::info!("Recording stopped with {} actions", session.actions.len());

        Ok(session.clone())
    }

    pub async fn get_state(&self) -> RecordingState {
        self.session.read().await.state.clone()
    }

    pub async fn get_session(&self) -> RecordingSession {
        self.session.read().await.clone()
    }

    pub async fn add_action(&self, action: RecordedAction) {
        let mut session = self.session.write().await;
        if session.state == RecordingState::Recording {
            session.actions.push(action);
        }
    }

    #[cfg(target_os = "macos")]
    async fn start_macos_event_tap(&self) -> Result<(), String> {
        let session_clone = self.session.clone();

        let tap = macos::EventTap::new(move |event| {
            let session = session_clone.clone();
            tokio::spawn(async move {
                let mut session = session.write().await;
                if session.state == RecordingState::Recording {
                    session.actions.push(event);
                }
            });
        })?;

        let mut event_tap = self.event_tap.write().await;
        *event_tap = Some(tap);

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn pause_macos_event_tap(&self) -> Result<(), String> {
        let event_tap = self.event_tap.read().await;
        if let Some(tap) = event_tap.as_ref() {
            tap.disable();
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn resume_macos_event_tap(&self) -> Result<(), String> {
        let event_tap = self.event_tap.read().await;
        if let Some(tap) = event_tap.as_ref() {
            tap.enable();
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn stop_macos_event_tap(&self) -> Result<(), String> {
        let mut event_tap = self.event_tap.write().await;
        *event_tap = None;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn start_windows_event_hook(&self) -> Result<(), String> {
        let session_clone = self.session.clone();

        let hook = windows::EventHook::new(move |event| {
            let session = session_clone.clone();
            tokio::spawn(async move {
                let mut session = session.write().await;
                if session.state == RecordingState::Recording {
                    session.actions.push(event);
                }
            });
        })?;

        let mut event_hook = self.event_hook.write().await;
        *event_hook = Some(hook);

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn pause_windows_event_hook(&self) -> Result<(), String> {
        let event_hook = self.event_hook.read().await;
        if let Some(hook) = event_hook.as_ref() {
            hook.disable();
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn resume_windows_event_hook(&self) -> Result<(), String> {
        let event_hook = self.event_hook.read().await;
        if let Some(hook) = event_hook.as_ref() {
            hook.enable();
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn stop_windows_event_hook(&self) -> Result<(), String> {
        let mut event_hook = self.event_hook.write().await;
        *event_hook = None;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn start_linux_event_monitor(&self) -> Result<(), String> {
        let session_clone = self.session.clone();

        let monitor = linux::EventMonitor::new(move |event| {
            let session = session_clone.clone();
            tokio::spawn(async move {
                let mut session = session.write().await;
                if session.state == RecordingState::Recording {
                    session.actions.push(event);
                }
            });
        })?;

        let mut event_monitor = self.event_monitor.write().await;
        *event_monitor = Some(monitor);

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn pause_linux_event_monitor(&self) -> Result<(), String> {
        let event_monitor = self.event_monitor.read().await;
        if let Some(monitor) = event_monitor.as_ref() {
            monitor.disable();
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn resume_linux_event_monitor(&self) -> Result<(), String> {
        let event_monitor = self.event_monitor.read().await;
        if let Some(monitor) = event_monitor.as_ref() {
            monitor.enable();
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn stop_linux_event_monitor(&self) -> Result<(), String> {
        let mut event_monitor = self.event_monitor.write().await;
        *event_monitor = None;
        Ok(())
    }
}

// Convert recorded actions to workflow nodes
pub fn convert_to_workflow(session: &RecordingSession) -> crate::engine::executor::Workflow {
    use crate::engine::executor::{Position, Workflow, WorkflowEdge, WorkflowNode};

    let mut nodes: Vec<WorkflowNode> = Vec::new();
    let mut edges: Vec<WorkflowEdge> = Vec::new();

    // Add start node
    let start_node = WorkflowNode {
        id: "start_node".to_string(),
        node_type: "start".to_string(),
        position: Position { x: 250.0, y: 0.0 },
        data: HashMap::new(),
        label: Some("Start".to_string()),
    };
    nodes.push(start_node);

    let mut prev_node_id = "start_node".to_string();
    let mut y_position = 100.0;

    for action in &session.actions {
        let (node_type, mut data) = match &action.action_type {
            RecordedActionType::Click => {
                let mut d = HashMap::new();
                d.insert(
                    "clickType".to_string(),
                    serde_json::Value::String("single".to_string()),
                );
                if let Some(element) = &action.element {
                    d.insert(
                        "elementId".to_string(),
                        serde_json::Value::String(element.id.clone()),
                    );
                    d.insert(
                        "elementName".to_string(),
                        serde_json::Value::String(element.name.clone()),
                    );
                }
                if let Some((x, y)) = action.position {
                    d.insert("x".to_string(), serde_json::Value::Number(x.into()));
                    d.insert("y".to_string(), serde_json::Value::Number(y.into()));
                }
                ("click".to_string(), d)
            }
            RecordedActionType::DoubleClick => {
                let mut d = HashMap::new();
                d.insert(
                    "clickType".to_string(),
                    serde_json::Value::String("double".to_string()),
                );
                if let Some(element) = &action.element {
                    d.insert(
                        "elementId".to_string(),
                        serde_json::Value::String(element.id.clone()),
                    );
                }
                if let Some((x, y)) = action.position {
                    d.insert("x".to_string(), serde_json::Value::Number(x.into()));
                    d.insert("y".to_string(), serde_json::Value::Number(y.into()));
                }
                ("click".to_string(), d)
            }
            RecordedActionType::RightClick => {
                let mut d = HashMap::new();
                d.insert(
                    "clickType".to_string(),
                    serde_json::Value::String("right".to_string()),
                );
                if let Some(element) = &action.element {
                    d.insert(
                        "elementId".to_string(),
                        serde_json::Value::String(element.id.clone()),
                    );
                }
                if let Some((x, y)) = action.position {
                    d.insert("x".to_string(), serde_json::Value::Number(x.into()));
                    d.insert("y".to_string(), serde_json::Value::Number(y.into()));
                }
                ("click".to_string(), d)
            }
            RecordedActionType::Input => {
                let mut d = HashMap::new();
                if let Some(text) = action.data.get("text") {
                    d.insert("text".to_string(), text.clone());
                }
                if let Some(element) = &action.element {
                    d.insert(
                        "elementId".to_string(),
                        serde_json::Value::String(element.id.clone()),
                    );
                }
                ("input".to_string(), d)
            }
            RecordedActionType::Scroll => {
                let mut d = HashMap::new();
                if let Some(delta_x) = action.data.get("deltaX") {
                    d.insert("deltaX".to_string(), delta_x.clone());
                }
                if let Some(delta_y) = action.data.get("deltaY") {
                    d.insert("deltaY".to_string(), delta_y.clone());
                }
                ("scroll".to_string(), d)
            }
            RecordedActionType::Hotkey => {
                let mut d = HashMap::new();
                if let Some(keys) = action.data.get("keys") {
                    d.insert("keys".to_string(), keys.clone());
                }
                if let Some(modifiers) = action.data.get("modifiers") {
                    d.insert("modifiers".to_string(), modifiers.clone());
                }
                ("hotkey".to_string(), d)
            }
            RecordedActionType::Wait => {
                let mut d = HashMap::new();
                let delay = action.data.get("delay").and_then(|v| v.as_u64()).unwrap_or(1000);
                d.insert("delay".to_string(), serde_json::Value::Number(delay.into()));
                ("delay".to_string(), d)
            }
        };

        // Copy additional data
        for (key, value) in &action.data {
            if !data.contains_key(key) {
                data.insert(key.clone(), value.clone());
            }
        }

        let node = WorkflowNode {
            id: action.id.clone(),
            node_type,
            position: Position {
                x: 250.0,
                y: y_position,
            },
            data,
            label: None,
        };
        nodes.push(node);

        // Create edge from previous node
        let edge = WorkflowEdge {
            id: format!("edge_{}_{}", prev_node_id, action.id),
            source: prev_node_id.clone(),
            target: action.id.clone(),
            source_handle: None,
            target_handle: None,
        };
        edges.push(edge);

        prev_node_id = action.id.clone();
        y_position += 100.0;
    }

    // Add end node
    let end_node = WorkflowNode {
        id: "end_node".to_string(),
        node_type: "end".to_string(),
        position: Position {
            x: 250.0,
            y: y_position,
        },
        data: HashMap::new(),
        label: Some("End".to_string()),
    };
    nodes.push(end_node);

    // Edge from last action to end
    let final_edge = WorkflowEdge {
        id: format!("edge_{}_end_node", prev_node_id),
        source: prev_node_id,
        target: "end_node".to_string(),
        source_handle: None,
        target_handle: None,
    };
    edges.push(final_edge);

    Workflow {
        id: uuid::Uuid::new_v4().to_string(),
        name: session.name.clone(),
        nodes,
        edges,
    }
}
