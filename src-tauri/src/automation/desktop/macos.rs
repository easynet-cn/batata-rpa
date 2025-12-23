#![cfg(target_os = "macos")]

use crate::automation::{AutomationError, AutomationResult, ClickType, InputMethod, Rect};
use crate::element::UIElement;
use async_trait::async_trait;
use super::DesktopAutomation;
use std::collections::HashMap;
use std::ffi::c_void;

use core_foundation::base::{CFRelease, CFTypeRef, TCFType, ToVoid};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::boolean::CFBoolean;

use core_graphics::display::CGDisplay;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

// Accessibility framework bindings
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyElementAtPosition(
        application: AXUIElementRef,
        x: f32,
        y: f32,
        element: *mut AXUIElementRef,
    ) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementCopyAttributeNames(
        element: AXUIElementRef,
        names: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> i32;
    fn AXUIElementPerformAction(
        element: AXUIElementRef,
        action: CFStringRef,
    ) -> i32;
    fn AXUIElementGetPid(
        element: AXUIElementRef,
        pid: *mut i32,
    ) -> i32;
}

type AXUIElementRef = *mut c_void;

// AXError codes
const K_AX_ERROR_SUCCESS: i32 = 0;
const K_AX_ERROR_ATTRIBUTE_UNSUPPORTED: i32 = -25205;
const K_AX_ERROR_NO_VALUE: i32 = -25212;

// Common attribute names
const K_AX_ROLE_ATTRIBUTE: &str = "AXRole";
const K_AX_TITLE_ATTRIBUTE: &str = "AXTitle";
const K_AX_DESCRIPTION_ATTRIBUTE: &str = "AXDescription";
const K_AX_VALUE_ATTRIBUTE: &str = "AXValue";
const K_AX_POSITION_ATTRIBUTE: &str = "AXPosition";
const K_AX_SIZE_ATTRIBUTE: &str = "AXSize";
const K_AX_CHILDREN_ATTRIBUTE: &str = "AXChildren";
const K_AX_PARENT_ATTRIBUTE: &str = "AXParent";
const K_AX_FOCUSED_ATTRIBUTE: &str = "AXFocused";
const K_AX_ENABLED_ATTRIBUTE: &str = "AXEnabled";
const K_AX_IDENTIFIER_ATTRIBUTE: &str = "AXIdentifier";
const K_AX_WINDOW_ATTRIBUTE: &str = "AXWindow";
const K_AX_WINDOWS_ATTRIBUTE: &str = "AXWindows";
const K_AX_FOCUSED_WINDOW_ATTRIBUTE: &str = "AXFocusedWindow";

// Action names
const K_AX_PRESS_ACTION: &str = "AXPress";
const K_AX_SHOW_MENU_ACTION: &str = "AXShowMenu";

pub struct MacOSAutomation {
    system_wide: AXUIElementRef,
}

unsafe impl Send for MacOSAutomation {}
unsafe impl Sync for MacOSAutomation {}

impl MacOSAutomation {
    pub fn new() -> Self {
        let system_wide = unsafe { AXUIElementCreateSystemWide() };
        Self { system_wide }
    }

    fn check_accessibility(&self) -> AutomationResult<()> {
        if !unsafe { AXIsProcessTrusted() } {
            return Err(AutomationError::ExecutionFailed(
                "Accessibility permission not granted. Please enable in System Preferences > Security & Privacy > Privacy > Accessibility".to_string()
            ));
        }
        Ok(())
    }

    fn get_element_at_position(&self, x: f32, y: f32) -> AutomationResult<AXUIElementRef> {
        self.check_accessibility()?;

        let mut element: AXUIElementRef = std::ptr::null_mut();
        let result = unsafe {
            AXUIElementCopyElementAtPosition(self.system_wide, x, y, &mut element)
        };

        if result != K_AX_ERROR_SUCCESS || element.is_null() {
            return Err(AutomationError::ElementNotFound(
                format!("No element found at position ({}, {})", x, y)
            ));
        }

        Ok(element)
    }

    fn get_attribute_string(&self, element: AXUIElementRef, attr: &str) -> Option<String> {
        let cf_attr = CFString::new(attr);
        let mut value: CFTypeRef = std::ptr::null_mut();

        let result = unsafe {
            AXUIElementCopyAttributeValue(element, cf_attr.as_concrete_TypeRef(), &mut value)
        };

        if result != K_AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }

        // Try to get as CFString
        let type_id = unsafe { core_foundation::base::CFGetTypeID(value) };
        if type_id == unsafe { core_foundation::string::CFStringGetTypeID() } {
            let cf_string: CFString = unsafe { TCFType::wrap_under_create_rule(value as CFStringRef) };
            return Some(cf_string.to_string());
        }

        unsafe { CFRelease(value) };
        None
    }

    fn get_attribute_bool(&self, element: AXUIElementRef, attr: &str) -> Option<bool> {
        let cf_attr = CFString::new(attr);
        let mut value: CFTypeRef = std::ptr::null_mut();

        let result = unsafe {
            AXUIElementCopyAttributeValue(element, cf_attr.as_concrete_TypeRef(), &mut value)
        };

        if result != K_AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }

        let type_id = unsafe { core_foundation::base::CFGetTypeID(value) };
        if type_id == unsafe { core_foundation::boolean::CFBooleanGetTypeID() } {
            let cf_bool: CFBoolean = unsafe { TCFType::wrap_under_create_rule(value as core_foundation::boolean::CFBooleanRef) };
            return Some(cf_bool.into());
        }

        unsafe { CFRelease(value) };
        None
    }

    fn get_element_bounds(&self, element: AXUIElementRef) -> AutomationResult<Rect> {
        // Get position
        let cf_pos_attr = CFString::new(K_AX_POSITION_ATTRIBUTE);
        let mut pos_value: CFTypeRef = std::ptr::null_mut();

        let pos_result = unsafe {
            AXUIElementCopyAttributeValue(element, cf_pos_attr.as_concrete_TypeRef(), &mut pos_value)
        };

        if pos_result != K_AX_ERROR_SUCCESS || pos_value.is_null() {
            return Err(AutomationError::ExecutionFailed("Failed to get element position".to_string()));
        }

        // Get size
        let cf_size_attr = CFString::new(K_AX_SIZE_ATTRIBUTE);
        let mut size_value: CFTypeRef = std::ptr::null_mut();

        let size_result = unsafe {
            AXUIElementCopyAttributeValue(element, cf_size_attr.as_concrete_TypeRef(), &mut size_value)
        };

        if size_result != K_AX_ERROR_SUCCESS || size_value.is_null() {
            unsafe { CFRelease(pos_value) };
            return Err(AutomationError::ExecutionFailed("Failed to get element size".to_string()));
        }

        // Parse AXValue for CGPoint and CGSize
        let (x, y) = self.extract_point(pos_value)?;
        let (width, height) = self.extract_size(size_value)?;

        unsafe {
            CFRelease(pos_value);
            CFRelease(size_value);
        }

        Ok(Rect {
            x: x as i32,
            y: y as i32,
            width: width as i32,
            height: height as i32,
        })
    }

    fn extract_point(&self, value: CFTypeRef) -> AutomationResult<(f64, f64)> {
        // AXValue containing CGPoint
        let mut point = core_graphics::geometry::CGPoint::new(0.0, 0.0);
        let result = unsafe {
            AXValueGetValue(
                value as AXValueRef,
                1, // kAXValueCGPointType
                &mut point as *mut _ as *mut c_void,
            )
        };

        if result {
            Ok((point.x, point.y))
        } else {
            Err(AutomationError::ExecutionFailed("Failed to extract point value".to_string()))
        }
    }

    fn extract_size(&self, value: CFTypeRef) -> AutomationResult<(f64, f64)> {
        // AXValue containing CGSize
        let mut size = core_graphics::geometry::CGSize::new(0.0, 0.0);
        let result = unsafe {
            AXValueGetValue(
                value as AXValueRef,
                2, // kAXValueCGSizeType
                &mut size as *mut _ as *mut c_void,
            )
        };

        if result {
            Ok((size.width, size.height))
        } else {
            Err(AutomationError::ExecutionFailed("Failed to extract size value".to_string()))
        }
    }

    fn element_to_ui_element(&self, ax_element: AXUIElementRef) -> AutomationResult<UIElement> {
        let role = self.get_attribute_string(ax_element, K_AX_ROLE_ATTRIBUTE)
            .unwrap_or_else(|| "Unknown".to_string());

        let title = self.get_attribute_string(ax_element, K_AX_TITLE_ATTRIBUTE)
            .or_else(|| self.get_attribute_string(ax_element, K_AX_DESCRIPTION_ATTRIBUTE))
            .unwrap_or_else(|| "Unnamed".to_string());

        let bounds = self.get_element_bounds(ax_element)?;

        let mut element = UIElement::new(title, role, bounds);

        // Get automation ID
        if let Some(identifier) = self.get_attribute_string(ax_element, K_AX_IDENTIFIER_ATTRIBUTE) {
            element.automation_id = Some(identifier);
        }

        // Get value if available
        if let Some(value) = self.get_attribute_string(ax_element, K_AX_VALUE_ATTRIBUTE) {
            element.attributes.insert("value".to_string(), value);
        }

        // Get enabled state
        if let Some(enabled) = self.get_attribute_bool(ax_element, K_AX_ENABLED_ATTRIBUTE) {
            element.attributes.insert("enabled".to_string(), enabled.to_string());
        }

        // Get focused state
        if let Some(focused) = self.get_attribute_bool(ax_element, K_AX_FOCUSED_ATTRIBUTE) {
            element.attributes.insert("focused".to_string(), focused.to_string());
        }

        // Get process info
        let mut pid: i32 = 0;
        let pid_result = unsafe { AXUIElementGetPid(ax_element, &mut pid) };
        if pid_result == K_AX_ERROR_SUCCESS && pid > 0 {
            element.attributes.insert("pid".to_string(), pid.to_string());
            // Try to get process name from PID
            if let Some(process_name) = self.get_process_name(pid) {
                element.process_name = Some(process_name);
            }
        }

        // Get window title
        if let Some(window_title) = self.get_window_title(ax_element) {
            element.window_title = Some(window_title);
        }

        Ok(element)
    }

    fn get_process_name(&self, pid: i32) -> Option<String> {
        use std::process::Command;
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .ok()?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
        None
    }

    fn get_window_title(&self, element: AXUIElementRef) -> Option<String> {
        // Try to get window from element
        let cf_window_attr = CFString::new(K_AX_WINDOW_ATTRIBUTE);
        let mut window_value: CFTypeRef = std::ptr::null_mut();

        let result = unsafe {
            AXUIElementCopyAttributeValue(element, cf_window_attr.as_concrete_TypeRef(), &mut window_value)
        };

        if result == K_AX_ERROR_SUCCESS && !window_value.is_null() {
            let title = self.get_attribute_string(window_value as AXUIElementRef, K_AX_TITLE_ATTRIBUTE);
            unsafe { CFRelease(window_value) };
            return title;
        }

        None
    }

    fn perform_click(&self, x: f64, y: f64, click_type: &ClickType) -> AutomationResult<()> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| AutomationError::ExecutionFailed("Failed to create event source".to_string()))?;

        let point = CGPoint::new(x, y);

        match click_type {
            ClickType::Single => {
                self.single_click(&source, point)?;
            }
            ClickType::Double => {
                self.double_click(&source, point)?;
            }
            ClickType::Right => {
                self.right_click(&source, point)?;
            }
        }

        Ok(())
    }

    fn single_click(&self, source: &CGEventSource, point: CGPoint) -> AutomationResult<()> {
        let mouse_down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            point,
            CGMouseButton::Left,
        ).map_err(|_| AutomationError::ExecutionFailed("Failed to create mouse down event".to_string()))?;

        let mouse_up = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseUp,
            point,
            CGMouseButton::Left,
        ).map_err(|_| AutomationError::ExecutionFailed("Failed to create mouse up event".to_string()))?;

        mouse_down.post(CGEventTapLocation::HID);
        std::thread::sleep(std::time::Duration::from_millis(10));
        mouse_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    fn double_click(&self, source: &CGEventSource, point: CGPoint) -> AutomationResult<()> {
        for i in 1..=2 {
            let mouse_down = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::LeftMouseDown,
                point,
                CGMouseButton::Left,
            ).map_err(|_| AutomationError::ExecutionFailed("Failed to create mouse down event".to_string()))?;

            let mouse_up = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::LeftMouseUp,
                point,
                CGMouseButton::Left,
            ).map_err(|_| AutomationError::ExecutionFailed("Failed to create mouse up event".to_string()))?;

            mouse_down.set_integer_value_field(core_graphics::event::EventField::MOUSE_EVENT_CLICK_STATE, i);
            mouse_up.set_integer_value_field(core_graphics::event::EventField::MOUSE_EVENT_CLICK_STATE, i);

            mouse_down.post(CGEventTapLocation::HID);
            std::thread::sleep(std::time::Duration::from_millis(10));
            mouse_up.post(CGEventTapLocation::HID);

            if i < 2 {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        Ok(())
    }

    fn right_click(&self, source: &CGEventSource, point: CGPoint) -> AutomationResult<()> {
        let mouse_down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::RightMouseDown,
            point,
            CGMouseButton::Right,
        ).map_err(|_| AutomationError::ExecutionFailed("Failed to create right mouse down event".to_string()))?;

        let mouse_up = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::RightMouseUp,
            point,
            CGMouseButton::Right,
        ).map_err(|_| AutomationError::ExecutionFailed("Failed to create right mouse up event".to_string()))?;

        mouse_down.post(CGEventTapLocation::HID);
        std::thread::sleep(std::time::Duration::from_millis(10));
        mouse_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    fn type_text(&self, text: &str) -> AutomationResult<()> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| AutomationError::ExecutionFailed("Failed to create event source".to_string()))?;

        for ch in text.chars() {
            self.type_character(&source, ch)?;
        }

        Ok(())
    }

    fn type_character(&self, source: &CGEventSource, ch: char) -> AutomationResult<()> {
        let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
            .map_err(|_| AutomationError::ExecutionFailed("Failed to create keyboard event".to_string()))?;

        // Set the Unicode character
        let chars = [ch as u16];
        event.set_string_from_utf16_unchecked(&chars);
        event.post(CGEventTapLocation::HID);

        let key_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
            .map_err(|_| AutomationError::ExecutionFailed("Failed to create key up event".to_string()))?;
        key_up.post(CGEventTapLocation::HID);

        std::thread::sleep(std::time::Duration::from_millis(5));

        Ok(())
    }

    fn focus_element(&self, element: AXUIElementRef) -> AutomationResult<()> {
        let cf_focused = CFString::new(K_AX_FOCUSED_ATTRIBUTE);
        let cf_true = CFBoolean::true_value();

        let result = unsafe {
            AXUIElementSetAttributeValue(
                element,
                cf_focused.as_concrete_TypeRef(),
                cf_true.to_void() as CFTypeRef,
            )
        };

        if result != K_AX_ERROR_SUCCESS {
            log::warn!("Failed to focus element, error: {}", result);
        }

        Ok(())
    }

    fn set_element_value(&self, element: AXUIElementRef, value: &str) -> AutomationResult<()> {
        let cf_value_attr = CFString::new(K_AX_VALUE_ATTRIBUTE);
        let cf_value = CFString::new(value);

        let result = unsafe {
            AXUIElementSetAttributeValue(
                element,
                cf_value_attr.as_concrete_TypeRef(),
                cf_value.to_void() as CFTypeRef,
            )
        };

        if result != K_AX_ERROR_SUCCESS {
            return Err(AutomationError::ExecutionFailed(
                format!("Failed to set element value, error: {}", result)
            ));
        }

        Ok(())
    }

    fn perform_action(&self, element: AXUIElementRef, action: &str) -> AutomationResult<()> {
        let cf_action = CFString::new(action);

        let result = unsafe {
            AXUIElementPerformAction(element, cf_action.as_concrete_TypeRef())
        };

        if result != K_AX_ERROR_SUCCESS {
            return Err(AutomationError::ExecutionFailed(
                format!("Failed to perform action '{}', error: {}", action, result)
            ));
        }

        Ok(())
    }
}

impl Drop for MacOSAutomation {
    fn drop(&mut self) {
        if !self.system_wide.is_null() {
            unsafe { CFRelease(self.system_wide as CFTypeRef) };
        }
    }
}

// Additional Accessibility framework binding for AXValue
type AXValueRef = *mut c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXValueGetValue(value: AXValueRef, theType: i32, valuePtr: *mut c_void) -> bool;
}

#[async_trait]
impl DesktopAutomation for MacOSAutomation {
    async fn click(&self, element: &UIElement, click_type: ClickType) -> AutomationResult<()> {
        self.check_accessibility()?;

        let (x, y) = element.bounds.center();
        log::info!("macOS: Click at ({}, {}) with type {:?}", x, y, click_type);

        self.perform_click(x as f64, y as f64, &click_type)?;

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
        self.check_accessibility()?;

        log::info!("macOS: Input '{}' to element {} with method {:?}", text, element.name, method);

        let (x, y) = element.bounds.center();

        match method {
            InputMethod::Type => {
                // Click to focus
                self.perform_click(x as f64, y as f64, &ClickType::Single)?;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Type the text
                self.type_text(text)?;
            }
            InputMethod::Set => {
                // Try to find and set value directly via Accessibility API
                let set_result = {
                    let ax_element = self.get_element_at_position(x as f32, y as f32)?;

                    // Focus the element
                    let _ = self.focus_element(ax_element);

                    // Try to set value directly
                    let result = self.set_element_value(ax_element, text);

                    unsafe { CFRelease(ax_element as CFTypeRef) };
                    result
                };

                // Wait after focus
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                // If set failed, fallback to typing
                if set_result.is_err() {
                    self.type_text(text)?;
                }
            }
        }

        Ok(())
    }

    async fn get_text(&self, element: &UIElement) -> AutomationResult<String> {
        self.check_accessibility()?;

        log::info!("macOS: Get text from element {}", element.name);

        let (x, y) = element.bounds.center();
        let ax_element = self.get_element_at_position(x as f32, y as f32)?;

        let text = self.get_attribute_string(ax_element, K_AX_VALUE_ATTRIBUTE)
            .or_else(|| self.get_attribute_string(ax_element, K_AX_TITLE_ATTRIBUTE))
            .or_else(|| self.get_attribute_string(ax_element, K_AX_DESCRIPTION_ATTRIBUTE))
            .unwrap_or_default();

        unsafe { CFRelease(ax_element as CFTypeRef) };

        Ok(text)
    }

    async fn get_attribute(&self, element: &UIElement, name: &str) -> AutomationResult<String> {
        self.check_accessibility()?;

        log::info!("macOS: Get attribute '{}' from element {}", name, element.name);

        let (x, y) = element.bounds.center();
        let ax_element = self.get_element_at_position(x as f32, y as f32)?;

        // Map common attribute names to AX attributes
        let ax_attr = match name.to_lowercase().as_str() {
            "value" => K_AX_VALUE_ATTRIBUTE,
            "title" | "name" => K_AX_TITLE_ATTRIBUTE,
            "description" => K_AX_DESCRIPTION_ATTRIBUTE,
            "role" | "controltype" => K_AX_ROLE_ATTRIBUTE,
            "enabled" => K_AX_ENABLED_ATTRIBUTE,
            "focused" => K_AX_FOCUSED_ATTRIBUTE,
            "identifier" | "automationid" => K_AX_IDENTIFIER_ATTRIBUTE,
            _ => name,
        };

        let value = self.get_attribute_string(ax_element, ax_attr)
            .unwrap_or_default();

        unsafe { CFRelease(ax_element as CFTypeRef) };

        Ok(value)
    }

    async fn wait_element(&self, locator: &str, timeout_ms: u64) -> AutomationResult<UIElement> {
        self.check_accessibility()?;

        log::info!("macOS: Wait for element '{}' with timeout {}ms", locator, timeout_ms);

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let poll_interval = std::time::Duration::from_millis(100);

        // Parse locator - support formats like "role:AXButton,title:OK"
        let criteria: HashMap<&str, &str> = locator
            .split(',')
            .filter_map(|part| {
                let mut parts = part.trim().splitn(2, ':');
                Some((parts.next()?, parts.next()?))
            })
            .collect();

        while start.elapsed() < timeout {
            // Search for element matching criteria
            if let Some(element) = self.find_element_by_criteria(&criteria).await {
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
        self.check_accessibility()?;

        log::info!("macOS: Capture element at ({}, {})", x, y);

        let ax_element = self.get_element_at_position(x as f32, y as f32)?;
        let ui_element = self.element_to_ui_element(ax_element)?;

        unsafe { CFRelease(ax_element as CFTypeRef) };

        Ok(ui_element)
    }

    async fn get_element_bounds(&self, element: &UIElement) -> AutomationResult<Rect> {
        Ok(element.bounds.clone())
    }

    async fn screenshot(&self, rect: Option<Rect>) -> AutomationResult<Vec<u8>> {
        log::info!("macOS: Take screenshot with rect {:?}", rect);

        let display = CGDisplay::main();

        let image = match rect {
            Some(r) => {
                let cg_rect = core_graphics::geometry::CGRect::new(
                    &core_graphics::geometry::CGPoint::new(r.x as f64, r.y as f64),
                    &core_graphics::geometry::CGSize::new(r.width as f64, r.height as f64),
                );
                display.image_for_rect(cg_rect)
            }
            None => display.image(),
        };

        let image = image.ok_or_else(|| {
            AutomationError::ExecutionFailed("Failed to capture screenshot".to_string())
        })?;

        // Convert CGImage to PNG data
        let png_data = self.cgimage_to_png(&image)?;

        Ok(png_data)
    }
}

impl MacOSAutomation {
    async fn find_element_by_criteria(&self, criteria: &HashMap<&str, &str>) -> Option<UIElement> {
        // Get all running applications and search for matching element
        // This is a simplified search - in production, you'd want a more sophisticated traversal

        // For now, we'll scan common screen positions
        let screen_bounds = CGDisplay::main().bounds();
        let step = 50.0;

        let mut y = 0.0;
        while y < screen_bounds.size.height {
            let mut x = 0.0;
            while x < screen_bounds.size.width {
                if let Ok(ax_element) = self.get_element_at_position(x as f32, y as f32) {
                    if self.element_matches_criteria(ax_element, criteria) {
                        if let Ok(ui_element) = self.element_to_ui_element(ax_element) {
                            unsafe { CFRelease(ax_element as CFTypeRef) };
                            return Some(ui_element);
                        }
                    }
                    unsafe { CFRelease(ax_element as CFTypeRef) };
                }
                x += step;
            }
            y += step;
        }

        None
    }

    fn element_matches_criteria(&self, element: AXUIElementRef, criteria: &HashMap<&str, &str>) -> bool {
        for (key, expected_value) in criteria {
            let ax_attr = match key.to_lowercase().as_str() {
                "role" => K_AX_ROLE_ATTRIBUTE,
                "title" | "name" => K_AX_TITLE_ATTRIBUTE,
                "description" => K_AX_DESCRIPTION_ATTRIBUTE,
                "value" => K_AX_VALUE_ATTRIBUTE,
                "identifier" | "automationid" => K_AX_IDENTIFIER_ATTRIBUTE,
                _ => *key,
            };

            if let Some(actual_value) = self.get_attribute_string(element, ax_attr) {
                if !actual_value.to_lowercase().contains(&expected_value.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn cgimage_to_png(&self, _image: &core_graphics::image::CGImage) -> AutomationResult<Vec<u8>> {
        // Use ImageIO framework to convert CGImage to PNG
        // This is a simplified implementation - for production, use proper ImageIO bindings

        use std::process::Command;
        use std::fs;

        // Take screenshot using screencapture command as fallback
        let temp_path = "/tmp/batata_rpa_screenshot.png";
        let output = Command::new("screencapture")
            .args(["-x", "-t", "png", temp_path])
            .output()
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to capture screenshot: {}", e)))?;

        if !output.status.success() {
            return Err(AutomationError::ExecutionFailed("screencapture command failed".to_string()));
        }

        let data = fs::read(temp_path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read screenshot: {}", e)))?;

        let _ = fs::remove_file(temp_path);

        Ok(data)
    }
}
