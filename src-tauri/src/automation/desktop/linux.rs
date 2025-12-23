#![cfg(target_os = "linux")]

use crate::automation::{AutomationError, AutomationResult, ClickType, InputMethod, Rect};
use crate::element::UIElement;
use async_trait::async_trait;
use super::DesktopAutomation;

pub struct LinuxAutomation {
    // Linux AT-SPI instance
}

impl LinuxAutomation {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl DesktopAutomation for LinuxAutomation {
    async fn click(&self, element: &UIElement, click_type: ClickType) -> AutomationResult<()> {
        let (x, y) = element.bounds.center();
        log::info!("Linux: Click at ({}, {}) with type {:?}", x, y, click_type);
        Ok(())
    }

    async fn input(
        &self,
        element: &UIElement,
        text: &str,
        method: InputMethod,
    ) -> AutomationResult<()> {
        log::info!("Linux: Input '{}' to element {} with method {:?}", text, element.name, method);
        Ok(())
    }

    async fn get_text(&self, element: &UIElement) -> AutomationResult<String> {
        log::info!("Linux: Get text from element {}", element.name);
        Ok(String::new())
    }

    async fn get_attribute(&self, element: &UIElement, name: &str) -> AutomationResult<String> {
        log::info!("Linux: Get attribute '{}' from element {}", name, element.name);
        Ok(String::new())
    }

    async fn wait_element(&self, locator: &str, timeout_ms: u64) -> AutomationResult<UIElement> {
        log::info!("Linux: Wait for element '{}' with timeout {}ms", locator, timeout_ms);
        Err(AutomationError::ElementNotFound(locator.to_string()))
    }

    async fn capture_element(&self, x: i32, y: i32) -> AutomationResult<UIElement> {
        log::info!("Linux: Capture element at ({}, {})", x, y);
        Err(AutomationError::PlatformNotSupported("Linux element capture not implemented".to_string()))
    }

    async fn get_element_bounds(&self, element: &UIElement) -> AutomationResult<Rect> {
        Ok(element.bounds.clone())
    }

    async fn screenshot(&self, rect: Option<Rect>) -> AutomationResult<Vec<u8>> {
        log::info!("Linux: Take screenshot with rect {:?}", rect);
        Ok(Vec::new())
    }
}
