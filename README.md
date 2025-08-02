# Tauri Plugin Printer

一个用于 Tauri V2 的打印机插件，支持获取打印机列表、打印 PDF 文件、管理打印任务等功能。

## ✨ 特性

- 🖨️ 获取系统打印机列表
- 📄 打印 PDF 文件
- 📋 管理打印任务（暂停、恢复、重启、删除）
- 🔍 按名称查询打印机
- 📊 获取打印任务状态
- 🌐 支持中文打印机名称
- 🔧 完全兼容 Tauri V2 稳定版

## 📦 安装

### 方法一：使用 Tauri CLI（推荐）

```bash
npx tauri add https://github.com/chen-collab/tauri-plugin-printer
```

### 方法二：手动安装

1. 在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
tauri-plugin-printer = { git = "https://github.com/chen-collab/tauri-plugin-printer", branch = "chen" }
```

2. 在 `src-tauri/src/lib.rs` 中注册插件：

```rust
use tauri_plugin_printer::init;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. 在 `src-tauri/capabilities/default.json` 中添加权限：

```json
{
  "permissions": [
    "printer:default"
  ]
}
```

4. 安装前端依赖：

```bash
npm install tauri-plugin-printer-api
```

## 🚀 使用方法

### 基础示例

```javascript
import { ping, getPrinters } from 'tauri-plugin-printer-api';

// 测试插件连接
const response = await ping({ value: 'Hello from frontend!' });
console.log(response);

// 获取打印机列表
const printers = await getPrinters();
console.log('可用打印机:', printers);
```

### 完整 API 示例

```javascript
import {
  ping,
  getPrinters,
  getPrintersByName,
  printPdf,
  getJobs,
  getJobsById,
  resumeJob,
  restartJob,
  pauseJob,
  removeJob
} from 'tauri-plugin-printer-api';

// 1. 获取所有打印机
const allPrinters = await getPrinters();

// 2. 按名称获取特定打印机
const specificPrinter = await getPrintersByName('Microsoft Print to PDF');

// 3. 打印 PDF 文件
const printResult = await printPdf({
  path: '/path/to/your/file.pdf',
  printer: 'Microsoft Print to PDF',
  pages: '1-3',
  subset: 'odd'
});

// 4. 获取打印任务
const jobs = await getJobs('Microsoft Print to PDF');

// 5. 管理打印任务
const jobId = '123';
const printerName = 'Microsoft Print to PDF';

// 暂停任务
await pauseJob(printerName, jobId);

// 恢复任务
await resumeJob(printerName, jobId);

// 重启任务
await restartJob(printerName, jobId);

// 删除任务
await removeJob(printerName, jobId);
```

## 📚 API 文档

### `ping(request: PingRequest): Promise<PingResponse>`
测试插件连接状态。

### `getPrinters(): Promise<string>`
获取系统中所有可用的打印机列表，返回 JSON 格式的字符串。

### `getPrintersByName(name: string): Promise<string>`
根据打印机名称获取特定打印机信息。

### `printPdf(options: PrintOptions): Promise<string>`
打印 PDF 文件。

**PrintOptions 参数：**
- `path`: PDF 文件路径
- `printer`: 打印机名称
- `pages`: 页面范围（可选）
- `subset`: 页面子集（可选）

### 打印任务管理

- `getJobs(printer: string): Promise<string>` - 获取打印机的所有任务
- `getJobsById(printer: string, jobId: string): Promise<string>` - 获取特定任务信息
- `pauseJob(printer: string, jobId: string): Promise<string>` - 暂停打印任务
- `resumeJob(printer: string, jobId: string): Promise<string>` - 恢复打印任务
- `restartJob(printer: string, jobId: string): Promise<string>` - 重启打印任务
- `removeJob(printer: string, jobId: string): Promise<string>` - 删除打印任务

## 🛠️ 开发

### 运行示例应用

```bash
# 克隆仓库
git clone https://github.com/chen-collab/tauri-plugin-printer.git
cd tauri-plugin-printer

# 构建插件
npm run build

# 运行示例应用
cd examples/tauri-app
npm install
npm run tauri:dev
```

### 项目结构

```
tauri-plugin-printer/
├── src/                    # Rust 源代码
│   ├── lib.rs             # 插件主入口
│   ├── commands.rs        # Tauri 命令定义
│   ├── desktop.rs         # 桌面端实现
│   ├── windows.rs         # Windows 特定实现
│   └── ...
├── guest-js/              # JavaScript API
│   └── index.ts           # 前端 API 定义
├── permissions/           # 权限配置
├── examples/              # 示例应用
│   └── tauri-app/         # Vue.js 示例
└── dist-js/               # 构建后的 JS 文件
```

## 🔧 权限配置

插件使用以下权限：

```toml
[default]
description = "Default permissions for the plugin"
permissions = [
  "allow-ping",
  "allow-create-temp-file", 
  "allow-remove-temp-file", 
  "allow-get-printers", 
  "allow-get-printers-by-name", 
  "allow-print-pdf", 
  "allow-get-jobs", 
  "allow-get-jobs-by-id", 
  "allow-resume-job", 
  "allow-restart-job", 
  "allow-pause-job", 
  "allow-remove-job"
]
```

## 🐛 已知问题

- 目前主要支持 Windows 系统
- 某些打印机驱动可能不完全兼容
- 大文件打印可能需要额外的内存管理

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 🙏 致谢

本项目基于以下仓库开发：

- [Alfian Lensun's Original Plugin Repository](https://github.com/alfianlensundev/tauri-plugin-printer)
- [adao99's Fork for Tauri V2 Beta](https://github.com/adao99/tauri-plugin-printer-v2)

感谢原作者的贡献！

## 📝 更新日志

### v0.1.0
- ✅ 兼容 Tauri V2 稳定版
- ✅ 修复中文打印机名称乱码问题
- ✅ 添加 ping 命令
- ✅ 完善权限配置
- ✅ 提供完整的示例应用