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
  printer: string;
  print_settings: string;
  remove_after_print: boolean;
}

export async function printPdf(options: PrintPdfOptions): Promise<string> {
  console.log('打印配置pdf:', options);
  return await invoke<string>('plugin:printer|print_pdf', {
    id: options.id,
    path: options.path,
    printer: options.printer,
    print_settings: options.print_settings,
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
  id: string;
  html: string;
  printer: string;
  print_settings?: string;
  remove_after_print?: boolean;
  page_size?: string;
  orientation?: string;
  margin?: PrintMargin;
  quality?: number;
  grayscale?: boolean;
  copies?: number;
  page_width?: number;  // 自定义页面宽度 (mm)
  page_height?: number; // 自定义页面高度 (mm)
}

export async function printHtml(options: PrintHtmlOptions): Promise<string> {
  console.log('打印配置html:', options);
  return await invoke<string>('plugin:printer|print_html', {
    options: options
  });
}

export interface PrintPdfUrlOptions {
  id: string;
  url: string;
  printer: string;
  print_settings: string;
  remove_after_print?: boolean;
  timeout_seconds?: number;
  temp_dir?: string;
}

export async function printPdfFromUrl(options: PrintPdfUrlOptions): Promise<string> {
  console.log('打印配置pdf from url:', options);
  return await invoke<string>('plugin:printer|print_pdf_from_url', {
    id: options.id,
    url: options.url,
    printer: options.printer,
    print_settings: options.print_settings,
    remove_after_print: options.remove_after_print,
    timeout_seconds: options.timeout_seconds,
    temp_dir: options.temp_dir,
  });
}
