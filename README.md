<div align="center">

# Tauri Plugin Printer

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-printer-v2.svg)](https://crates.io/crates/tauri-plugin-printer-v2)
[![NPM](https://img.shields.io/npm/v/tauri-plugin-printer-v2.svg)](https://www.npmjs.com/package/tauri-plugin-printer-v2)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2.11-orange.svg)](https://tauri.app/)

**Tauri V2 打印机插件 — WebView2 COM 静默打印 · 三层架构模板引擎**

基于 WebView2 COM `ICoreWebView2_16::Print` 实现真正的静默打印。
**推荐三层架构**：前端传 `{模板+数据+打印机配置}` → Rust 原子调度 → 隐藏 WebView 渲染引擎，
前端彻底解脱 hiprint 渲染、图片等待、纸张计算等底层细节。

[快速开始](#-快速开始) • [三层架构](#-三层架构) • [API 文档](#-api-文档) • [示例](#-示例) • [开发](#-开发)

</div>

---

## ✨ 特性

- 🏗️ **三层架构模板打印** — 前端只传 JSON，Rust 原子操作（建窗→渲染→打印→销窗），引擎纯计算
- 🖨️ **系统打印机管理** — 原生 `EnumPrintersW`，强类型 `PrinterInfo[]`
- 🌐 **HTML 静默打印** — 隐藏 WebView + `ICoreWebView2_16::Print`，自定义尺寸（热敏小票/标签）
- 📄 **PDF 静默打印** — WebView2 内置 PDFium，无需 SumatraPDF 等外部依赖
- 📋 **打印任务管理** — 查询/暂停/恢复/重启/删除（`EnumJobsW` / `SetJobW`）
- 🎨 **vue-plugin-hiprint 深度整合** — 引擎内置 hiprint，图片自动转 Base64，高度自动计算
- 🎯 **强类型 API** — 前端 camelCase 类型化，IDE 友好
- 🚀 **零外部二进制** — 完全基于 Windows 内置 WebView2 Runtime（Win10+ 自带）

## 🏗️ 三层架构

> **医疗门诊打印推荐方案**，强制职责分离，严禁跨层承担无关逻辑。

```
┌────────────────────────────────────────────────────────────────────┐
│  前端 Vue 层（指挥官）                                             │
│  ✅ 做：患者/检验数据组装、模板 JSON 存储选择、打印机参数、结果提示   │
│  ❌ 不做：hiprint 渲染、DOM 生成、图片加载等待、纸张高度计算         │
│  API: printTemplate({ template, data, paperWidth, printerId, ... }) │
└──────────────────────┬─────────────────────────────────────────────┘
                       │ invoke (IPC)
                       ▼
┌────────────────────────────────────────────────────────────────────┐
│  Rust Tauri 层（调度中心）                                         │
│  ✅ 做：全生命周期调度、原子化流程管控、隐藏窗口管理、异常捕获       │
│  ❌ 不做：hiprint 实例化、页面渲染、尺寸计算                        │
│  流程：创建窗口 → 加载引擎 → 注入模板数据 → 等待渲染 → 写入页面 →    │
│        WebView2 Print → 销毁窗口 → 返回结果                        │
└──────────────────────┬─────────────────────────────────────────────┘
                       │ WebView2
                       ▼
┌────────────────────────────────────────────────────────────────────┐
│  隐藏 WebView 引擎页（纯计算引擎）                                 │
│  ✅ 做：实例化 hiprint → 等待图片加载 → 计算内容高度 → 输出完整HTML   │
│  ❌ 不做：业务判断、打印机选择、打印动作、文件读写、网络请求         │
│  文件：print-engine/print-render.html（资源目录静态页面）            │
│  API : window.renderAndCalculate({ templateJson, dataJson, ... })   │
└────────────────────────────────────────────────────────────────────┘
```

**为什么用三层架构？**

| 对比项 | 传统模式（前端渲染） | 三层架构（推荐） |
|---|---|---|
| 前端依赖 | 需引入 vue-plugin-hiprint（~900KB） | **不需要**，只传 JSON |
| 图片处理 | 前端手动转 Base64 | 引擎自动转 |
| 纸张高度 | 前端硬编码或估算 | 引擎精确计算 |
| 并发安全 | 需管理模板实例 | 每次打印独立窗口，天然隔离 |
| 可测试性 | 渲染逻辑散落在前端 | 引擎纯函数，可独立测试 |
| 代码复用 | 每个页面复制粘贴渲染逻辑 | 一次接入，处处可用 |

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

### 配置模板引擎资源（使用 printTemplate 时需要）

将插件仓库 `resources/print-engine/` 目录拷贝到你的项目：

```
your-project/
├── resources/
│   └── print-engine/
│       ├── print-render.html      # 渲染引擎页面
│       ├── jquery.min.js          # hiprint 依赖
│       ├── jsbarcode.min.js       # 条形码
│       ├── bwip-js.js             # 二维码
│       └── vue-plugin-hiprint.js  # hiprint 核心库
```

在 `tauri.conf.json` 中配置打包：

```json
{
  "bundle": {
    "resources": ["resources/print-engine/"]
  }
}
```

> `bundle.resources` 路径相对于 `src-tauri/` 目录，根据实际项目结构调整。

### ⚠️ WebView2 Runtime 版本要求

打印功能依赖 `ICoreWebView2_16::Print` API，需 WebView2 Runtime **>= 1.0.1518.46**。
Windows 10 1809+ / Win11 自带且自动更新，通常已满足。

## 🚀 快速开始

### 方式一：模板打印（推荐 · 三层架构）

前端只需传模板 JSON + 数据 JSON + 打印参数，**不需要引入 hiprint**：

```typescript
import { printTemplate } from 'tauri-plugin-printer-v2';

// 模板 JSON（从后端接口或本地存储读取）
const templateJson = JSON.stringify({
  panels: [{
    index: 0, height: 297, width: 210,
    printElements: [
      { options: { left: 10, top: 10, height: 20, width: 190,
        title: "门诊处方笺", fontSize: 16, fontWeight: "700",
        textAlign: "center" }, printElementType: { type: "text" }},
      // ... 更多模板元素
    ]
  }]
});

// 打印数据
const dataJson = JSON.stringify({
  patientName: "张三",
  age: 45,
  department: "内科",
  diagnosis: "急性上呼吸道感染",
  medicines: [
    { name: "阿莫西林胶囊", spec: "0.5g", dosage: "口服 tid", qty: "24粒" },
    { name: "布洛芬缓释胶囊", spec: "0.3g", dosage: "口服 bid", qty: "10粒" },
  ]
});

// 一键打印（Rust 端原子操作：建窗 → 渲染 → 打印 → 销窗）
const result = await printTemplate({
  template: templateJson,
  data: dataJson,
  paperWidth: 210,          // 纸张宽度 mm（A4）
  paperHeight: 297,         // 纸张高度 mm，不传则自动计算内容高度
  orientation: 'Portrait',
  printerId: 'HP LaserJet',
  copies: 2,
  grayscale: false,
  renderTimeoutMs: 15000,   // 渲染超时（可选，默认 15s）
});

console.log('打印结果:', result);
```

> 👉 **前端无需处理图片转 Base64、纸张高度计算、CSS @page 构建** —— 全部由引擎层完成。

### 方式二：HTML 打印（底层能力）

直接传完整 HTML 字符串打印（适合简单场景或自定义渲染）：

```typescript
import { printHtml } from 'tauri-plugin-printer-v2';

await printHtml({
  html: '<h1>Hello World</h1>',
  pageSize: 'A4',
  orientation: 'Portrait',
  margin: { top: 10, bottom: 10, left: 10, right: 10, unit: 'mm' },
  copies: 1,
  grayscale: false,
  printerId: 'Microsoft Print to PDF',
});
```

> ⚠️ HTML 中的图片必须转 Base64 内嵌（隐藏窗口无法加载鉴权网络图片）。

### 方式三：PDF 打印

```typescript
import { printPdf } from 'tauri-plugin-printer-v2';

await printPdf({
  id: 'report-001',
  path: 'C:/reports/medical-report.pdf',
  printerSetting: 'HP LaserJet',
  copies: 3,
  grayscale: true,
  removeAfterPrint: false,
});
```

### 获取打印机列表

```typescript
import { getPrinters } from 'tauri-plugin-printer-v2';

const printers = await getPrinters();
console.log('可用打印机:', printers.map(p => p.name));
```

## 📚 API 文档

### PrintTemplateOptions（模板打印）

```typescript
interface PrintTemplateOptions {
  template: string          // hiprint 模板 JSON 字符串
  data: string              // 打印数据 JSON 字符串
  paperWidth: number        // 纸张宽度（mm）
  paperHeight?: number      // 纸张高度（mm），不传则自动计算内容高度
  orientation?: string      // Portrait / Landscape
  printerId?: string        // 打印机名称，为空用默认打印机
  copies?: number           // 打印份数（默认 1）
  grayscale?: boolean       // 是否灰度打印
  renderTimeoutMs?: number  // 渲染超时毫秒（默认 15000）
}
```

### PrintHtmlOptions（HTML 打印）

```typescript
interface PrintHtmlOptions {
  html: string
  printerId?: string
  removeAfterPrint?: boolean
  pageSize?: string         // A4, A5, Letter 等
  pageWidth?: number        // 自定义宽度（mm），与 pageSize 二选一
  pageHeight?: number       // 自定义高度（mm）
  orientation?: string      // Portrait, Landscape
  margin?: PrintMargin
  grayscale?: boolean
  copies?: number
}
```

### PrintPdfOptions（PDF 打印）

```typescript
interface PrintPdfOptions {
  id: string
  path: string
  printerSetting: string    // 打印机名称，空字符串 = 默认打印机
  removeAfterPrint: boolean
  copies?: number
  grayscale?: boolean
  orientation?: string
}
```

### PrintMargin

```typescript
interface PrintMargin {
  top: number
  bottom: number
  left: number
  right: number
  unit: string              // mm, cm, inch
}
```

### PrinterInfo / JobInfo

见 [类型定义](#类型定义) 或 TypeScript 智能提示。

### 完整 API 列表

| 函数 | 返回类型 | 说明 |
|---|---|---|
| `printTemplate(options)` | `Promise<string>` | **[推荐]** 模板打印（三层架构，前端只传 JSON） |
| `printHtml(options)` | `Promise<string>` | HTML 内容打印 |
| `printPdf(options)` | `Promise<string>` | PDF 文件打印 |
| `getPrinters()` | `Promise<PrinterInfo[]>` | 获取所有打印机 |
| `getPrinterByName(name)` | `Promise<PrinterInfo>` | 按名称查询打印机 |
| `getJobs(printerName)` | `Promise<JobInfo[]>` | 获取打印任务列表 |
| `getJobById(printer, jobId)` | `Promise<JobInfo>` | 按 ID 获取任务 |
| `resumeJob(printer, jobId)` | `Promise<void>` | 恢复任务 |
| `restartJob(printer, jobId)` | `Promise<void>` | 重启任务 |
| `pauseJob(printer, jobId)` | `Promise<void>` | 暂停任务 |
| `removeJob(printer, jobId)` | `Promise<void>` | 删除任务 |
| `createTempFile(base64, name)` | `Promise<string>` | 创建临时文件 |
| `removeTempFile(name)` | `Promise<boolean>` | 删除临时文件 |
| `ping(value)` | `Promise<string \| null>` | 测试连接 |

## 📋 示例

### 医疗小票打印（80mm 热敏）

```typescript
import { printTemplate } from 'tauri-plugin-printer-v2';

// 收费小票模板 + 数据
await printTemplate({
  template: receiptTemplateJson,   // 80mm 小票模板
  data: receiptDataJson,           // 患者 + 费用明细
  paperWidth: 80,                  // 80mm 热敏纸
  // paperHeight 不传 → 引擎根据内容自动计算高度
  printerId: '热敏小票打印机',
  copies: 1,
});
```

### 门诊处方打印（A5）

```typescript
import { printTemplate } from 'tauri-plugin-printer-v2';

await printTemplate({
  template: prescriptionTemplateJson,
  data: prescriptionDataJson,
  paperWidth: 148,     // A5
  paperHeight: 210,
  orientation: 'Portrait',
  printerId: '处方打印机',
  copies: 2,
});
```

### 打印任务管理

```typescript
import { getJobs, pauseJob, resumeJob, removeJob } from 'tauri-plugin-printer-v2';

const jobs = await getJobs('HP LaserJet');
console.log(`${jobs.length} 个待处理任务`);

if (jobs.length > 0) {
  await pauseJob('HP LaserJet', String(jobs[0].id));
}
```

## 🏛️ 架构详解

### 模板打印原子流程（printTemplate）

```
前端 invoke print_template
    │
    ▼
print_service::print_template（async，tokio runtime）
    │
    ├─ 1. 输入校验（模板/数据/纸张宽度）
    ├─ 2. 解析引擎路径（resource_dir/print-engine/print-render.html）
    ├─ 3. 生成唯一窗口 label，注册 oneshot sender 到 RenderRegistry
    ├─ 4. 创建隐藏 WebviewWindow（visible=false, skip_taskbar）
    ├─ 5. 导航到引擎 HTML（file:// URL）
    │   └─ 用 NavigationCompleted 事件等待加载完成
    ├─ 6. eval 注入模板+数据，调用 window.renderAndCalculate()
    │
    │   ┌────────── 引擎页面（print-render.html）──────────┐
    │   │ 1. hiprint.init()（首次）                         │
    │   │ 2. new PrintTemplate({ template })                │
    │   │ 3. template.getHtml(data) → 渲染到离屏容器        │
    │   │ 4. 等待所有 <img> 加载完成（超时 10s）             │
    │   │ 5. 所有图片转 Base64（canvas.toDataURL）           │
    │   │ 6. 计算内容像素高度（scrollHeight）                 │
    │   │ 7. 构建完整 HTML（含 @page CSS）                   │
    │   │ 8. invoke print_render_done → 通知 Rust            │
    │   └───────────────────────────────────────────────────┘
    │
    ├─ 7. 收到渲染结果（oneshot channel，带超时）
    ├─ 8. eval 写入 HTML 到当前页面（document.write）
    ├─ 9. 构建 PrintSettings（纸张尺寸/边距/份数/灰度/打印机）
    ├─ 10. ICoreWebView2_16::Print 静默打印
    │    └─ PrintCompletedHandler → mpsc channel → tokio 等待
    ├─ 11. 销毁窗口（webview.close()）
    └─ 返回打印结果
```

### 模块结构

```
src/
├── lib.rs             # 插件入口，命令注册 + RenderRegistry 状态
├── print_engine.rs    # 模板打印引擎调度：print_template、回调路由、文档写入
├── print_service.rs   # 打印服务：print_html/print_pdf、隐藏窗口管理、流程编排
├── webview2_print.rs  # WebView2 COM 封装：PrintSettings 构建、Print 调用、mm_to_inch
├── spooler.rs         # Windows Spooler API 封装
├── windows.rs         # Spooler 转发（打印机列表、任务管理）
├── desktop.rs         # 桌面端 Printer 实例
├── declare.rs         # 数据结构定义（PrintTemplateOptions / PrintHtmlOptions / ...）
├── error.rs           # 错误类型
├── fsys.rs            # 文件操作（路径遍历防护）
└── models.rs          # Ping 请求/响应模型

resources/print-engine/
├── print-render.html  # 渲染引擎页面（纯计算，零业务耦合）
├── jquery.min.js      # hiprint 依赖
├── jsbarcode.min.js   # 条形码渲染
├── bwip-js.js         # 二维码渲染
└── vue-plugin-hiprint.js  # hiprint 核心库
```

### 关键设计决策

| 决策 | 原因 |
|---|---|
| **每次打印独立窗口** | 天然隔离，无并发冲突，打印完毕销毁不留状态 |
| **引擎 HTML 放在 resources/** | 纯静态页面，可独立测试、可替换（换其他模板引擎只需改这一个文件） |
| **oneshot + HashMap 回调路由** | 支持并发打印，每个窗口 label 唯一对应一个 sender |
| **图片引擎端转 Base64** | 隐藏窗口无法加载鉴权网络图片，在引擎层统一处理 |
| **document.write 替换文档** | 拿到渲染 HTML 后同步写入当前页面，直接打印，无需二次导航 |
| **WebView2 COM 直接打印** | 不经过 Edge headless / SumatraPDF 子进程，零命令注入风险 |

## 🛠️ 开发

```bash
git clone https://github.com/chen-collab/tauri-plugin-printer.git
cd tauri-plugin-printer

# 构建前端 API
npm install && npm run build

# Rust 构建 / 测试 / 格式化 / Lint
cargo build
cargo test          # 含 print_engine / print_service / webview2_print 单测
cargo fmt
cargo clippy

# 运行示例应用
cd examples/tauri-app
npm install
npm run tauri:dev
```

## 🔒 安全设计

| 措施 | 说明 |
|---|---|
| **WebView2 COM 直接打印** | 不经过 Edge headless 命令行 / SumatraPDF 子进程，无命令注入风险 |
| **原生 Spooler API** | 打印机管理用 Windows API（`EnumPrintersW`/`EnumJobsW`/`SetJobW`），零 Shell 调用 |
| **路径遍历防护** | 临时文件 API 对文件名做 `sanitize_filename` 校验，拒绝 `..`、绝对路径和路径分隔符 |
| **无外部二进制** | 完全基于系统内置 WebView2 Runtime，插件不嵌入也不依赖任何外部可执行文件 |
| **隐藏窗口隔离** | 每次打印独立隐藏窗口，打印完毕立即销毁，无状态泄漏 |
| **引擎纯计算** | 渲染引擎无业务逻辑、无网络请求、无文件读写，只做模板→HTML 转换 |

## 🐛 已知问题

- 仅完整支持 **Windows** 平台（非 Windows 返回 `UnsupportedPlatform` 错误）
- 打印功能依赖 **WebView2 Runtime >= 1.0.1518.46**
- PDF 打印用固定延时等待渲染（1.5s），超大 PDF 可能需要调整
- 某些热敏打印机驱动对自定义纸张尺寸支持不同，建议实测
- 模板打印的 `vue-plugin-hiprint.js` 约 900KB，随资源打包进应用

## 🔧 故障排除

### printTemplate 提示「渲染引擎文件不存在」
- 确认 `resources/print-engine/print-render.html` 存在
- 确认 `tauri.conf.json > bundle.resources` 配置正确
- 开发模式下确保资源目录能被 Tauri 找到

### 打印失败提示「WebView2 Runtime 版本过低」
- 系统 WebView2 Runtime 版本过旧，需升级到 >= 1.0.1518.46
- 下载地址：https://developer.microsoft.com/microsoft-edge/webview2/

### 模板打印内容不对 / 样式错乱
- 确认模板 JSON 格式正确（`hiprintTemplate.getJson()` 输出的标准格式）
- 确认打印数据 JSON 与模板字段对应
- 若图片不显示，检查是否为跨域图片（引擎已自动尝试转 Base64，但受 CORS 限制）

### 热敏小票打印尺寸不对
- 确认打印机驱动支持自定义纸张尺寸
- `paperWidth` 单位为毫米（mm），需与打印机实际纸张一致
- `paperHeight` 不传则自动计算内容高度，适合卷纸打印机

## 📝 更新日志

### v0.6.0
- 🏗️ **三层架构模板打印** — 新增 `printTemplate` API，前端只传 JSON
- 🎨 **渲染引擎** — 新增 `print-engine/print-render.html`，内置 hiprint，自动图片 Base64 + 高度计算
- 🧩 **`print_engine.rs`** — 新增模块，原子流程调度 + oneshot 回调路由
- 📦 **`PrintTemplateOptions`** — 新增数据结构
- ✨ 前端 demo 全面改造，四个 designer 组件均改用三层架构
- 📚 README 全面更新，三层架构文档 + 流程图

### v0.5.0
- 🏗️ **架构重构**：HTML/PDF 打印改为 **WebView2 COM `ICoreWebView2_16::Print` 直接静默打印**
- 📦 **零外部依赖**：移除 SumatraPDF，完全基于 Windows 内置 WebView2 Runtime
- 🎯 `print_pdf` 新增 `copies` / `grayscale` / `orientation` 可选参数

### 更早版本
见 [CHANGELOG.md 历史归档](#)

## 🙏 致谢

基于以下开源项目开发：
- [alfianlensundev/tauri-plugin-printer](https://github.com/alfianlensundev/tauri-plugin-printer) — 原作者
- [adao99/tauri-plugin-printer-v2](https://github.com/adao99/tauri-plugin-printer-v2) — 早期 V2 适配
- [wravery/webview2-rs](https://github.com/wravery/webview2-rs) — WebView2 COM Rust 绑定
- [vue-plugin-hiprint](https://github.com/CcSimple/vue-plugin-hiprint) — 打印模板引擎

## 📄 许可证

MIT License
