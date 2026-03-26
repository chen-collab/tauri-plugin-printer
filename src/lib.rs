mod declare;
mod fsys;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(target_os = "windows"))]
mod linux;

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

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the printer APIs.
pub trait PrinterExt<R: Runtime> {
    fn printer(&self) -> &Printer<R>;
}

impl<R: Runtime, T: Manager<R>> crate::PrinterExt<R> for T {
    fn printer(&self) -> &Printer<R> {
        self.state::<Printer<R>>().inner()
    }
}

#[tauri::command]
async fn ping<R: Runtime>(
    app: tauri::AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.printer().ping(payload)
}

#[tauri::command(rename_all = "snake_case")]
async fn print_html<R: Runtime>(
    app: tauri::AppHandle<R>,
    options: PrintHtmlOptions,
) -> Result<String> {
    app.printer().print_html(options)
}

/// Write a base64-encoded buffer to a temp file; returns the file path on success.
#[tauri::command(rename_all = "snake_case")]
fn create_temp_file(buffer_data: String, filename: String) -> String {
    let dir = env::temp_dir();
    let path = format!("{}{}", dir.display(), filename);
    match fsys::create_file_from_base64(buffer_data.as_str(), &path) {
        Ok(_) => path,
        Err(_) => String::new(),
    }
}

/// Remove a file from the temp directory.
#[tauri::command(rename_all = "snake_case")]
fn remove_temp_file(filename: String) -> bool {
    let dir = env::temp_dir();
    fsys::remove_file(format!("{}{}", dir.display(), filename).as_str()).is_ok()
}

#[tauri::command]
fn get_printers() -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_printers()
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::get_printers()
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_printers_by_name(printername: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_printers_by_name(printername)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::get_printers_by_name(printername)
    }
}

/// Print a PDF file.
/// `print_setting` is the destination printer name; leave empty to use the system default.
#[tauri::command(rename_all = "snake_case")]
fn print_pdf(
    id: String,
    path: String,
    print_setting: String,
    remove_after_print: bool,
) -> String {
    let options = declare::PrintOptions {
        id,
        path,
        print_setting,
        remove_after_print,
    };
    #[cfg(target_os = "windows")]
    {
        windows::print_pdf(options)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::print_pdf(options)
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_jobs(printername: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_jobs(printername)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::get_jobs(printername)
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_jobs_by_id(printername: String, jobid: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::get_jobs_by_id(printername, jobid)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::get_jobs_by_id(printername, jobid)
    }
}

#[tauri::command(rename_all = "snake_case")]
fn resume_job(printername: String, jobid: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::resume_job(printername, jobid)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::resume_job(printername, jobid)
    }
}

#[tauri::command(rename_all = "snake_case")]
fn restart_job(printername: String, jobid: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::restart_job(printername, jobid)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::restart_job(printername, jobid)
    }
}

#[tauri::command(rename_all = "snake_case")]
fn pause_job(printername: String, jobid: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::pause_job(printername, jobid)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::pause_job(printername, jobid)
    }
}

#[tauri::command(rename_all = "snake_case")]
fn remove_job(printername: String, jobid: String) -> String {
    #[cfg(target_os = "windows")]
    {
        windows::remove_job(printername, jobid)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux::remove_job(printername, jobid)
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    #[cfg(target_os = "windows")]
    windows::init_windows();

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
            remove_job,
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
