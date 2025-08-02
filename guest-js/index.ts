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
  console.log('打印配置pdf:', options);
  return await invoke<string>('plugin:printer|print_pdf', {
    id: options.id,
    path: options.path,
    printer_setting: options.printer_setting,
    remove_after_print: options.remove_after_print,
  });
}

export interface PrintMargin {
  top: number;
  bottom: number;
  left: number;
  right: number;
  unit: string;
}

export interface PrintHtmlOptions {
  html: string;
  printer_id?: string;
  print_settings?: string;
  remove_after_print?: boolean;
  page_size?: string;
  orientation?: string;
  margin?: PrintMargin;
  quality?: number;
  grayscale?: boolean;
  copies?: number;
}

export async function printHtml(options: PrintHtmlOptions): Promise<string> {
  return await invoke<string>('plugin:printer|print_html', {
    html: options.html,
    printer_id: options.printer_id,
    print_settings: options.print_settings,
    remove_after_print: options.remove_after_print,
    page_size: options.page_size,
    orientation: options.orientation,
    margin: options.margin,
    quality: options.quality,
    grayscale: options.grayscale,
    copies: options.copies,
  });
}
