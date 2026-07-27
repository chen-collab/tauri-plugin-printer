mod declare;
mod fsys;
#[cfg(target_os = "windows")]
pub mod print_engine;
#[cfg(target_os = "windows")]
pub mod print_service;
#[cfg(target_os = "windows")]
mod spooler;
#[cfg(target_os = "windows")]
mod webview2_print;
#[cfg(target_os = "windows")]
mod windows;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

use crate::declare::{JobInfo, PrintHtmlOptions, PrintOptions, PrintTemplateOptions, PrinterInfo};
pub use crate::models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Printer;
#[cfg(mobile)]
use mobile::Printer;

/// 测试打印机连接
#[tauri::command]
async fn ping<R: Runtime>(app: tauri::AppHandle<R>, payload: PingRequest) -> Result<PingResponse> {
    app.printer().ping(payload)
}

/// 打印 HTML 内容
#[tauri::command(rename_all = "camelCase")]
async fn print_html<R: Runtime>(
    app: tauri::AppHandle<R>,
    options: PrintHtmlOptions,
) -> Result<String> {
    app.printer().print_html(options).await
}

/// 创建临时文件
#[tauri::command(rename_all = "camelCase")]
fn create_temp_file(buffer_data: String, filename: String) -> Result<String> {
    let safe_name = fsys::sanitize_filename(&filename)?;
    let dir = std::env::temp_dir();
    let file_path = dir.join(&safe_name);
    fsys::create_file_from_base64(&buffer_data, file_path.to_str().unwrap_or(""))?;
    Ok(file_path.to_string_lossy().to_string())
}

/// 删除临时文件
#[tauri::command(rename_all = "camelCase")]
fn remove_temp_file(filename: String) -> Result<bool> {
    let safe_name = fsys::sanitize_filename(&filename)?;
    let dir = std::env::temp_dir();
    let file_path = dir.join(&safe_name);
    fsys::remove_file(file_path.to_str().unwrap_or(""))?;
    Ok(true)
}

/// 获取打印机列表
#[tauri::command]
async fn get_printers<R: Runtime>(app: tauri::AppHandle<R>) -> Result<Vec<PrinterInfo>> {
    app.printer().get_printers()
}

/// 按名称获取打印机
#[tauri::command(rename_all = "camelCase")]
async fn get_printers_by_name<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
) -> Result<PrinterInfo> {
    app.printer().get_printer_by_name(&printername)
}

/// 打印 PDF
#[tauri::command(rename_all = "camelCase")]
async fn print_pdf<R: Runtime>(
    app: tauri::AppHandle<R>,
    id: String,
    path: String,
    printer_setting: String,
    remove_after_print: bool,
) -> Result<String> {
    let options = PrintOptions {
        id,
        path,
        printer_setting,
        remove_after_print,
        orientation: None,
        grayscale: None,
        copies: None,
    };
    app.printer().print_pdf(options).await
}

/// 模板打印（三层架构：前端传数据+模板，Rust 原子调度，引擎负责渲染）
#[tauri::command(rename_all = "camelCase")]
async fn print_template<R: Runtime>(
    app: tauri::AppHandle<R>,
    options: PrintTemplateOptions,
) -> Result<String> {
    app.printer().print_template(options).await
}

/// 渲染完成回调（内部 command，由 print-render.html 引擎页面调用）
#[tauri::command(rename_all = "camelCase")]
async fn print_render_done<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    app: tauri::AppHandle<R>,
    html: String,
    content_height_px: f64,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        crate::print_engine::print_render_done(window, app, html, content_height_px).await
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, app, html, content_height_px);
        Err(crate::Error::UnsupportedPlatform)
    }
}

/// 获取打印任务列表
#[tauri::command(rename_all = "camelCase")]
async fn get_jobs<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
) -> Result<Vec<JobInfo>> {
    app.printer().get_jobs(&printername)
}

/// 按 ID 获取打印任务
#[tauri::command(rename_all = "camelCase")]
async fn get_jobs_by_id<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
    jobid: String,
) -> Result<JobInfo> {
    let jid: u32 = jobid
        .parse()
        .map_err(|_| Error::InvalidInput(format!("无效的 jobid: {}", jobid)))?;
    app.printer().get_job_by_id(&printername, jid)
}

/// 恢复打印任务
#[tauri::command(rename_all = "camelCase")]
async fn resume_job<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
    jobid: String,
) -> Result<()> {
    let jid: u32 = jobid
        .parse()
        .map_err(|_| Error::InvalidInput(format!("无效的 jobid: {}", jobid)))?;
    app.printer().resume_job(&printername, jid)
}

/// 重启打印任务
#[tauri::command(rename_all = "camelCase")]
async fn restart_job<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
    jobid: String,
) -> Result<()> {
    let jid: u32 = jobid
        .parse()
        .map_err(|_| Error::InvalidInput(format!("无效的 jobid: {}", jobid)))?;
    app.printer().restart_job(&printername, jid)
}

/// 暂停打印任务
#[tauri::command(rename_all = "camelCase")]
async fn pause_job<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
    jobid: String,
) -> Result<()> {
    let jid: u32 = jobid
        .parse()
        .map_err(|_| Error::InvalidInput(format!("无效的 jobid: {}", jobid)))?;
    app.printer().pause_job(&printername, jid)
}

/// 删除打印任务
#[tauri::command(rename_all = "camelCase")]
async fn remove_job<R: Runtime>(
    app: tauri::AppHandle<R>,
    printername: String,
    jobid: String,
) -> Result<()> {
    let jid: u32 = jobid
        .parse()
        .map_err(|_| Error::InvalidInput(format!("无效的 jobid: {}", jobid)))?;
    app.printer().remove_job(&printername, jid)
}

/// 扩展方法，从 Tauri 状态中获取 Printer 实例
pub trait PrinterExt<R: Runtime> {
    fn printer(&self) -> &Printer<R>;
}

impl<R: Runtime, T: Manager<R>> crate::PrinterExt<R> for T {
    fn printer(&self) -> &Printer<R> {
        self.state::<Printer<R>>().inner()
    }
}

/// 初始化插件
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("printer-v2")
        .invoke_handler(tauri::generate_handler![
            ping,
            print_html,
            create_temp_file,
            remove_temp_file,
            get_printers,
            get_printers_by_name,
            print_pdf,
            print_template,
            print_render_done,
            get_jobs,
            get_jobs_by_id,
            resume_job,
            restart_job,
            pause_job,
            remove_job
        ])
        .setup(|app, api| {
            #[cfg(desktop)]
            let printer = desktop::init(app, api)?;
            #[cfg(mobile)]
            let printer = mobile::init(app, api)?;

            app.manage(printer);

            // 注册渲染完成回调注册表（用于模板打印引擎的回调路由）
            #[cfg(target_os = "windows")]
            app.manage(crate::print_engine::RenderRegistry::new());

            Ok(())
        })
        .build()
}
