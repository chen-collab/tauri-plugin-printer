//! 模板打印引擎（三层架构：前端指挥官 -> Rust 调度中心 -> 隐藏 WebView 渲染引擎）
//!
//! 架构：
//! - 前端：只传 { 模板 JSON, 数据 JSON, 纸张参数, 打印机配置 }
//! - Rust：创建隐藏窗口 -> 加载 print-render.html -> ExecuteScript 触发渲染 -> 轮询结果 -> 写入页面 -> 打印 -> 销毁
//! - 引擎：纯静态 HTML，加载 hiprint 库，提供 renderAndCalculate() 全局方法（返回 Promise）
//!
//! 渲染完成通知机制：
//! ExecuteScript 不等待 Promise resolve（webview2-com 0.38 限制），
//! 因此用「触发渲染 + 轮询检查全局变量」模式。
//! 轮询 JS 返回结构化对象（含诊断信息），避免 JSON 双重转义解析问题。

use tauri::{AppHandle, Manager, Runtime};

use crate::declare::PrintTemplateOptions;
use crate::print_service::{
    create_hidden_window, do_print, unique_window_label,
};
use crate::webview2_print::PrintSettingsData;
use crate::Error;

/// 渲染完成结果（从 JS 引擎读取）
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    pub html: String,
    pub content_height_px: f64,
}

/// 轮询返回的结构化状态
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PollStatus {
    /// "ok" | "error" | "waiting"
    status: String,
    #[serde(default)]
    html: String,
    #[serde(default)]
    content_height_px: f64,
    #[serde(default)]
    message: String,
    /// 诊断信息（waiting 状态时返回）
    #[serde(default)]
    engine_ready: bool,
    #[serde(default)]
    has_render_fn: bool,
    #[serde(default)]
    has_jquery: bool,
    #[serde(default)]
    has_vue_plugin_obj: bool,
    #[serde(default)]
    vue_plugin_keys: String,
    #[serde(default)]
    has_hiprint: bool,
    #[serde(default)]
    has_js_barcode: bool,
    #[serde(default)]
    has_bwipjs: bool,
}

/// 渲染超时（毫秒）
const RENDER_TIMEOUT_MS: u64 = 30_000;
/// 轮询间隔（毫秒）
const POLL_INTERVAL_MS: u64 = 100;

/// 模板打印（原子操作：创建窗口 -> 加载引擎 -> 渲染 -> 写入页面 -> 打印 -> 销毁）
pub async fn print_template<R: Runtime>(
    app: &AppHandle<R>,
    options: PrintTemplateOptions,
) -> Result<String, Error> {
    // 1. 校验输入
    if options.template.trim().is_empty() {
        return Err(Error::InvalidInput("模板不能为空".to_string()));
    }
    if options.data.trim().is_empty() {
        return Err(Error::InvalidInput("打印数据不能为空".to_string()));
    }
    if options.paper_width <= 0.0 {
        return Err(Error::InvalidInput(format!(
            "纸张宽度必须大于 0: {}",
            options.paper_width
        )));
    }

    // 2. 解析引擎 HTML 路径 + 库目录
    let engine_url = resolve_engine_url(app)?;
    let engine_dir = engine_url
        .strip_prefix("file:///")
        .map(|p| std::path::PathBuf::from(p.replace('/', "\\")).parent().map(|p| p.to_path_buf()))
        .flatten()
        .ok_or_else(|| Error::RenderEngine("无法解析引擎目录".to_string()))?;

    // 3. 创建隐藏窗口
    let label = unique_window_label();
    let webview = create_hidden_window(app, &label)?;

    // 4. 导航到引擎页面
    wait_for_navigation(&webview, &engine_url, 10_000).await?;

    // 4.5 注入 JS 库（file:// 页面的 <script src> 被 WebView2 阻止，改用 ExecuteScript 注入）
    inject_js_libs(&webview, &engine_dir).await?;

    // 5. 触发渲染（ExecuteScript 执行 IIFE，不等 Promise）
    let trigger_js = build_trigger_js(&options);
    eval_script_async(&webview, &trigger_js, 5_000).await?;

    // 6. 轮询等待渲染完成
    let render_timeout = options.render_timeout_ms.unwrap_or(RENDER_TIMEOUT_MS);
    let render_result = poll_render_result(&webview, render_timeout).await?;

    // 7. 根据内容高度计算纸张高度
    let paper_height_mm = if options.paper_height.unwrap_or(0.0) > 0.0 {
        options.paper_height.unwrap()
    } else {
        // 引擎按内容自适应，最小保留 10mm，避免 WebView2 SetPageHeight 因 0 拒绝
        px_to_mm(render_result.content_height_px).max(10.0)
    };

    // 8. 将渲染好的 HTML 写入当前 WebView 页面
    write_html_to_page(&webview, &render_result.html).await?;

    // 9. 构建打印设置
    use crate::declare::PrintMargin;
    let orientation = options.orientation.clone().unwrap_or_else(|| {
        if options.paper_width > paper_height_mm {
            "landscape".to_string()
        } else {
            "portrait".to_string()
        }
    });
    let settings_data = PrintSettingsData {
        printer_name: options.printer_id.clone().unwrap_or_default(),
        page_width_mm: Some(options.paper_width),
        page_height_mm: Some(paper_height_mm),
        orientation: Some(orientation),
        margin: Some(PrintMargin {
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
            left: Some(0.0),
            unit: Some("mm".to_string()),
        }),
        copies: options.copies,
        grayscale: options.grayscale,
        print_backgrounds: true,
    };

    // 10. 触发打印
    let outcome = do_print(&webview, settings_data).await?;

    // 11. 清理窗口
    let _ = webview.close();

    Ok(format!("{}: {}", outcome.status, outcome.message))
}

// ===== 轮询等待渲染结果 =====

async fn poll_render_result<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    timeout_ms: u64,
) -> Result<RenderResult, Error> {
    use tokio::time::{sleep, Duration, Instant};

    let start = Instant::now();
    let mut last_status: Option<PollStatus> = None;

    loop {
        if start.elapsed() > Duration::from_millis(timeout_ms) {
            // 超时时返回最后一次轮询的诊断信息
            let diag = match &last_status {
                Some(s) => format!(
                    "引擎就绪:{} jQuery:{} vuePluginObj:{} keys:{} hiprint:{} JsBarcode:{} bwipjs:{} renderFn:{}",
                    s.engine_ready, s.has_jquery, s.has_vue_plugin_obj, s.vue_plugin_keys,
                    s.has_hiprint, s.has_js_barcode, s.has_bwipjs, s.has_render_fn
                ),
                None => "无轮询数据（ExecuteScript 可能全部失败）".to_string(),
            };
            return Err(Error::PrintTimeout(format!(
                "渲染超时 ({}ms)。诊断: {}",
                timeout_ms, diag
            )));
        }

        // 执行轮询 JS，返回结构化对象
        let raw = eval_script_async(webview, POLL_JS, 3_000).await?;
        let status = parse_poll_status(&raw)?;

        match status.status.as_str() {
            "ok" => {
                return Ok(RenderResult {
                    html: status.html,
                    content_height_px: status.content_height_px,
                });
            }
            "error" => {
                return Err(Error::RenderEngine(format!(
                    "渲染失败: {}",
                    if status.message.is_empty() {
                        "未知错误"
                    } else {
                        &status.message
                    }
                )));
            }
            _ => {
                // "waiting" -- 继续轮询
                last_status = Some(status);
            }
        }

        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
}

/// 轮询 JS：返回结构化对象（不返回字符串，避免双重 JSON 转义）
const POLL_JS: &str = r#"
(function() {
    if (window.__printError__) {
        return { status: 'error', message: window.__printError__ };
    }
    if (window.__printResult__) {
        return {
            status: 'ok',
            html: window.__printResult__.html,
            contentHeightPx: window.__printResult__.contentHeightPx
        };
    }
    // 诊断信息
    var vpo = window['vue-plugin-hiprint'];
    var hiprintObj = window.hiprint || (vpo && vpo.hiprint);
    return {
        status: 'waiting',
        engineReady: !!window.__printEngineReady,
        hasRenderFn: typeof window.renderAndCalculate === 'function',
        hasJQuery: !!(window.jQuery || window.$),
        hasVuePluginObj: !!vpo,
        vuePluginKeys: vpo ? Object.keys(vpo).join(',') : '',
        hasHiprint: !!hiprintObj,
        hasJsBarcode: !!window.JsBarcode,
        hasBwipjs: !!(window.bwipjs || window['bwip-js'])
    };
})()
"#;

/// 解析轮询返回的状态对象
fn parse_poll_status(raw: &str) -> Result<PollStatus, Error> {
    // ExecuteScript 返回值的 JSON 表示：
    // - JS 返回 {status:"waiting",...} -> raw = "{\"status\":\"waiting\",...}"
    // - JS 返回 null -> raw = "null"
    if raw.is_empty() || raw == "null" || raw == "undefined" {
        return Err(Error::RenderEngine(format!(
            "轮询返回空值（引擎页面可能未加载）: {}",
            raw
        )));
    }

    // 直接解析
    if let Ok(status) = serde_json::from_str::<PollStatus>(raw) {
        return Ok(status);
    }

    Err(Error::RenderEngine(format!(
        "轮询结果解析失败: {}",
        raw
    )))
}

// ===== ExecuteScript 执行 =====

/// 异步执行 JS 并返回字符串结果
async fn eval_script_async<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    js: &str,
    timeout_ms: u64,
) -> Result<String, Error> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<String, Error>>();
    let js = js.to_string();
    let webview_clone = webview.clone();

    use webview2_com::ExecuteScriptCompletedHandler;
    use windows::core::HSTRING;

    webview_clone
        .with_webview(move |pw| {
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
                let handler = ExecuteScriptCompletedHandler::create(Box::new(
                    move |_sender, result| -> windows::core::Result<()> {
                        let _ = tx_clone.send(Ok(result));
                        Ok(())
                    },
                ));

                if let Err(e) = core.ExecuteScript(&HSTRING::from(&js), &handler) {
                    let _ = tx.send(Err(Error::WebView2(format!(
                        "ExecuteScript 调用失败: {}",
                        e
                    ))));
                }
            }
        })
        .map_err(|e| Error::WebView2(format!("with_webview 调用失败: {}", e)))?;

    let result = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_millis(timeout_ms))
            .map_err(|e| match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => {
                    Error::PrintTimeout(format!("ExecuteScript 超时 ({}ms)", timeout_ms))
                }
                std::sync::mpsc::RecvTimeoutError::Disconnected => {
                    Error::WebView2("ExecuteScript channel 已断开".to_string())
                }
            })
    })
    .await
    .map_err(|e| Error::WebView2(format!("spawn_blocking 失败: {}", e)))??;

    result
}

// ===== 工具函数 =====

fn px_to_mm(px: f64) -> f64 {
    px * 25.4 / 96.0
}

/// 构建触发渲染的 JS 代码
///
/// IIFE 调用 renderAndCalculate()，结果存入 window.__printResult__，
/// 错误存入 window.__printError__。IIFE 本身返回 undefined（不返回 Promise）。
fn build_trigger_js(options: &PrintTemplateOptions) -> String {
    let template_json = escape_js_string(&options.template);
    let data_json = escape_js_string(&options.data);
    let paper_width = options.paper_width;
    let paper_height = options.paper_height.unwrap_or(0.0);

    format!(
        r#"
        (function() {{
            window.__printResult__ = null;
            window.__printError__ = null;

            if (typeof window.renderAndCalculate !== 'function') {{
                window.__printError__ = 'renderAndCalculate 函数不存在（hiprint 可能未加载）';
                return;
            }}

            window.renderAndCalculate({{
                templateJson: {template},
                dataJson: {data},
                paperWidthMm: {width},
                paperHeightMm: {height}
            }}).then(function(result) {{
                window.__printResult__ = result;
            }}).catch(function(err) {{
                window.__printError__ = (err && err.message) ? err.message : String(err);
            }});
        }})();
        "#,
        template = template_json,
        data = data_json,
        width = paper_width,
        height = paper_height
    )
}

fn escape_js_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// 将 HTML 内容写入当前 WebView 页面（替换整个文档）
async fn write_html_to_page<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    html: &str,
) -> Result<(), Error> {
    let html_json =
        serde_json::to_string(html).map_err(|e| Error::InvalidInput(format!("HTML 转义失败: {}", e)))?;

    let js_code = format!(
        "(function() {{ document.open(); document.write({}); document.close(); }})();",
        html_json
    );

    eval_script_async(webview, &js_code, 5_000).await?;

    // document.write 同步完成，但浏览器布局需要一点时间
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    Ok(())
}

// ===== 导航等待 =====

async fn wait_for_navigation<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    url: &str,
    timeout_ms: u64,
) -> Result<(), Error> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), Error>>();
    let url = url.to_string();
    let webview_clone = webview.clone();

    use webview2_com::NavigationCompletedEventHandler;
    use windows::core::HSTRING;

    webview_clone
        .with_webview(move |pw| {
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

                if let Err(e) = core.Navigate(&HSTRING::from(&url)) {
                    let _ = tx.send(Err(Error::WebView2(format!("导航失败: {}", e))));
                    return;
                }
                let _ = token;
            }
        })
        .map_err(|e| Error::WebView2(format!("with_webview 调用失败: {}", e)))?;

    let result = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_millis(timeout_ms))
            .map_err(|e| match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => {
                    Error::PrintTimeout(format!("导航超时 ({}ms)", timeout_ms))
                }
                std::sync::mpsc::RecvTimeoutError::Disconnected => {
                    Error::WebView2("NavigationCompleted channel 已断开".to_string())
                }
            })
    })
    .await
    .map_err(|e| Error::WebView2(format!("等待导航结果失败: {}", e)))??;

    result
}

/// 注入 JS 库文件（绕过 file:// 的 <script src> 限制）
///
/// 按顺序读取库文件内容，用 ExecuteScript 依次注入：
/// jquery -> socket.io -> jsbarcode -> bwip-js -> vue-plugin-hiprint
async fn inject_js_libs<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    engine_dir: &std::path::Path,
) -> Result<(), Error> {
    // 1. 注入 jquery
    inject_and_check(
        webview,
        engine_dir,
        "jquery.min.js",
        "window.jQuery || window.$",
        10_000,
    )
    .await?;

    // 2. 注入 socket.io（hiprint 内部需要 window.io）
    inject_and_check(
        webview,
        engine_dir,
        "socket.io.min.js",
        "typeof window.io === 'function'",
        10_000,
    )
    .await?;

    // 3. 给 vue-plugin-hiprint 的其他可选依赖设置占位符（防止内部 undefined.xxx 抛异常）
    let stubs = r#"
        window.JsBarcode = window.JsBarcode || {};
        window['bwip-js'] = window['bwip-js'] || window.bwipjs || {};
        window.jspdf = window.jspdf || {};
        window.html2canvas = window.html2canvas || {};
        window.canvg = window.canvg || {};
        "done"
    "#;
    eval_script_async(webview, stubs, 3_000).await?;

    // 4. 注入 jsbarcode（可选，收据场景可能不用，但 hiprint UMD 需要它存在）
    let _ = inject_and_check(
        webview,
        engine_dir,
        "jsbarcode.min.js",
        "window.JsBarcode",
        10_000,
    )
    .await;

    // 5. 注入 bwip-js（可选，收据场景可能不用）
    let _ = inject_and_check(
        webview,
        engine_dir,
        "bwip-js.js",
        "window.bwipjs || window['bwip-js']",
        10_000,
    )
    .await;

    // 6. 注入 vue-plugin-hiprint
    inject_and_check(
        webview,
        engine_dir,
        "vue-plugin-hiprint.js",
        "window['vue-plugin-hiprint'] && window['vue-plugin-hiprint'].hiprint",
        30_000,
    )
    .await?;

    // 7. 注入 hiprint 样式表（离屏量高 + 最终打印文档均依赖）。
    //    hiprint 的表格边框/内边距/表头背景、线框/矩形椭圆描边、SimSun 字体、
    //    word-break 等行为完全由 CSS 定义，不注入会导致渲染结果与预览不一致。
    //    本引擎以 file:// 加载且无 HTTP 服务，改用内联 <style> 注入（等价官方 styleHandler）。
    let css_path = engine_dir.join("print-lock.css");
    let css = std::fs::read_to_string(&css_path).map_err(|e| {
        Error::RenderEngine(format!(
            "读取样式文件失败: {} ({})",
            css_path.display(),
            e
        ))
    })?;
    let css_json = serde_json::to_string(&css).unwrap_or_else(|_| "\"\"".to_string());
    let inject_css = format!(
        "(function(){{ \
            var s=document.createElement('style'); \
            s.setAttribute('data-hiprint-css','1'); \
            s.textContent={css}; \
            document.head.appendChild(s); \
            window.__hiprintCss__={css}; \
            return 'ok'; \
        }})()",
        css = css_json
    );
    let raw = eval_script_async(webview, &inject_css, 5_000).await?;
    if raw.trim().trim_matches('"') != "ok" {
        return Err(Error::RenderEngine(format!(
            "注入 hiprint CSS 失败，返回值: {}",
            raw
        )));
    }

    Ok(())
}

/// 注入单个 JS 库并检查全局变量
async fn inject_and_check<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    engine_dir: &std::path::Path,
    lib_name: &str,
    check_expr: &str,
    timeout_ms: u64,
) -> Result<(), Error> {
    let lib_path = engine_dir.join(lib_name);
    let lib_content = std::fs::read_to_string(&lib_path).map_err(|e| {
        Error::RenderEngine(format!(
            "读取库文件失败: {} ({})",
            lib_path.display(),
            e
        ))
    })?;

    eval_script_async(webview, &lib_content, timeout_ms).await?;

    // 检查全局变量是否存在
    let check_js = format!("(function() {{ return !!({}); }})()", check_expr);
    let raw = eval_script_async(webview, &check_js, 3_000).await?;
    let exists = raw == "true";

    if !exists {
        return Err(Error::RenderEngine(format!(
            "注入 {} 后全局变量未找到（检查表达式: {}），返回值: {}",
            lib_name, check_expr, raw
        )));
    }

    Ok(())
}

/// 解析引擎 HTML 的 file:// URL
fn resolve_engine_url<R: Runtime>(app: &AppHandle<R>) -> Result<String, Error> {
    use tauri::path::BaseDirectory;

    let candidates: Vec<std::path::PathBuf> = vec![
        app.path()
            .resolve("print-engine/print-render.html", BaseDirectory::Resource)
            .unwrap_or_default(),
        std::env::current_dir()
            .unwrap_or_default()
            .join("resources/print-engine/print-render.html"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_default()
            .join("resources/print-engine/print-render.html"),
        std::env::current_dir()
            .unwrap_or_default()
            .join("src-tauri/resources/print-engine/print-render.html"),
    ];

    for path in &candidates {
        if path.exists() {
            let display = path.display().to_string().replace('\\', "/");
            return Ok(format!("file:///{}", display));
        }
    }

    let tried = candidates
        .iter()
        .map(|p| format!("  - {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Err(Error::RenderEngine(format!(
        "渲染引擎文件不存在，已尝试以下路径：\n{}",
        tried
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_px_to_mm() {
        assert!((px_to_mm(96.0) - 25.4).abs() < 0.01);
        assert!((px_to_mm(960.0) - 254.0).abs() < 0.01);
        assert!((px_to_mm(0.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_escape_js_string() {
        let s = escape_js_string(r#"hello "world""#);
        assert!(s.contains("\\\"world\\\""));
        assert!(s.starts_with('"'));
        assert!(s.ends_with('"'));
    }

    #[test]
    fn test_parse_poll_status_waiting() {
        let json = r#"{"status":"waiting","html":"","contentHeightPx":0,"message":"","engineReady":true,"hasRenderFn":true,"hasJQuery":true,"hasVuePluginObj":true,"vuePluginKeys":"hiprint,defaultElementTypeProvider","hasHiprint":true,"hasJsBarcode":true,"hasBwipjs":true}"#;
        let status = parse_poll_status(json).unwrap();
        assert_eq!(status.status, "waiting");
        assert!(status.engine_ready);
        assert!(status.has_render_fn);
    }

    #[test]
    fn test_parse_poll_status_ok() {
        let json = r#"{"status":"ok","html":"<div>test</div>","contentHeightPx":120.5,"message":"","engineReady":false,"hasRenderFn":false,"hasJQuery":false,"hasVuePluginObj":false,"vuePluginKeys":"","hasHiprint":false,"hasJsBarcode":false,"hasBwipjs":false}"#;
        let status = parse_poll_status(json).unwrap();
        assert_eq!(status.status, "ok");
        assert_eq!(status.html, "<div>test</div>");
        assert!((status.content_height_px - 120.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_poll_status_error() {
        let json = r#"{"status":"error","html":"","contentHeightPx":0,"message":"hiprint not loaded","engineReady":false,"hasRenderFn":false,"hasJQuery":false,"hasVuePluginObj":false,"vuePluginKeys":"","hasHiprint":false,"hasJsBarcode":false,"hasBwipjs":false}"#;
        let status = parse_poll_status(json).unwrap();
        assert_eq!(status.status, "error");
        assert_eq!(status.message, "hiprint not loaded");
    }

    #[test]
    fn test_parse_poll_status_null() {
        assert!(parse_poll_status("null").is_err());
        assert!(parse_poll_status("").is_err());
    }
}
