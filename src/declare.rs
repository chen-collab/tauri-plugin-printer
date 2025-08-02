use serde::{Deserialize, Serialize};

pub struct PrintOptions {
    // pub content_type: String,  // "pdf" 或 "html"
    // pub html: String,  // HTML 内容
    pub id: String,
    pub path: String,
    pub print_setting: String,
    pub remove_after_print: bool
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PrintHtmlOptions {
    pub html: String,
    pub printer_id: Option<String>,
    pub print_settings: Option<String>,
    pub remove_after_print: Option<bool>,
    pub page_size: Option<String>,  // A4, Letter 等
    pub orientation: Option<String>,  // portrait, landscape
    pub margin: Option<PrintMargin>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintMargin {
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
    pub unit: Option<String>,  // mm, cm, inch
}