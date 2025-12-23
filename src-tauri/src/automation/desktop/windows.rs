#![cfg(target_os = "windows")]

use crate::automation::{AutomationError, AutomationResult, ClickType, InputMethod, Rect};
use crate::element::UIElement;
use async_trait::async_trait;
use super::DesktopAutomation;
use std::collections::HashMap;

use uiautomation::{
    UIAutomation,
    UIElement as UIAElement,
    types::UIProperty,
    controls::ControlType,
};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, INPUT_KEYBOARD,
    MOUSEINPUT, KEYBDINPUT,
    MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SetCursorPos, GetSystemMetrics,
    SM_CXSCREEN, SM_CYSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
};
use windows::Win32::Foundation::POINT;

pub struct WindowsAutomation {
    automation: UIAutomation,
}

unsafe impl Send for WindowsAutomation {}
unsafe impl Sync for WindowsAutomation {}

impl WindowsAutomation {
    pub fn new() -> Self {
        let automation = UIAutomation::new().expect("Failed to initialize UI Automation");
        Self { automation }
    }

    fn get_element_at_point(&self, x: i32, y: i32) -> AutomationResult<UIAElement> {
        let point = POINT { x, y };
        self.automation
            .element_from_point(point)
            .map_err(|e| AutomationError::ElementNotFound(format!("No element at ({}, {}): {}", x, y, e)))
    }

    fn uia_element_to_ui_element(&self, element: &UIAElement) -> AutomationResult<UIElement> {
        // Get control type
        let control_type = element
            .get_control_type()
            .map(|ct| control_type_to_string(ct))
            .unwrap_or_else(|_| "Unknown".to_string());

        // Get name
        let name = element
            .get_name()
            .unwrap_or_else(|_| String::new());

        let display_name = if name.is_empty() {
            control_type.clone()
        } else {
            name
        };

        // Get bounding rectangle
        let rect = element
            .get_bounding_rectangle()
            .map(|r| Rect {
                x: r.left,
                y: r.top,
                width: r.right - r.left,
                height: r.bottom - r.top,
            })
            .unwrap_or_else(|_| Rect { x: 0, y: 0, width: 0, height: 0 });

        let mut ui_element = UIElement::new(display_name, control_type, rect);

        // Get automation ID
        if let Ok(automation_id) = element.get_automation_id() {
            if !automation_id.is_empty() {
                ui_element.automation_id = Some(automation_id);
            }
        }

        // Get class name
        if let Ok(class_name) = element.get_classname() {
            if !class_name.is_empty() {
                ui_element.class_name = Some(class_name);
            }
        }

        // Get process ID and name
        if let Ok(pid) = element.get_process_id() {
            ui_element.attributes.insert("pid".to_string(), pid.to_string());
            if let Some(process_name) = self.get_process_name(pid) {
                ui_element.process_name = Some(process_name);
            }
        }

        // Try to get window title from parent window
        if let Some(window_title) = self.get_window_title(element) {
            ui_element.window_title = Some(window_title);
        }

        // Get additional properties
        if let Ok(is_enabled) = element.get_is_enabled() {
            ui_element.attributes.insert("enabled".to_string(), is_enabled.to_string());
        }

        if let Ok(has_keyboard_focus) = element.get_has_keyboard_focus() {
            ui_element.attributes.insert("focused".to_string(), has_keyboard_focus.to_string());
        }

        // Get value if it's a value pattern element
        if let Ok(value) = self.get_element_value(element) {
            if !value.is_empty() {
                ui_element.attributes.insert("value".to_string(), value);
            }
        }

        Ok(ui_element)
    }

    fn get_process_name(&self, pid: u32) -> Option<String> {
        use std::process::Command;

        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output()
            .ok()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Parse CSV: "process_name","pid",...
            let parts: Vec<&str> = output_str.trim().split(',').collect();
            if !parts.is_empty() {
                let name = parts[0].trim_matches('"').to_string();
                if !name.is_empty() && name != "INFO:" {
                    return Some(name);
                }
            }
        }
        None
    }

    fn get_window_title(&self, element: &UIAElement) -> Option<String> {
        // Walk up to find a window element
        let mut current = element.clone();

        for _ in 0..20 {  // Max depth to prevent infinite loop
            if let Ok(control_type) = current.get_control_type() {
                if control_type == ControlType::Window {
                    if let Ok(name) = current.get_name() {
                        if !name.is_empty() {
                            return Some(name);
                        }
                    }
                }
            }

            // Try to get parent
            match self.automation.create_tree_walker().and_then(|w| w.get_parent(&current)) {
                Ok(parent) => current = parent,
                Err(_) => break,
            }
        }

        None
    }

    fn get_element_value(&self, element: &UIAElement) -> AutomationResult<String> {
        // Try Value pattern
        if let Ok(pattern) = element.get_value_pattern() {
            if let Ok(value) = pattern.get_value() {
                return Ok(value);
            }
        }

        // Try Text pattern
        if let Ok(pattern) = element.get_text_pattern() {
            if let Ok(range) = pattern.get_document_range() {
                if let Ok(text) = range.get_text(-1) {
                    return Ok(text);
                }
            }
        }

        Ok(String::new())
    }

    fn set_element_value(&self, element: &UIAElement, value: &str) -> AutomationResult<()> {
        // Try Value pattern
        if let Ok(pattern) = element.get_value_pattern() {
            pattern.set_value(value)
                .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to set value: {}", e)))?;
            return Ok(());
        }

        Err(AutomationError::ExecutionFailed("Element does not support value pattern".to_string()))
    }

    fn focus_element(&self, element: &UIAElement) -> AutomationResult<()> {
        element.set_focus()
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to focus element: {}", e)))
    }

    fn perform_click(&self, x: i32, y: i32, click_type: &ClickType) -> AutomationResult<()> {
        // Move cursor to position
        unsafe {
            SetCursorPos(x, y)
                .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to set cursor position: {}", e)))?;
        }

        // Small delay after moving
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Get screen dimensions for absolute coordinates
        let (screen_width, screen_height) = unsafe {
            (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
        };

        let abs_x = (x as f64 / screen_width as f64 * 65535.0) as i32;
        let abs_y = (y as f64 / screen_height as f64 * 65535.0) as i32;

        match click_type {
            ClickType::Single => {
                self.send_mouse_click(abs_x, abs_y, false, 1)?;
            }
            ClickType::Double => {
                self.send_mouse_click(abs_x, abs_y, false, 2)?;
            }
            ClickType::Right => {
                self.send_mouse_click(abs_x, abs_y, true, 1)?;
            }
        }

        Ok(())
    }

    fn send_mouse_click(&self, abs_x: i32, abs_y: i32, right_button: bool, click_count: u32) -> AutomationResult<()> {
        let (down_flag, up_flag) = if right_button {
            (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
        } else {
            (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
        };

        for _ in 0..click_count {
            // Mouse down
            let mut input_down = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: abs_x,
                        dy: abs_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | down_flag,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            // Mouse up
            let mut input_up = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: abs_x,
                        dy: abs_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | up_flag,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            unsafe {
                SendInput(&[input_down], std::mem::size_of::<INPUT>() as i32);
                std::thread::sleep(std::time::Duration::from_millis(10));
                SendInput(&[input_up], std::mem::size_of::<INPUT>() as i32);
            }

            if click_count > 1 {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        Ok(())
    }

    fn type_text(&self, text: &str) -> AutomationResult<()> {
        for ch in text.chars() {
            self.type_character(ch)?;
        }
        Ok(())
    }

    fn type_character(&self, ch: char) -> AutomationResult<()> {
        let char_code = ch as u16;

        // Key down
        let mut input_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: char_code,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // Key up
        let mut input_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: char_code,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        unsafe {
            SendInput(&[input_down], std::mem::size_of::<INPUT>() as i32);
            std::thread::sleep(std::time::Duration::from_millis(5));
            SendInput(&[input_up], std::mem::size_of::<INPUT>() as i32);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
        Ok(())
    }

    fn invoke_element(&self, element: &UIAElement) -> AutomationResult<()> {
        if let Ok(pattern) = element.get_invoke_pattern() {
            pattern.invoke()
                .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to invoke: {}", e)))?;
            return Ok(());
        }

        // Fallback to click
        if let Ok(rect) = element.get_bounding_rectangle() {
            let x = (rect.left + rect.right) / 2;
            let y = (rect.top + rect.bottom) / 2;
            return self.perform_click(x, y, &ClickType::Single);
        }

        Err(AutomationError::ExecutionFailed("Cannot invoke element".to_string()))
    }

    fn find_element_by_criteria(&self, criteria: &HashMap<&str, &str>) -> Option<UIAElement> {
        let root = self.automation.get_root_element().ok()?;
        self.search_element_recursive(&root, criteria, 10)
    }

    fn search_element_recursive(
        &self,
        element: &UIAElement,
        criteria: &HashMap<&str, &str>,
        max_depth: u32,
    ) -> Option<UIAElement> {
        if max_depth == 0 {
            return None;
        }

        if self.element_matches_criteria(element, criteria) {
            return Some(element.clone());
        }

        // Get children and search
        let walker = self.automation.create_tree_walker().ok()?;
        let mut child = walker.get_first_child(element).ok();

        while let Some(ref c) = child {
            if let Some(found) = self.search_element_recursive(c, criteria, max_depth - 1) {
                return Some(found);
            }
            child = walker.get_next_sibling(c).ok();
        }

        None
    }

    fn element_matches_criteria(&self, element: &UIAElement, criteria: &HashMap<&str, &str>) -> bool {
        for (key, expected_value) in criteria {
            let actual_value = match key.to_lowercase().as_str() {
                "name" | "title" => element.get_name().ok(),
                "automationid" | "id" => element.get_automation_id().ok(),
                "classname" | "class" => element.get_classname().ok(),
                "controltype" | "type" => element.get_control_type().ok().map(|ct| control_type_to_string(ct)),
                _ => None,
            };

            match actual_value {
                Some(actual) if actual.to_lowercase().contains(&expected_value.to_lowercase()) => continue,
                _ => return false,
            }
        }
        true
    }
}

#[async_trait]
impl DesktopAutomation for WindowsAutomation {
    async fn click(&self, element: &UIElement, click_type: ClickType) -> AutomationResult<()> {
        let (x, y) = element.bounds.center();
        log::info!("Windows: Click at ({}, {}) with type {:?}", x, y, click_type);

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
        log::info!("Windows: Input '{}' to element {} with method {:?}", text, element.name, method);

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
                // Try to set value directly via UI Automation
                let uia_element = self.get_element_at_point(x, y)?;

                // Focus the element
                let _ = self.focus_element(&uia_element);
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                // Try to set value directly
                if self.set_element_value(&uia_element, text).is_err() {
                    // Fallback to typing
                    self.type_text(text)?;
                }
            }
        }

        Ok(())
    }

    async fn get_text(&self, element: &UIElement) -> AutomationResult<String> {
        log::info!("Windows: Get text from element {}", element.name);

        let (x, y) = element.bounds.center();
        let uia_element = self.get_element_at_point(x, y)?;

        // Try to get value
        if let Ok(value) = self.get_element_value(&uia_element) {
            if !value.is_empty() {
                return Ok(value);
            }
        }

        // Fallback to name
        if let Ok(name) = uia_element.get_name() {
            return Ok(name);
        }

        Ok(String::new())
    }

    async fn get_attribute(&self, element: &UIElement, name: &str) -> AutomationResult<String> {
        log::info!("Windows: Get attribute '{}' from element {}", name, element.name);

        let (x, y) = element.bounds.center();
        let uia_element = self.get_element_at_point(x, y)?;

        let value = match name.to_lowercase().as_str() {
            "value" => self.get_element_value(&uia_element).unwrap_or_default(),
            "name" | "title" => uia_element.get_name().unwrap_or_default(),
            "automationid" | "id" => uia_element.get_automation_id().unwrap_or_default(),
            "classname" | "class" => uia_element.get_classname().unwrap_or_default(),
            "controltype" | "type" => uia_element.get_control_type()
                .map(|ct| control_type_to_string(ct))
                .unwrap_or_default(),
            "enabled" => uia_element.get_is_enabled()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            "focused" => uia_element.get_has_keyboard_focus()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        };

        Ok(value)
    }

    async fn wait_element(&self, locator: &str, timeout_ms: u64) -> AutomationResult<UIElement> {
        log::info!("Windows: Wait for element '{}' with timeout {}ms", locator, timeout_ms);

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let poll_interval = std::time::Duration::from_millis(100);

        // Parse locator - support formats like "name:Button1,type:Button"
        let criteria: HashMap<&str, &str> = locator
            .split(',')
            .filter_map(|part| {
                let mut parts = part.trim().splitn(2, ':');
                Some((parts.next()?, parts.next()?))
            })
            .collect();

        while start.elapsed() < timeout {
            if let Some(element) = self.find_element_by_criteria(&criteria) {
                return self.uia_element_to_ui_element(&element);
            }
            tokio::time::sleep(poll_interval).await;
        }

        Err(AutomationError::Timeout(format!(
            "Element '{}' not found within {}ms",
            locator, timeout_ms
        )))
    }

    async fn capture_element(&self, x: i32, y: i32) -> AutomationResult<UIElement> {
        log::info!("Windows: Capture element at ({}, {})", x, y);

        let uia_element = self.get_element_at_point(x, y)?;
        self.uia_element_to_ui_element(&uia_element)
    }

    async fn get_element_bounds(&self, element: &UIElement) -> AutomationResult<Rect> {
        Ok(element.bounds.clone())
    }

    async fn screenshot(&self, rect: Option<Rect>) -> AutomationResult<Vec<u8>> {
        log::info!("Windows: Take screenshot with rect {:?}", rect);

        use std::process::Command;
        use std::fs;

        // Use PowerShell to capture screenshot
        let temp_path = std::env::temp_dir().join("batata_rpa_screenshot.png");
        let temp_path_str = temp_path.to_string_lossy();

        let script = match rect {
            Some(r) => format!(
                r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bitmap = New-Object System.Drawing.Bitmap({}, {})
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen({}, {}, 0, 0, $bitmap.Size)
$bitmap.Save('{}')
$graphics.Dispose()
$bitmap.Dispose()
"#,
                r.width, r.height, r.x, r.y, temp_path_str
            ),
            None => format!(
                r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$screen = [System.Windows.Forms.Screen]::PrimaryScreen
$bitmap = New-Object System.Drawing.Bitmap($screen.Bounds.Width, $screen.Bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($screen.Bounds.Location, [System.Drawing.Point]::Empty, $screen.Bounds.Size)
$bitmap.Save('{}')
$graphics.Dispose()
$bitmap.Dispose()
"#,
                temp_path_str
            ),
        };

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to run PowerShell: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AutomationError::ExecutionFailed(format!("PowerShell screenshot failed: {}", error)));
        }

        let data = fs::read(&temp_path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read screenshot: {}", e)))?;

        let _ = fs::remove_file(&temp_path);

        Ok(data)
    }
}

fn control_type_to_string(ct: ControlType) -> String {
    match ct {
        ControlType::Button => "Button",
        ControlType::Calendar => "Calendar",
        ControlType::CheckBox => "CheckBox",
        ControlType::ComboBox => "ComboBox",
        ControlType::Edit => "Edit",
        ControlType::Hyperlink => "Hyperlink",
        ControlType::Image => "Image",
        ControlType::ListItem => "ListItem",
        ControlType::List => "List",
        ControlType::Menu => "Menu",
        ControlType::MenuBar => "MenuBar",
        ControlType::MenuItem => "MenuItem",
        ControlType::ProgressBar => "ProgressBar",
        ControlType::RadioButton => "RadioButton",
        ControlType::ScrollBar => "ScrollBar",
        ControlType::Slider => "Slider",
        ControlType::Spinner => "Spinner",
        ControlType::StatusBar => "StatusBar",
        ControlType::Tab => "Tab",
        ControlType::TabItem => "TabItem",
        ControlType::Text => "Text",
        ControlType::ToolBar => "ToolBar",
        ControlType::ToolTip => "ToolTip",
        ControlType::Tree => "Tree",
        ControlType::TreeItem => "TreeItem",
        ControlType::Custom => "Custom",
        ControlType::Group => "Group",
        ControlType::Thumb => "Thumb",
        ControlType::DataGrid => "DataGrid",
        ControlType::DataItem => "DataItem",
        ControlType::Document => "Document",
        ControlType::SplitButton => "SplitButton",
        ControlType::Window => "Window",
        ControlType::Pane => "Pane",
        ControlType::Header => "Header",
        ControlType::HeaderItem => "HeaderItem",
        ControlType::Table => "Table",
        ControlType::TitleBar => "TitleBar",
        ControlType::Separator => "Separator",
        ControlType::SemanticZoom => "SemanticZoom",
        ControlType::AppBar => "AppBar",
        _ => "Unknown",
    }.to_string()
}
