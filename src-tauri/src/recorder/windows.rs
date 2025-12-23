#![cfg(target_os = "windows")]

use super::{RecordedAction, RecordedActionType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx,
    GetMessageW, MSG, HHOOK,
    WH_MOUSE_LL, WH_KEYBOARD_LL,
    KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_LBUTTONDBLCLK, WM_MOUSEWHEEL, WM_KEYDOWN, WM_KEYUP,
};

static MOUSE_HOOK: std::sync::OnceLock<HHOOK> = std::sync::OnceLock::new();
static KEYBOARD_HOOK: std::sync::OnceLock<HHOOK> = std::sync::OnceLock::new();
static EVENT_CALLBACK: std::sync::OnceLock<Box<dyn Fn(RecordedAction) + Send + Sync>> = std::sync::OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(false);

pub struct EventHook {
    _marker: std::marker::PhantomData<()>,
}

impl EventHook {
    pub fn new<F>(callback: F) -> Result<Self, String>
    where
        F: Fn(RecordedAction) + Send + Sync + 'static,
    {
        // Store the callback
        EVENT_CALLBACK.set(Box::new(callback))
            .map_err(|_| "Callback already set".to_string())?;

        // Install hooks in a separate thread
        std::thread::spawn(|| {
            unsafe {
                // Install mouse hook
                let mouse_hook = SetWindowsHookExW(
                    WH_MOUSE_LL,
                    Some(mouse_proc),
                    None,
                    0,
                );

                if let Ok(hook) = mouse_hook {
                    let _ = MOUSE_HOOK.set(hook);
                }

                // Install keyboard hook
                let keyboard_hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_proc),
                    None,
                    0,
                );

                if let Ok(hook) = keyboard_hook {
                    let _ = KEYBOARD_HOOK.set(hook);
                }

                ENABLED.store(true, Ordering::SeqCst);

                // Message loop
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    // Process messages
                }
            }
        });

        Ok(Self {
            _marker: std::marker::PhantomData,
        })
    }

    pub fn enable(&self) {
        ENABLED.store(true, Ordering::SeqCst);
    }

    pub fn disable(&self) {
        ENABLED.store(false, Ordering::SeqCst);
    }
}

impl Drop for EventHook {
    fn drop(&mut self) {
        ENABLED.store(false, Ordering::SeqCst);

        unsafe {
            if let Some(hook) = MOUSE_HOOK.get() {
                let _ = UnhookWindowsHookEx(*hook);
            }
            if let Some(hook) = KEYBOARD_HOOK.get() {
                let _ = UnhookWindowsHookEx(*hook);
            }
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && ENABLED.load(Ordering::SeqCst) {
        if let Some(callback) = EVENT_CALLBACK.get() {
            let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            let x = info.pt.x;
            let y = info.pt.y;

            let action = match wparam.0 as u32 {
                WM_LBUTTONDOWN => {
                    Some(RecordedAction::new(RecordedActionType::Click)
                        .with_position(x, y))
                }
                WM_LBUTTONDBLCLK => {
                    Some(RecordedAction::new(RecordedActionType::DoubleClick)
                        .with_position(x, y))
                }
                WM_RBUTTONDOWN => {
                    Some(RecordedAction::new(RecordedActionType::RightClick)
                        .with_position(x, y))
                }
                WM_MOUSEWHEEL => {
                    let delta = (info.mouseData >> 16) as i16 as i32;
                    Some(RecordedAction::new(RecordedActionType::Scroll)
                        .with_position(x, y)
                        .with_data("deltaY", serde_json::json!(delta)))
                }
                _ => None,
            };

            if let Some(action) = action {
                callback(action);
            }
        }
    }

    CallNextHookEx(MOUSE_HOOK.get().copied(), code, wparam, lparam)
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && ENABLED.load(Ordering::SeqCst) {
        if let Some(callback) = EVENT_CALLBACK.get() {
            let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

            if wparam.0 as u32 == WM_KEYDOWN {
                let vk_code = info.vkCode;
                let scan_code = info.scanCode;

                // Convert to character if possible
                let text = vk_code_to_char(vk_code);

                if let Some(ch) = text {
                    let action = RecordedAction::new(RecordedActionType::Input)
                        .with_data("text", serde_json::json!(ch.to_string()))
                        .with_data("vkCode", serde_json::json!(vk_code))
                        .with_data("scanCode", serde_json::json!(scan_code));
                    callback(action);
                } else {
                    // It's a special key (hotkey)
                    let action = RecordedAction::new(RecordedActionType::Hotkey)
                        .with_data("vkCode", serde_json::json!(vk_code))
                        .with_data("scanCode", serde_json::json!(scan_code));
                    callback(action);
                }
            }
        }
    }

    CallNextHookEx(KEYBOARD_HOOK.get().copied(), code, wparam, lparam)
}

fn vk_code_to_char(vk_code: u32) -> Option<char> {
    // Simple mapping for printable characters
    match vk_code {
        0x30..=0x39 => Some((vk_code as u8) as char), // 0-9
        0x41..=0x5A => Some((vk_code as u8 + 32) as char), // A-Z -> a-z
        0x20 => Some(' '), // Space
        _ => None,
    }
}
