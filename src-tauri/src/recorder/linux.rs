#![cfg(target_os = "linux")]

use super::{RecordedAction, RecordedActionType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Linux event recording using xinput
pub struct EventMonitor {
    xinput_process: Option<Child>,
    enabled: Arc<AtomicBool>,
}

impl EventMonitor {
    pub fn new<F>(callback: F) -> Result<Self, String>
    where
        F: Fn(RecordedAction) + Send + Sync + 'static,
    {
        let enabled = Arc::new(AtomicBool::new(true));
        let enabled_clone = enabled.clone();

        // Start xinput test-xi2 to monitor events
        let mut child = Command::new("xinput")
            .args(["test-xi2", "--root"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start xinput: {}. Please install xinput.", e))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| "Failed to get xinput stdout".to_string())?;

        // Process xinput output in a separate thread
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut current_event: Option<EventData> = None;

            for line in reader.lines() {
                if !enabled_clone.load(Ordering::SeqCst) {
                    break;
                }

                let line = match line {
                    Ok(l) => l,
                    Err(_) => continue,
                };

                // Parse xinput output
                if let Some(event) = parse_xinput_line(&line, &mut current_event) {
                    callback(event);
                }
            }
        });

        Ok(Self {
            xinput_process: Some(child),
            enabled,
        })
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
        ENABLED.store(true, Ordering::SeqCst);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        ENABLED.store(false, Ordering::SeqCst);
    }
}

impl Drop for EventMonitor {
    fn drop(&mut self) {
        self.enabled.store(false, Ordering::SeqCst);
        if let Some(mut child) = self.xinput_process.take() {
            let _ = child.kill();
        }
    }
}

#[derive(Default)]
struct EventData {
    event_type: String,
    x: i32,
    y: i32,
    button: u32,
    keycode: u32,
}

fn parse_xinput_line(line: &str, current_event: &mut Option<EventData>) -> Option<RecordedAction> {
    let line = line.trim();

    // Event start
    if line.starts_with("EVENT type") {
        let mut event = EventData::default();

        // Parse event type
        if line.contains("ButtonPress") {
            event.event_type = "ButtonPress".to_string();
        } else if line.contains("ButtonRelease") {
            event.event_type = "ButtonRelease".to_string();
        } else if line.contains("KeyPress") {
            event.event_type = "KeyPress".to_string();
        } else if line.contains("KeyRelease") {
            event.event_type = "KeyRelease".to_string();
        } else if line.contains("Motion") {
            event.event_type = "Motion".to_string();
        }

        *current_event = Some(event);
        return None;
    }

    // Parse event details
    if let Some(ref mut event) = current_event {
        if line.starts_with("root:") {
            // Parse coordinates: root: 123.45/678.90
            if let Some(coords) = line.strip_prefix("root:") {
                let parts: Vec<&str> = coords.trim().split('/').collect();
                if parts.len() >= 2 {
                    event.x = parts[0].trim().parse::<f64>().unwrap_or(0.0) as i32;
                    event.y = parts[1].trim().parse::<f64>().unwrap_or(0.0) as i32;
                }
            }
        } else if line.starts_with("detail:") {
            // Parse button or keycode
            if let Some(detail) = line.strip_prefix("detail:") {
                let num = detail.trim().parse::<u32>().unwrap_or(0);
                if event.event_type.contains("Button") {
                    event.button = num;
                } else if event.event_type.contains("Key") {
                    event.keycode = num;
                }
            }
        }

        // Check if we have enough info to create an action
        // Only process on Press events to avoid duplicates
        if event.event_type == "ButtonPress" && event.button > 0 {
            let action = match event.button {
                1 => Some(RecordedAction::new(RecordedActionType::Click)
                    .with_position(event.x, event.y)),
                3 => Some(RecordedAction::new(RecordedActionType::RightClick)
                    .with_position(event.x, event.y)),
                4 | 5 => {
                    // Scroll wheel (4=up, 5=down)
                    let delta = if event.button == 4 { 120 } else { -120 };
                    Some(RecordedAction::new(RecordedActionType::Scroll)
                        .with_position(event.x, event.y)
                        .with_data("deltaY", serde_json::json!(delta)))
                }
                _ => None,
            };

            *current_event = None;
            return action;
        } else if event.event_type == "KeyPress" && event.keycode > 0 {
            // Convert keycode to key name using xdotool
            let key_name = get_key_name(event.keycode);

            let action = if is_printable_key(&key_name) {
                RecordedAction::new(RecordedActionType::Input)
                    .with_data("text", serde_json::json!(key_name))
                    .with_data("keycode", serde_json::json!(event.keycode))
            } else {
                RecordedAction::new(RecordedActionType::Hotkey)
                    .with_data("key", serde_json::json!(key_name))
                    .with_data("keycode", serde_json::json!(event.keycode))
            };

            *current_event = None;
            return Some(action);
        }
    }

    None
}

fn get_key_name(keycode: u32) -> String {
    // Use xdotool to convert keycode to key name
    let output = Command::new("xdotool")
        .args(["key", "--clearmodifiers", &format!("0x{:x}", keycode)])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => format!("key_{}", keycode),
    }
}

fn is_printable_key(key: &str) -> bool {
    // Check if it's a single printable character
    key.chars().count() == 1 && key.chars().next().map_or(false, |c| c.is_alphanumeric() || c.is_ascii_punctuation() || c == ' ')
}
