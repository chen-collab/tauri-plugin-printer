import { invoke } from '@tauri-apps/api/core'

export async function ping(value: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:printer|ping', {
    payload: {
      value,
    },
  }).then((r) => (r.value ? r.value : null));
}

export async function getPrinters(): Promise<string> {
  return await invoke<string>('plugin:printer|get_printers');
}

export async function getPrinterByName(printerName: string): Promise<string> {
  return await invoke<string>('plugin:printer|get_printers_by_name', {
    printername: printerName,
  });
}

export interface PrintPdfOptions {
  id: string;
  path: string;
  printer_setting: string;
  remove_after_print: boolean;
}

export async function printPdf(options: PrintPdfOptions): Promise<string> {
  return await invoke<string>('plugin:printer|print_pdf', {
    id: options.id,
    path: options.path,
    printerSetting: options.printer_setting,
    removeAfterPrint: options.remove_after_print,
  });
}
