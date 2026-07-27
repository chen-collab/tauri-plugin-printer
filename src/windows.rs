//! Windows 平台打印机管理（Spooler API 转发）。
//!
//! 打印实现已迁移至 `print_service` + `webview2_print` 模块（WebView2 COM 方式）。
//! 本模块仅保留打印机列表、打印任务管理等 Spooler 相关功能。

use crate::declare::{JobInfo, PrinterInfo};
use crate::Error;

/// 获取所有打印机列表
pub fn get_printers() -> Result<Vec<PrinterInfo>, Error> {
    crate::spooler::list_printers()
}

/// 按名称获取打印机
pub fn get_printers_by_name(name: &str) -> Result<PrinterInfo, Error> {
    crate::spooler::get_printer(name)
}

/// 获取打印机任务列表
pub fn get_jobs(printer_name: &str) -> Result<Vec<JobInfo>, Error> {
    crate::spooler::list_jobs(printer_name)
}

/// 按 ID 获取打印机任务
pub fn get_jobs_by_id(printer_name: &str, job_id: u32) -> Result<JobInfo, Error> {
    let jobs = crate::spooler::list_jobs(printer_name)?;
    jobs.into_iter()
        .find(|j| j.id == job_id)
        .ok_or_else(|| Error::WindowsApi(format!("任务 {} 不存在", job_id)))
}

/// 恢复打印任务
pub fn resume_job(printer_name: &str, job_id: u32) -> Result<(), Error> {
    crate::spooler::control_job(printer_name, job_id, crate::spooler::JobCommand::Resume)
}

/// 重启打印任务
pub fn restart_job(printer_name: &str, job_id: u32) -> Result<(), Error> {
    crate::spooler::control_job(printer_name, job_id, crate::spooler::JobCommand::Restart)
}

/// 暂停打印任务
pub fn pause_job(printer_name: &str, job_id: u32) -> Result<(), Error> {
    crate::spooler::control_job(printer_name, job_id, crate::spooler::JobCommand::Pause)
}

/// 删除打印任务
pub fn remove_job(printer_name: &str, job_id: u32) -> Result<(), Error> {
    crate::spooler::control_job(printer_name, job_id, crate::spooler::JobCommand::Delete)
}
