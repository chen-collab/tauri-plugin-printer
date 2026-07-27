//! 模板打印引擎（三层架构：前端指挥官 → Rust 调度中心 → 隐藏 WebView 渲染引擎）
//!
//! 架构：
//! - 前端：只传 { 模板 JSON, 数据 JSON, 纸张参数, 打印机配置 }
//! - Rust：创建隐藏窗口 → 加载 print-render.html → 注入模板数据 → 等待渲染完成 → 调 WebView2 Print → 销毁窗口
//! - 引擎：纯静态 HTML，加载 hiprint 库，提供 renderAndCalculate() 全局方法
//!
//! 渲染完成回调：
//! 引擎页面渲染完成后调用 print_render_done command（带渲染好的完整 HTML + 内容像素高度），
//! Rust 端通过全局注册表 HashMap<label, oneshot::Sender> 路由回对应打印任务。

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime, State};

use crate::declare::PrintTemplateOptions;
use crate::print_service::{
    create_hidden_window, do_print, unique_window_label,
};
use crate::webview2_print::PrintSettingsData;
use crate::Error;

/// 渲染完成结果（从 JS 引擎传回）
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    /// 渲染好的完整 HTML（含 body 内容，图片已转 base64）
    pub html: String,
    /// 内容像素高度（px）
    pub content_height_px: f64,
}

/// 渲染完成回调 Sender 注册表
/// key = 窗口 label，value = oneshot sender
pub struct RenderRegistry {
    inner: Mutex<HashMap<String, tokio::sync::oneshot::Sender<RenderResult>>>,
}

impl RenderRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, label: String, tx: tokio::sync::oneshot::Sender<RenderResult>) {
        self.inner.lock().unwrap().insert(label, tx);
    }

    pub fn take(&self, label: &str) -> Option<tokio::sync::oneshot::Sender<RenderResult>> {
        self.inner.lock().unwrap().remove(label)
    }
}

/// 渲染超时（毫秒）
const RENDER_TIMEOUT_MS: u64 = 15_000;

/// 模板打印（原子操作：创建窗口 → 渲染 → 算高 → 打印 → 销毁）
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

    // 2. 解析引擎 HTML 路径（从资源目录加载）
    let engine_url = resolve_engine_url(app)?;

    // 3. 创建隐藏窗口
    let label = unique_window_label();
    let webview = create_hidden_window(app, &label)?;

    // 4. 注册渲染完成回调
    let (render_tx, render_rx) = tokio::sync::oneshot::channel::<RenderResult>();
    {
        let registry = app.state::<RenderRegistry>();
        registry.register(label.clone(), render_tx);
    }

    // 5. 导航到引擎页面
    let nav_timeout = 5_000u64;
    wait_for_navigation(&webview, &engine_url, nav_timeout).await?;

    // 6. 注入模板 + 数据 + 纸张参数，触发渲染
    let render_timeout = options.render_timeout_ms.unwrap_or(RENDER_TIMEOUT_MS);
    let js_code = build_render_invoke_js(&options);

    // 用 webview.eval() 执行渲染 JS（返回值我们不关心，结果通过 print_render_done 回调通知）
    let _ = webview
        .eval(&js_code)
        .map_err(|e| Error::WebView2(format!("执行渲染 JS 失败: {}", e)));

    // 7. 等待渲染完成（带超时）
    let render_result = tokio::time::timeout(
        std::time::Duration::from_millis(render_timeout),
        render_rx,
    )
    .await
    .map_err(|_| Error::PrintTimeout(format!("渲染超时 ({}ms)", render_timeout)))?
    .map_err(|_| Error::RenderEngine("渲染回调 channel 已关闭".to_string()))?;

    // 8. 根据内容高度计算纸张高度（如果 paper_height 为 None 或 0）
    let paper_height_mm = if options.paper_height.unwrap_or(0.0) > 0.0 {
        options.paper_height.unwrap()
    } else {
        // 从像素转换为毫米：假设 96 DPI
        // 1 inch = 25.4 mm, 96px = 1 inch => 1px = 25.4/96 mm
        px_to_mm(render_result.content_height_px)
    };

    // 9. 将渲染好的 HTML 写入当前 WebView 页面（替换文档内容）
    write_html_to_page(&webview, &render_result.html).await?;

    // 10. 构建打印设置
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
        // 模板打印需要打印背景（hiprint 的颜色、背景色等）
        print_backgrounds: true,
    };

    // 10. 触发打印（复用 print_service 的 do_print）
    let outcome = do_print(&webview, settings_data).await?;

    // 11. 清理窗口
    let _ = webview.close();

    Ok(format!("{}: {}", outcome.status, outcome.message))
}

/// 像素转毫米（假设 96 DPI）
fn px_to_mm(px: f64) -> f64 {
    px * 25.4 / 96.0
}

/// 构建渲染调用 JS 代码
fn build_render_invoke_js(options: &PrintTemplateOptions) -> String {
    // 将模板、数据、纸张参数传给全局 renderAndCalculate 函数
    // 注意：JSON 字符串需要安全转义后嵌入 JS
    let template_json = escape_js_string(&options.template);
    let data_json = escape_js_string(&options.data);
    let paper_width = options.paper_width;
    let paper_height = options.paper_height.unwrap_or(0.0);

    format!(
        r#"
        (function() {{
            if (typeof window.renderAndCalculate !== 'function') {{
                console.error('renderAndCalculate not found');
                return;
            }}
            window.renderAndCalculate({{
                templateJson: {template},
                dataJson: {data},
                paperWidthMm: {width},
                paperHeightMm: {height}
            }}).then(function(result) {{
                window.__TAURI__.invoke('plugin:printer-v2|print_render_done', {{
                    html: result.html,
                    contentHeightPx: result.contentHeightPx
                }});
            }}).catch(function(err) {{
                console.error('render error:', err);
                window.__TAURI__.invoke('plugin:printer-v2|print_render_done', {{
                    html: '',
                    contentHeightPx: 0
                }});
            }});
        }})();
        "#,
        template = template_json,
        data = data_json,
        width = paper_width,
        height = paper_height
    )
}

/// 将字符串安全嵌入 JS 代码（用 JSON.stringify 处理转义）
fn escape_js_string(s: &str) -> String {
    // 直接用 serde_json 转义，保证 JSON 字符串合法
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

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

/// 解析引擎 HTML 的 file:// URL
///
/// 引擎文件位于 Tauri 资源目录的 `print-engine/print-render.html`
fn resolve_engine_url<R: Runtime>(app: &AppHandle<R>) -> Result<String, Error> {
    use tauri::path::BaseDirectory;

    let engine_path = app
        .path()
        .resolve("print-engine/print-render.html", BaseDirectory::Resource)
        .map_err(|e| Error::WebView2(format!("解析引擎路径失败: {}", e)))?;

    if !engine_path.exists() {
        return Err(Error::RenderEngine(format!(
            "渲染引擎文件不存在: {}",
            engine_path.display()
        )));
    }

    let display = engine_path.display().to_string().replace('\\', "/");
    Ok(format!("file:///{}", display))
}

/// 渲染完成回调（由 print-render.html 中的 JS 调用，lib.rs 中作为 tauri command 注册）
pub async fn print_render_done<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    app: AppHandle<R>,
    html: String,
    content_height_px: f64,
) -> Result<(), Error> {
    let label = window.label().to_string();
    let registry: State<'_, RenderRegistry> = app.state();

    if let Some(tx) = registry.take(&label) {
        let result = RenderResult {
            html,
            content_height_px,
        };
        let _ = tx.send(result);
    } else {
        // 可能是超时后才到的回调，忽略
    }

    Ok(())
}

/// 将 HTML 内容写入当前 WebView 页面（替换整个文档）
///
/// 通过 document.open/write/close 同步替换，完成后即可直接打印。
async fn write_html_to_page<R: Runtime>(
    webview: &tauri::WebviewWindow<R>,
    html: &str,
) -> Result<(), Error> {
    // 用 JSON 转义确保 HTML 内容安全嵌入 JS 字符串
    let html_json =
        serde_json::to_string(html).map_err(|e| Error::InvalidInput(format!("HTML 转义失败: {}", e)))?;

    let js_code = format!(
        "(function() {{ document.open(); document.write({}); document.close(); }})();",
        html_json
    );

    webview
        .eval(&js_code)
        .map_err(|e| Error::WebView2(format!("写入 HTML 到页面失败: {}", e)))?;

    // document.write 是同步的，但浏览器解析和布局需要一点时间
    // 给一个短暂延时确保布局完成（50ms 足够简单页面）
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_px_to_mm() {
        // 96px = 1 inch = 25.4mm
        assert!((px_to_mm(96.0) - 25.4).abs() < 0.01);
        assert!((px_to_mm(960.0) - 254.0).abs() < 0.01);
        assert!((px_to_mm(0.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_escape_js_string() {
        let s = escape_js_string(r#"hello "world""#);
        // serde_json 会生成带引号的字符串
        assert!(s.contains("\\\"world\\\""));
        assert!(s.starts_with('"'));
        assert!(s.ends_with('"'));
    }
}
