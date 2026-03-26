use std::collections::HashMap;
use std::env;
use std::process::Command;

use serde_json::{json, Value};

use crate::declare::{PrintHtmlOptions, PrintOptions};
use crate::fsys::remove_file;

/// List all printers using CUPS `lpstat`.
/// Returns a JSON array compatible with the Windows Get-Printer format.
pub fn get_printers() -> String {
    let printers_out = Command::new("lpstat").args(["-p"]).output().ok();
    let default_out = Command::new("lpstat").args(["-d"]).output().ok();
    let devices_out = Command::new("lpstat").args(["-v"]).output().ok();

    let printers_bytes = printers_out.map(|o| o.stdout).unwrap_or_default();
    let default_bytes = default_out.map(|o| o.stdout).unwrap_or_default();
    let devices_bytes = devices_out.map(|o| o.stdout).unwrap_or_default();

    let printers_str = String::from_utf8_lossy(&printers_bytes);
    let default_str = String::from_utf8_lossy(&default_bytes);
    let devices_str = String::from_utf8_lossy(&devices_bytes);

    // "system default destination: <name>"
    let default_printer = default_str
        .trim()
        .strip_prefix("system default destination: ")
        .unwrap_or("")
        .to_string();

    // Build name → device-URI map from "device for <name>: <uri>"
    let device_map: HashMap<String, String> = devices_str
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("device for ")?;
            let colon = rest.find(": ")?;
            Some((rest[..colon].to_string(), rest[colon + 2..].to_string()))
        })
        .collect();

    // Parse "printer <NAME> is <STATUS>..." lines
    let printers: Vec<Value> = printers_str
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("printer ")?;
            let sep = rest.find(" is ")?;
            let name = &rest[..sep];
            let status_str = &rest[sep + 4..];
            let status = if status_str.starts_with("idle") {
                "Idle"
            } else if status_str.starts_with("processing") {
                "Printing"
            } else if status_str.starts_with("disabled") || status_str.starts_with("stopped") {
                "Paused"
            } else {
                "Unknown"
            };

            Some(json!({
                "Name": name,
                "DriverName": null,
                "JobCount": 0,
                "PrintProcessor": null,
                "PortName": device_map.get(name).cloned().unwrap_or_default(),
                "ShareName": null,
                "ComputerName": null,
                "PrinterStatus": status,
                "Shared": false,
                "Type": 0,
                "Priority": 1,
                "IsDefault": name == default_printer,
            }))
        })
        .collect();

    serde_json::to_string(&printers).unwrap_or_else(|_| "[]".to_string())
}

/// Return info for a single printer by name.
pub fn get_printers_by_name(name: String) -> String {
    let all: Vec<Value> = serde_json::from_str(&get_printers()).unwrap_or_default();
    let matched: Vec<&Value> = all
        .iter()
        .filter(|p| p["Name"].as_str().unwrap_or("") == name)
        .collect();
    serde_json::to_string(&matched).unwrap_or_else(|_| "[]".to_string())
}

/// Print a PDF file using CUPS `lp`.
/// `options.print_setting` holds the destination printer name (empty = default).
pub fn print_pdf(options: PrintOptions) -> String {
    let mut args: Vec<String> = Vec::new();

    if !options.print_setting.is_empty() {
        args.push("-d".to_string());
        args.push(options.print_setting.clone());
    }

    args.push(options.path.clone());

    let result = match Command::new("lp").args(&args).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Ok(out) => format!("lp error: {}", String::from_utf8_lossy(&out.stderr).trim()),
        Err(e) => format!("Failed to run lp: {}", e),
    };

    if options.remove_after_print {
        let _ = remove_file(&options.path);
    }

    result
}

/// Print HTML by converting to PDF with `wkhtmltopdf`, then sending to `lp`.
pub fn print_html(options: PrintHtmlOptions) -> String {
    match print_html_internal(options) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("HTML print failed: {}", e);
            format!("Print failed: {}", e)
        }
    }
}

fn print_html_internal(options: PrintHtmlOptions) -> Result<String, String> {
    if options.html.trim().is_empty() {
        return Err("HTML content cannot be empty".to_string());
    }

    // Verify wkhtmltopdf is available
    Command::new("wkhtmltopdf")
        .arg("--version")
        .output()
        .map_err(|_| {
            "wkhtmltopdf is not installed. Install it with: sudo apt install wkhtmltopdf"
                .to_string()
        })?;

    let temp_dir = env::temp_dir();
    let id = format!(
        "{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    let html_path = temp_dir.join(format!("tauri_printer_{}.html", id));
    let pdf_path = temp_dir.join(format!("tauri_printer_{}.pdf", id));

    std::fs::write(&html_path, &options.html)
        .map_err(|e| format!("Failed to write temp HTML: {}", e))?;

    let mut wk_args: Vec<String> = vec![
        "--encoding".to_string(),
        "UTF-8".to_string(),
        "--enable-local-file-access".to_string(),
        "--print-media-type".to_string(),
        "--quiet".to_string(),
        "--page-size".to_string(),
        options.page_size.as_deref().unwrap_or("A4").to_string(),
    ];

    if let Some(ref orientation) = options.orientation {
        wk_args.extend(["--orientation".to_string(), orientation.clone()]);
    }

    if let Some(ref margin) = options.margin {
        let unit = margin.unit.as_deref().unwrap_or("mm");
        for (flag, val) in [
            ("--margin-top", margin.top),
            ("--margin-right", margin.right),
            ("--margin-bottom", margin.bottom),
            ("--margin-left", margin.left),
        ] {
            if let Some(v) = val {
                wk_args.extend([flag.to_string(), format!("{}{}", v, unit)]);
            }
        }
    }

    if options.grayscale == Some(true) {
        wk_args.push("--grayscale".to_string());
    }

    wk_args.push(html_path.to_string_lossy().to_string());
    wk_args.push(pdf_path.to_string_lossy().to_string());

    let wk_out = Command::new("wkhtmltopdf")
        .args(&wk_args)
        .output()
        .map_err(|e| format!("Failed to run wkhtmltopdf: {}", e))?;

    let _ = std::fs::remove_file(&html_path);

    if !wk_out.status.success() {
        return Err(format!(
            "wkhtmltopdf failed: {}",
            String::from_utf8_lossy(&wk_out.stderr).trim()
        ));
    }

    if !pdf_path.exists() {
        return Err("PDF generation failed".to_string());
    }

    Ok(print_pdf(PrintOptions {
        id: "html_print".to_string(),
        path: pdf_path.to_string_lossy().to_string(),
        print_setting: options.printer_id.unwrap_or_default(),
        remove_after_print: options.remove_after_print.unwrap_or(true),
    }))
}

/// List print jobs for a printer using `lpq`.
pub fn get_jobs(printername: String) -> String {
    let stdout = Command::new("lpq")
        .args(["-P", &printername])
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();

    parse_lpq_output(&String::from_utf8_lossy(&stdout), &printername)
}

/// Return a specific job by its ID.
pub fn get_jobs_by_id(printername: String, jobid: String) -> String {
    let all: Vec<Value> = serde_json::from_str(&get_jobs(printername)).unwrap_or_default();
    let matched: Vec<&Value> = all
        .iter()
        .filter(|j| j["Id"].as_str().unwrap_or("") == jobid)
        .collect();
    serde_json::to_string(&matched).unwrap_or_else(|_| "[]".to_string())
}

/// Release (resume) a held job: `lp -i <printer>-<id> -H release`
pub fn resume_job(printername: String, jobid: String) -> String {
    run_lp_job_command(&printername, &jobid, "release")
}

/// Restart a completed/failed job: `lp -i <printer>-<id> -H restart`
pub fn restart_job(printername: String, jobid: String) -> String {
    run_lp_job_command(&printername, &jobid, "restart")
}

/// Hold (pause) a job: `lp -i <printer>-<id> -H hold`
pub fn pause_job(printername: String, jobid: String) -> String {
    run_lp_job_command(&printername, &jobid, "hold")
}

/// Cancel a job: `cancel <printer>-<id>`
pub fn remove_job(printername: String, jobid: String) -> String {
    let job_ref = format!("{}-{}", printername, jobid);
    match Command::new("cancel").args([&job_ref]).output() {
        Ok(out) if out.status.success() => "Job cancelled".to_string(),
        Ok(out) => String::from_utf8_lossy(&out.stderr).trim().to_string(),
        Err(e) => format!("Failed to cancel job: {}", e),
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn run_lp_job_command(printername: &str, jobid: &str, action: &str) -> String {
    let job_ref = format!("{}-{}", printername, jobid);
    match Command::new("lp")
        .args(["-i", &job_ref, "-H", action])
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Ok(out) => String::from_utf8_lossy(&out.stderr).trim().to_string(),
        Err(e) => format!("Failed to run lp: {}", e),
    }
}

/// Parse `lpq -P <printer>` output into a JSON array.
///
/// Example lpq output:
/// ```text
/// HP_LaserJet is ready
/// Rank    Owner   Job     File(s)              Total Size
/// 1st     alice   42      report.pdf           102400 bytes
/// ```
fn parse_lpq_output(output: &str, printername: &str) -> String {
    let jobs: Vec<Value> = output
        .lines()
        .skip(2) // skip status line and header
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 4 {
                return None;
            }
            let (rank, owner, job_id, doc_name) = (cols[0], cols[1], cols[2], cols[3]);
            let size = cols.get(4).copied().unwrap_or("0");
            let status = if rank == "active" { "Printing" } else { "Queued" };

            Some(json!({
                "DocumentName": doc_name,
                "Id": job_id,
                "TotalPages": null,
                "Position": rank,
                "Size": size,
                "UserName": owner,
                "PrinterName": printername,
                "JobStatus": status,
                "ComputerName": null,
                "Priority": 1,
            }))
        })
        .collect();

    serde_json::to_string(&jobs).unwrap_or_else(|_| "[]".to_string())
}
