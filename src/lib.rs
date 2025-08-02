mod declare;
mod fsys;
mod windows;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

use std::env;

pub use crate::models::*;
use crate::declare::PrintHtmlOptions;

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

/**
 * 测试打印机连接
 */
#[tauri::command]
async fn ping<R: Runtime>(app: tauri::AppHandle<R>, payload: PingRequest) -> Result<PingResponse> {
    app.printer().ping(payload)
}

/**
 * 打印 HTML 内容
 */
#[tauri::command]
async fn print_html<R: Runtime>(app: tauri::AppHandle<R>, options: PrintHtmlOptions) -> Result<String> {
    app.printer().print_html(options)
}


/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the printer APIs.
pub trait PrinterExt<R: Runtime> {
    fn printer(&self) -> &Printer<R>;
}

impl<R: Runtime, T: Manager<R>> crate::PrinterExt<R> for T {
    fn printer(&self) -> &Printer<R> {
        self.state::<Printer<R>>().inner()
    }
}

/**
 * 创建临时文件
 * @param buffer_data base64字符串
 * @param filename 文件名
 * @returns 临时文件路径
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|create_temp_file')`.
fn create_temp_file(buffer_data: String, filename: String) -> String {
    let dir = env::temp_dir();
    let result = fsys::create_file_from_base64(
        buffer_data.as_str(),
        format!("{}{}", dir.display(), filename).as_str(),
    );
    if result.is_ok() {
        return format!("{}{}", dir.display(), filename);
    }
    return "".to_owned();
}

/**
 * 删除临时文件
 * @param filename 文件名
 * @returns 删除结果
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|create_temp_file')`.
fn remove_temp_file(filename: String) -> bool {
    let dir = env::temp_dir();
    let result = fsys::remove_file(format!("{}{}", dir.display(), filename).as_str());
    if result.is_ok() {
        return true;
    }
    return false;
}

/**
 * 获取打印机列表
 */
#[tauri::command]
// this will be accessible with `invoke('plugin:printer|get_printers')`.
fn get_printers() -> String {
    if cfg!(windows) {
        return windows::get_printers();
    }

    return "Unsupported OS".to_string();
}

/**
 * 获取打印机列表
 * @param printername 打印机名称
 * @returns 打印机列表
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|get_printer_by_name')`.
fn get_printers_by_name(printername: String) -> String {
    println!("获取打印机列表: {}", printername);
    if cfg!(windows) {
        return windows::get_printers_by_name(printername);
    }

    return "Unsupported OS".to_string();
}

/**
 * 打印PDF
 * @param id 打印机ID
 * @param path PDF文件路径
 * @param printer_setting 打印机设置
 * @param remove_after_print 打印完成后删除文件
 * @returns 打印结果
 */
#[tauri::command(rename_all = "snake_case")]    
// this will be accessible with `invoke('plugin:printer|print_pdf')`.
fn print_pdf(
    id: String,
    path: String,
    printer_setting: String,
    remove_after_print: bool,
) -> String {
    println!("打印PDF: {:?}", id);
    println!("打印PDF: {:?}", path);
    println!("打印PDF: {:?}", printer_setting);
    println!("打印PDF: {:?}", remove_after_print);
    if cfg!(windows) {
        let options = declare::PrintOptions { 
            id,
            path,
            print_setting: printer_setting,
            remove_after_print: remove_after_print,
        };
        return windows::print_pdf(options);
    }

    return "Unsupported OS".to_string();
}

#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|get_jobs')`.
fn get_jobs(printername: String) -> String {
    if cfg!(windows) {
        return windows::get_jobs(printername);
    }
    return "Unsupported OS".to_string();
}

/**
 * 获取打印任务列表
 * @param printername 打印机名称
 * @param jobid 打印任务ID
 * @returns 打印任务列表
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|get_jobs_by_id')`.
fn get_jobs_by_id(printername: String, jobid: String) -> String {
    if cfg!(windows) {
        return windows::get_jobs_by_id(printername, jobid);
    }
    return "Unsupported OS".to_string();
}

/**
 * 恢复打印任务
 * @param printername 打印机名称
 * @param jobid 打印任务ID
 * @returns 恢复结果
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|restart_job')`.
fn resume_job(printername: String, jobid: String) -> String {
    if cfg!(windows) {
        return windows::resume_job(printername, jobid);
    }
    return "Unsupported OS".to_string();
}

/**
 * 重启打印任务
 * @param printername 打印机名称
 * @param jobid 打印任务ID
 * @returns 重启结果
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|restart_job')`.
fn restart_job(printername: String, jobid: String) -> String {
    if cfg!(windows) {
        return windows::restart_job(printername, jobid);
    }
    return "Unsupported OS".to_string();
}

/**
 * 暂停打印任务
 * @param printername 打印机名称
 * @param jobid 打印任务ID
 * @returns 暂停结果
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|pause_job')`.
fn pause_job(printername: String, jobid: String) -> String {
    if cfg!(windows) {
        return windows::pause_job(printername, jobid);
    }
    return "Unsupported OS".to_string();
}

/**
 * 删除打印任务
 * @param printername 打印机名称
 * @param jobid 打印任务ID
 * @returns 删除结果
 */
#[tauri::command(rename_all = "snake_case")]
// this will be accessible with `invoke('plugin:printer|remove_job')`.
fn remove_job(printername: String, jobid: String) -> String {
    if cfg!(windows) {
        return windows::remove_job(printername, jobid);
    }
    return "Unsupported OS".to_string();
}

/**
 * 获取打印机列表
 * @param printername 打印机名称
 * @returns 打印机列表
 */
pub fn custom_get_printers_by_name(printername: String) -> String {
    if cfg!(windows) {
        return windows::get_printers_by_name(printername);
    }

    return "Unsupported OS".to_string();
}

/**
 * 打印PDF
 * @param id 打印机ID
 * @param path PDF文件路径
 * @param printer_setting 打印机设置
 * @param remove_after_print 打印完成后删除文件
 * @returns 打印结果
 */
pub fn custom_print_pdf(
    id: String,
    path: String,
    printer_setting: String,
    remove_after_print: bool,
) -> String {
    if cfg!(windows) {
        let options = declare::PrintOptions {
            id,
            path,
            print_setting: printer_setting,
            remove_after_print: remove_after_print,
        };
        return windows::print_pdf(options);
    }

    return "Unsupported OS".to_string();
}

/**
 * 初始化插件
 * @returns 初始化结果
 */
/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  if cfg!(windows) {
    windows::init_windows();
  }
    Builder::new("printer")
        .invoke_handler(tauri::generate_handler![
            ping,
            print_html,
            create_temp_file,
            remove_temp_file,
            get_printers,
            get_printers_by_name,
            print_pdf,
            get_jobs,
            get_jobs_by_id,
            resume_job,
            restart_job,
            pause_job,
            remove_job
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let printer = mobile::init(app, api)?;
            #[cfg(desktop)]
            let printer = desktop::init(app, api)?;
            app.manage(printer);
            Ok(())
        })
        .build()
}
