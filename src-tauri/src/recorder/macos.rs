#![cfg(target_os = "macos")]

use super::{RecordedAction, RecordedActionType};
use crate::element::UIElement;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use core_foundation::base::{CFRelease, CFTypeRef};
use core_graphics::event::{
    CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};

// Run loop FFI bindings
type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopMode = *const c_void;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFRunLoopCommonModes: CFRunLoopMode;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
}

// Event tap callback type
type CGEventTapCallBack = extern "C" fn(
    proxy: *mut c_void,
    event_type: CGEventType,
    event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> *mut c_void;

    fn CGEventTapEnable(tap: *mut c_void, enable: bool);
}

// Event masks
const K_CG_EVENT_LEFT_MOUSE_DOWN: u64 = 1 << 1;
const K_CG_EVENT_LEFT_MOUSE_UP: u64 = 1 << 2;
const K_CG_EVENT_RIGHT_MOUSE_DOWN: u64 = 1 << 3;
const K_CG_EVENT_RIGHT_MOUSE_UP: u64 = 1 << 4;
const K_CG_EVENT_MOUSE_MOVED: u64 = 1 << 5;
const K_CG_EVENT_LEFT_MOUSE_DRAGGED: u64 = 1 << 6;
const K_CG_EVENT_RIGHT_MOUSE_DRAGGED: u64 = 1 << 7;
const K_CG_EVENT_KEY_DOWN: u64 = 1 << 10;
const K_CG_EVENT_KEY_UP: u64 = 1 << 11;
const K_CG_EVENT_SCROLL_WHEEL: u64 = 1 << 22;

// All events we want to capture
const EVENT_MASK: u64 = K_CG_EVENT_LEFT_MOUSE_DOWN
    | K_CG_EVENT_LEFT_MOUSE_UP
    | K_CG_EVENT_RIGHT_MOUSE_DOWN
    | K_CG_EVENT_RIGHT_MOUSE_UP
    | K_CG_EVENT_KEY_DOWN
    | K_CG_EVENT_SCROLL_WHEEL;

// Callback data structure
struct CallbackData {
    callback: Box<dyn Fn(RecordedAction) + Send + Sync>,
    enabled: Arc<AtomicBool>,
    last_click_time: std::sync::atomic::AtomicU64,
    pending_text: std::sync::Mutex<String>,
}

pub struct EventTap {
    tap_port: *mut c_void,
    callback_data: Box<CallbackData>,
    run_loop_source: Option<*mut c_void>,
    enabled: Arc<AtomicBool>,
}

unsafe impl Send for EventTap {}
unsafe impl Sync for EventTap {}

impl EventTap {
    pub fn new<F>(callback: F) -> Result<Self, String>
    where
        F: Fn(RecordedAction) + Send + Sync + 'static,
    {
        let enabled = Arc::new(AtomicBool::new(true));

        let callback_data = Box::new(CallbackData {
            callback: Box::new(callback),
            enabled: enabled.clone(),
            last_click_time: std::sync::atomic::AtomicU64::new(0),
            pending_text: std::sync::Mutex::new(String::new()),
        });

        let callback_ptr = Box::into_raw(callback_data) as *mut c_void;

        let tap_port = unsafe {
            CGEventTapCreate(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                EVENT_MASK,
                event_tap_callback,
                callback_ptr,
            )
        };

        if tap_port.is_null() {
            unsafe {
                let _ = Box::from_raw(callback_ptr as *mut CallbackData);
            }
            return Err(
                "Failed to create event tap. Please grant Accessibility permission.".to_string(),
            );
        }

        // Create a run loop source
        let run_loop_source = unsafe {
            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap_port, 0);
            if !source.is_null() {
                CFRunLoopAddSource(
                    CFRunLoopGetCurrent(),
                    source as *mut c_void,
                    kCFRunLoopCommonModes,
                );
            }
            Some(source)
        };

        Ok(Self {
            tap_port,
            callback_data: unsafe { Box::from_raw(callback_ptr as *mut CallbackData) },
            run_loop_source,
            enabled,
        })
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
        unsafe {
            CGEventTapEnable(self.tap_port, true);
        }
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        unsafe {
            CGEventTapEnable(self.tap_port, false);
        }
    }
}

impl Drop for EventTap {
    fn drop(&mut self) {
        unsafe {
            CGEventTapEnable(self.tap_port, false);
            if let Some(source) = self.run_loop_source {
                CFRelease(source as CFTypeRef);
            }
            CFRelease(self.tap_port as CFTypeRef);
        }
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
}

// Additional CGEvent FFI bindings
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventGetLocation(event: *mut c_void) -> CGPoint;
    fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
    fn CGEventGetFlags(event: *mut c_void) -> u64;
}

// CGPoint struct (we use it raw here)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

// Event field constants
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_1: u32 = 11;
const K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_2: u32 = 12;

// Event flag constants
const K_CG_EVENT_FLAG_COMMAND: u64 = 0x00100000;
const K_CG_EVENT_FLAG_CONTROL: u64 = 0x00040000;
const K_CG_EVENT_FLAG_ALTERNATE: u64 = 0x00080000;
const K_CG_EVENT_FLAG_SHIFT: u64 = 0x00020000;

extern "C" fn event_tap_callback(
    _proxy: *mut c_void,
    event_type: CGEventType,
    event: *mut c_void,
    user_info: *mut c_void,
) -> *mut c_void {
    if user_info.is_null() || event.is_null() {
        return event;
    }

    let callback_data = unsafe { &*(user_info as *const CallbackData) };

    if !callback_data.enabled.load(Ordering::SeqCst) {
        return event;
    }

    // Process the event using raw FFI
    if let Some(recorded_action) = process_event_raw(event, event_type, callback_data) {
        (callback_data.callback)(recorded_action);
    }

    // Return the original event (don't consume it)
    event
}

fn process_event_raw(
    event: *mut c_void,
    event_type: CGEventType,
    callback_data: &CallbackData,
) -> Option<RecordedAction> {
    let location = unsafe { CGEventGetLocation(event) };
    let x = location.x as i32;
    let y = location.y as i32;

    match event_type {
        CGEventType::LeftMouseDown => {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let last_click = callback_data.last_click_time.load(Ordering::SeqCst);

            // Check for double-click (within 500ms)
            let is_double_click = current_time - last_click < 500;
            callback_data
                .last_click_time
                .store(current_time, Ordering::SeqCst);

            // Flush any pending text input
            flush_pending_text(callback_data);

            if is_double_click {
                // Replace last click with double-click (handled in frontend)
                let action = RecordedAction::new(RecordedActionType::DoubleClick)
                    .with_position(x, y);

                // Try to get element at position
                if let Some(element) = try_capture_element(x, y) {
                    return Some(action.with_element(element));
                }

                return Some(action);
            }

            let action = RecordedAction::new(RecordedActionType::Click)
                .with_position(x, y);

            // Try to get element at position
            if let Some(element) = try_capture_element(x, y) {
                return Some(action.with_element(element));
            }

            Some(action)
        }
        CGEventType::RightMouseDown => {
            flush_pending_text(callback_data);

            let action = RecordedAction::new(RecordedActionType::RightClick)
                .with_position(x, y);

            if let Some(element) = try_capture_element(x, y) {
                return Some(action.with_element(element));
            }

            Some(action)
        }
        CGEventType::KeyDown => {
            let keycode = unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) };
            let flags = unsafe { CGEventGetFlags(event) };

            // Check for modifier keys (Cmd, Ctrl, Alt)
            if (flags & K_CG_EVENT_FLAG_COMMAND) != 0
                || (flags & K_CG_EVENT_FLAG_CONTROL) != 0
                || (flags & K_CG_EVENT_FLAG_ALTERNATE) != 0
            {
                // This is a hotkey
                let modifiers = get_modifier_names_raw(flags);
                let key_name = keycode_to_name(keycode as u16);

                let action = RecordedAction::new(RecordedActionType::Hotkey)
                    .with_data("key", serde_json::Value::String(key_name))
                    .with_data(
                        "modifiers",
                        serde_json::Value::Array(
                            modifiers
                                .into_iter()
                                .map(serde_json::Value::String)
                                .collect(),
                        ),
                    );

                return Some(action);
            }

            // Regular text input - accumulate characters
            if let Some(chars) = get_key_characters_raw(keycode as u16, flags) {
                let mut pending = callback_data.pending_text.lock().unwrap();
                pending.push_str(&chars);
            }

            None
        }
        CGEventType::ScrollWheel => {
            flush_pending_text(callback_data);

            let delta_y = unsafe {
                CGEventGetIntegerValueField(event, K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_1)
            };
            let delta_x = unsafe {
                CGEventGetIntegerValueField(event, K_CG_SCROLL_WHEEL_EVENT_DELTA_AXIS_2)
            };

            let action = RecordedAction::new(RecordedActionType::Scroll)
                .with_position(x, y)
                .with_data("deltaX", serde_json::Value::Number(delta_x.into()))
                .with_data("deltaY", serde_json::Value::Number(delta_y.into()));

            Some(action)
        }
        _ => None,
    }
}

fn flush_pending_text(callback_data: &CallbackData) -> Option<RecordedAction> {
    let mut pending = callback_data.pending_text.lock().unwrap();
    if pending.is_empty() {
        return None;
    }

    let text = pending.clone();
    pending.clear();

    Some(
        RecordedAction::new(RecordedActionType::Input)
            .with_data("text", serde_json::Value::String(text)),
    )
}

fn get_modifier_names_raw(flags: u64) -> Vec<String> {
    let mut names = Vec::new();

    if (flags & K_CG_EVENT_FLAG_COMMAND) != 0 {
        names.push("Cmd".to_string());
    }
    if (flags & K_CG_EVENT_FLAG_CONTROL) != 0 {
        names.push("Ctrl".to_string());
    }
    if (flags & K_CG_EVENT_FLAG_ALTERNATE) != 0 {
        names.push("Alt".to_string());
    }
    if (flags & K_CG_EVENT_FLAG_SHIFT) != 0 {
        names.push("Shift".to_string());
    }

    names
}

fn keycode_to_name(keycode: u16) -> String {
    match keycode {
        0 => "A",
        1 => "S",
        2 => "D",
        3 => "F",
        4 => "H",
        5 => "G",
        6 => "Z",
        7 => "X",
        8 => "C",
        9 => "V",
        11 => "B",
        12 => "Q",
        13 => "W",
        14 => "E",
        15 => "R",
        16 => "Y",
        17 => "T",
        18 => "1",
        19 => "2",
        20 => "3",
        21 => "4",
        22 => "6",
        23 => "5",
        24 => "=",
        25 => "9",
        26 => "7",
        27 => "-",
        28 => "8",
        29 => "0",
        30 => "]",
        31 => "O",
        32 => "U",
        33 => "[",
        34 => "I",
        35 => "P",
        36 => "Return",
        37 => "L",
        38 => "J",
        39 => "'",
        40 => "K",
        41 => ";",
        42 => "\\",
        43 => ",",
        44 => "/",
        45 => "N",
        46 => "M",
        47 => ".",
        48 => "Tab",
        49 => "Space",
        50 => "`",
        51 => "Delete",
        53 => "Escape",
        55 => "Command",
        56 => "Shift",
        57 => "CapsLock",
        58 => "Option",
        59 => "Control",
        96 => "F5",
        97 => "F6",
        98 => "F7",
        99 => "F3",
        100 => "F8",
        101 => "F9",
        103 => "F11",
        109 => "F10",
        111 => "F12",
        115 => "Home",
        116 => "PageUp",
        117 => "ForwardDelete",
        118 => "F4",
        119 => "End",
        120 => "F2",
        121 => "PageDown",
        122 => "F1",
        123 => "Left",
        124 => "Right",
        125 => "Down",
        126 => "Up",
        _ => "Unknown",
    }
    .to_string()
}

fn get_key_characters_raw(keycode: u16, flags: u64) -> Option<String> {
    // Skip non-printable keys
    if keycode > 50 && keycode != 49 {
        // Space is 49
        return None;
    }

    // For simplicity, convert keycode to character
    // This is a simplified version - a full implementation would use Text Input Services
    let ch = match keycode {
        0 => 'a',
        1 => 's',
        2 => 'd',
        3 => 'f',
        4 => 'h',
        5 => 'g',
        6 => 'z',
        7 => 'x',
        8 => 'c',
        9 => 'v',
        11 => 'b',
        12 => 'q',
        13 => 'w',
        14 => 'e',
        15 => 'r',
        16 => 'y',
        17 => 't',
        18 => '1',
        19 => '2',
        20 => '3',
        21 => '4',
        22 => '6',
        23 => '5',
        24 => '=',
        25 => '9',
        26 => '7',
        27 => '-',
        28 => '8',
        29 => '0',
        30 => ']',
        31 => 'o',
        32 => 'u',
        33 => '[',
        34 => 'i',
        35 => 'p',
        37 => 'l',
        38 => 'j',
        39 => '\'',
        40 => 'k',
        41 => ';',
        42 => '\\',
        43 => ',',
        44 => '/',
        45 => 'n',
        46 => 'm',
        47 => '.',
        49 => ' ',
        50 => '`',
        _ => return None,
    };

    let ch = if (flags & K_CG_EVENT_FLAG_SHIFT) != 0 {
        ch.to_uppercase().next().unwrap_or(ch)
    } else {
        ch
    };

    Some(ch.to_string())
}

fn try_capture_element(x: i32, y: i32) -> Option<UIElement> {
    use crate::automation::desktop::macos::MacOSAutomation;
    use crate::automation::desktop::DesktopAutomation;

    // Create automation instance and try to capture element
    let automation = MacOSAutomation::new();

    // Use tokio runtime to run async code
    let rt = tokio::runtime::Handle::try_current().ok()?;
    rt.block_on(async { automation.capture_element(x, y).await.ok() })
}
