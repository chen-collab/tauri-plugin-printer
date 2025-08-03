

use std::process::Command;
use std::{sync::mpsc, io::Write};
use std::thread;
use std::fs::{File};
use std::env;
use std::path::{Path, PathBuf};
use crate::declare::{PrintOptions, PrintHtmlOptions};
use crate::{ fsys::remove_file};
/**
 * Create sm.exe to temp
 */
fn create_file(path: String, bin: &[u8]) -> std::io::Result<()> {
    let mut f = File::create(format!("{}sm.exe", path))?;
    f.write_all(bin)?;
  
    f.sync_all()?;
    Ok(())
}

  
/**
 * init sm.exe
 */
pub fn init_windows() {
    let sm = include_bytes!("bin/sm");
    let dir: std::path::PathBuf = env::temp_dir();
    let result: Result<(), std::io::Error>  = create_file(dir.display().to_string(),sm);
    if result.is_err() {
        panic!("Gagal")
    }
}

/**
 * Get printers on windows using powershell
 */
pub fn get_printers() -> String {
    // Create a channel for communication
    let (sender, receiver) = mpsc::channel();

    // Spawn a new thread
    thread::spawn(move || {
        // let output: tauri_plugin_shell::process::Output = Command::new("powershell").args(["Get-Printer | Select-Object Name, DriverName, JobCount, PrintProcessor, PortName, ShareName, ComputerName, PrinterStatus, Shared, Type, Priority | ConvertTo-Json"]).output().unwrap();

        let output = Command::new("powershell")
            .args(["-Command", "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-Printer | Select-Object Name, DriverName, JobCount, PrintProcessor, PortName, ShareName, ComputerName, PrinterStatus, Shared, Type, Priority | ConvertTo-Json"])
            .output().unwrap();
        
        let output_string = String::from_utf8_lossy(&output.stdout).to_string();
        sender.send(output_string).unwrap();
    });

    // Do other non-blocking work on the main thread

    // Receive the result from the spawned thread
    let result: String = receiver.recv().unwrap();


    return result;
}

/**
 * Get printers by name on windows using powershell
 */
pub fn get_printers_by_name(printername: String) -> String {
    // Create a channel for communication
    let (sender, receiver) = mpsc::channel();

    // Spawn a new thread
    thread::spawn(move || {
        let output = Command::new("powershell")
            .args(["-Command", &format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-Printer -Name '{}' | Select-Object Name, DriverName, JobCount, PrintProcessor, PortName, ShareName, ComputerName, PrinterStatus, Shared, Type, Priority | ConvertTo-Json", printername)])
            .output().unwrap();

        let output_string = String::from_utf8_lossy(&output.stdout).to_string();
        sender.send(output_string).unwrap();
    });

    // Receive the result from the spawned thread
    let result: String = receiver.recv().unwrap();

    return result;
}

/**
 * Print pdf file 
 */
pub fn print_pdf (options: PrintOptions) -> String {
    println!("options id {}", options.id);
    println!("options print_setting {}", options.print_setting);

    let dir: std::path::PathBuf = env::temp_dir();
    let print_setting: String = options.print_setting;
    let shell_command = if print_setting.is_empty() {
        format!("{}sm.exe -print-to-default -silent \"{}\"", dir.display(), options.path)
    } else {
        format!("{}sm.exe -print-to \"{}\" -silent \"{}\"", dir.display(), print_setting, options.path)
    };
    

    // Create a channel for communication
    let (sender, receiver) = mpsc::channel();
    println!("{}", shell_command);
    // Spawn a new thread
    thread::spawn(move || {
        // let output: tauri_plugin_shell::process::Output = Command::new("powershell").args([shell_command]).output().unwrap();

        // sender.send(output.stdout.to_string()).unwrap();

        let output = Command::new("powershell").args([shell_command]).output().unwrap();

        sender.send(String::from_utf8(output.stdout).unwrap()).unwrap();
    });

    // Do other non-blocking work on the main thread

    // Receive the result from the spawned thread
    let result = receiver.recv().unwrap();
    
    

    if options.remove_after_print == true {
        let _ = remove_file(&options.path);
    }
    
    return result;
}




/**
 * 打印 HTML 内容
 * 
 * 优化特性:
 * - 改进的错误处理和资源管理
 * - 更好的临时文件清理机制
 * - 增强的 wkhtmltopdf 参数配置
 * - 支持更多打印选项和边距单位
 */
pub fn print_html(options: PrintHtmlOptions) -> String {
    // 使用 Result 类型进行更好的错误处理
    match print_html_internal(options) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("HTML 打印失败: {}", e);
            format!("打印失败: {}", e)
        }
    }
}

/// 生成唯一的临时文件路径
fn generate_temp_file_path(extension: &str) -> Result<PathBuf, String> {
    let temp_dir = env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("获取时间戳失败: {}", e))?
        .as_nanos();
    let filename = format!("tauri_printer_{}_{}.{}", std::process::id(), timestamp, extension);
    Ok(temp_dir.join(filename))
}

/// 内部实现函数，使用 Result 进行错误处理
fn print_html_internal(options: PrintHtmlOptions) -> Result<String, String> {
    // 验证 HTML 内容
    if options.html.trim().is_empty() {
        return Err("HTML 内容不能为空".to_string());
    }

    // 检查 wkhtmltopdf 是否可用
    check_wkhtmltopdf_availability()?;

    // 生成临时文件路径
    let html_path = generate_temp_file_path("html")?;
    let pdf_path = generate_temp_file_path("pdf")?;
    
    println!("html_path: {:?}, pdf_path: {:?}", html_path, pdf_path);

    // 写入 HTML 内容到临时文件
    std::fs::write(&html_path, &options.html)
        .map_err(|e| format!("写入 HTML 内容失败: {}", e))?;

    // 构建 wkhtmltopdf 命令参数
    let args = build_wkhtmltopdf_args(&options, &html_path, &pdf_path)?;

    println!("wkhtmltopdf args: {:?}", args);

    // 执行 HTML 到 PDF 转换
    let conversion_result = execute_wkhtmltopdf(&args);
    
    // 如果转换失败，清理 HTML 文件并返回错误
    if let Err(e) = conversion_result {
        let _ = remove_file(&html_path.to_string_lossy());
        return Err(e);
    }

    // 验证 PDF 文件是否生成成功
    if !pdf_path.exists() {
        // 清理 HTML 文件
        let _ = remove_file(&html_path.to_string_lossy());
        return Err("PDF 文件生成失败".to_string());
    }
    
    println!("PDF 文件生成成功: {:?}", pdf_path);

    // 创建打印选项并执行打印
    let print_options = PrintOptions {
        path: pdf_path.to_string_lossy().to_string(),
        id: options.printer_id.unwrap_or_default(),
        print_setting: options.print_settings.unwrap_or_default(),
        remove_after_print: options.remove_after_print.unwrap_or(true),
    };

    // 执行打印
    let result = print_pdf(print_options);

    // 清理 HTML 临时文件（PDF 文件由 print_pdf 函数根据 remove_after_print 选项处理）
    let _ = remove_file(&html_path.to_string_lossy());
    
    Ok(result)
}

/// 检查 wkhtmltopdf 是否可用
fn check_wkhtmltopdf_availability() -> Result<(), String> {
    Command::new("wkhtmltopdf")
        .arg("--version")
        .output()
        .map_err(|_| "wkhtmltopdf 未安装或不在 PATH 中。请先安装 wkhtmltopdf。".to_string())?;
    Ok(())
}

/// 构建 wkhtmltopdf 命令参数
fn build_wkhtmltopdf_args(
    options: &PrintHtmlOptions,
    html_path: &Path,
    pdf_path: &Path,
) -> Result<Vec<String>, String> {
    let mut args = vec![
        "--encoding".to_string(),
        "UTF-8".to_string(),
        "--enable-local-file-access".to_string(),
        "--disable-smart-shrinking".to_string(), // 禁用智能缩放以获得更好的打印质量
        "--print-media-type".to_string(),
        "--no-pdf-compression".to_string(),      // 禁用 PDF 压缩以提高质量
        "--load-error-handling".to_string(),
        "ignore".to_string(),                    // 忽略加载错误
        "--load-media-error-handling".to_string(),
        "ignore".to_string(),                    // 忽略媒体加载错误
    ];

    // 设置默认边距
    let default_margin = "10mm";
    args.extend([
        "--margin-top".to_string(),
        default_margin.to_string(),
        "--margin-right".to_string(),
        default_margin.to_string(),
        "--margin-bottom".to_string(),
        default_margin.to_string(),
        "--margin-left".to_string(),
        default_margin.to_string(),
    ]);

    // 设置页面大小
    if let Some(ref page_size) = options.page_size {
        args.extend(["--page-size".to_string(), page_size.clone()]);
    } else {
        args.extend(["--page-size".to_string(), "A4".to_string()]);
    }

    // 设置方向
    if let Some(ref orientation) = options.orientation {
        args.extend(["--orientation".to_string(), orientation.clone()]);
    } else {
        args.extend(["--orientation".to_string(), "Portrait".to_string()]);
    }

    // 设置自定义边距（会覆盖默认边距）
    if let Some(ref margin) = options.margin {
        let unit = margin.unit.as_deref().unwrap_or("mm");
        
        if let Some(top) = margin.top {
            args.extend(["--margin-top".to_string(), format!("{}{}", top, unit)]);
        }
        if let Some(right) = margin.right {
            args.extend(["--margin-right".to_string(), format!("{}{}", right, unit)]);
        }
        if let Some(bottom) = margin.bottom {
            args.extend(["--margin-bottom".to_string(), format!("{}{}", bottom, unit)]);
        }
        if let Some(left) = margin.left {
            args.extend(["--margin-left".to_string(), format!("{}{}", left, unit)]);
        }
    }

    // 添加输入和输出文件路径
    args.push(html_path.to_string_lossy().to_string());
    args.push(pdf_path.to_string_lossy().to_string());

    Ok(args)
}

/// 执行 wkhtmltopdf 命令
fn execute_wkhtmltopdf(args: &[String]) -> Result<(), String> {
    let output = Command::new("wkhtmltopdf")
        .args(args)
        .output()
        .map_err(|e| format!("执行 wkhtmltopdf 失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "wkhtmltopdf 转换失败 (退出码: {})\n标准错误: {}\n标准输出: {}",
            output.status.code().unwrap_or(-1),
            stderr,
            stdout
        ));
    }

    Ok(())
}

/**
 * Get printer job on windows using powershell
 */
pub fn get_jobs(printername: String) -> String {
    // let output = Command::new("powershell").args([format!("Get-PrintJob -PrinterName \"{}\"  | Select-Object DocumentName,Id,TotalPages,Position,Size,SubmmitedTime,UserName,PagesPrinted,JobTime,ComputerName,Datatype,PrinterName,Priority,SubmittedTime,JobStatus | ConvertTo-Json", printername)]).output().unwrap();
    // return output.stdout.to_string();

    let output = Command::new("powershell").args([format!("Get-PrintJob -PrinterName \"{}\"  | Select-Object DocumentName,Id,TotalPages,Position,Size,SubmmitedTime,UserName,PagesPrinted,JobTime,ComputerName,Datatype,PrinterName,Priority,SubmittedTime,JobStatus | ConvertTo-Json", printername)]).output().unwrap();
    return String::from_utf8(output.stdout).unwrap();
}

/**
 * Get printer job by id on windows using powershell
 */
pub fn get_jobs_by_id(printername: String, jobid: String) -> String {
    // let output = Command::new("powershell").args([format!("Get-PrintJob -PrinterName \"{}\" -ID \"{}\"  | Select-Object DocumentName,Id,TotalPages,Position,Size,SubmmitedTime,UserName,PagesPrinted,JobTime,ComputerName,Datatype,PrinterName,Priority,SubmittedTime,JobStatus | ConvertTo-Json", printername, jobid)]).output().unwrap();
    // return output.stdout.to_string();

    let output = Command::new("powershell").args([format!("Get-PrintJob -PrinterName \"{}\" -ID \"{}\"  | Select-Object DocumentName,Id,TotalPages,Position,Size,SubmmitedTime,UserName,PagesPrinted,JobTime,ComputerName,Datatype,PrinterName,Priority,SubmittedTime,JobStatus | ConvertTo-Json", printername, jobid)]).output().unwrap();
    return String::from_utf8(output.stdout).unwrap();
}


/**
 * Resume printers job on windows using powershell
 */
pub fn resume_job(printername: String, jobid: String) -> String {
    // let output = Command::new("powershell").args([format!("Resume-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    // return output.stdout.to_string();

    let output = Command::new("powershell").args([format!("Resume-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    return String::from_utf8(output.stdout).unwrap();
}

/**
 * Restart printers job on windows using powershell
 */
pub fn restart_job(printername: String, jobid: String) -> String {
    // let output = Command::new("powershell").args([format!("Restart-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    // return output.stdout.to_string();

    let output = Command::new("powershell").args([format!("Restart-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    return String::from_utf8(output.stdout).unwrap();
}

/**
 * pause printers job on windows using powershell
 */
pub fn pause_job(printername: String, jobid: String) -> String {
    // let output = Command::new("powershell").args([format!("Suspend-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    // return output.stdout.to_string();

    let output = Command::new("powershell").args([format!("Suspend-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    return String::from_utf8(output.stdout).unwrap();
}

/**
 * remove printers job on windows using powershell
 */
pub fn remove_job(printername: String, jobid: String) -> String {
    // let output = Command::new("powershell").args([format!("Remove-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    // return output.stdout.to_string();
    let output = Command::new("powershell").args([format!("Remove-PrintJob -PrinterName \"{}\" -ID \"{}\" ", printername, jobid)]).output().unwrap();
    return String::from_utf8(output.stdout).unwrap();
}