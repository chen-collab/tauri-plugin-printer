use std::path::PathBuf;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, Manager};

use crate::declare::{PrintHtmlOptions, PrintOptions, PrinterInfo, JobInfo};
use crate::models::*;

/// 打印机 API 访问入口
pub struct Printer<R: Runtime> {
    app: AppHandle<R>,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Printer<R>> {
    Ok(Printer {
        app: app.clone(),
    })
}

impl<R: Runtime> Printer<R> {
    /// 运行时从 Tauri 资源目录解析 sm 路径（延迟到打印时检查，不在 setup 阶段报错）
    #[cfg(target_os = "windows")]
    fn resolve_sm_exe(&self) -> crate::Result<PathBuf> {
        // 1. 生产模式：resource_dir() → 可执行文件所在目录
        if let Ok(dir) = self.app.path().resource_dir() {
            let exe = dir.join("sm");
            if exe.exists() {
                return Ok(exe);
            }
        }

        // 2. 开发模式：从当前工作目录查找（tauri dev 时 CWD 为项目根目录）
        if let Ok(cwd) = std::env::current_dir() {
            let dev_paths = [
                cwd.join("resources").join("sm"),           // 项目根/resources/sm
                cwd.join("src-tauri").join("resources").join("sm"), // 项目根/src-tauri/resources/sm
            ];
            for p in &dev_paths {
                if p.exists() {
                    return Ok(p.clone());
                }
            }
        }

        // 3. 都未找到，报错
        let dir = self.app.path().resource_dir()
            .unwrap_or_else(|_| PathBuf::from("(unknown)"));
        Err(crate::Error::InvalidInput(
            format!("未找到 PDF 打印引擎 sm。\n  - 已检查: {}\\sm\n  - 开发模式: resources/sm 或 src-tauri/resources/sm\n请确保已按 README 放置 sm 文件", dir.display())
        ))
    }

    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse { value: payload.value })
    }

    pub fn get_printers(&self) -> crate::Result<Vec<PrinterInfo>> {
        #[cfg(target_os = "windows")]
        { crate::windows::get_printers().map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn get_printer_by_name(&self, name: &str) -> crate::Result<PrinterInfo> {
        #[cfg(target_os = "windows")]
        { crate::windows::get_printers_by_name(name).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn print_pdf(&self, options: PrintOptions) -> crate::Result<String> {
        #[cfg(target_os = "windows")]
        {
            let sm_exe = self.resolve_sm_exe()?;
            crate::windows::print_pdf(options, &sm_exe)
        }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn print_html(&self, options: PrintHtmlOptions) -> crate::Result<String> {
        #[cfg(target_os = "windows")]
        {
            let sm_exe = self.resolve_sm_exe()?;
            crate::windows::print_html(options, &sm_exe)
        }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn get_jobs(&self, printer_name: &str) -> crate::Result<Vec<JobInfo>> {
        #[cfg(target_os = "windows")]
        { crate::windows::get_jobs(printer_name).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn get_job_by_id(&self, printer_name: &str, job_id: u32) -> crate::Result<JobInfo> {
        #[cfg(target_os = "windows")]
        { crate::windows::get_jobs_by_id(printer_name, job_id).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn resume_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        { crate::windows::resume_job(printer_name, job_id).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn restart_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        { crate::windows::restart_job(printer_name, job_id).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn pause_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        { crate::windows::pause_job(printer_name, job_id).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }

    pub fn remove_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        { crate::windows::remove_job(printer_name, job_id).map_err(Into::into) }
        #[cfg(not(target_os = "windows"))]
        { Err(crate::Error::UnsupportedPlatform) }
    }
}