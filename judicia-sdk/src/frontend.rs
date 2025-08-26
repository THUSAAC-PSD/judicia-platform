//! Frontend integration utilities for Judicia plugins

use crate::{error::PluginResult, types::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Props structure for frontend components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendProps {
    pub data: serde_json::Value,
    pub user_context: Option<UserContext>,
    pub theme: Option<String>,
    pub locale: Option<String>,
}

/// User context information for frontend components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// Component type definitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComponentType {
    /// Simple display component
    Display,
    /// Interactive form component
    Form,
    /// Data table/list component
    List,
    /// Chart/graph component
    Chart,
    /// Modal dialog component
    Modal,
    /// Navigation component
    Navigation,
    /// Dashboard widget
    Widget,
    /// Custom component type
    Custom,
}

/// Frontend component trait
pub trait FrontendComponent {
    /// Render the component to HTML
    fn render(&self, props: &serde_json::Value) -> Result<String, JsValue>;
    
    /// Handle frontend events
    fn handle_event(&mut self, event: &str, data: &serde_json::Value) -> Result<(), JsValue>;
    
    /// Get component metadata
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
    
    /// Get required CSS stylesheets
    fn stylesheets(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Get required JavaScript modules
    fn scripts(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub component_type: ComponentType,
    pub supported_events: Vec<String>,
    pub required_props: Vec<String>,
    pub optional_props: Vec<String>,
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            description: "A Judicia frontend component".to_string(),
            component_type: ComponentType::Custom,
            supported_events: Vec::new(),
            required_props: Vec::new(),
            optional_props: Vec::new(),
        }
    }
}

/// HTML builder utility for creating component markup
pub struct HtmlBuilder {
    content: String,
}

impl HtmlBuilder {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }
    
    pub fn element(mut self, tag: &str, attributes: HashMap<String, String>, content: &str) -> Self {
        self.content.push('<');
        self.content.push_str(tag);
        
        for (key, value) in attributes {
            self.content.push(' ');
            self.content.push_str(&key);
            self.content.push_str("=\"");
            self.content.push_str(&html_escape(&value));
            self.content.push('"');
        }
        
        self.content.push('>');
        self.content.push_str(content);
        self.content.push_str("</");
        self.content.push_str(tag);
        self.content.push('>');
        
        self
    }
    
    pub fn div(self, class: Option<&str>, content: &str) -> Self {
        let mut attrs = HashMap::new();
        if let Some(class_name) = class {
            attrs.insert("class".to_string(), class_name.to_string());
        }
        self.element("div", attrs, content)
    }
    
    pub fn span(self, class: Option<&str>, content: &str) -> Self {
        let mut attrs = HashMap::new();
        if let Some(class_name) = class {
            attrs.insert("class".to_string(), class_name.to_string());
        }
        self.element("span", attrs, content)
    }
    
    pub fn button(self, id: Option<&str>, class: Option<&str>, onclick: Option<&str>, text: &str) -> Self {
        let mut attrs = HashMap::new();
        if let Some(button_id) = id {
            attrs.insert("id".to_string(), button_id.to_string());
        }
        if let Some(class_name) = class {
            attrs.insert("class".to_string(), class_name.to_string());
        }
        if let Some(click_handler) = onclick {
            attrs.insert("onclick".to_string(), click_handler.to_string());
        }
        attrs.insert("type".to_string(), "button".to_string());
        self.element("button", attrs, text)
    }
    
    pub fn input(self, input_type: &str, name: Option<&str>, value: Option<&str>, placeholder: Option<&str>) -> Self {
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), input_type.to_string());
        
        if let Some(input_name) = name {
            attrs.insert("name".to_string(), input_name.to_string());
        }
        if let Some(input_value) = value {
            attrs.insert("value".to_string(), input_value.to_string());
        }
        if let Some(input_placeholder) = placeholder {
            attrs.insert("placeholder".to_string(), input_placeholder.to_string());
        }
        
        self.element("input", attrs, "")
    }
    
    pub fn table(mut self, headers: Vec<&str>, rows: Vec<Vec<&str>>) -> Self {
        self.content.push_str("<table class='judicia-table'>");
        
        // Header
        self.content.push_str("<thead><tr>");
        for header in headers {
            self.content.push_str("<th>");
            self.content.push_str(&html_escape(header));
            self.content.push_str("</th>");
        }
        self.content.push_str("</tr></thead>");
        
        // Body
        self.content.push_str("<tbody>");
        for row in rows {
            self.content.push_str("<tr>");
            for cell in row {
                self.content.push_str("<td>");
                self.content.push_str(&html_escape(cell));
                self.content.push_str("</td>");
            }
            self.content.push_str("</tr>");
        }
        self.content.push_str("</tbody>");
        
        self.content.push_str("</table>");
        self
    }
    
    pub fn raw(mut self, html: &str) -> Self {
        self.content.push_str(html);
        self
    }
    
    pub fn build(self) -> String {
        self.content
    }
}

impl Default for HtmlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS builder utility for component styling
pub struct CssBuilder {
    rules: Vec<String>,
}

impl CssBuilder {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    pub fn rule(mut self, selector: &str, properties: HashMap<String, String>) -> Self {
        let mut rule = format!("{} {{", selector);
        
        for (property, value) in properties {
            rule.push_str(&format!(" {}: {};", property, value));
        }
        
        rule.push_str(" }");
        self.rules.push(rule);
        self
    }
    
    pub fn class(self, class_name: &str, properties: HashMap<String, String>) -> Self {
        self.rule(&format!(".{}", class_name), properties)
    }
    
    pub fn id(self, id_name: &str, properties: HashMap<String, String>) -> Self {
        self.rule(&format!("#{}", id_name), properties)
    }
    
    pub fn media_query(mut self, query: &str, rules: &str) -> Self {
        let media_rule = format!("@media {} {{ {} }}", query, rules);
        self.rules.push(media_rule);
        self
    }
    
    pub fn build(self) -> String {
        self.rules.join("\n")
    }
}

impl Default for CssBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript event handling utilities
pub struct EventHandler {
    handlers: HashMap<String, String>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    pub fn on(mut self, event: &str, handler_code: &str) -> Self {
        self.handlers.insert(event.to_string(), handler_code.to_string());
        self
    }
    
    pub fn onclick(self, handler_code: &str) -> Self {
        self.on("click", handler_code)
    }
    
    pub fn onchange(self, handler_code: &str) -> Self {
        self.on("change", handler_code)
    }
    
    pub fn onsubmit(self, handler_code: &str) -> Self {
        self.on("submit", handler_code)
    }
    
    pub fn generate_script(&self, element_id: &str) -> String {
        let mut script = format!("(function() {{");
        script.push_str(&format!("const element = document.getElementById('{}');", element_id));
        
        for (event, handler) in &self.handlers {
            script.push_str(&format!(
                "element.addEventListener('{}', function(event) {{ {} }});",
                event, handler
            ));
        }
        
        script.push_str("})();");
        script
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for frontend development
pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn json_to_props(json: &serde_json::Value) -> FrontendProps {
    FrontendProps {
        data: json.clone(),
        user_context: None,
        theme: None,
        locale: None,
    }
}

/// Common component implementations
pub mod components {
    use super::*;
    
    /// Simple text display component
    pub struct TextDisplay {
        pub text: String,
        pub class_name: Option<String>,
    }
    
    impl TextDisplay {
        pub fn new(text: &str) -> Self {
            Self {
                text: text.to_string(),
                class_name: None,
            }
        }
        
        pub fn with_class(mut self, class_name: &str) -> Self {
            self.class_name = Some(class_name.to_string());
            self
        }
    }
    
    impl FrontendComponent for TextDisplay {
        fn render(&self, _props: &serde_json::Value) -> Result<String, JsValue> {
            let html = HtmlBuilder::new()
                .div(self.class_name.as_deref(), &html_escape(&self.text))
                .build();
            Ok(html)
        }
        
        fn handle_event(&mut self, _event: &str, _data: &serde_json::Value) -> Result<(), JsValue> {
            // Text display doesn't handle events
            Ok(())
        }
        
        fn metadata(&self) -> ComponentMetadata {
            ComponentMetadata {
                name: "TextDisplay".to_string(),
                version: "1.0.0".to_string(),
                description: "Simple text display component".to_string(),
                component_type: ComponentType::Display,
                supported_events: vec![],
                required_props: vec!["text".to_string()],
                optional_props: vec!["className".to_string()],
            }
        }
    }
    
    /// Data table component
    pub struct DataTable {
        pub headers: Vec<String>,
        pub rows: Vec<Vec<String>>,
        pub class_name: Option<String>,
    }
    
    impl DataTable {
        pub fn new(headers: Vec<String>) -> Self {
            Self {
                headers,
                rows: Vec::new(),
                class_name: None,
            }
        }
        
        pub fn add_row(mut self, row: Vec<String>) -> Self {
            self.rows.push(row);
            self
        }
        
        pub fn with_class(mut self, class_name: &str) -> Self {
            self.class_name = Some(class_name.to_string());
            self
        }
    }
    
    impl FrontendComponent for DataTable {
        fn render(&self, _props: &serde_json::Value) -> Result<String, JsValue> {
            let headers: Vec<&str> = self.headers.iter().map(|s| s.as_str()).collect();
            let rows: Vec<Vec<&str>> = self.rows.iter()
                .map(|row| row.iter().map(|s| s.as_str()).collect())
                .collect();
            
            let mut html_builder = HtmlBuilder::new();
            if let Some(class) = &self.class_name {
                html_builder = html_builder.div(Some(class), "");
            }
            
            let html = html_builder.table(headers, rows).build();
            Ok(html)
        }
        
        fn handle_event(&mut self, event: &str, data: &serde_json::Value) -> Result<(), JsValue> {
            match event {
                "row_click" => {
                    // Handle row selection
                    if let Some(row_index) = data.get("rowIndex").and_then(|v| v.as_u64()) {
                        // Could emit an event or update internal state
                        let _ = row_index; // Suppress unused warning
                    }
                }
                _ => {}
            }
            Ok(())
        }
        
        fn metadata(&self) -> ComponentMetadata {
            ComponentMetadata {
                name: "DataTable".to_string(),
                version: "1.0.0".to_string(),
                description: "Data table component with row selection".to_string(),
                component_type: ComponentType::List,
                supported_events: vec!["row_click".to_string()],
                required_props: vec!["headers".to_string(), "rows".to_string()],
                optional_props: vec!["className".to_string()],
            }
        }
    }
    
    /// Form component
    pub struct SimpleForm {
        pub fields: Vec<FormField>,
        pub submit_text: String,
        pub action: String,
    }
    
    pub struct FormField {
        pub name: String,
        pub label: String,
        pub field_type: String,
        pub required: bool,
        pub placeholder: Option<String>,
    }
    
    impl SimpleForm {
        pub fn new(action: &str, submit_text: &str) -> Self {
            Self {
                fields: Vec::new(),
                submit_text: submit_text.to_string(),
                action: action.to_string(),
            }
        }
        
        pub fn add_field(mut self, field: FormField) -> Self {
            self.fields.push(field);
            self
        }
        
        pub fn text_field(self, name: &str, label: &str, required: bool) -> Self {
            self.add_field(FormField {
                name: name.to_string(),
                label: label.to_string(),
                field_type: "text".to_string(),
                required,
                placeholder: None,
            })
        }
        
        pub fn email_field(self, name: &str, label: &str, required: bool) -> Self {
            self.add_field(FormField {
                name: name.to_string(),
                label: label.to_string(),
                field_type: "email".to_string(),
                required,
                placeholder: None,
            })
        }
    }
    
    impl FrontendComponent for SimpleForm {
        fn render(&self, _props: &serde_json::Value) -> Result<String, JsValue> {
            let mut html = format!("<form action='{}' method='POST' class='judicia-form'>", self.action);
            
            for field in &self.fields {
                html.push_str(&format!(
                    "<div class='form-field'><label for='{}'>{}</label>",
                    field.name, field.label
                ));
                
                let required_attr = if field.required { " required" } else { "" };
                let placeholder_attr = field.placeholder.as_ref()
                    .map(|p| format!(" placeholder='{}'", html_escape(p)))
                    .unwrap_or_default();
                
                html.push_str(&format!(
                    "<input type='{}' name='{}' id='{}'{}{} />",
                    field.field_type, field.name, field.name, required_attr, placeholder_attr
                ));
                
                html.push_str("</div>");
            }
            
            html.push_str(&format!(
                "<button type='submit' class='submit-btn'>{}</button>",
                self.submit_text
            ));
            html.push_str("</form>");
            
            Ok(html)
        }
        
        fn handle_event(&mut self, event: &str, data: &serde_json::Value) -> Result<(), JsValue> {
            match event {
                "submit" => {
                    // Handle form submission
                    let _ = data; // Suppress unused warning
                }
                _ => {}
            }
            Ok(())
        }
        
        fn metadata(&self) -> ComponentMetadata {
            ComponentMetadata {
                name: "SimpleForm".to_string(),
                version: "1.0.0".to_string(),
                description: "Simple form component".to_string(),
                component_type: ComponentType::Form,
                supported_events: vec!["submit".to_string()],
                required_props: vec!["action".to_string(), "fields".to_string()],
                optional_props: vec!["submitText".to_string()],
            }
        }
    }
}