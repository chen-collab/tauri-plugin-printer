use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::PrinterExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.printer().ping(payload)
}

#[command]
pub(crate) async fn get_printers<R: Runtime>(
    app: AppHandle<R>,
) -> Result<String> {
    app.printer().get_printers()
}

#[command]
pub(crate) async fn get_printer_by_name<R: Runtime>(
    app: AppHandle<R>,
    name: String,
) -> Result<String> {
    app.printer().get_printer_by_name(name)
}
