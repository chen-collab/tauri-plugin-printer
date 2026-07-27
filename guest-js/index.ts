import { invoke } from '@tauri-apps/api/core'

// ========== 类型定义 ==========

/** 打印机信息 */
export interface PrinterInfo {
  name: string
  driverName: string
  jobCount: number
  printProcessor: string
  portName: string
  shareName: string
  computerName: string
  printerStatus: string[]
  shared: boolean
  printerType: number
  priority: number
}

/** 打印任务信息 */
export interface JobInfo {
  documentName: string
  id: number
  totalPages: number
  position: number
  size: number
  submittedTime: string
  userName: string
  pagesPrinted: number
  jobTime: number
  computerName: string
  datatype: string
  printerName: string
  priority: number
  jobStatus: string[]
}

/** PDF 打印选项 */
export interface PrintPdfOptions {
  id: string
  path: string
  printerSetting: string
  removeAfterPrint: boolean
  /** 打印份数（默认 1） */
  copies?: number
  /** 是否灰度打印 */
  grayscale?: boolean
  /** 方向：Portrait, Landscape（可选，默认 PDF 自带尺寸） */
  orientation?: string
}

/** HTML 打印选项 */
export interface PrintHtmlOptions {
  html: string
  printerId?: string
  printSettings?: string
  removeAfterPrint?: boolean
  /** 页面大小：A4, A5, Letter 等 */
  pageSize?: string
  /** 自定义纸张宽度（mm），与 pageSize 二选一，有自定义宽高时优先 */
  pageWidth?: number
  /** 自定义纸张高度（mm） */
  pageHeight?: number
  /** 方向：Portrait, Landscape */
  orientation?: string
  margin?: PrintMargin
  quality?: number
  grayscale?: boolean
  copies?: number
}

/** 模板打印选项（三层架构：前端传数据+模板，Rust 调度，引擎渲染） */
export interface PrintTemplateOptions {
  /** hiprint 模板 JSON 字符串 */
  template: string
  /** 打印数据 JSON 字符串 */
  data: string
  /** 纸张宽度（mm） */
  paperWidth: number
  /** 纸张高度（mm），不传或为 0 表示由内容高度自适应 */
  paperHeight?: number
  /** 方向：Portrait, Landscape */
  orientation?: string
  /** 打印机 ID（名称），为空使用默认打印机 */
  printerId?: string
  /** 打印份数（默认 1） */
  copies?: number
  /** 是否灰度打印 */
  grayscale?: boolean
  /** 渲染超时（毫秒），默认 15000 */
  renderTimeoutMs?: number
}

/** 打印边距 */
export interface PrintMargin {
  top: number
  bottom: number
  left: number
  right: number
  unit: string
}

// ========== API 函数 ==========

/** 测试连接 */
export async function ping(value: string): Promise<string | null> {
  return await invoke<{ value?: string }>('plugin:printer-v2|ping', {
    payload: { value },
  }).then((r) => (r.value ? r.value : null))
}

/** 获取所有打印机 */
export async function getPrinters(): Promise<PrinterInfo[]> {
  return await invoke<PrinterInfo[]>('plugin:printer-v2|get_printers')
}

/** 按名称获取打印机 */
export async function getPrinterByName(printerName: string): Promise<PrinterInfo> {
  return await invoke<PrinterInfo>('plugin:printer-v2|get_printers_by_name', {
    printername: printerName,
  })
}

/** 打印 PDF 文件 */
export async function printPdf(options: PrintPdfOptions): Promise<string> {
  return await invoke<string>('plugin:printer-v2|print_pdf', {
    id: options.id,
    path: options.path,
    printerSetting: options.printerSetting,
    removeAfterPrint: options.removeAfterPrint,
  })
}

/** 打印 HTML 内容 */
export async function printHtml(options: PrintHtmlOptions): Promise<string> {
  return await invoke<string>('plugin:printer-v2|print_html', {
    options: options,
  })
}

/**
 * 模板打印（三层架构推荐用法）
 *
 * 前端只传 { 模板 JSON, 数据 JSON, 纸张参数, 打印机配置 }，
 * Rust 端原子操作：创建隐藏窗口 → 加载渲染引擎 → hiprint 渲染 + 图片等待 + 高度计算 → WebView2 静默打印 → 销毁窗口。
 *
 * 前端无需引入 hiprint，无需关心渲染、图片加载、纸张高度计算等底层细节。
 */
export async function printTemplate(options: PrintTemplateOptions): Promise<string> {
  return await invoke<string>('plugin:printer-v2|print_template', {
    options: options,
  })
}

/** 获取打印任务列表 */
export async function getJobs(printerName: string): Promise<JobInfo[]> {
  return await invoke<JobInfo[]>('plugin:printer-v2|get_jobs', {
    printername: printerName,
  })
}

/** 按 ID 获取打印任务 */
export async function getJobById(printerName: string, jobId: string): Promise<JobInfo> {
  return await invoke<JobInfo>('plugin:printer-v2|get_jobs_by_id', {
    printername: printerName,
    jobid: jobId,
  })
}

/** 恢复打印任务 */
export async function resumeJob(printerName: string, jobId: string): Promise<void> {
  return await invoke<void>('plugin:printer-v2|resume_job', {
    printername: printerName,
    jobid: jobId,
  })
}

/** 重启打印任务 */
export async function restartJob(printerName: string, jobId: string): Promise<void> {
  return await invoke<void>('plugin:printer-v2|restart_job', {
    printername: printerName,
    jobid: jobId,
  })
}

/** 暂停打印任务 */
export async function pauseJob(printerName: string, jobId: string): Promise<void> {
  return await invoke<void>('plugin:printer-v2|pause_job', {
    printername: printerName,
    jobid: jobId,
  })
}

/** 删除打印任务 */
export async function removeJob(printerName: string, jobId: string): Promise<void> {
  return await invoke<void>('plugin:printer-v2|remove_job', {
    printername: printerName,
    jobid: jobId,
  })
}

/** 创建临时文件 */
export async function createTempFile(base64Data: string, filename: string): Promise<string> {
  return await invoke<string>('plugin:printer-v2|create_temp_file', {
    bufferData: base64Data,
    filename: filename,
  })
}

/** 删除临时文件 */
export async function removeTempFile(filename: string): Promise<boolean> {
  return await invoke<boolean>('plugin:printer-v2|remove_temp_file', {
    filename: filename,
  })
}