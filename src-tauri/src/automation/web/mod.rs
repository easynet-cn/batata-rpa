use crate::automation::{AutomationError, AutomationResult, Rect};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::Element;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebElement {
    pub id: String,
    pub tag_name: String,
    pub xpath: String,
    pub css_selector: String,
    pub text: Option<String>,
    pub attributes: HashMap<String, String>,
    pub bounds: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserOptions {
    pub headless: bool,
    pub browser_path: Option<String>,
    pub user_data_dir: Option<String>,
    pub window_size: Option<(u32, u32)>,
}

impl Default for BrowserOptions {
    fn default() -> Self {
        Self {
            headless: false,
            browser_path: None,
            user_data_dir: None,
            window_size: Some((1280, 800)),
        }
    }
}

pub struct BrowserSession {
    browser: Browser,
    page: chromiumoxide::Page,
}

pub struct WebAutomation {
    sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
}

impl WebAutomation {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn open_browser(
        &self,
        session_id: &str,
        options: BrowserOptions,
    ) -> AutomationResult<()> {
        log::info!("Opening browser with session: {}", session_id);

        let mut config_builder = BrowserConfig::builder();

        if options.headless {
            config_builder = config_builder.with_head();
        }

        if let Some((width, height)) = options.window_size {
            config_builder = config_builder.window_size(width, height);
        }

        if let Some(path) = options.browser_path {
            config_builder = config_builder.chrome_executable(path);
        }

        let config = config_builder
            .build()
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to build browser config: {}", e)))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to launch browser: {}", e)))?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to create page: {}", e)))?;

        let session = BrowserSession { browser, page };
        self.sessions.write().await.insert(session_id.to_string(), session);

        log::info!("Browser opened successfully with session: {}", session_id);
        Ok(())
    }

    pub async fn navigate(&self, session_id: &str, url: &str) -> AutomationResult<()> {
        log::info!("Navigating to: {}", url);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        session
            .page
            .goto(url)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to navigate: {}", e)))?;

        // Wait for page to load
        session
            .page
            .wait_for_navigation()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Navigation timeout: {}", e)))?;

        log::info!("Navigation completed");
        Ok(())
    }

    pub async fn click(&self, session_id: &str, selector: &str) -> AutomationResult<()> {
        log::info!("Clicking element: {}", selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        element
            .click()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to click: {}", e)))?;

        log::info!("Click completed");
        Ok(())
    }

    pub async fn input(&self, session_id: &str, selector: &str, text: &str) -> AutomationResult<()> {
        log::info!("Inputting text to element: {}", selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        // Clear existing content
        element
            .click()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to focus: {}", e)))?;

        // Type the text
        element
            .type_str(text)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to type: {}", e)))?;

        log::info!("Input completed");
        Ok(())
    }

    pub async fn get_text(&self, session_id: &str, selector: &str) -> AutomationResult<String> {
        log::info!("Getting text from element: {}", selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        let text = element
            .inner_text()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to get text: {}", e)))?
            .unwrap_or_default();

        log::info!("Got text: {}", text);
        Ok(text)
    }

    pub async fn get_attribute(
        &self,
        session_id: &str,
        selector: &str,
        attribute: &str,
    ) -> AutomationResult<Option<String>> {
        log::info!("Getting attribute '{}' from element: {}", attribute, selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        let value = element
            .attribute(attribute)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to get attribute: {}", e)))?;

        Ok(value)
    }

    pub async fn execute_js(&self, session_id: &str, script: &str) -> AutomationResult<String> {
        log::info!("Executing JavaScript");

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let result = session
            .page
            .evaluate(script)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to execute JS: {}", e)))?;

        let json_value = result.value().cloned().unwrap_or(serde_json::Value::Null);
        Ok(json_value.to_string())
    }

    pub async fn wait_element(
        &self,
        session_id: &str,
        selector: &str,
        timeout_ms: u64,
    ) -> AutomationResult<WebElement> {
        log::info!("Waiting for element '{}' with timeout {}ms", selector, timeout_ms);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            match self.find_element(&session.page, selector).await {
                Ok(element) => {
                    // Element found, get its properties using JavaScript
                    let text = element.inner_text().await.ok().flatten();

                    // Get tag name via JavaScript
                    let tag_name_result = session.page.evaluate(format!(
                        "document.querySelector('{}')?.tagName?.toLowerCase() || 'unknown'",
                        selector.replace('\'', "\\'")
                    )).await;

                    let tag_name = tag_name_result
                        .ok()
                        .and_then(|r| r.value().cloned())
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "unknown".to_string());

                    return Ok(WebElement {
                        id: uuid::Uuid::new_v4().to_string(),
                        tag_name,
                        xpath: String::new(),
                        css_selector: selector.to_string(),
                        text,
                        attributes: HashMap::new(),
                        bounds: Rect {
                            x: 0,
                            y: 0,
                            width: 0,
                            height: 0,
                        },
                    });
                }
                Err(_) => {
                    if start.elapsed() >= timeout {
                        return Err(AutomationError::Timeout(format!(
                            "Element '{}' not found within {}ms",
                            selector, timeout_ms
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    pub async fn screenshot(&self, session_id: &str) -> AutomationResult<Vec<u8>> {
        log::info!("Taking screenshot");

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .build();

        let screenshot = session
            .page
            .screenshot(params)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to take screenshot: {}", e)))?;

        log::info!("Screenshot taken: {} bytes", screenshot.len());
        Ok(screenshot)
    }

    pub async fn screenshot_element(
        &self,
        session_id: &str,
        selector: &str,
    ) -> AutomationResult<Vec<u8>> {
        log::info!("Taking element screenshot: {}", selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        let screenshot = element
            .screenshot(CaptureScreenshotFormat::Png)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to screenshot element: {}", e)))?;

        Ok(screenshot)
    }

    pub async fn close(&self, session_id: &str) -> AutomationResult<()> {
        log::info!("Closing browser session: {}", session_id);

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(session_id) {
            // Close the page first
            let _ = session.page.close().await;
            // Browser will be dropped
        }

        log::info!("Browser session closed");
        Ok(())
    }

    pub async fn close_all(&self) -> AutomationResult<()> {
        log::info!("Closing all browser sessions");

        let mut sessions = self.sessions.write().await;
        for (id, session) in sessions.drain() {
            log::info!("Closing session: {}", id);
            let _ = session.page.close().await;
        }

        Ok(())
    }

    pub async fn get_page_title(&self, session_id: &str) -> AutomationResult<String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let title = session
            .page
            .get_title()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to get title: {}", e)))?
            .unwrap_or_default();

        Ok(title)
    }

    pub async fn get_page_url(&self, session_id: &str) -> AutomationResult<String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let url = session.page.url().await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to get URL: {}", e)))?
            .map(|u| u.to_string())
            .unwrap_or_default();

        Ok(url)
    }

    pub async fn select_option(
        &self,
        session_id: &str,
        selector: &str,
        value: &str,
    ) -> AutomationResult<()> {
        log::info!("Selecting option '{}' in: {}", value, selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let script = format!(
            r#"
            const select = document.querySelector('{}');
            if (select) {{
                select.value = '{}';
                select.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }}
            "#,
            selector, value
        );

        session
            .page
            .evaluate(script)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to select option: {}", e)))?;

        Ok(())
    }

    pub async fn hover(&self, session_id: &str, selector: &str) -> AutomationResult<()> {
        log::info!("Hovering over element: {}", selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        element
            .scroll_into_view()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to scroll: {}", e)))?;

        // Use JavaScript to trigger hover
        let script = format!(
            r#"
            const el = document.querySelector('{}');
            if (el) {{
                el.dispatchEvent(new MouseEvent('mouseenter', {{ bubbles: true }}));
                el.dispatchEvent(new MouseEvent('mouseover', {{ bubbles: true }}));
            }}
            "#,
            selector
        );

        session
            .page
            .evaluate(script)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to hover: {}", e)))?;

        Ok(())
    }

    pub async fn scroll_to(&self, session_id: &str, x: i32, y: i32) -> AutomationResult<()> {
        log::info!("Scrolling to: ({}, {})", x, y);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let script = format!("window.scrollTo({}, {})", x, y);
        session
            .page
            .evaluate(script)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to scroll: {}", e)))?;

        Ok(())
    }

    pub async fn scroll_to_element(&self, session_id: &str, selector: &str) -> AutomationResult<()> {
        log::info!("Scrolling to element: {}", selector);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AutomationError::ExecutionFailed(format!("Session not found: {}", session_id)))?;

        let element = self.find_element(&session.page, selector).await?;

        element
            .scroll_into_view()
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to scroll to element: {}", e)))?;

        Ok(())
    }

    async fn find_element(&self, page: &chromiumoxide::Page, selector: &str) -> AutomationResult<Element> {
        page.find_element(selector)
            .await
            .map_err(|_| AutomationError::ElementNotFound(selector.to_string()))
    }
}

impl Default for WebAutomation {
    fn default() -> Self {
        Self::new()
    }
}
