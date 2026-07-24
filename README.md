<div align="center">

# Tauri Plugin Printer

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-printer-v2.svg)](https://crates.io/crates/tauri-plugin-printer-v2)
[![NPM](https://img.shields.io/npm/v/tauri-plugin-printer-v2.svg)](https://www.npmjs.com/package/tauri-plugin-printer-v2)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) 
[![Tauri](https://img.shields.io/badge/Tauri-v2.11-orange.svg)](https://tauri.app/)

**Tauri V2 打印机插件 — 安全、高性能、强类型**

基于原生 Windows Spooler API，支持 PDF/HTML 打印、打印机管理、打印任务控制。

[安装](#-安装) • [使用](#-快速开始) • [API 文档](#-api-文档) • [示例](#-示例)

</div>

---

## ✨ 特性

- 🖨️ **获取系统打印机列表** — 基于原生 `EnumPrintersW` API，返回强类型 `PrinterInfo[]`
- 📄 **打印 PDF 文件** — 内置 SumatraPDF 引擎，**直接子进程执行，无 Shell 注入风险**
- 🌐 **打印 HTML 内容** — 通过 Microsoft Edge (Chromium) headless 转换，支持标准纸张 + 自定义尺寸（mm），完美适配 vue-plugin-hiprint
- 📋 **打印任务管理** — 查询/暂停/恢复/重启/删除打印任务，基于 `EnumJobsW`/`SetJobW` API
- 🔍 **按名称查询打印机** — 获取单个打印机详细信息
- 📁 **临时文件管理** — 创建/删除临时文件，含路径遍历防护
- 🔒 **安全设计** — 弃用 PowerShell，零命令注入；路径遍历校验；打印引擎由主程序通过 Tauri 资源系统打包管理
- 🎯 **强类型 API** — 前端返回 camelCase 类型化对象，IDE 友好

## 📦 安装

```bash
# Rust 依赖
cargo add tauri-plugin-printer-v2

# 前端 API
npm install tauri-plugin-printer-v2
```

### ⚠️ 重要说明（Windows）

此插件需要 PDF 打印引擎（基于 SumatraPDF）才能打印 PDF/HTML。 

**二进制不嵌入插件**，由主应用通过 Tauri 资源系统打包管理：

1. 下载 [SumatraPDF](https://www.sumatrapdfreader.org/download-free-pdf-viewer)
2. 将 `SumatraPDF.exe` 重命名为 `sm`
3. 放置于 `src-tauri/resources/sm`
4. 在 `tauri.conf.json` 的 `bundle` 中配置资源打包：

```json
{
  "bundle": {
    "resources": ["resources/*"]
  }
}
```

插件运行时从 Tauri 资源目录自动读取 `sm`。


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
  "permissions": ["printer:default"]
}
```

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
  removeAfterPrint: false
});

// 打印 HTML
await printHtml({
  html: '<h1>Hello World</h1>',
  pageSize: 'A4',
  orientation: 'Portrait',
  margin: { top: 10, bottom: 10, left: 10, right: 10, unit: 'mm' }
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
}

interface PrintHtmlOptions {
  html: string
  printerId?: string
  printSettings?: string
  removeAfterPrint?: boolean
  pageSize?: string           // A4, A5, Letter 等（标准纸张）
  pageWidth?: number          // 自定义纸张宽度（mm），与 pageSize 二选一
  pageHeight?: number         // 自定义纸张高度（mm）
  orientation?: string        // Portrait, Landscape
  margin?: PrintMargin
  quality?: number            // 1-100
  grayscale?: boolean
  copies?: number
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
| `printPdf(options)` | `Promise<string>` | 打印 PDF 文件 |
| `printHtml(options)` | `Promise<string>` | 打印 HTML 内容 |
| `getJobs(printerName)` | `Promise<JobInfo[]>` | 获取打印任务列表 |
| `getJobById(printer, jobId)` | `Promise<JobInfo>` | 按 ID 获取任务 |
| `resumeJob(printer, jobId)` | `Promise<void>` | 恢复任务 |
| `restartJob(printer, jobId)` | `Promise<void>` | 重启任务 |
| `pauseJob(printer, jobId)` | `Promise<void>` | 暂停任务 |
| `removeJob(printer, jobId)` | `Promise<void>` | 删除任务 |
| `createTempFile(base64, name)` | `Promise<string>` | 创建临时文件 |
| `removeTempFile(name)` | `Promise<boolean>` | 删除临时文件 |

## 📋 示例

### 医疗打印（vue-plugin-hiprint + Edge headless）

```typescript
import { printHtml } from 'tauri-plugin-printer-v2';
import { HiprintTemplate } from 'vue-plugin-hiprint';

// hiprint 模板生成 HTML
const template = new HiprintTemplate({ template: jsonData });
const html = template.getHtml(printData);

// 自定义纸张尺寸打印（如 58×40mm 标签）
await printHtml({
  html: html,
  pageWidth: 58,           // 标签宽度（mm）
  pageHeight: 40,          // 标签高度（mm）
  printerId: '标签打印机',
  margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' },
  removeAfterPrint: true
});

// 标准 A4 处方打印
await printHtml({
  html: prescriptionHtml,
  pageSize: 'A4',
  orientation: 'Portrait',
  printerId: '处方打印机',
  margin: { top: 10, bottom: 10, left: 10, right: 10, unit: 'mm' },
  removeAfterPrint: true
});
```

### 打印发票

```typescript
import { printHtml } from 'tauri-plugin-printer-v2';

const invoiceHtml = `<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
  body { font-family: Arial, sans-serif; padding: 20px; }
  table { width: 100%; border-collapse: collapse; }
  th, td { border: 1px solid #000; padding: 8px; }
</style></head><body>
  <h1>发票 INV-2024-001</h1>
  <table>
    <tr><th>项目</th><th>数量</th><th>单价</th><th>金额</th></tr>
    <tr><td>产品A</td><td>2</td><td>¥100</td><td>¥200</td></tr>
  </table>
</body></html>`;

await printHtml({
  html: invoiceHtml,
  printerId: 'Microsoft Print to PDF',
  pageSize: 'A4',
  removeAfterPrint: true
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
  console.log(`任务 ${jobs[0].documentName} 已暂停`);
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
```

### 项目结构

```
tauri-plugin-printer/
├── src/                    # Rust 后端
│   ├── lib.rs             # 插件入口，命令注册
│   ├── spooler.rs         # Windows Spooler API 封装（EnumPrintersW/OpenPrinterW 等）
│   ├── windows.rs         # 打印逻辑（PDF/HTML/job 控制）
│   ├── desktop.rs         # 桌面端 Printer 实例
│   ├── declare.rs         # 数据结构定义（PrinterInfo/JobInfo/PrintOptions）
│   ├── error.rs           # 错误类型
│   ├── fsys.rs            # 文件操作（含路径遍历防护）
│   └── models.rs          # Ping 请求/响应模型
├── guest-js/              # TypeScript 前端 API
│   └── index.ts           # 类型定义 + 所有命令封装
├── permissions/           # Tauri 权限配置
└── examples/tauri-app/    # Vue 3 示例应用
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
| **弃用 PowerShell** | 所有打印机操作改用原生 Windows Spooler API（`EnumPrintersW`/`OpenPrinterW`/`EnumJobsW`/`SetJobW`），**零命令注入风险** |
| **直接子进程执行** | `sm` 通过 `Command::new().args([])` 调用，参数分离，不经 Shell 解析，不从插件内部嵌入二进制 |
| **路径遍历防护** | `createTempFile`/`removeTempFile` 对文件名做 `sanitize_filename` 校验，拒绝 `..`、绝对路径和路径分隔符 |
| **主程序打包资源** | `sm` 由主应用通过 `tauri.conf.json` 的 `bundle.resources` 打包，插件运行时从 Tauri 资源目录读取，不嵌入插件二进制，减小发布包体积 |
| **无硬编码密钥** | 所有敏感信息均从环境变量或系统 API 获取 |

## 🐛 已知问题

- 目前仅完整支持 **Windows** 平台（非 Windows 返回 `UnsupportedPlatform` 错误）
- HTML 打印依赖 **Microsoft Edge**（Windows 10+ 自带，无需额外安装）
- 某些打印机驱动的状态报告可能不完整

## 🔧 故障排除

### 无法获取打印机列表
- 确保 Print Spooler 服务正在运行（`services.msc` → Print Spooler）
- 检查应用权限配置是否包含 `printer:allow-get-printers`

### PDF 打印失败
- 确保 PDF 文件路径正确且文件存在
- 检查 `printerSetting` 是否与可用打印机名称一致（留空使用默认打印机）
- 检查主程序是否已通过 `bundle.resources` 打包 `sm` 到 `resources/` 目录

### HTML 打印失败
- 确保 Microsoft Edge 可用（Windows 10+ 自带，无需额外安装）
- 若 Edge 被卸载，请重新安装或恢复系统组件

## 📝 更新日志

### v0.4.0 (BREAKING)
- 🏗️ **架构重构**：`sm` 不再嵌入插件二进制，改为由主应用通过 `bundle.resources` 打包管理
- 📦 **极致瘦身**：发布包从 13.2MiB 降至 212KiB（减少 98.5%）
- 🔧 **资源路径**：插件运行时从 Tauri `resource_dir()` 读取 `sm`，延迟到打印时检查存在性，setup 阶段不报错
- 📚 **文档更新**：安装说明、安全设计、故障排除同步更新

### v0.3.0 (BREAKING)
- 🔒 **安全重构**：弃用 PowerShell，改用原生 Windows Spooler API，消除命令注入风险
- 🌐 **HTML 打印引擎升级**：**wkhtmltopdf → Microsoft Edge (Chromium) headless**，CSS 支持完美，中文渲染无问题，无需额外安装
- 📐 **自定义纸张尺寸**：`pageWidth`/`pageHeight`（mm）支持标签/小票/卷纸等非标准纸张，完美适配 vue-plugin-hiprint
- 📦 **包名统一**：`tauri-plugin-printer-v2`（Cargo + npm 统一命名）
- 🎯 **强类型 API**：`getPrinters()` 返回 `PrinterInfo[]`（不再返回 JSON 字符串）
- 📋 **新增 API**：`getJobs`/`getJobById`/`resumeJob`/`restartJob`/`pauseJob`/`removeJob`/`createTempFile`/`removeTempFile`
- 🏗️ **架构重构**：新增 `spooler.rs` 模块，统一命令入口，删除死代码 `commands.rs`
- 🔒 **路径遍历防护**：`createTempFile`/`removeTempFile` 添加文件名校验
- 📦 **依赖升级**：`tauri = "2"`（兼容 2.11+），`windows = "0.61"`
- 🗑️ **清理**：移除 `custom_*` 函数，修正权限配置，删除 Vue 模板残留
- 📚 **完善文档**：更新 API 文档、类型定义、安全设计说明

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

## 📄 许可证

MIT License