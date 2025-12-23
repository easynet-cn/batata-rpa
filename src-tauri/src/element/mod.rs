use crate::automation::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElement {
    pub id: String,
    pub name: String,
    pub control_type: String,
    pub automation_id: Option<String>,
    pub class_name: Option<String>,
    pub xpath: Option<String>,
    pub css_selector: Option<String>,
    pub bounds: Rect,
    pub screenshot: Option<Vec<u8>>,
    pub process_name: Option<String>,
    pub window_title: Option<String>,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: String,
}

impl UIElement {
    pub fn new(name: String, control_type: String, bounds: Rect) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            control_type,
            automation_id: None,
            class_name: None,
            xpath: None,
            css_selector: None,
            bounds,
            screenshot: None,
            process_name: None,
            window_title: None,
            parent_id: None,
            children: Vec::new(),
            attributes: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementLibrary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub elements: Vec<UIElement>,
    pub created_at: String,
    pub updated_at: String,
}

impl ElementLibrary {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            elements: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_element(&mut self, element: UIElement) {
        self.elements.push(element);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn remove_element(&mut self, id: &str) -> Option<UIElement> {
        if let Some(pos) = self.elements.iter().position(|e| e.id == id) {
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Some(self.elements.remove(pos))
        } else {
            None
        }
    }

    pub fn find_element(&self, id: &str) -> Option<&UIElement> {
        self.elements.iter().find(|e| e.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocatorStrategy {
    AutomationId(String),
    Name(String),
    ClassName(String),
    XPath(String),
    CssSelector(String),
    Image(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLocator {
    pub primary: LocatorStrategy,
    pub fallback: Option<Box<ElementLocator>>,
}

impl ElementLocator {
    pub fn new(strategy: LocatorStrategy) -> Self {
        Self {
            primary: strategy,
            fallback: None,
        }
    }

    pub fn with_fallback(mut self, fallback: ElementLocator) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }
}
