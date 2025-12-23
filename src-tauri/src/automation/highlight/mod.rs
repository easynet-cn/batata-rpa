//! Element highlight overlay module
//! Provides platform-specific overlay windows to highlight UI elements on screen

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::automation::Rect;

/// Duration to show the highlight overlay (milliseconds)
const DEFAULT_HIGHLIGHT_DURATION_MS: u64 = 2000;

/// Highlight configuration
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub color: (u8, u8, u8),  // RGB
    pub border_width: f64,
    pub duration_ms: u64,
    pub opacity: f64,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            color: (255, 0, 0),  // Red
            border_width: 3.0,
            duration_ms: DEFAULT_HIGHLIGHT_DURATION_MS,
            opacity: 0.8,
        }
    }
}

/// Show highlight overlay around the given bounds
pub async fn highlight_element(bounds: Rect, config: Option<HighlightConfig>) -> Result<(), String> {
    let config = config.unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        macos::show_highlight(bounds, config).await
    }

    #[cfg(target_os = "windows")]
    {
        windows::show_highlight(bounds, config).await
    }

    #[cfg(target_os = "linux")]
    {
        linux::show_highlight(bounds, config).await
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        log::warn!("Highlight overlay not supported on this platform");
        Ok(())
    }
}
