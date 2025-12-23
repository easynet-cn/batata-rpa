#![cfg(target_os = "linux")]

use crate::automation::Rect;
use super::HighlightConfig;
use std::process::Command;

/// Show highlight overlay on Linux using notify-send or zenity
pub async fn show_highlight(bounds: Rect, config: HighlightConfig) -> Result<(), String> {
    log::info!(
        "Linux: Highlighting element at ({}, {}) size {}x{} for {}ms",
        bounds.x, bounds.y, bounds.width, bounds.height, config.duration_ms
    );

    // Try to use notify-send for visual feedback
    let message = format!(
        "元素位置: ({}, {})  大小: {}x{}",
        bounds.x, bounds.y, bounds.width, bounds.height
    );

    // Try notify-send first
    let result = Command::new("notify-send")
        .args([
            "-t", &config.duration_ms.to_string(),
            "元素高亮",
            &message,
        ])
        .output();

    if result.is_err() || !result.as_ref().unwrap().status.success() {
        // Fallback to zenity
        let _ = Command::new("zenity")
            .args([
                "--notification",
                "--text", &message,
            ])
            .output();
    }

    // Wait for the duration
    tokio::time::sleep(std::time::Duration::from_millis(config.duration_ms)).await;

    Ok(())
}

// Note: For a true overlay window on Linux, we would need to:
// 1. Create an X11 or Wayland overlay window
// 2. Set it as always-on-top and transparent
// 3. Draw a colored rectangle border
// This requires significant platform-specific code (GTK, Qt, or raw X11/Wayland)
