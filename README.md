<div align="center">

# Tauri Plugin Printer

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-printer-v2.svg)](https://crates.io/crates/tauri-plugin-printer-v2)
[![NPM](https://img.shields.io/npm/v/tauri-plugin-printer-v2.svg)](https://www.npmjs.com/package/tauri-plugin-printer-v2)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2.11-orange.svg)](https://tauri.app/)

**Tauri V2 打印机插件 — WebView2 COM 静默打印**

基于 WebView2 COM `ICoreWebView2_16::Print` 实现真正的静默打印，无需中间转 PDF，无需额外打印引擎。支持 PDF/HTML、标准纸张 + 自定义尺寸（热敏小票/标签），完美适配 vue-plugin-hiprint。

[安装](#-安装) • [快速开始](#-快速开始) • [API 文档](#-api-文档) • [示例](#-示例) • [架构](#-架构)

</div>

---

## ✨ 特性

- 🖨️ **获取系统打印机列表** — 基于原生 `EnumPrintersW` API，返回强类型 `PrinterInfo[]`
- 🌐 **HTML 静默打印** — 隐藏 WebView 窗口 + `ICoreWebView2_16::Print` COM 接口，支持自定义纸张尺寸（mm），完美适配 vue-plugin-hiprint
- 📄 **PDF 静默打印** — WebView2 内置 PDFium 引擎，加载 PDF 后直接静默打印，无需 SumatraPDF 等外部依赖
- 📋 **打印任务管理** — 查询/暂停/恢复/重启/删除打印任务，基于 `EnumJobsW`/`SetJobW` API
- 🔍 **按名称查询打印机** — 获取单个打印机详细信息
- 📁 **临时文件管理** — 创建/删除临时文件，含路径遍历防护
- 🎯 **强类型 API** — 前端返回 camelCase 类型化对象，IDE 友好
- 🚀 **零外部依赖** — 完全基于 Windows 内置 WebView2 Runtime（Win10+ 自带），无需打包额外二进制

## 📦 安装

```bash
# Rust 依赖
cargo add tauri-plugin-printer-v2

# 前端 API
npm install tauri-plugin-printer-v2
```

### 注册插件

```rust
// src-tauri/src/lib.rs
use tauri_plugin_printer_v2::init;

pub fn run() {
    tauri::Builder::default()
        .plugin(init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 配置权限

```json
// src-tauri/capabilities/default.json
{
  "permissions": ["printer-v2:default"]
}
```

### ⚠️ WebView2 Runtime 版本要求

HTML/PDF 打印依赖 `ICoreWebView2_16::Print` API，需 WebView2 Runtime **>= 1.0.1518.46**。Windows 10/11 自动更新的 WebView2 通常已满足。若版本过低，会返回 `WebView2 Runtime 版本过低` 错误。

## 🚀 快速开始

```typescript
import { getPrinters, printPdf, printHtml } from 'tauri-plugin-printer-v2';

// 获取打印机列表（强类型数组）
const printers = await getPrinters();
console.log('可用打印机:', printers.map(p => p.name));

// 打印 PDF
await printPdf({
  id: 'print-001',
  path: 'C:/docs/report.pdf',
  printerSetting: 'Microsoft Print to PDF',
  removeAfterPrint: false,
  copies: 2,
  grayscale: true
});

// 打印 HTML
await printHtml({
  html: '<h1>Hello World</h1>',
  pageSize: 'A4',
  orientation: 'Portrait',
  margin: { top: 10, bottom: 10, left: 10, right: 10, unit: 'mm' },
  copies: 1,
  grayscale: false
});
```

## 📚 API 文档

### 类型定义

```typescript
interface PrinterInfo {
  name: string
  driverName: string
  jobCount: number
  printProcessor: string
  portName: string
  shareName: string
  computerName: string
  printerStatus: string[]  // 如 ["normal"] / ["paused","error"]
  shared: boolean
  printerType: number
  priority: number
}

interface JobInfo {
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
  jobStatus: string[]  // 如 ["printing"] / ["paused"]
}

interface PrintPdfOptions {
  id: string
  path: string
  printerSetting: string     // 打印机名称，空字符串 = 默认打印机
  removeAfterPrint: boolean
  copies?: number            // 打印份数（默认 1）
  grayscale?: boolean        // 是否灰度打印
  orientation?: string       // 方向：Portrait / Landscape（可选，默认 PDF 自带尺寸）
}

interface PrintHtmlOptions {
  html: string
  printerId?: string
  printSettings?: string
  removeAfterPrint?: boolean
  pageSize?: string           // A4, A5, Letter 等（标准纸张）
  pageWidth?: number          // 自定义纸张宽度（mm），与 pageSize 二选一，有自定义宽高时优先
  pageHeight?: number         // 自定义纸张高度（mm）
  orientation?: string        // Portrait, Landscape
  margin?: PrintMargin
  quality?: number            // 保留字段，暂未映射（WebView2 无对应参数）
  grayscale?: boolean         // 是否灰度打印
  copies?: number             // 打印份数
}

interface PrintMargin {
  top: number
  bottom: number
  left: number
  right: number
  unit: string               // mm, cm, inch
}
```

### 命令列表

| 函数 | 返回类型 | 说明 |
|---|---|---|
| `ping(value)` | `Promise<string \| null>` | 测试插件连接 |
| `getPrinters()` | `Promise<PrinterInfo[]>` | 获取所有打印机 |
| `getPrinterByName(name)` | `Promise<PrinterInfo>` | 按名称查询打印机 |
| `printPdf(options)` | `Promise<string>` | 打印 PDF 文件（WebView2 静默打印） |
| `printHtml(options)` | `Promise<string>` | 打印 HTML 内容（WebView2 静默打印） |
| `getJobs(printerName)` | `Promise<JobInfo[]>` | 获取打印任务列表 |
| `getJobById(printer, jobId)` | `Promise<JobInfo>` | 按 ID 获取任务 |
| `resumeJob(printer, jobId)` | `Promise<void>` | 恢复任务 |
| `restartJob(printer, jobId)` | `Promise<void>` | 重启任务 |
| `pauseJob(printer, jobId)` | `Promise<void>` | 暂停任务 |
| `removeJob(printer, jobId)` | `Promise<void>` | 删除任务 |
| `createTempFile(base64, name)` | `Promise<string>` | 创建临时文件 |
| `removeTempFile(name)` | `Promise<boolean>` | 删除临时文件 |

## 🏗️ 架构

### 核心流程

```
print_html / print_pdf （async command，tokio runtime）
    │
    ├─ 创建隐藏 WebviewWindow（visible=false, skip_taskbar）
    ├─ 导航到 file:// URL（HTML 临时文件 或 PDF 文件）
    │
    ├─ 第 1 次 with_webview（主线程，注册 NavigationCompleted + 触发导航）
    │   └─ NavigationCompletedEventHandler::create(FnMut) → mpsc channel
    │
    ├─ tokio::task::spawn_blocking 等待 mpsc::recv_timeout（导航完成）
    │
    ├─ 第 2 次 with_webview（主线程，构建 settings + 调 Print）
    │   ├─ environment.cast::<ICoreWebView2Environment6>()
    │   ├─ env6.CreatePrintSettings() → cast PrintSettings2
    │   ├─ SetPageWidth/Height/Margin/Orientation/Copies/ColorMode/PrinterName/...
    │   └─ PrintCompletedHandler::create(FnOnce) → ICoreWebView2_16::Print
    │
    ├─ tokio::task::spawn_blocking 等待 mpsc::recv_timeout（打印完成）
    │
    └─ 清理：webview.close() + 删临时文件
```

### 设计要点

- **非阻塞架构**：`with_webview` 闭包仅做"注册 + 触发"立即返回，结果通过 `mpsc::channel` 异步回传，主线程不阻塞
- **隐藏窗口**：每次打印创建独立隐藏窗口（`visible=false`, `skip_taskbar=true`），打印完毕销毁，天然隔离无并发冲突
- **单位换算**：前端传毫米（mm），Rust 端 `mm_to_inch = mm / 25.4` 转英寸后传给 WebView2 Print API（WebView2 仅接受英寸）
- **渲染就绪**：HTML 用 `NavigationCompleted` 事件检测；PDF 用固定延时（PDFium viewer 的 NavigationCompleted 行为不稳定）
- **零外部依赖**：完全基于 Windows 10+ 自带的 WebView2 Runtime，无需 SumatraPDF / Edge headless / wkhtmltopdf 等额外组件

### 模块结构

```
src/
├── lib.rs             # 插件入口，命令注册
├── print_service.rs   # 打印服务：print_html/print_pdf、隐藏窗口管理、流程编排
├── webview2_print.rs  # WebView2 COM 封装：PrintSettings 构建、Print 调用、结果映射、mm_to_inch
├── spooler.rs         # Windows Spooler API 封装（EnumPrintersW/EnumJobsW/SetJobW 等）
├── windows.rs         # Spooler 转发（打印机列表、任务管理）
├── desktop.rs         # 桌面端 Printer 实例
├── declare.rs         # 数据结构定义
├── error.rs           # 错误类型
├── fsys.rs            # 文件操作（路径遍历防护）
└── models.rs          # Ping 请求/响应模型
```

## 📋 示例

### 医疗打印（vue-plugin-hiprint + WebView2 静默打印）

```typescript
import { printHtml } from 'tauri-plugin-printer-v2';
import { HiprintTemplate } from 'vue-plugin-hiprint';

// hiprint 模板生成 HTML（图片需转 base64 内嵌，避免网络加载失败）
const template = new HiprintTemplate({ template: jsonData });
const html = template.getHtml(printData);

// 58mm 热敏小票打印
await printHtml({
  html: html,
  pageWidth: 58,           // 小票宽度（mm）
  pageHeight: 120,         // 小票高度（mm），可根据内容动态计算
  printerId: '热敏打印机',
  margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' },
  copies: 1,
  removeAfterPrint: true
});

// 标准 A4 处方打印
await printHtml({
  html: prescriptionHtml,
  pageSize: 'A4',
  orientation: 'Portrait',
  printerId: '处方打印机',
  margin: { top: 10, bottom: 10, left: 10, right: 10, unit: 'mm' },
  copies: 2,
  grayscale: false,
  removeAfterPrint: true
});
```

> **注意**：HTML 中的图片必须转为 Base64 编码内嵌，隐藏窗口无法加载需要鉴权的网络图片。

### 打印 PDF 报告

```typescript
import { printPdf } from 'tauri-plugin-printer-v2';

await printPdf({
  id: 'report-001',
  path: 'C:/reports/medical-report.pdf',
  printerSetting: 'HP LaserJet',
  copies: 3,
  grayscale: true,
  removeAfterPrint: false
});
```

### 管理打印任务

```typescript
import { getJobs, pauseJob, resumeJob, removeJob } from 'tauri-plugin-printer-v2';

// 获取所有任务
const jobs = await getJobs('Microsoft Print to PDF');
console.log(`${jobs.length} 个待处理任务`);

// 暂停第一个任务
if (jobs.length > 0) {
  await pauseJob('Microsoft Print to PDF', String(jobs[0].id));
}
```

### 打印机状态监控

```typescript
import { getPrinters } from 'tauri-plugin-printer-v2';

async function monitorPrinters() {
  const printers = await getPrinters();
  for (const p of printers) {
    const status = p.printerStatus.join(', ');
    console.log(`${p.name}: ${status} (${p.jobCount} 个任务)`);
  }
}

setInterval(monitorPrinters, 30000);
```

## 🛠️ 开发

```bash
git clone https://github.com/chen-collab/tauri-plugin-printer.git
cd tauri-plugin-printer

# 构建前端 API
npm install && npm run build

# 运行示例应用
cd examples/tauri-app
npm install
npm run tauri:dev

# Rust 测试 / 格式化 / Lint
cargo test
cargo fmt
cargo clippy
```

## 🔧 权限配置

```toml
# permissions/default.toml
[default]
description = "Default permissions for the plugin"
permissions = [
  "allow-ping",
  "allow-create-temp-file",
  "allow-remove-temp-file",
  "allow-get-printers",
  "allow-get-printers-by-name",
  "allow-print-html",
  "allow-print-pdf",
  "allow-get-jobs",
  "allow-get-jobs-by-id",
  "allow-resume-job",
  "allow-restart-job",
  "allow-pause-job",
  "allow-remove-job"
]
```

## 🔒 安全设计

| 措施 | 说明 |
|---|---|
| **WebView2 COM 直接打印** | 不经过 Edge headless 命令行 / SumatraPDF 子进程，无命令注入风险 |
| **原生 Spooler API** | 打印机管理改用 Windows API（`EnumPrintersW`/`OpenPrinterW`/`EnumJobsW`/`SetJobW`），零 Shell 调用 |
| **路径遍历防护** | `createTempFile`/`removeTempFile` 对文件名做 `sanitize_filename` 校验，拒绝 `..`、绝对路径和路径分隔符 |
| **无外部二进制** | 完全基于系统内置 WebView2 Runtime，插件不嵌入也不依赖任何外部可执行文件 |
| **隐藏窗口隔离** | 每次打印独立隐藏窗口，打印完毕立即销毁，无状态泄漏 |
| **无硬编码密钥** | 所有敏感信息均从环境变量或系统 API 获取 |

## 🐛 已知问题

- 目前仅完整支持 **Windows** 平台（非 Windows 返回 `UnsupportedPlatform` 错误）
- 打印功能依赖 **WebView2 Runtime >= 1.0.1518.46**（Windows 10 1809+ / Win11 自带，通常已自动更新）
- PDF 打印用固定延时等待渲染完成（1.5s），超大 PDF 可能需要调整
- 某些热敏打印机驱动对自定义纸张尺寸的支持程度不同，建议实测

## 🔧 故障排除

### 无法获取打印机列表
- 确保 Print Spooler 服务正在运行（`services.msc` → Print Spooler）
- 检查应用权限配置是否包含 `printer:allow-get-printers`

### 打印失败提示"WebView2 Runtime 版本过低"
- 系统 WebView2 Runtime 版本过旧，需升级到 >= 1.0.1518.46
- 下载地址：https://developer.microsoft.com/microsoft-edge/webview2/

### HTML 打印内容不对 / 样式错乱
- 确保 HTML 是完整文档（含 `<html><head><body>`）
- 图片必须用 Base64 内嵌（隐藏窗口无法加载需要鉴权的网络图片）
- 自定义尺寸时同时设置 `pageWidth` 和 `pageHeight`（单位 mm），并将 `margin` 设为 0

### 热敏小票打印尺寸不对
- 确认打印机驱动支持自定义纸张尺寸
- `pageWidth`/`pageHeight` 单位为毫米（mm），需与打印机实际纸张一致
- 建议设置 `margin` 全为 0，由 HTML/CSS 内部控制留白

## 📝 更新日志

### v0.5.0 (BREAKING)
- 🏗️ **架构重构**：HTML 打印从 "Edge headless + SumatraPDF" 改为 **WebView2 COM `ICoreWebView2_16::Print` 直接静默打印**
- 🚀 **PDF 打印重构**：移除 SumatraPDF 依赖，改用 WebView2 内置 PDFium 引擎
- 📦 **零外部依赖**：完全基于 Windows 内置 WebView2 Runtime，不再需要打包 `sm` 二进制
- ⚡ **性能提升**：省去 PDF 中转步骤，HTML → 打印一步到位
- 🧩 **架构清晰**：新增 `print_service.rs` + `webview2_print.rs` 两文件分层（流程编排 / COM 封装）
- 🎯 **强类型**：`print_pdf` 新增 `copies` / `grayscale` / `orientation` 可选参数
- 📚 **文档更新**：README 全面更新为 WebView2 架构

### v0.4.0 (BREAKING)
- 🏗️ **架构重构**：`sm` 不再嵌入插件二进制，改为主应用通过 `bundle.resources` 打包管理
- 📦 **极致瘦身**：发布包从 13.2MiB 降至 212KiB（减少 98.5%）
- 🔧 **资源路径**：插件运行时从 Tauri `resource_dir()` 读取 `sm`
- 📚 **文档更新**：安装说明、安全设计、故障排除同步更新

### v0.3.0 (BREAKING)
- 🔒 **安全重构**：弃用 PowerShell，改用原生 Windows Spooler API
- 🌐 **HTML 打印引擎升级**：wkhtmltopdf → Microsoft Edge headless
- 📐 **自定义纸张尺寸**：`pageWidth`/`pageHeight`（mm）支持标签/小票/卷纸
- 🎯 **强类型 API**：`getPrinters()` 返回 `PrinterInfo[]`
- 📋 **新增 API**：`getJobs`/`getJobById`/`resumeJob`/`restartJob`/`pauseJob`/`removeJob`/`createTempFile`/`removeTempFile`
- 🏗️ **架构重构**：新增 `spooler.rs` 模块
- 🔒 **路径遍历防护**：`createTempFile`/`removeTempFile` 文件名校验

### v0.2.0
- ✨ HTML 打印功能（wkhtmltopdf）
- 🔧 改进错误处理和调试支持
- 📚 完善文档和示例

### v0.1.0
- 🎉 初始版本，兼容 Tauri V2

## 🙏 致谢

基于以下开源项目开发：
- [alfianlensundev/tauri-plugin-printer](https://github.com/alfianlensundev/tauri-plugin-printer) — 原作者
- [adao99/tauri-plugin-printer-v2](https://github.com/adao99/tauri-plugin-printer-v2) — 早期 V2 适配参考
- [wravery/webview2-rs](https://github.com/wravery/webview2-rs) — WebView2 COM Rust 绑定

## 📄 许可证

MIT License
