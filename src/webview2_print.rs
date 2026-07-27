//! WebView2 COM 打印封装。
//!
//! 基于 webview2-com 0.38 提供的 `PrintCompletedHandler::create` 内置封装，
//! 通过 ICoreWebView2_16::Print 实现静默打印。
//!
//! 参考官方模式（callback.rs + sample.rs）：
//! - `PrintCompletedHandler::create(closure)` 自动生成 COM vtable，无需手写 #[implement]
//! - 闭包签名：`FnOnce(Result<()>, COREWEBVIEW2_PRINT_STATUS) -> Result<()>`
//! - 所有 COM 对象 `!Send`，必须在创建它的线程（主线程）上使用

use std::sync::mpsc;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Environment6, ICoreWebView2PrintSettings, ICoreWebView2PrintSettings2,
    ICoreWebView2_16, COREWEBVIEW2_PRINT_COLOR_MODE_COLOR, COREWEBVIEW2_PRINT_COLOR_MODE_GRAYSCALE,
    COREWEBVIEW2_PRINT_MEDIA_SIZE_CUSTOM, COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE,
    COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT, COREWEBVIEW2_PRINT_STATUS,
    COREWEBVIEW2_PRINT_STATUS_OTHER_ERROR, COREWEBVIEW2_PRINT_STATUS_PRINTER_UNAVAILABLE,
    COREWEBVIEW2_PRINT_STATUS_SUCCEEDED,
};
use webview2_com::PrintCompletedHandler;
use windows::core::{Interface, HSTRING};

use crate::declare::{PrintHtmlOptions, PrintMargin, PrintOptions};
use crate::Error;

/// 打印设置中间数据结构（在各线程间传递，必须 Send）
///
/// 单位：页面尺寸、边距均为毫米（mm），调用 COM 前统一转英寸。
#[derive(Debug, Clone)]
pub struct PrintSettingsData {
    pub printer_name: String,
    pub page_width_mm: Option<f64>,
    pub page_height_mm: Option<f64>,
    pub orientation: Option<String>, // "portrait" | "landscape"
    pub margin: Option<PrintMargin>,
    pub copies: Option<u32>,
    pub grayscale: Option<bool>,
    /// 是否打印背景色/图片（hiprint 模板需要）
    pub print_backgrounds: bool,
}

impl From<&PrintHtmlOptions> for PrintSettingsData {
    fn from(options: &PrintHtmlOptions) -> Self {
        Self {
            printer_name: options.printer_id.clone().unwrap_or_default(),
            page_width_mm: options.page_width,
            page_height_mm: options.page_height,
            orientation: options.orientation.clone(),
            margin: options.margin.clone(),
            copies: options.copies,
            grayscale: options.grayscale,
            print_backgrounds: true,
        }
    }
}

impl From<&PrintOptions> for PrintSettingsData {
    fn from(options: &PrintOptions) -> Self {
        Self {
            printer_name: options.printer_setting.clone(),
            page_width_mm: None, // PDF 用自带尺寸
            page_height_mm: None,
            orientation: options.orientation.clone(),
            margin: None,
            copies: options.copies,
            grayscale: options.grayscale,
            print_backgrounds: true,
        }
    }
}

/// 毫米转英寸（WebView2 Print API 全部使用英寸）
#[inline]
pub fn mm_to_inch(mm: f64) -> f64 {
    mm / 25.4
}

/// 构建打印设置对象（必须在主线程调用）。
///
/// 步骤：
/// 1. `env6.CreatePrintSettings()` 创建基础 settings
/// 2. cast 到 `ICoreWebView2PrintSettings2` 获得 PrinterName/Copies 等扩展字段
/// 3. 逐字段设置（PageWidth/Height、Margin、Orientation 等均为英寸）
pub unsafe fn build_print_settings(
    env6: &ICoreWebView2Environment6,
    data: &PrintSettingsData,
) -> Result<ICoreWebView2PrintSettings, Error> {
    let settings = env6
        .CreatePrintSettings()
        .map_err(|e| Error::WebView2(format!("创建 PrintSettings 失败: {}", e)))?;
    let s2: ICoreWebView2PrintSettings2 = settings
        .cast()
        .map_err(|e| Error::WebView2(format!("QueryInterface PrintSettings2 失败: {}", e)))?;

    // PrinterName（空字符串 = 使用系统默认打印机）
    if !data.printer_name.trim().is_empty() {
        s2.SetPrinterName(&HSTRING::from(data.printer_name.trim()))
            .map_err(|e| Error::WebView2(format!("SetPrinterName 失败: {}", e)))?;
    }

    // 自定义页面尺寸（英寸）：必须先把 MediaSize 设为 CUSTOM，PageWidth/PageHeight 才会生效
    // 否则 WebView2 会使用打印机驱动默认纸张（通常是 A4），自定义宽高被静默忽略
    if let (Some(w), Some(h)) = (data.page_width_mm, data.page_height_mm) {
        s2.SetMediaSize(COREWEBVIEW2_PRINT_MEDIA_SIZE_CUSTOM)
            .map_err(|e| Error::WebView2(format!("SetMediaSize 失败: {}", e)))?;
        settings
            .SetPageWidth(mm_to_inch(w))
            .map_err(|e| Error::WebView2(format!("SetPageWidth 失败: {}", e)))?;
        settings
            .SetPageHeight(mm_to_inch(h))
            .map_err(|e| Error::WebView2(format!("SetPageHeight 失败: {}", e)))?;
    }

    // 边距（英寸）
    if let Some(m) = &data.margin {
        let unit = m.unit.as_deref().unwrap_or("mm");
        let to_inch = |v: Option<f64>| -> f64 {
            match v {
                None => 0.0,
                Some(val) if unit.eq_ignore_ascii_case("inch") => val,
                Some(val) if unit.eq_ignore_ascii_case("cm") => val / 2.54,
                Some(val) => mm_to_inch(val), // 默认 mm
            }
        };
        settings
            .SetMarginTop(to_inch(m.top))
            .map_err(|e| Error::WebView2(format!("SetMarginTop 失败: {}", e)))?;
        settings
            .SetMarginBottom(to_inch(m.bottom))
            .map_err(|e| Error::WebView2(format!("SetMarginBottom 失败: {}", e)))?;
        settings
            .SetMarginLeft(to_inch(m.left))
            .map_err(|e| Error::WebView2(format!("SetMarginLeft 失败: {}", e)))?;
        settings
            .SetMarginRight(to_inch(m.right))
            .map_err(|e| Error::WebView2(format!("SetMarginRight 失败: {}", e)))?;
    }

    // 方向
    if let Some(o) = &data.orientation {
        let orientation = if o.eq_ignore_ascii_case("landscape") {
            COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE
        } else {
            COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT
        };
        settings
            .SetOrientation(orientation)
            .map_err(|e| Error::WebView2(format!("SetOrientation 失败: {}", e)))?;
    }

    // 份数
    if let Some(copies) = data.copies {
        s2.SetCopies(copies as i32)
            .map_err(|e| Error::WebView2(format!("SetCopies 失败: {}", e)))?;
    }

    // 黑白/彩色
    if let Some(grayscale) = data.grayscale {
        let mode = if grayscale {
            COREWEBVIEW2_PRINT_COLOR_MODE_GRAYSCALE
        } else {
            COREWEBVIEW2_PRINT_COLOR_MODE_COLOR
        };
        s2.SetColorMode(mode)
            .map_err(|e| Error::WebView2(format!("SetColorMode 失败: {}", e)))?;
    }

    // 背景色/图片（hiprint 模板有背景条码，必须开启）
    settings
        .SetShouldPrintBackgrounds(data.print_backgrounds)
        .map_err(|e| Error::WebView2(format!("SetShouldPrintBackgrounds 失败: {}", e)))?;

    Ok(settings)
}

/// 触发异步打印（非阻塞，立即返回）。
///
/// 调用 `ICoreWebView2_16::Print`，通过 `PrintCompletedHandler::create` 注册回调；
/// 回调在主线程触发时将结果通过 `mpsc::Sender` 发送出去。
///
/// 必须在主线程调用（`with_webview` 闭包内）。
///
/// # Safety
/// 调用方必须保证 `core_16` 和 `settings` 有效且在当前 STA 线程。
pub unsafe fn invoke_print(
    core_16: &ICoreWebView2_16,
    settings: &ICoreWebView2PrintSettings,
    tx: &mpsc::Sender<Result<PrintOutcome, Error>>,
) -> Result<(), Error> {
    let tx = tx.clone();
    let handler = PrintCompletedHandler::create(Box::new(
        move |error_code: windows::core::Result<()>,
              print_status: COREWEBVIEW2_PRINT_STATUS|
              -> windows::core::Result<()> {
            let outcome = map_print_result(error_code, print_status);
            let _ = tx.send(outcome);
            Ok(())
        },
    ));

    core_16
        .Print(settings, &handler)
        .map_err(|e| Error::WebView2(format!("调用 Print 失败: {}", e)))?;

    Ok(())
}

/// 打印结果（从 COM 回调映射后返回）
#[derive(Debug, Clone)]
pub struct PrintOutcome {
    /// 打印状态："succeeded" / "printer_unavailable" / "other_error"
    pub status: String,
    /// 状态描述
    pub message: String,
}

/// 将 COM 回调的 (error_code, print_status) 映射为业务结果
fn map_print_result(
    error_code: windows::core::Result<()>,
    print_status: COREWEBVIEW2_PRINT_STATUS,
) -> Result<PrintOutcome, Error> {
    match (error_code, print_status) {
        (Ok(()), COREWEBVIEW2_PRINT_STATUS_SUCCEEDED) => Ok(PrintOutcome {
            status: "succeeded".to_string(),
            message: "打印成功".to_string(),
        }),
        (Ok(()), COREWEBVIEW2_PRINT_STATUS_PRINTER_UNAVAILABLE) => Err(Error::WebView2(
            "指定打印机不可用、离线或处于错误状态".to_string(),
        )),
        (Ok(()), COREWEBVIEW2_PRINT_STATUS_OTHER_ERROR) => {
            Err(Error::WebView2("打印失败（OTHER_ERROR）".to_string()))
        }
        (Ok(()), other) => Err(Error::WebView2(format!("打印返回未知状态码: {}", other.0))),
        (Err(e), _) => {
            let msg = format!("Print 回调返回错误 (0x{:08X}): {}", e.code().0, e.message());
            Err(Error::WebView2(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mm_to_inch() {
        assert!((mm_to_inch(25.4) - 1.0).abs() < 1e-9);
        assert!((mm_to_inch(210.0) - 8.267_716_535).abs() < 1e-6); // A4 宽
        assert!((mm_to_inch(297.0) - 11.692_913_385).abs() < 1e-6); // A4 高
        assert!((mm_to_inch(80.0) - 3.149_606_299).abs() < 1e-6); // 80mm 热敏
    }

    /// 80x80mm 自定义尺寸应正确映射到 page_width_mm/page_height_mm
    #[test]
    fn test_settings_data_from_html_options_custom_size() {
        let options = PrintHtmlOptions {
            html: String::new(),
            printer_id: Some("标签打印机".to_string()),
            print_settings: None,
            remove_after_print: None,
            page_size: None,
            page_width: Some(80.0),
            page_height: Some(80.0),
            orientation: Some("portrait".to_string()),
            margin: Some(PrintMargin {
                top: Some(0.0),
                right: Some(0.0),
                bottom: Some(0.0),
                left: Some(0.0),
                unit: Some("mm".to_string()),
            }),
            quality: None,
            grayscale: Some(false),
            copies: Some(2),
        };
        let data = PrintSettingsData::from(&options);
        assert_eq!(data.printer_name, "标签打印机");
        assert_eq!(
            data.page_width_mm,
            Some(80.0),
            "page_width 必须映射到 page_width_mm"
        );
        assert_eq!(
            data.page_height_mm,
            Some(80.0),
            "page_height 必须映射到 page_height_mm"
        );
        assert_eq!(data.copies, Some(2));
        assert_eq!(data.grayscale, Some(false));
        assert!(data.print_backgrounds, "默认应开启背景打印");
    }

    /// 无自定义尺寸时 page_width_mm/page_height_mm 应为 None
    #[test]
    fn test_settings_data_from_html_options_no_custom_size() {
        let options = PrintHtmlOptions {
            html: String::new(),
            printer_id: None,
            print_settings: None,
            remove_after_print: None,
            page_size: Some("A4".to_string()),
            page_width: None,
            page_height: None,
            orientation: None,
            margin: None,
            quality: None,
            grayscale: None,
            copies: None,
        };
        let data = PrintSettingsData::from(&options);
        assert_eq!(data.page_width_mm, None);
        assert_eq!(data.page_height_mm, None);
        assert_eq!(data.printer_name, "", "无 printer_id 时应为空字符串");
    }
}
