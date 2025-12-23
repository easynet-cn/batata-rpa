#![cfg(target_os = "linux")]

use crate::automation::{AutomationError, AutomationResult, ClickType, InputMethod, Rect};
use crate::element::UIElement;
use async_trait::async_trait;
use super::DesktopAutomation;
use std::collections::HashMap;
use std::process::Command;

use atspi::{
    connection::AccessibilityConnection,
    proxy::accessible::AccessibleProxy,
    AccessibilityBus,
    Role,
};
use zbus::Connection;

/// Linux automation using AT-SPI for accessibility and xdotool for input
pub struct LinuxAutomation {
    connection: Option<AccessibilityConnection>,
}

unsafe impl Send for LinuxAutomation {}
unsafe impl Sync for LinuxAutomation {}

impl LinuxAutomation {
    pub fn new() -> Self {
        // Initialize AT-SPI connection lazily
        Self { connection: None }
    }

    async fn ensure_connection(&mut self) -> AutomationResult<&AccessibilityConnection> {
        if self.connection.is_none() {
            let conn = AccessibilityConnection::new()
                .await
                .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to connect to AT-SPI: {}", e)))?;
            self.connection = Some(conn);
        }
        Ok(self.connection.as_ref().unwrap())
    }

    /// Get element at screen position using AT-SPI
    async fn get_element_at_position(&mut self, x: i32, y: i32) -> AutomationResult<UIElement> {
        // Use xdotool to get window at position, then query AT-SPI
        let output = Command::new("xdotool")
            .args(["getmouselocation", "--shell"])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("xdotool failed: {}", e)))?;

        if !output.status.success() {
            return Err(AutomationError::ExecutionFailed("xdotool getmouselocation failed".to_string()));
        }

        // Get window ID at position
        let output = Command::new("xdotool")
            .args(["mousemove", &x.to_string(), &y.to_string()])
            .output();

        let window_output = Command::new("xdotool")
            .args(["getwindowfocus"])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("xdotool failed: {}", e)))?;

        let window_id = String::from_utf8_lossy(&window_output.stdout).trim().to_string();

        // Get window name
        let name_output = Command::new("xdotool")
            .args(["getwindowname", &window_id])
            .output()
            .ok();

        let window_name = name_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Create element with available information
        let element = UIElement::new(
            window_name.clone(),
            "Window".to_string(),
            Rect { x, y, width: 100, height: 50 },
        );

        Ok(element)
    }

    /// Perform mouse click using xdotool
    fn perform_click(&self, x: i32, y: i32, click_type: &ClickType) -> AutomationResult<()> {
        // Move mouse to position
        Command::new("xdotool")
            .args(["mousemove", &x.to_string(), &y.to_string()])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("xdotool mousemove failed: {}", e)))?;

        // Perform click
        let button = match click_type {
            ClickType::Single | ClickType::Double => "1",
            ClickType::Right => "3",
        };

        let click_count = match click_type {
            ClickType::Double => "2",
            _ => "1",
        };

        Command::new("xdotool")
            .args(["click", "--repeat", click_count, button])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("xdotool click failed: {}", e)))?;

        Ok(())
    }

    /// Type text using xdotool
    fn type_text(&self, text: &str) -> AutomationResult<()> {
        Command::new("xdotool")
            .args(["type", "--clearmodifiers", text])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("xdotool type failed: {}", e)))?;

        Ok(())
    }

    /// Get window title at position
    fn get_window_title_at(&self, x: i32, y: i32) -> Option<String> {
        // Move mouse temporarily and get window
        let _ = Command::new("xdotool")
            .args(["mousemove", &x.to_string(), &y.to_string()])
            .output();

        let output = Command::new("xdotool")
            .args(["getactivewindow", "getwindowname"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Get process name from window
    fn get_process_name(&self, window_id: &str) -> Option<String> {
        let output = Command::new("xdotool")
            .args(["getwindowpid", window_id])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Get process name from /proc
        let cmdline = std::fs::read_to_string(format!("/proc/{}/comm", pid)).ok()?;
        Some(cmdline.trim().to_string())
    }

    /// Take screenshot using scrot or gnome-screenshot
    fn take_screenshot(&self, rect: Option<Rect>) -> AutomationResult<Vec<u8>> {
        let temp_path = "/tmp/batata_rpa_screenshot.png";

        let result = match rect {
            Some(r) => {
                // Use scrot with geometry
                Command::new("scrot")
                    .args([
                        "-a", &format!("{},{},{},{}", r.x, r.y, r.width, r.height),
                        temp_path
                    ])
                    .output()
            }
            None => {
                // Full screen
                Command::new("scrot")
                    .args([temp_path])
                    .output()
            }
        };

        match result {
            Ok(output) if output.status.success() => {
                let data = std::fs::read(temp_path)
                    .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read screenshot: {}", e)))?;
                let _ = std::fs::remove_file(temp_path);
                Ok(data)
            }
            Ok(output) => {
                // Try gnome-screenshot as fallback
                let fallback = Command::new("gnome-screenshot")
                    .args(["-f", temp_path])
                    .output();

                match fallback {
                    Ok(out) if out.status.success() => {
                        let data = std::fs::read(temp_path)
                            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read screenshot: {}", e)))?;
                        let _ = std::fs::remove_file(temp_path);
                        Ok(data)
                    }
                    _ => Err(AutomationError::ExecutionFailed(
                        "Screenshot failed. Please install scrot or gnome-screenshot".to_string()
                    ))
                }
            }
            Err(e) => Err(AutomationError::ExecutionFailed(format!("Screenshot command failed: {}", e)))
        }
    }

    /// Find element matching criteria using AT-SPI
    async fn find_element_by_criteria(&mut self, criteria: &HashMap<&str, &str>) -> Option<UIElement> {
        // This is a simplified search - full AT-SPI traversal would be more complex
        // For now, use xdotool to search by window name

        if let Some(name) = criteria.get("name").or(criteria.get("title")) {
            let output = Command::new("xdotool")
                .args(["search", "--name", name])
                .output()
                .ok()?;

            if output.status.success() {
                let window_ids: Vec<&str> = std::str::from_utf8(&output.stdout)
                    .ok()?
                    .trim()
                    .lines()
                    .collect();

                if let Some(window_id) = window_ids.first() {
                    // Get window geometry
                    let geo_output = Command::new("xdotool")
                        .args(["getwindowgeometry", "--shell", window_id])
                        .output()
                        .ok()?;

                    let geo_str = String::from_utf8_lossy(&geo_output.stdout);
                    let (x, y, width, height) = parse_geometry(&geo_str)?;

                    // Get window name
                    let name_output = Command::new("xdotool")
                        .args(["getwindowname", window_id])
                        .output()
                        .ok()?;

                    let window_name = String::from_utf8_lossy(&name_output.stdout).trim().to_string();

                    let mut element = UIElement::new(
                        window_name,
                        "Window".to_string(),
                        Rect { x, y, width, height },
                    );

                    element.window_title = Some(window_id.to_string());

                    if let Some(process_name) = self.get_process_name(window_id) {
                        element.process_name = Some(process_name);
                    }

                    return Some(element);
                }
            }
        }

        None
    }
}

/// Parse xdotool geometry output
fn parse_geometry(geo_str: &str) -> Option<(i32, i32, i32, i32)> {
    let mut x = 0;
    let mut y = 0;
    let mut width = 0;
    let mut height = 0;

    for line in geo_str.lines() {
        if line.starts_with("X=") {
            x = line[2..].parse().ok()?;
        } else if line.starts_with("Y=") {
            y = line[2..].parse().ok()?;
        } else if line.starts_with("WIDTH=") {
            width = line[6..].parse().ok()?;
        } else if line.starts_with("HEIGHT=") {
            height = line[7..].parse().ok()?;
        }
    }

    Some((x, y, width, height))
}

#[async_trait]
impl DesktopAutomation for LinuxAutomation {
    async fn click(&self, element: &UIElement, click_type: ClickType) -> AutomationResult<()> {
        let (x, y) = element.bounds.center();
        log::info!("Linux: Click at ({}, {}) with type {:?}", x, y, click_type);

        self.perform_click(x, y, &click_type)?;

        // Small delay after click
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Ok(())
    }

    async fn input(
        &self,
        element: &UIElement,
        text: &str,
        method: InputMethod,
    ) -> AutomationResult<()> {
        log::info!("Linux: Input '{}' to element {} with method {:?}", text, element.name, method);

        let (x, y) = element.bounds.center();

        match method {
            InputMethod::Type => {
                // Click to focus
                self.perform_click(x, y, &ClickType::Single)?;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Type the text
                self.type_text(text)?;
            }
            InputMethod::Set => {
                // For Set mode, we also use typing on Linux
                // (clipboard paste could be an alternative)
                self.perform_click(x, y, &ClickType::Single)?;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Select all and replace
                Command::new("xdotool")
                    .args(["key", "ctrl+a"])
                    .output()
                    .ok();

                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                self.type_text(text)?;
            }
        }

        Ok(())
    }

    async fn get_text(&self, element: &UIElement) -> AutomationResult<String> {
        log::info!("Linux: Get text from element {}", element.name);

        // Getting text from arbitrary elements is limited without full AT-SPI support
        // Return element name as fallback
        Ok(element.name.clone())
    }

    async fn get_attribute(&self, element: &UIElement, name: &str) -> AutomationResult<String> {
        log::info!("Linux: Get attribute '{}' from element {}", name, element.name);

        // Return from stored attributes or empty
        Ok(element.attributes.get(name).cloned().unwrap_or_default())
    }

    async fn wait_element(&self, locator: &str, timeout_ms: u64) -> AutomationResult<UIElement> {
        log::info!("Linux: Wait for element '{}' with timeout {}ms", locator, timeout_ms);

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let poll_interval = std::time::Duration::from_millis(100);

        // Parse locator
        let criteria: HashMap<&str, &str> = locator
            .split(',')
            .filter_map(|part| {
                let mut parts = part.trim().splitn(2, ':');
                Some((parts.next()?, parts.next()?))
            })
            .collect();

        // Create mutable self for async operations
        let mut self_mut = LinuxAutomation::new();

        while start.elapsed() < timeout {
            if let Some(element) = self_mut.find_element_by_criteria(&criteria).await {
                return Ok(element);
            }
            tokio::time::sleep(poll_interval).await;
        }

        Err(AutomationError::Timeout(format!(
            "Element '{}' not found within {}ms",
            locator, timeout_ms
        )))
    }

    async fn capture_element(&self, x: i32, y: i32) -> AutomationResult<UIElement> {
        log::info!("Linux: Capture element at ({}, {})", x, y);

        let mut self_mut = LinuxAutomation::new();
        self_mut.get_element_at_position(x, y).await
    }

    async fn get_element_bounds(&self, element: &UIElement) -> AutomationResult<Rect> {
        Ok(element.bounds.clone())
    }

    async fn screenshot(&self, rect: Option<Rect>) -> AutomationResult<Vec<u8>> {
        log::info!("Linux: Take screenshot with rect {:?}", rect);
        self.take_screenshot(rect)
    }
}
