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
    /// 运行时从 Tauri 资源目录解析 sm.exe 路径（延迟到打印时检查，不在 setup 阶段报错）
    #[cfg(target_os = "windows")]
    fn resolve_sm_exe(&self) -> crate::Result<PathBuf> {
        let dir = self.app.path().resource_dir()
            .map_err(|e| crate::Error::InvalidInput(format!("获取资源目录失败: {}", e)))?;
        let exe = dir.join("sm.exe");
        if !exe.exists() {
            return Err(crate::Error::InvalidInput(
                "未找到 PDF 打印引擎 sm.exe。请确保主程序已通过 tauri.conf.json 的 bundle.resources 打包 sm.exe 到 resources/ 目录".into()
            ));
        }
        Ok(exe)
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