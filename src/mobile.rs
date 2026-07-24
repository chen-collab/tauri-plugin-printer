use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::declare::{PrintHtmlOptions, PrintOptions, PrinterInfo, JobInfo};
use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_printer);

/// 初始化 Kotlin 或 Swift 插件类
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Printer<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("", "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_printer)?;
  Ok(Printer(handle))
}

/// 移动端打印机 API 访问入口（大部分功能不可用）
pub struct Printer<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Printer<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self.0.run_mobile_plugin("ping", payload).map_err(Into::into)
  }

  pub fn get_printers(&self) -> crate::Result<Vec<PrinterInfo>> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn get_printer_by_name(&self, _name: &str) -> crate::Result<PrinterInfo> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn print_pdf(&self, _options: PrintOptions) -> crate::Result<String> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn print_html(&self, _options: PrintHtmlOptions) -> crate::Result<String> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn get_jobs(&self, _printer_name: &str) -> crate::Result<Vec<JobInfo>> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn get_job_by_id(&self, _printer_name: &str, _job_id: u32) -> crate::Result<JobInfo> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn resume_job(&self, _printer_name: &str, _job_id: u32) -> crate::Result<()> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn restart_job(&self, _printer_name: &str, _job_id: u32) -> crate::Result<()> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn pause_job(&self, _printer_name: &str, _job_id: u32) -> crate::Result<()> {
    Err(crate::Error::UnsupportedPlatform)
  }

  pub fn remove_job(&self, _printer_name: &str, _job_id: u32) -> crate::Result<()> {
    Err(crate::Error::UnsupportedPlatform)
  }
}