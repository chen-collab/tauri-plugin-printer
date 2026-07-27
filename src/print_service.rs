//! 打印服务层：编排隐藏 WebView 窗口的全生命周期。
//!
//! 流程总览：
//! 1. 创建隐藏 WebviewWindow（visible=false, skip_taskbar）
//! 2. 导航到 file:// URL（HTML 临时文件或 PDF 文件）
//! 3. 等待渲染就绪（HTML 用 NavigationCompleted 事件，PDF 用固定延时）
//! 4. 调用 ICoreWebView2_16::Print 静默打印
//! 5. 清理窗口与临时文件

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Runtime, WebviewUrl, WebviewWindowBuilder};
use url::Url;
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment6;
use webview2_com::NavigationCompletedEventHandler;
use windows::core::{Interface, HSTRING};

use crate::declare::{PrintHtmlOptions, PrintOptions};
use crate::webview2_print::{build_print_settings, invoke_print, PrintOutcome, PrintSettingsData};
use crate::{fsys, Error};

/// 打印 HTML 内容（对外入口）
pub async fn print_html<R: Runtime>(
    app: &AppHandle<R>,
    options: PrintHtmlOptions,
) -> Result<String, Error> {
    if options.html.trim().is_empty() {
        return Err(Error::InvalidInput("HTML 内容不能为空".to_string()));
    }

    // 1. 构建带 @page CSS 的完整 HTML
    let page_css = build_page_css(&options);
    let html_with_css = inject_page_css(&options.html, &page_css);

    // 2. 写临时 HTML 文件
    let temp_path = generate_temp_file_path("html")?;
    std::fs::write(&temp_path, &html_with_css).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("写入 HTML 失败: {}", e),
        ))
    })?;

    let url = file_url_from_path(&temp_path);
    let settings_data = PrintSettingsData::from(&options);
    let remove_after = options.remove_after_print.unwrap_or(true);

    // 3. 走统一打印流程
    let result = print_via_webview(
        app,
        &url,
        settings_data,
        ReadyStrategy::NavigationCompleted(NAV_TIMEOUT_MS),
    )
    .await;

    // 4. 清理临时文件
    if remove_after {
        let _ = std::fs::remove_file(&temp_path);
    }

    result.map(|o| format!("{}: {}", o.status, o.message))
}

/// 打印 PDF 文件（对外入口）
pub async fn print_pdf<R: Runtime>(
    app: &AppHandle<R>,
    options: PrintOptions,
) -> Result<String, Error> {
    if options.path.trim().is_empty() {
        return Err(Error::InvalidInput("PDF 路径不能为空".to_string()));
    }

    let pdf_path = PathBuf::from(&options.path);
    if !pdf_path.exists() {
        return Err(Error::InvalidInput(format!(
            "PDF 文件不存在: {}",
            options.path
        )));
    }

    let url = file_url_from_path(&pdf_path);
    let settings_data = PrintSettingsData::from(&options);
    let remove_after = options.remove_after_print;

    // PDF 加载就绪用固定延时（PDFium viewer 的 NavigationCompleted 行为不稳定）
    let result = print_via_webview(
        app,
        &url,
        settings_data,
        ReadyStrategy::FixedDelay(PDF_READY_DELAY_MS),
    )
    .await;

    if remove_after {
        let _ = std::fs::remove_file(&pdf_path);
    }

    result.map(|o| format!("{}: {}", o.status, o.message))
}

// ===== 常量 =====

/// 导航超时（毫秒）
const NAV_TIMEOUT_MS: u64 = 10_000;
/// PDF 渲染就绪等待（毫秒）
const PDF_READY_DELAY_MS: u64 = 1500;
/// 打印操作超时（毫秒）
const PRINT_TIMEOUT_MS: u64 = 30_000;
/// 隐藏窗口标签前缀
const WINDOW_LABEL_PREFIX: &str = "__print_silent_";

// ===== 就绪策略 =====

pub(crate) enum ReadyStrategy {
    /// 等待 NavigationCompleted 事件（带超时，毫秒）
    NavigationCompleted(u64),
    /// 固定延时等待（毫秒）
    FixedDelay(u64),
}

// ===== 核心统一流程 =====

async fn print_via_webview<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    settings_data: PrintSettingsData,
    ready_strategy: ReadyStrategy,
) -> Result<PrintOutcome, Error> {
    let label = unique_window_label();

    // 1. 创建隐藏窗口（初始加载 about:blank）
    let webview = create_hidden_window(app, &label)?;

    // 2. 等渲染就绪
    wait_for_ready(&webview, url, ready_strategy).await?;

    // 3. 触发打印并等结果
    let outcome = do_print(&webview, settings_data).await?;

    // 4. 清理窗口
    let _ = webview.close();

    Ok(outcome)
}

// ===== 创建隐藏窗口 =====

pub(crate) fn create_hidden_window<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
) -> Result<tauri::WebviewWindow<R>, Error> {
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("about:blank".into()))
        .visible(false)
        .skip_taskbar(true)
        .inner_size(1.0, 1.0)
        .title("")
        .decorations(false)
        .focused(false)
        .build()
        .map_err(|e| Error::WebView2(format!("创建隐藏 WebView 窗口失败: {}", e)))?;
    Ok(window)
}

pub(crate) fn unique_window_label() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}{}_{}", WINDOW_LABEL_PREFIX, std::process::id(), ts)
}

// ===== 就绪等待 =====

async fn wait_for_ready<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    url: &str,
    strategy: ReadyStrategy,
) -> Result<(), Error> {
    match strategy {
        ReadyStrategy::NavigationCompleted(timeout_ms) => {
            wait_for_navigation_completed(webview, url, timeout_ms).await
        }
        ReadyStrategy::FixedDelay(delay_ms) => {
            // 先导航
            let parsed_url =
                Url::parse(url).map_err(|e| Error::InvalidInput(format!("URL 解析失败: {}", e)))?;
            webview
                .navigate(parsed_url)
                .map_err(|e| Error::WebView2(format!("导航失败: {}", e)))?;
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            Ok(())
        }
    }
}

async fn wait_for_navigation_completed<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    url: &str,
    timeout_ms: u64,
) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel::<Result<(), Error>>();
    let url = url.to_string();
    let webview_clone = webview.clone();

    // 用 with_webview 注册 NavigationCompleted handler 并触发导航
    // with_webview 闭包在主线程同步执行，但只做注册+触发，微秒级
    webview_clone
        .with_webview(move |pw| {
            // 闭包返回 ()，所有错误通过 tx 回传
            // SAFETY: 闭包在主线程（WebView STA 线程）执行，COM 对象操作安全
            unsafe {
                let core = match pw.controller().CoreWebView2() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(Error::WebView2(format!(
                            "获取 CoreWebView2 失败: {}",
                            e
                        ))));
                        return;
                    }
                };

                let tx_clone = tx.clone();
                let handler = NavigationCompletedEventHandler::create(Box::new(
                    move |_sender, _args| -> windows::core::Result<()> {
                        let _ = tx_clone.send(Ok(()));
                        Ok(())
                    },
                ));

                let mut token = 0i64;
                if let Err(e) = core.add_NavigationCompleted(&handler, &mut token) {
                    let _ = tx.send(Err(Error::WebView2(format!(
                        "注册 NavigationCompleted 失败: {}",
                        e
                    ))));
                    return;
                }

                // 触发导航
                if let Err(e) = core.Navigate(&HSTRING::from(&url)) {
                    let _ = tx.send(Err(Error::WebView2(format!("导航失败: {}", e))));
                    return;
                }

                // token 不主动 remove——窗口销毁时会自动释放
                let _ = token;
            }
        })
        .map_err(|e| Error::WebView2(format!("with_webview 调用失败: {}", e)))?;

    // 异步等待导航完成（带超时）
    let result = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_millis(timeout_ms))
            .map_err(|e| match e {
                mpsc::RecvTimeoutError::Timeout => {
                    Error::PrintTimeout(format!("导航超时 ({}ms)", timeout_ms))
                }
                mpsc::RecvTimeoutError::Disconnected => {
                    Error::WebView2("NavigationCompleted channel 已断开".to_string())
                }
            })
    })
    .await
    .map_err(|e| Error::WebView2(format!("等待导航结果失败: {}", e)))??;

    result
}

// ===== 触发打印 =====

pub(crate) async fn do_print<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    settings_data: PrintSettingsData,
) -> Result<PrintOutcome, Error> {
    let (tx, rx) = mpsc::channel::<Result<PrintOutcome, Error>>();
    let webview_clone = webview.clone();

    // with_webview 闭包在主线程执行：构建 settings + 调 Print + 注册完成回调
    // 闭包立即返回（不阻塞），结果通过 channel 异步回传
    webview_clone
        .with_webview(move |pw| {
            // 闭包返回 ()，同步阶段错误通过 tx 回传
            // SAFETY: 闭包在主线程（WebView STA 线程）执行
            unsafe {
                let controller = pw.controller();
                let env = pw.environment();

                let core = match controller.CoreWebView2() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(Error::WebView2(format!(
                            "获取 CoreWebView2 失败: {}",
                            e
                        ))));
                        return;
                    }
                };

                let core_16: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_16 =
                    match core.cast() {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx.send(Err(Error::WebView2(format!(
                                "WebView2 Runtime 版本过低（需 >= 1.0.1518.46），cast ICoreWebView2_16 失败: {}",
                                e
                            ))));
                            return;
                        }
                    };

                let env6: ICoreWebView2Environment6 = match env.cast() {
                    Ok(e) => e,
                    Err(e) => {
                        let _ = tx.send(Err(Error::WebView2(format!(
                            "WebView2 Environment6 不可用: {}",
                            e
                        ))));
                        return;
                    }
                };

                let settings = match build_print_settings(&env6, &settings_data) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        return;
                    }
                };

                if let Err(e) = invoke_print(&core_16, &settings, &tx) {
                    // invoke_print 失败（如 Print 调用本身失败），通过 tx 回传
                    let _ = tx.send(Err(e));
                }
            }
        })
        .map_err(|e| Error::WebView2(format!("with_webview 调用失败: {}", e)))?;

    // 异步等待打印结果（带超时）
    // std::sync::mpsc::Receiver 没有 async recv，用 spawn_blocking 包装
    let outcome = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_millis(PRINT_TIMEOUT_MS))
            .map_err(|e| match e {
                mpsc::RecvTimeoutError::Timeout => {
                    Error::PrintTimeout(format!("打印超时 ({}ms)", PRINT_TIMEOUT_MS))
                }
                mpsc::RecvTimeoutError::Disconnected => {
                    Error::WebView2("Print channel 已断开".to_string())
                }
            })
    })
    .await
    .map_err(|e| Error::WebView2(format!("等待打印结果失败: {}", e)))??;

    outcome
}

// ===== 工具函数 =====

/// 生成唯一临时文件路径
fn generate_temp_file_path(extension: &str) -> Result<PathBuf, Error> {
    let temp_dir = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::InvalidInput(format!("获取时间戳失败: {}", e)))?
        .as_nanos();
    let filename = format!(
        "tauri_print_webview2_{}_{}.{}",
        std::process::id(),
        timestamp,
        extension
    );
    Ok(temp_dir.join(filename))
}

/// 将本地文件路径转为 file:// URL
fn file_url_from_path(path: &std::path::Path) -> String {
    let display = path.display().to_string().replace('\\', "/");
    format!("file:///{}", display)
}

/// 从打印选项构建 @page CSS 规则
fn build_page_css(options: &PrintHtmlOptions) -> String {
    let size = if let (Some(w), Some(h)) = (options.page_width, options.page_height) {
        format!("{}mm {}mm", w, h)
    } else if let Some(ref page_size) = options.page_size {
        page_size.clone()
    } else {
        "A4".to_string()
    };

    let orientation = if let Some(ref o) = options.orientation {
        if o.eq_ignore_ascii_case("landscape") {
            " landscape"
        } else {
            ""
        }
    } else {
        ""
    };

    let margin = if let Some(ref m) = options.margin {
        let unit = m.unit.as_deref().unwrap_or("mm");
        let top = m.top.map_or("0".to_string(), |v| format!("{}{}", v, unit));
        let right = m
            .right
            .map_or("0".to_string(), |v| format!("{}{}", v, unit));
        let bottom = m
            .bottom
            .map_or("0".to_string(), |v| format!("{}{}", v, unit));
        let left = m.left.map_or("0".to_string(), |v| format!("{}{}", v, unit));
        format!("{} {} {} {}", top, right, bottom, left)
    } else {
        "0".to_string()
    };

    format!(
        "@page {{ size: {}{}; margin: {}; }}",
        size, orientation, margin
    )
}

/// 将 @page CSS 注入 HTML 头部
fn inject_page_css(html: &str, css: &str) -> String {
    let style_tag = format!("<style>{}</style>", css);
    if let Some(head_end) = html.find("</head>") {
        let mut result = String::with_capacity(html.len() + style_tag.len());
        result.push_str(&html[..head_end]);
        result.push_str(&style_tag);
        result.push_str(&html[head_end..]);
        result
    } else if let Some(head_start) = html.find("<head>") {
        let insert_pos = head_start + "<head>".len();
        let mut result = String::with_capacity(html.len() + style_tag.len());
        result.push_str(&html[..insert_pos]);
        result.push_str(&style_tag);
        result.push_str(&html[insert_pos..]);
        result
    } else {
        format!("{}{}", style_tag, html)
    }
}

#[allow(dead_code)]
fn _sanitize_check(filename: &str) -> Result<String, Error> {
    fsys::sanitize_filename(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declare::PrintMargin;

    /// 构造测试用 PrintHtmlOptions
    fn make_options(
        page_width: Option<f64>,
        page_height: Option<f64>,
        orientation: Option<&str>,
        page_size: Option<&str>,
    ) -> PrintHtmlOptions {
        PrintHtmlOptions {
            html: String::new(),
            printer_id: None,
            print_settings: None,
            remove_after_print: None,
            page_size: page_size.map(|s| s.to_string()),
            page_width,
            page_height,
            orientation: orientation.map(|s| s.to_string()),
            margin: Some(PrintMargin {
                top: Some(0.0),
                right: Some(0.0),
                bottom: Some(0.0),
                left: Some(0.0),
                unit: Some("mm".to_string()),
            }),
            quality: None,
            grayscale: None,
            copies: None,
        }
    }

    /// 80x80mm 自定义尺寸（输液标签场景）应生成 size: 80mm 80mm
    #[test]
    fn test_build_page_css_custom_80x80() {
        let css = build_page_css(&make_options(Some(80.0), Some(80.0), None, None));
        assert!(
            css.contains("size: 80mm 80mm"),
            "自定义尺寸应出现在 @page size 中，实际: {}",
            css
        );
        assert!(
            css.contains("margin: 0mm 0mm 0mm 0mm"),
            "零边距应正确渲染，实际: {}",
            css
        );
    }

    /// 58x40mm 腕带标签也应支持
    #[test]
    fn test_build_page_css_custom_58x40() {
        let css = build_page_css(&make_options(Some(58.0), Some(40.0), None, None));
        assert!(css.contains("size: 58mm 40mm"), "实际: {}", css);
    }

    /// 无尺寸时回退 A4
    #[test]
    fn test_build_page_css_default_a4() {
        let css = build_page_css(&make_options(None, None, None, None));
        assert!(css.contains("size: A4"), "实际: {}", css);
    }

    /// 显式 page_size（如 A5）也应生效
    #[test]
    fn test_build_page_css_named_size() {
        let css = build_page_css(&make_options(None, None, None, Some("A5")));
        assert!(css.contains("size: A5"), "实际: {}", css);
    }

    /// landscape 方向应追加到 size 之后
    #[test]
    fn test_build_page_css_landscape() {
        let css = build_page_css(&make_options(None, None, Some("Landscape"), None));
        assert!(css.contains("size: A4 landscape"), "实际: {}", css);
    }

    /// @page CSS 应注入到 </head> 之前
    #[test]
    fn test_inject_page_css_before_head_end() {
        let html = "<html><head><title>x</title></head><body></body></html>";
        let css = "@page { size: 80mm 80mm; }";
        let result = inject_page_css(html, css);
        let head_end_pos = result.find("</head>").unwrap();
        let style_pos = result.find("@page").unwrap();
        assert!(
            style_pos < head_end_pos,
            "@page CSS 应在 </head> 之前，实际位置 style={} head_end={}",
            style_pos,
            head_end_pos
        );
    }

    /// 无 <head> 时应前置注入
    #[test]
    fn test_inject_page_css_no_head() {
        let html = "<html><body></body></html>";
        let css = "@page { size: 80mm 80mm; }";
        let result = inject_page_css(html, css);
        assert!(
            result.starts_with("<style>"),
            "无 head 时应前置 <style>，实际: {}",
            &result[..result.len().min(20)]
        );
    }
}
