use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::declare::{JobInfo, PrintHtmlOptions, PrintOptions, PrinterInfo};
use crate::models::*;

/// 打印机 API 访问入口
pub struct Printer<R: Runtime> {
    app: AppHandle<R>,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Printer<R>> {
    Ok(Printer { app: app.clone() })
}

impl<R: Runtime> Printer<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

    pub fn get_printers(&self) -> crate::Result<Vec<PrinterInfo>> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::get_printers().map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn get_printer_by_name(&self, name: &str) -> crate::Result<PrinterInfo> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::get_printers_by_name(name).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    /// 打印 PDF（WebView2 静默打印）
    pub async fn print_pdf(&self, options: PrintOptions) -> crate::Result<String> {
        #[cfg(target_os = "windows")]
        {
            crate::print_service::print_pdf(&self.app, options).await
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    /// 打印 HTML（WebView2 静默打印）
    pub async fn print_html(&self, options: PrintHtmlOptions) -> crate::Result<String> {
        #[cfg(target_os = "windows")]
        {
            crate::print_service::print_html(&self.app, options).await
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn get_jobs(&self, printer_name: &str) -> crate::Result<Vec<JobInfo>> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::get_jobs(printer_name).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn get_job_by_id(&self, printer_name: &str, job_id: u32) -> crate::Result<JobInfo> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::get_jobs_by_id(printer_name, job_id).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn resume_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::resume_job(printer_name, job_id).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn restart_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::restart_job(printer_name, job_id).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn pause_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::pause_job(printer_name, job_id).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }

    pub fn remove_job(&self, printer_name: &str, job_id: u32) -> crate::Result<()> {
        #[cfg(target_os = "windows")]
        {
            crate::windows::remove_job(printer_name, job_id).map_err(Into::into)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(crate::Error::UnsupportedPlatform)
        }
    }
}
