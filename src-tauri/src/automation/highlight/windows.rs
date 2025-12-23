#![cfg(target_os = "windows")]

use crate::automation::Rect;
use super::HighlightConfig;

use windows::Win32::Foundation::{HWND, RECT, COLORREF, LPARAM, WPARAM, LRESULT};
use windows::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, SelectObject, DeleteObject,
    Rectangle, GetDC, ReleaseDC, InvalidateRect,
    PS_SOLID, HBRUSH, HPEN, HDC,
    GetStockObject, NULL_BRUSH,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, ShowWindow, UpdateWindow, DestroyWindow,
    SetWindowPos, GetWindowRect, DefWindowProcW,
    RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW,
    WS_POPUP, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOPMOST, WS_EX_TOOLWINDOW,
    SW_SHOWNOACTIVATE, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE,
    WM_PAINT, WM_DESTROY, WM_CREATE,
    SetLayeredWindowAttributes, LWA_COLORKEY, LWA_ALPHA,
    SetTimer, KillTimer, WM_TIMER, PostQuitMessage,
    GetMessageW, TranslateMessage, DispatchMessageW, MSG,
};
use windows::core::{PCWSTR, w};
use std::sync::atomic::{AtomicPtr, Ordering};

static HIGHLIGHT_WINDOW: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
static mut HIGHLIGHT_CONFIG: Option<HighlightConfig> = None;
static mut HIGHLIGHT_BOUNDS: Option<Rect> = None;

/// Show highlight overlay on Windows
pub async fn show_highlight(bounds: Rect, config: HighlightConfig) -> Result<(), String> {
    log::info!(
        "Windows: Highlighting element at ({}, {}) size {}x{} for {}ms",
        bounds.x, bounds.y, bounds.width, bounds.height, config.duration_ms
    );

    let duration_ms = config.duration_ms;

    // Store config for use in window procedure
    unsafe {
        HIGHLIGHT_CONFIG = Some(config.clone());
        HIGHLIGHT_BOUNDS = Some(bounds.clone());
    }

    // Create overlay window in a separate thread
    std::thread::spawn(move || {
        unsafe {
            create_highlight_window(bounds, config);
        }
    });

    // Wait for duration then close
    tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;

    // Close the window
    let hwnd = HIGHLIGHT_WINDOW.load(Ordering::SeqCst);
    if !hwnd.is_null() {
        unsafe {
            let _ = DestroyWindow(HWND(hwnd as isize));
        }
        HIGHLIGHT_WINDOW.store(std::ptr::null_mut(), Ordering::SeqCst);
    }

    Ok(())
}

unsafe fn create_highlight_window(bounds: Rect, config: HighlightConfig) {
    // Register window class
    let class_name = w!("BatataRPAHighlightOverlay");

    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(highlight_wnd_proc),
        hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
            .unwrap_or_default()
            .into(),
        lpszClassName: class_name,
        ..Default::default()
    };

    RegisterClassW(&wc);

    // Create window
    let padding = config.border_width as i32;
    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        class_name,
        w!(""),
        WS_POPUP,
        bounds.x - padding,
        bounds.y - padding,
        bounds.width + padding * 2,
        bounds.height + padding * 2,
        None,
        None,
        None,
        None,
    ).unwrap_or_default();

    if hwnd.0 == 0 {
        log::error!("Failed to create highlight window");
        return;
    }

    HIGHLIGHT_WINDOW.store(hwnd.0 as *mut std::ffi::c_void, Ordering::SeqCst);

    // Set transparency (transparent background with opaque border)
    let alpha = (config.opacity * 255.0) as u8;
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);

    // Show window
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    let _ = UpdateWindow(hwnd);

    // Set timer to close window after duration
    SetTimer(hwnd, 1, config.duration_ms as u32, None);

    // Message loop
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn highlight_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint_highlight(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            KillTimer(hwnd, 1);
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_highlight(hwnd: HWND) {
    let config = HIGHLIGHT_CONFIG.as_ref().unwrap_or(&HighlightConfig::default());
    let bounds = HIGHLIGHT_BOUNDS.as_ref();

    let hdc = GetDC(hwnd);
    if hdc.0 == 0 {
        return;
    }

    // Get window rect
    let mut rect = RECT::default();
    let _ = GetWindowRect(hwnd, &mut rect);

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    // Create pen for border
    let (r, g, b) = config.color;
    let color = COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));
    let pen = CreatePen(PS_SOLID, config.border_width as i32, color);

    // Use null brush for hollow rectangle
    let null_brush = GetStockObject(NULL_BRUSH);

    let old_pen = SelectObject(hdc, pen);
    let old_brush = SelectObject(hdc, null_brush);

    // Draw rectangle
    Rectangle(hdc, 0, 0, width, height);

    // Restore and cleanup
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    let _ = DeleteObject(pen);

    ReleaseDC(hwnd, hdc);
}
