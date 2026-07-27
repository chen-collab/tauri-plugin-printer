use serde::{Deserialize, Serialize};

/// 打印机信息（camelCase JSON 输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterInfo {
    pub name: String,
    pub driver_name: String,
    pub job_count: u32,
    pub print_processor: String,
    pub port_name: String,
    pub share_name: String,
    pub computer_name: String,
    /// 打印机状态数组（可同时包含多个状态，如 ["paused","error"]）
    pub printer_status: Vec<String>,
    pub shared: bool,
    pub printer_type: u32,
    pub priority: u32,
}

/// 打印任务信息（camelCase JSON 输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub document_name: String,
    pub id: u32,
    pub total_pages: u32,
    pub position: u32,
    pub size: u32,
    pub submitted_time: String,
    pub user_name: String,
    pub pages_printed: u32,
    pub job_time: u32,
    pub computer_name: String,
    pub datatype: String,
    pub printer_name: String,
    pub priority: u32,
    /// 任务状态数组（可同时包含多个状态位）
    pub job_status: Vec<String>,
}

/// PDF 打印选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintOptions {
    pub id: String,
    pub path: String,
    /// 打印机名称（空字符串表示使用默认打印机）
    pub printer_setting: String,
    pub remove_after_print: bool,
    /// 方向：portrait, landscape（可选，默认由 PDF/驱动决定）
    pub orientation: Option<String>,
    /// 是否灰度打印（可选）
    pub grayscale: Option<bool>,
    /// 打印份数（可选，默认 1）
    pub copies: Option<u32>,
}

/// HTML 打印选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintHtmlOptions {
    pub html: String,
    pub printer_id: Option<String>,
    pub print_settings: Option<String>,
    pub remove_after_print: Option<bool>,
    /// 页面大小：A4, Letter 等（与 page_width/page_height 二选一）
    pub page_size: Option<String>,
    /// 自定义纸张宽度（mm），与 page_size 二选一，有自定义宽高时优先
    pub page_width: Option<f64>,
    /// 自定义纸张高度（mm）
    pub page_height: Option<f64>,
    /// 方向：portrait, landscape
    pub orientation: Option<String>,
    pub margin: Option<PrintMargin>,
    /// 质量：1-100
    pub quality: Option<u32>,
    pub grayscale: Option<bool>,
    pub copies: Option<u32>,
}

/// 打印边距设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintMargin {
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
    /// 单位：mm, cm, inch
    pub unit: Option<String>,
}
