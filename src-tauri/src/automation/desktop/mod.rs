#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::automation::{AutomationResult, ClickType, InputMethod, Rect};
use crate::element::UIElement;
use async_trait::async_trait;

#[async_trait]
pub trait DesktopAutomation: Send + Sync {
    async fn click(&self, element: &UIElement, click_type: ClickType) -> AutomationResult<()>;

    async fn input(&self, element: &UIElement, text: &str, method: InputMethod)
        -> AutomationResult<()>;

    async fn get_text(&self, element: &UIElement) -> AutomationResult<String>;

    async fn get_attribute(&self, element: &UIElement, name: &str) -> AutomationResult<String>;

    async fn wait_element(
        &self,
        locator: &str,
        timeout_ms: u64,
    ) -> AutomationResult<UIElement>;

    async fn capture_element(&self, x: i32, y: i32) -> AutomationResult<UIElement>;

    async fn get_element_bounds(&self, element: &UIElement) -> AutomationResult<Rect>;

    async fn screenshot(&self, rect: Option<Rect>) -> AutomationResult<Vec<u8>>;
}

pub fn create_automation() -> Box<dyn DesktopAutomation> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsAutomation::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSAutomation::new())
    }

    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxAutomation::new())
    }
}
