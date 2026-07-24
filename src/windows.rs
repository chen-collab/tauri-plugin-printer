use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::declare::{JobInfo, PrintHtmlOptions, PrintOptions, PrinterInfo};
use crate::fsys::remove_file;
use crate::Error;

/// 将 sm.exe 释放到目标目录
fn create_file(target_dir: &Path, bin: &[u8]) -> std::io::Result<PathBuf> {
    let exe_path = target_dir.join("sm.exe");
    let mut f = File::create(&exe_path)?;
    f.write_all(bin)?;
    f.sync_all()?;
    Ok(exe_path)
}

/// 初始化 sm.exe，返回其路径
pub fn init_windows(target_dir: &Path) -> Result<PathBuf, Error> {
    std::fs::create_dir_all(target_dir)?;
    let sm = include_bytes!("bin/sm");
    create_file(target_dir, sm).map_err(|e| Error::Io(e))
}

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

/// 打印 PDF 文件（直接调用 sm.exe，不经 PowerShell）
pub fn print_pdf(options: PrintOptions, sm_exe_path: &Path) -> Result<String, Error> {
    let printer = options.printer_setting.trim();
    let mut cmd = Command::new(sm_exe_path);
    // 打印机名称为空或 "default" 时使用默认打印机
    if printer.is_empty() || printer.eq_ignore_ascii_case("default") {
        cmd.arg("-print-to-default");
    } else {
        cmd.args(["-print-to", printer]);
    }
    cmd.arg("-silent").arg(&options.path);

    let output = cmd
        .output()
        .map_err(|e| Error::WindowsApi(format!("执行 sm.exe 失败: {}", e)))?;

    let result = String::from_utf8_lossy(&output.stdout).to_string();

    if options.remove_after_print {
        let _ = remove_file(&options.path);
    }

    Ok(result)
}

/// 打印 HTML 内容
pub fn print_html(options: PrintHtmlOptions, sm_exe_path: &Path) -> Result<String, Error> {
    print_html_internal(options, sm_exe_path)
        .map_err(|e| Error::WindowsApi(format!("HTML 打印失败: {}", e)))
}

/// 生成唯一的临时文件路径
fn generate_temp_file_path(extension: &str) -> Result<PathBuf, String> {
    let temp_dir = env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("获取时间戳失败: {}", e))?
        .as_nanos();
    let filename = format!(
        "tauri_printer_{}_{}.{}",
        std::process::id(),
        timestamp,
        extension
    );
    Ok(temp_dir.join(filename))
}

/// 内部 HTML 打印实现（使用 Edge headless 替代 wkhtmltopdf）
fn print_html_internal(options: PrintHtmlOptions, sm_exe_path: &Path) -> Result<String, String> {
    if options.html.trim().is_empty() {
        return Err("HTML 内容不能为空".to_string());
    }

    let edge_path = find_edge_path()?;
    let page_css = build_page_css(&options);
    let html_with_css = inject_page_css(&options.html, &page_css);

    let html_path = generate_temp_file_path("html")?;
    let pdf_path = generate_temp_file_path("pdf")?;

    std::fs::write(&html_path, &html_with_css)
        .map_err(|e| format!("写入 HTML 内容失败: {}", e))?;

    // Edge 需要独立的用户数据目录，否则多个 headless 实例会冲突（退出码 13）
    let profile_dir = generate_temp_file_path("edge-profile")?;
    std::fs::create_dir_all(&profile_dir)
        .map_err(|e| format!("创建 Edge 临时目录失败: {}", e))?;

    // 调用 Edge headless 将 HTML 转为 PDF
    // 使用 --flag=value 单参格式，避免 Windows 参数转义问题
    let edge_output = match Command::new(&edge_path)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--no-pdf-header-footer")
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg(format!("--print-to-pdf={}", pdf_path.display()))
        .arg(format!("file:///{}", html_path.display()))
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            let _ = remove_file(&html_path.to_string_lossy());
            let _ = remove_file(&pdf_path.to_string_lossy());
            let _ = std::fs::remove_dir_all(&profile_dir);
            return Err(format!("执行 Edge 失败: {}", e));
        }
    };

    let _ = remove_file(&html_path.to_string_lossy());

    if !edge_output.status.success() {
        let stderr = String::from_utf8_lossy(&edge_output.stderr);
        let _ = remove_file(&pdf_path.to_string_lossy());
        let _ = std::fs::remove_dir_all(&profile_dir);
        return Err(format!(
            "Edge PDF 转换失败 (退出码: {}): {}",
            edge_output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    if !pdf_path.exists() {
        let _ = remove_file(&pdf_path.to_string_lossy());
        let _ = std::fs::remove_dir_all(&profile_dir);
        return Err("PDF 文件未生成".to_string());
    }

    let printer_id = options.printer_id.unwrap_or_default();

    let print_options = PrintOptions {
        path: pdf_path.to_string_lossy().to_string(),
        id: printer_id.clone(),
        printer_setting: printer_id,
        remove_after_print: options.remove_after_print.unwrap_or(true),
    };

    let result = print_pdf(print_options, sm_exe_path).map_err(|e| format!("{}", e))?;

    // 清理临时目录
    let _ = std::fs::remove_dir_all(&profile_dir);

    Ok(result)
}

/// 查找 Edge 浏览器路径（注册表 → 默认路径回退）
fn find_edge_path() -> Result<PathBuf, String> {
    // 1. 尝试从注册表读取
    let reg_output = Command::new("reg")
        .args([
            "query",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\msedge.exe",
            "/ve",
        ])
        .output();
    if let Ok(output) = reg_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().find(|l| l.contains("msedge.exe")) {
                let path = line
                    .trim()
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .trim();
                if !path.is_empty() && Path::new(path).exists() {
                    return Ok(PathBuf::from(path));
                }
            }
        }
    }

    // 2. 回退到默认安装路径
    let candidates = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for candidate in &candidates {
        let p = Path::new(candidate);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }

    Err("未找到 Microsoft Edge。Windows 10+ 自带 Edge，请检查系统环境。".to_string())
}

/// 根据打印选项构建 @page CSS 规则
fn build_page_css(options: &PrintHtmlOptions) -> String {
    let size = if let (Some(w), Some(h)) = (options.page_width, options.page_height) {
        // 自定义纸张尺寸（mm）
        format!("{}mm {}mm", w, h)
    } else if let Some(ref page_size) = options.page_size {
        // 标准纸张名
        page_size.clone()
    } else {
        // 默认 A4
        "A4".to_string()
    };

    let orientation = if let Some(ref o) = options.orientation {
        if o.eq_ignore_ascii_case("landscape") {
            " landscape"
        } else {
            ""
        }
    } else {
        ""
    };

    let margin = if let Some(ref m) = options.margin {
        let unit = m.unit.as_deref().unwrap_or("mm");
        let top = m.top.map_or("0".to_string(), |v| format!("{}{}", v, unit));
        let right = m
            .right
            .map_or("0".to_string(), |v| format!("{}{}", v, unit));
        let bottom = m
            .bottom
            .map_or("0".to_string(), |v| format!("{}{}", v, unit));
        let left = m
            .left
            .map_or("0".to_string(), |v| format!("{}{}", v, unit));
        format!("{} {} {} {}", top, right, bottom, left)
    } else {
        "0".to_string() // hiprint 模板自控留白，默认零边距
    };

    format!(
        "@page {{ size: {}{}; margin: {}; }}",
        size, orientation, margin
    )
}

/// 将 @page CSS 注入到 HTML 头部
fn inject_page_css(html: &str, css: &str) -> String {
    let style_tag = format!("<style>{}</style>", css);
    if let Some(head_end) = html.find("</head>") {
        // 插入到 </head> 之前
        let mut result = String::with_capacity(html.len() + style_tag.len());
        result.push_str(&html[..head_end]);
        result.push_str(&style_tag);
        result.push_str(&html[head_end..]);
        result
    } else if let Some(head_start) = html.find("<head>") {
        // 插入到 <head> 之后
        let insert_pos = head_start + "<head>".len();
        let mut result = String::with_capacity(html.len() + style_tag.len());
        result.push_str(&html[..insert_pos]);
        result.push_str(&style_tag);
        result.push_str(&html[insert_pos..]);
        result
    } else {
        // 没有 <head> 标签，在开头拼接
        format!("{}{}", style_tag, html)
    }
}