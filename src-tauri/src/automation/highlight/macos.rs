#![cfg(target_os = "macos")]

use crate::automation::Rect;
use super::HighlightConfig;
use std::process::Command;

/// Show highlight overlay on macOS
/// Uses osascript to display a temporary notification/overlay effect
pub async fn show_highlight(bounds: Rect, config: HighlightConfig) -> Result<(), String> {
    log::info!(
        "Highlighting element at ({}, {}) size {}x{} for {}ms",
        bounds.x, bounds.y, bounds.width, bounds.height, config.duration_ms
    );

    // For now, we use a simpler approach: flash the screen region using screencapture
    // and display it temporarily as a visual feedback
    // A full overlay window implementation would require more complex AppKit integration

    // Use Automator/AppleScript to show a visual notification
    let script = format!(
        r#"display notification "元素位置: ({}, {}) 大小: {}x{}" with title "元素高亮""#,
        bounds.x, bounds.y, bounds.width, bounds.height
    );

    let _ = Command::new("osascript")
        .args(["-e", &script])
        .output();

    // For actual visual overlay, we could also use a Python script with PyObjC
    // or create a standalone overlay app, but for now this provides feedback

    // Wait for the duration
    tokio::time::sleep(std::time::Duration::from_millis(config.duration_ms)).await;

    Ok(())
}

// Note: A full overlay window implementation using AppKit would require
// more complex thread-safe bindings. The notification approach above
// provides immediate feedback while a proper solution is developed.
