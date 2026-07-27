# Changelog

All notable changes to the `tauri-plugin-printer-v2` plugin are documented here.

## v0.6.0

- 🏗️ **三层架构模板打印** — 新增 `printTemplate` API，前端只传 JSON（模板 + 数据 + 纸张参数 + 打印机配置），Rust 端原子调度，隐藏 WebView 渲染引擎负责 hiprint 渲染、图片转 Base64、纸张高度计算
- 🎨 **渲染引擎** — 新增 `resources/print-engine/print-render.html`，内置 vue-plugin-hiprint，图片自动转 Base64 + 内容高度自适应
- 🧩 **`print_engine.rs` / `print_service.rs`** — 新增模块，原子流程调度 + oneshot 回调路由，支持并发打印
- 📦 **`PrintTemplateOptions`** — 新增数据结构，强类型前端 API `printTemplate`
- ✅ 前端 API 与权限（`allow-print-template`）同步更新，`build.rs` COMMANDS 与 `lib.rs` 命令清单对齐

## v0.5.0

- 🏗️ **架构重构**：HTML/PDF 打印改为 **WebView2 COM `ICoreWebView2_16::Print` 直接静默打印**
- 📦 **零外部依赖**：移除 SumatraPDF，完全基于 Windows 内置 WebView2 Runtime
- 🎯 `print_pdf` 新增 `copies` / `grayscale` / `orientation` 可选参数

## v0.4.0

- 稳定化基础打印能力（模板/HTML/PDF 打印、打印机列表、打印任务管理）

## v0.2.x

- 早期版本发布与问题修复
