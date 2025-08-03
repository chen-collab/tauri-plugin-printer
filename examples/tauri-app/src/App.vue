<script setup>
import { ref } from 'vue'
import { ping, getPrinters, getPrinterByName, printPdf, printHtml } from 'tauri-plugin-printer-v2'
import { open } from '@tauri-apps/plugin-dialog'
// import { writeTextFile, BaseDirectory } from '@tauri-apps/api/fs'

const response = ref('')
const printerName = ref('')
const pdfFilePath = ref('')
const selectedFileName = ref('')
const printersList = ref([])
const selectedPrinter = ref('')

const updateResponse = (returnValue) => {
  const timestamp = new Date().toLocaleTimeString()
  const value = typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)
  response.value += `[${timestamp}] ${value}\n`
}

const handlePing = async () => {
  try {
    const result = await ping("Pong!")
    updateResponse(result)
  } catch (error) {
    updateResponse(error)
  }
}

const handleGetPrinters = async () => {
  try {
    const result = await getPrinters()
    updateResponse(`打印机列表: ${result}`)
    
    // 解析打印机列表数据
    try {
      const parsedResult = JSON.parse(result)
      if (Array.isArray(parsedResult)) {
        printersList.value = parsedResult.map((printer, index) => ({
          id: index + 1,
          name: printer.Name || printer,
          status: printer.PrinterStatus || '未知',
          isDefault: printer.isDefault || false,
          driver: printer.DriverName || '未知',
          port: printer.PortName || '未知'
        }))
        updateResponse(`成功解析 ${printersList.value.length} 台打印机`)
      } else {
        // 如果返回的是字符串列表，按行分割
        const printerNames = result.split('\n').filter(name => name.trim())
        printersList.value = printerNames.map((name, index) => ({
          id: index + 1,
          name: name.trim(),
          status: '可用',
          isDefault: index === 0,
          driver: '未知',
          port: '未知'
        }))
        updateResponse(`成功解析 ${printersList.value.length} 台打印机`)
      }
    } catch (parseError) {
      updateResponse(`解析打印机列表失败: ${parseError}`)
      // 作为备选方案，尝试简单的字符串分割
      const printerNames = result.split(',').filter(name => name.trim())
      printersList.value = printerNames.map((name, index) => ({
        id: index + 1,
        name: name.trim(),
        status: '可用',
        isDefault: index === 0,
        driver: '未知',
        port: '未知'
      }))
    }
  } catch (error) {
    updateResponse(`获取打印机列表失败: ${error}`)
    printersList.value = []
  }
}

const handleGetPrinterByName = async () => {
  if (!printerName.value.trim()) {
    updateResponse('请输入打印机名称')
    return
  }
  
  try {
    const result = await getPrinterByName(printerName.value.trim())
    updateResponse(`打印机信息 [${printerName.value}]: ${result}`)
  } catch (error) {
    updateResponse(`获取打印机信息失败 [${printerName.value}]: ${error}`)
  }
}

const handleSelectPrinter = (printer) => {
  selectedPrinter.value = printer.name
  printerName.value = printer.name
  updateResponse(`已选择打印机: ${printer.name}`)
}

const handleSelectPdfFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'PDF文件',
        extensions: ['pdf']
      }]
    })
    
    if (selected && typeof selected === 'string') {
      pdfFilePath.value = selected
      // 提取文件名
      const fileName = selected.split('\\').pop() || selected.split('/').pop() || selected
      selectedFileName.value = fileName
      updateResponse(`已选择PDF文件: ${fileName}`)
      updateResponse(`文件路径: ${selected}`)
    } else {
      updateResponse('未选择文件')
    }
  } catch (error) {
    updateResponse(`选择文件失败: ${error}`)
  }
}

const handlePrintCurrentPage = async () => {
  try {
    updateResponse('🖨️ 开始打印当前页面...')
    
    // 验证打印机设置
    const currentPrinter = selectedPrinter.value || printerName.value.trim()
    if (!currentPrinter) {
      updateResponse('⚠️ 警告: 未指定打印机，将使用默认打印机')
    } else {
      updateResponse(`📋 使用打印机: ${currentPrinter}`)
    }
    
    // 生成优化的HTML内容
    const currentTime = new Date().toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
    
    const htmlContent = `
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tauri Plugin Printer 测试页面</title>
    <style>
        @page {
            size: A4;
            margin: 15mm;
        }
        * {
            box-sizing: border-box;
        }
        body {
            font-family: 'Microsoft YaHei', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            margin: 0;
            padding: 20px;
            background: white;
        }
        .header {
            text-align: center;
            border-bottom: 2px solid #2c3e50;
            padding-bottom: 15px;
            margin-bottom: 25px;
        }
        h1 {
            color: #2c3e50;
            margin: 0;
            font-size: 24px;
            font-weight: bold;
        }
        .subtitle {
            color: #7f8c8d;
            font-size: 14px;
            margin-top: 5px;
        }
        .content {
            margin: 20px 0;
        }
        .info-box {
            background: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 8px;
            padding: 15px;
            margin: 15px 0;
        }
        .info-title {
            font-weight: bold;
            color: #495057;
            margin-bottom: 10px;
            font-size: 16px;
        }
        .log-section {
            background: #f1f3f4;
            border-radius: 8px;
            padding: 15px;
            margin-top: 20px;
            border-left: 4px solid #3498db;
        }
        .log-content {
            background: white;
            border: 1px solid #dee2e6;
            border-radius: 4px;
            padding: 10px;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            max-height: 200px;
            overflow-y: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
        }
        .footer {
            margin-top: 30px;
            padding-top: 15px;
            border-top: 1px solid #dee2e6;
            text-align: center;
            font-size: 12px;
            color: #6c757d;
        }
        .print-info {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin: 15px 0;
            padding: 10px;
            background: #e8f4fd;
            border-radius: 6px;
        }
        @media print {
            body { background: white; }
            .no-print { display: none; }
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🖨️ Tauri Plugin Printer 演示应用</h1>
        <div class="subtitle">HTML 打印功能测试页面</div>
    </div>
    
    <div class="content">
        <div class="info-box">
            <div class="info-title">📄 当前页面打印测试</div>
            <p>这是一个测试 Tauri Plugin Printer 的 HTML 打印功能的页面。</p>
            <div class="print-info">
                <span><strong>打印时间:</strong> ${currentTime}</span>
                <span><strong>页面类型:</strong> HTML 转 PDF</span>
            </div>
        </div>
        
        <div class="info-box">
            <div class="info-title">⚙️ 打印配置信息</div>
            <p><strong>打印机:</strong> ${currentPrinter || '默认打印机'}</p>
            <p><strong>页面大小:</strong> A4</p>
            <p><strong>方向:</strong> 纵向 (Portrait)</p>
            <p><strong>边距:</strong> 10mm (上下左右)</p>
            <p><strong>质量:</strong> 300 DPI</p>
        </div>
    </div>
    
    <div class="log-section">
        <div class="info-title">📋 响应日志</div>
        <div class="log-content">${response.value || '暂无日志信息'}</div>
    </div>
    
    <div class="footer">
        <p>由 Tauri Plugin Printer 生成 | ${currentTime}</p>
        <p>此页面通过 wkhtmltopdf 转换为 PDF 后打印</p>
    </div>
</body>
</html>
    `
    
    updateResponse(`📝 生成的HTML内容长度: ${htmlContent.length} 字符`)
    updateResponse(`🔧 准备打印配置...`)
    
    // 构建优化的打印选项
    const printOptions = {
      html: htmlContent,
      printer_id: currentPrinter || undefined,
      print_settings: undefined,
      remove_after_print: true,
      page_size: 'A4',
      orientation: 'Portrait',
      margin: {
        top: 10.0,
        bottom: 10.0,
        left: 10.0,
        right: 10.0,
        unit: 'mm'
      },
      quality: 300,
      grayscale: false,
      copies: 1
    }
    
    // updateResponse(`⚙️ 打印配置详情:\n${JSON.stringify(printOptions, null, 2)}`)
    updateResponse(`🚀 正在提交打印任务...`)
    
    const result = await printHtml(printOptions)
    updateResponse(`✅ 打印任务已成功提交: ${result}`)
    
    // 可选：同时触发浏览器打印对话框作为备选方案
    updateResponse('🌐 同时准备浏览器打印对话框作为备选方案...')
    setTimeout(() => {
      try {
        window.print()
        updateResponse('🖨️ 浏览器打印对话框已打开')
      } catch (printError) {
        updateResponse(`⚠️ 浏览器打印失败: ${printError}`)
      }
    }, 1000)
    
  } catch (error) {
    updateResponse(`❌ 打印失败: ${error}`)
    updateResponse(`💡 建议检查: 1) 打印机是否可用 2) wkhtmltopdf 是否已安装 3) 网络连接是否正常`)
  }
}

const handlePrintSpecificPdf = async () => {
  try {
    // 检查是否选择了PDF文件
    if (!pdfFilePath.value.trim()) {
      updateResponse('❌ 请先选择要打印的PDF文件')
      return
    }
    
    updateResponse(`📄 开始打印PDF文件: ${selectedFileName.value || pdfFilePath.value}`)
    
    // 检查是否设置了打印机
    const currentPrinter = selectedPrinter.value || printerName.value.trim()
    if (!currentPrinter) {
      updateResponse('⚠️ 警告: 未指定打印机，将使用默认打印机')
    } else {
      updateResponse(`🖨️ 使用打印机: ${currentPrinter}`)
    }
    
    // 构建打印选项
    const printId = `pdf_print_${Date.now()}`
    const printOptions = {
      id: printId, 
      path: pdfFilePath.value, 
      printer_setting: currentPrinter || 'default',
      remove_after_print: false // 不删除原文件
    }
    
    updateResponse(`⚙️ 打印配置: ID=${printId}`)
    updateResponse(`📁 文件路径: ${pdfFilePath.value}`)
    updateResponse(`🖨️ 打印机设置: ${printOptions.printer_setting}`)
    
    // 调用打印PDF API
    console.log('打印配置:', { id: printId, path: pdfFilePath.value, options: printOptions });
    const result = await printPdf(printOptions)
    updateResponse(`✅ PDF打印任务已成功提交: ${result}`)
    
  } catch (error) {
    updateResponse(`❌ 打印PDF失败: ${error.message || error}`)
  }
}
</script>

<template>
  <div class="container">
    <header>
      <h1>🖨️ Tauri Plugin Printer Example</h1>
      <p>基于 Vue 3 + Vite 的 Tauri 插件演示应用</p>
    </header>

    <main class="desktop-layout">
      <!-- 左侧控制面板 -->
      <div class="control-panel">
        <div class="section-card">
          <h3>🔧 基础功能</h3>
          <div class="button-group vertical">
            <button @click="handlePing" class="action-button">
              🏓 测试 Ping 功能
            </button>
            <button @click="handleGetPrinters" class="action-button">
              📋 获取打印机列表
            </button>
          </div>
        </div>

        <div class="section-card">
          <h3>🖨️ 打印功能</h3>
          <div class="button-group vertical">
            <button @click="handlePrintCurrentPage" class="action-button print-button">
              📄 打印当前页面
            </button>
          </div>
        </div>

        <div class="section-card">
          <h3>📁 PDF 文件打印</h3>
          <div class="pdf-section">
            <button @click="handleSelectPdfFile" class="action-button file-select-button">
              📂 选择 PDF 文件
            </button>
            <div v-if="selectedFileName" class="selected-file-info">
              <div class="file-icon">📄</div>
              <div class="file-details">
                <div class="file-name">{{ selectedFileName }}</div>
                <div class="file-path">{{ pdfFilePath }}</div>
              </div>
            </div>
            <button 
              @click="handlePrintSpecificPdf" 
              class="action-button pdf-print-button"
              :disabled="!pdfFilePath"
            >
              🖨️ 打印选中的PDF
            </button>
          </div>
        </div>

        <div class="section-card">
          <h3>🔍 打印机查询</h3>
          <div class="search-group">
            <input 
              v-model="printerName" 
              type="text" 
              placeholder="请输入打印机名称" 
              class="printer-input"
              @keyup.enter="handleGetPrinterByName"
            />
            <button @click="handleGetPrinterByName" class="search-button">
              🔍 查询
            </button>
          </div>
        </div>
      </div>

      <!-- 右侧信息面板 -->
      <div class="info-panel">
        
        <div class="printer-list-section" v-if="printersList.length > 0">
          <h3>📋 可用打印机列表：</h3>
          <div class="current-printer" v-if="selectedPrinter">
            <span class="current-label">当前选择：</span>
            <span class="current-name">{{ selectedPrinter }}</span>
          </div>
          <div class="table-container">
            <table class="printers-table">
              <thead>
                <tr>
                  <th>序号</th>
                  <th>打印机名称</th>
                  <th>状态</th>
                  <th>默认</th>
                  <th>驱动</th>
                  <th>端口</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr 
                  v-for="printer in printersList" 
                  :key="printer.id"
                  :class="{ 'selected-row': selectedPrinter === printer.name }"
                >
                  <td>{{ printer.id }}</td>
                  <td class="printer-name">{{ printer.name }}</td>
                  <td>
                    <span class="status-badge" :class="printer.status === '可用' ? 'status-available' : 'status-unknown'">
                      {{ printer.status }}
                    </span>
                  </td>
                  <td>
                    <span v-if="printer.isDefault" class="default-badge">✓</span>
                    <span v-else>-</span>
                  </td>
                  <td class="driver-info">{{ printer.driver }}</td>
                  <td class="port-info">{{ printer.port }}</td>
                  <td>
                    <button 
                      @click="handleSelectPrinter(printer)" 
                      class="select-button"
                      :class="{ 'selected': selectedPrinter === printer.name }"
                    >
                      {{ selectedPrinter === printer.name ? '已选择' : '选择' }}
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="response-area">
          <h3>📋 响应日志</h3>
          <pre>{{ response || '点击按钮测试插件功能...' }}</pre>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

header {
  text-align: center;
  margin-bottom: 2rem;
}

header h1 {
  color: #2c3e50;
  margin-bottom: 0.5rem;
  font-size: 2.5rem;
}

header p {
  color: #7f8c8d;
  font-size: 1.1rem;
}

/* 桌面布局 */
.desktop-layout {
  display: grid;
  grid-template-columns: 400px 1fr;
  gap: 2rem;
  min-height: 80vh;
}

/* 控制面板 */
.control-panel {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.section-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  border: 1px solid #e1e8ed;
}

.section-card h3 {
  margin: 0 0 1rem 0;
  color: #2c3e50;
  font-size: 1.1rem;
  font-weight: 600;
}

/* 信息面板 */
.info-panel {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.button-group {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.button-group.vertical {
  flex-direction: column;
  gap: 0.75rem;
}

/* PDF 文件选择区域 */
.pdf-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.selected-file-info {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  background: #f8f9fa;
  border-radius: 8px;
  border: 2px dashed #dee2e6;
}

.file-icon {
  font-size: 2rem;
}

.file-details {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 0.25rem;
  word-break: break-word;
}

.file-path {
  font-size: 0.85rem;
  color: #6c757d;
  word-break: break-all;
  font-family: 'Courier New', monospace;
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .desktop-layout {
    grid-template-columns: 1fr;
    gap: 1.5rem;
  }
  
  .container {
    padding: 1rem;
  }
  
  header h1 {
    font-size: 2rem;
  }
}

.action-button {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  padding: 12px 20px;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  width: 100%;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.action-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
}

.action-button:disabled {
  background: #6c757d;
  cursor: not-allowed;
  opacity: 0.6;
}

.action-button:disabled:hover {
  transform: none;
  box-shadow: none;
}

.file-select-button {
  background: linear-gradient(135deg, #17a2b8 0%, #138496 100%) !important;
}

.file-select-button:hover:not(:disabled) {
  box-shadow: 0 8px 25px rgba(23, 162, 184, 0.3) !important;
}

.print-button {
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%) !important;
}

.print-button:hover {
  box-shadow: 0 8px 25px rgba(255, 107, 107, 0.3) !important;
}

.pdf-print-button {
  background: linear-gradient(135deg, #ffa726 0%, #ff7043 100%) !important;
}

.pdf-print-button:hover {
  box-shadow: 0 8px 25px rgba(255, 167, 38, 0.3) !important;
}

.printer-list-section {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  border: 1px solid #e1e8ed;
}

.printer-list-section h3 {
  margin: 0 0 1rem 0;
  color: #2c3e50;
  font-size: 1.1rem;
  font-weight: 600;
}

.current-printer {
  margin-bottom: 1rem;
  padding: 0.75rem 1rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.current-label {
  font-weight: 600;
}

.current-name {
  font-weight: 700;
  background: rgba(255, 255, 255, 0.2);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
}

.table-container {
  overflow-x: auto;
  border-radius: 8px;
  border: 1px solid #e1e8ed;
}

.printers-table {
  width: 100%;
  border-collapse: collapse;
  background: white;
}

.printers-table th {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  color: #495057;
  font-weight: 600;
  padding: 1rem 0.75rem;
  text-align: left;
  border-bottom: 2px solid #dee2e6;
  font-size: 0.9rem;
}

.printers-table td {
  padding: 0.75rem;
  border-bottom: 1px solid #f1f3f4;
  vertical-align: middle;
}

.printers-table tbody tr {
  transition: all 0.2s ease;
}

.printers-table tbody tr:hover {
  background: #f8f9fa;
}

.selected-row {
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%) !important;
  border-left: 4px solid #667eea;
}

.printer-name {
  font-weight: 600;
  color: #2c3e50;
  max-width: 200px;
  word-break: break-word;
}

.status-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
}

.status-available {
  background: #d4edda;
  color: #155724;
}

.status-unknown {
  background: #f8d7da;
  color: #721c24;
}

.default-badge {
  color: #28a745;
  font-weight: bold;
  font-size: 1.2rem;
}

.driver-info,
.port-info {
  color: #6c757d;
  font-size: 0.9rem;
  max-width: 120px;
  word-break: break-word;
}

.select-button {
  background: linear-gradient(135deg, #28a745 0%, #20c997 100%);
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  min-width: 70px;
}

.select-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3);
}

.select-button.selected {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  cursor: default;
}

.select-button.selected:hover {
  transform: none;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
}

.search-group {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  flex-wrap: wrap;
}

.printer-input {
  flex: 1;
  min-width: 200px;
  padding: 10px 14px;
  border: 2px solid #e1e8ed;
  border-radius: 6px;
  font-size: 0.95rem;
  transition: border-color 0.3s ease;
}

.printer-input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.search-button {
  background: linear-gradient(135deg, #28a745 0%, #20c997 100%);
  color: white;
  border: none;
  padding: 10px 16px;
  border-radius: 6px;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.search-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(40, 167, 69, 0.3);
}



.response-area {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  border: 1px solid #e1e8ed;
  flex: 1;
}

.response-area h3 {
  margin: 0 0 1rem 0;
  color: #2c3e50;
  font-size: 1.1rem;
  font-weight: 600;
}

.response-area pre {
  background: #f8f9fa;
  padding: 1.5rem;
  border-radius: 8px;
  border-left: 4px solid #667eea;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: #2c3e50;
  margin: 0;
  min-height: 300px;
  max-height: 500px;
  overflow-y: auto;
  font-size: 0.9rem;
  line-height: 1.5;
}
</style>
