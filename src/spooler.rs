/// Windows Spooler API 封装（适配 windows crate 0.61）
use crate::declare::{JobInfo, PrinterInfo};
use crate::Error;

use windows::core::HSTRING;
use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::Graphics::Printing::*;

/// 自定义 PRINTER_INFO_2W 结构体（windows crate 0.61 中未包含此类型）
#[allow(non_snake_case)]
#[repr(C)]
struct PRINTER_INFO_2W {
    pServerName: *mut u16,
    pPrinterName: *mut u16,
    pShareName: *mut u16,
    pPortName: *mut u16,
    pDriverName: *mut u16,
    pComment: *mut u16,
    pLocation: *mut u16,
    pDevMode: *mut u8,
    pSepFile: *mut u16,
    pPrintProcessor: *mut u16,
    pDatatype: *mut u16,
    pParameters: *mut u16,
    pSecurityDescriptor: *mut u8,
    Attributes: u32,
    Priority: u32,
    DefaultPriority: u32,
    StartTime: u32,
    UntilTime: u32,
    Status: u32,
    cJobs: u32,
    AveragePPM: u32,
}

/// 宽字符串指针转换为 Rust String
unsafe fn pwstr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
}

/// 格式化 SYSTEMTIME 为字符串
unsafe fn systemtime_to_string(st: &SYSTEMTIME) -> String {
    format!(
        "{}-{}-{} {}:{}:{}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
    )
}

/// 打印机状态位掩码映射为友好字符串数组
fn map_printer_status(status: u32) -> Vec<String> {
    let mut result = Vec::new();
    let pairs: &[(u32, &str)] = &[
        (0x00000001, "paused"),
        (0x00000002, "error"),
        (0x00000004, "pendingDeletion"),
        (0x00000008, "paperJam"),
        (0x00000010, "paperOut"),
        (0x00000020, "manualFeed"),
        (0x00000040, "paperProblem"),
        (0x00000080, "offline"),
        (0x00000100, "ioActive"),
        (0x00000200, "busy"),
        (0x00000400, "printing"),
        (0x00000800, "outputBinFull"),
        (0x00001000, "notAvailable"),
        (0x00002000, "waiting"),
        (0x00004000, "processing"),
        (0x00008000, "initializing"),
        (0x00010000, "warmingUp"),
        (0x00020000, "tonerLow"),
        (0x00040000, "noToner"),
        (0x00080000, "pagePunt"),
        (0x00100000, "userIntervention"),
        (0x00200000, "outOfMemory"),
        (0x00400000, "doorOpen"),
        (0x00800000, "serverUnknown"),
        (0x01000000, "powerSave"),
    ];
    for (mask, name) in pairs {
        if status & mask != 0 {
            result.push(name.to_string());
        }
    }
    if result.is_empty() {
        result.push("normal".to_string());
    }
    result
}

/// 打印任务状态位掩码映射为友好字符串数组
fn map_job_status(status: u32) -> Vec<String> {
    let mut result = Vec::new();
    let pairs: &[(u32, &str)] = &[
        (0x00000001, "paused"),
        (0x00000002, "error"),
        (0x00000004, "deleting"),
        (0x00000008, "spooling"),
        (0x00000010, "printing"),
        (0x00000020, "offline"),
        (0x00000040, "paperOut"),
        (0x00000080, "printed"),
        (0x00000100, "deleted"),
        (0x00000200, "blocked"),
        (0x00000400, "userIntervention"),
        (0x00000800, "restart"),
    ];
    for (mask, name) in pairs {
        if status & mask != 0 {
            result.push(name.to_string());
        }
    }
    if result.is_empty() {
        result.push("normal".to_string());
    }
    result
}

/// RAII 打印机句柄，Drop 时自动调用 ClosePrinter
pub struct PrinterHandle(pub PRINTER_HANDLE);

impl Drop for PrinterHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = ClosePrinter(self.0);
        }
    }
}

/// 打开打印机句柄
pub fn open_printer(name: &str) -> Result<PrinterHandle, Error> {
    let name_hstring = HSTRING::from(name);
    let mut handle = PRINTER_HANDLE::default();
    unsafe {
        OpenPrinterW(&name_hstring, &mut handle, None)
            .map_err(|e| Error::WindowsApi(format!("无法打开打印机 '{}': {:?}", name, e)))?;
    }
    Ok(PrinterHandle(handle))
}

/// 枚举所有打印机（本地 + 网络连接）
pub fn list_printers() -> Result<Vec<PrinterInfo>, Error> {
    let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    let mut needed: u32 = 0;
    let mut returned: u32 = 0;
    // 第一次调用：获取缓冲区大小
    unsafe {
        let _ = EnumPrintersW(flags, None, 2, None, &mut needed, &mut returned);
    }
    if needed == 0 {
        return Ok(Vec::new());
    }
    let mut buffer: Vec<u8> = vec![0u8; needed as usize];
    unsafe {
        EnumPrintersW(
            flags,
            None,
            2,
            Some(&mut buffer[..]),
            &mut needed,
            &mut returned,
        )
        .map_err(|e| Error::WindowsApi(format!("EnumPrintersW failed: {:?}", e)))?;
    }
    let mut printers = Vec::with_capacity(returned as usize);
    unsafe {
        let base = buffer.as_ptr() as *const PRINTER_INFO_2W;
        for i in 0..returned as usize {
            let info = &*base.add(i);
            printers.push(PrinterInfo {
                name: pwstr_to_string(info.pPrinterName),
                driver_name: pwstr_to_string(info.pDriverName),
                job_count: info.cJobs,
                print_processor: pwstr_to_string(info.pPrintProcessor),
                port_name: pwstr_to_string(info.pPortName),
                share_name: pwstr_to_string(info.pShareName),
                computer_name: pwstr_to_string(info.pServerName),
                printer_status: map_printer_status(info.Status),
                shared: (info.Attributes & 0x00000008) != 0,
                printer_type: info.Attributes,
                priority: info.Priority,
            });
        }
    }
    Ok(printers)
}

/// 按名称获取打印机信息
pub fn get_printer(name: &str) -> Result<PrinterInfo, Error> {
    let handle = open_printer(name)?;
    let mut needed: u32 = 0;
    unsafe {
        let _ = GetPrinterW(handle.0, 2, None, &mut needed);
    }
    if needed == 0 {
        return Err(Error::WindowsApi(format!("printer '{}' not found", name)));
    }
    let mut buffer: Vec<u8> = vec![0u8; needed as usize];
    unsafe {
        GetPrinterW(handle.0, 2, Some(&mut buffer[..]), &mut needed)
            .map_err(|e| Error::WindowsApi(format!("GetPrinterW failed: {:?}", e)))?;
    }
    unsafe {
        let info = &*(buffer.as_ptr() as *const PRINTER_INFO_2W);
        Ok(PrinterInfo {
            name: pwstr_to_string(info.pPrinterName),
            driver_name: pwstr_to_string(info.pDriverName),
            job_count: info.cJobs,
            print_processor: pwstr_to_string(info.pPrintProcessor),
            port_name: pwstr_to_string(info.pPortName),
            share_name: pwstr_to_string(info.pShareName),
            computer_name: pwstr_to_string(info.pServerName),
            printer_status: map_printer_status(info.Status),
            shared: (info.Attributes & 0x00000008) != 0,
            printer_type: info.Attributes,
            priority: info.Priority,
        })
    }
}

/// 列出打印机的所有任务
pub fn list_jobs(printer_name: &str) -> Result<Vec<JobInfo>, Error> {
    let handle = open_printer(printer_name)?;
    let mut needed: u32 = 0;
    let mut returned: u32 = 0;
    unsafe {
        let _ = EnumJobsW(handle.0, 0, 99, 1, None, &mut needed, &mut returned);
    }
    if needed == 0 {
        return Ok(Vec::new());
    }
    let mut buffer: Vec<u8> = vec![0u8; needed as usize];
    unsafe {
        EnumJobsW(
            handle.0,
            0,
            99,
            1,
            Some(&mut buffer[..]),
            &mut needed,
            &mut returned,
        )
        .map_err(|e| Error::WindowsApi(format!("EnumJobsW failed: {:?}", e)))?;
    }
    let mut jobs = Vec::with_capacity(returned as usize);
    unsafe {
        let base = buffer.as_ptr() as *const JOB_INFO_1W;
        for i in 0..returned as usize {
            let job = &*base.add(i);
            jobs.push(JobInfo {
                document_name: pwstr_to_string(job.pDocument.0),
                id: job.JobId,
                total_pages: job.TotalPages,
                position: job.Position,
                size: 0,
                submitted_time: systemtime_to_string(&job.Submitted),
                user_name: pwstr_to_string(job.pUserName.0),
                pages_printed: job.PagesPrinted,
                job_time: 0,
                computer_name: pwstr_to_string(job.pMachineName.0),
                datatype: pwstr_to_string(job.pDatatype.0),
                printer_name: pwstr_to_string(job.pPrinterName.0),
                priority: job.Priority,
                job_status: map_job_status(job.Status),
            });
        }
    }
    Ok(jobs)
}

/// 打印任务控制命令
#[derive(Debug, Clone, Copy)]
pub enum JobCommand {
    Pause,
    Resume,
    Restart,
    Delete,
}

/// 控制打印任务
pub fn control_job(printer_name: &str, job_id: u32, command: JobCommand) -> Result<(), Error> {
    let handle = open_printer(printer_name)?;
    let cmd = match command {
        JobCommand::Pause => JOB_CONTROL_PAUSE,
        JobCommand::Resume => JOB_CONTROL_RESUME,
        JobCommand::Restart => JOB_CONTROL_RESTART,
        JobCommand::Delete => JOB_CONTROL_DELETE,
    };
    unsafe {
        let _ = SetJobW(handle.0, job_id, 0, None, cmd);
    }
    Ok(())
}
