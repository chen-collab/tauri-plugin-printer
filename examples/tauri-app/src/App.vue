<script setup>
import { ref } from 'vue'
import { ping, getPrinters, getPrinterByName, printPdf, printHtml } from 'tauri-plugin-printer-api'
// import { writeTextFile, BaseDirectory } from '@tauri-apps/api/fs'

 


const response = ref('')
const printerName = ref('')
const pdfFilePath = ref('D:\\Downloads\\平心堂项目报价.pdf')

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
  } catch (error) {
    updateResponse(`获取打印机列表失败: ${error}`)
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

const handlePrintCurrentPage = async () => {
  try {
    updateResponse('🖨️ 开始打印当前页面...')
    
    // 验证打印机设置
    const selectedPrinter = printerName.value.trim()
    if (!selectedPrinter) {
      updateResponse('⚠️ 警告: 未指定打印机，将使用默认打印机')
    } else {
      updateResponse(`📋 使用打印机: ${selectedPrinter}`)
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
            <p><strong>打印机:</strong> ${selectedPrinter || '默认打印机'}</p>
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
      printer_id: selectedPrinter || undefined,
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
    updateResponse(`开始打印指定PDF文件: ${pdfFilePath.value}`)
    
    // 检查是否设置了打印机
    if (!printerName.value.trim()) {
      updateResponse('警告: 未指定打印机，将使用默认打印机')
    }
    
    // 构建打印选项
    const printId = `pdf_print_${Date.now()}`
    const printOptions = {
      id: printId, path: pdfFilePath.value, 
      printer_setting: printerName.value.trim() || 'default',
      remove_after_print: false // 不删除原文件
    }
    
    updateResponse(`打印配置: ID=${printId}, Path=${pdfFilePath.value}, Options=${JSON.stringify(printOptions, null, 2)}`)
    
    // 调用打印PDF API
    console.log('打印配置:', { id: printId, path: pdfFilePath.value, options: printOptions });
    const result = await printPdf( printOptions)
    updateResponse(`PDF打印任务已提交: ${result}`)
    updateResponse(`文件路径: ${printOptions.path}`)
    updateResponse(`使用打印机: ${printOptions.printer_setting}`)
    
  } catch (error) {
    updateResponse(`打印PDF失败: ${error.message || error}`)
  }
}
</script>

<template>
  <div class="container">
    <header>
      <h1>Tauri Plugin Printer Example</h1>
      <p>基于 Vue 3 + Vite 的 Tauri 插件演示应用</p>
    </header>

    <main>
      <div class="demo-section">
        <div class="button-group">
          <button @click="handlePing" class="action-button">
            测试 Ping 功能
          </button>
          <button @click="handleGetPrinters" class="action-button">
            获取打印机列表
          </button>
          <button @click="handlePrintCurrentPage" class="action-button print-button">
            打印当前页面
          </button>
          <button @click="handlePrintSpecificPdf" class="action-button pdf-print-button">
            打印指定PDF文件
          </button>
        </div>
        
        <div class="printer-search-section">
          <h3>根据名称获取打印机信息：</h3>
          <div class="search-group">
            <input 
              v-model="printerName" 
              type="text" 
              placeholder="请输入打印机名称" 
              class="printer-input"
              @keyup.enter="handleGetPrinterByName"
            />
            <button @click="handleGetPrinterByName" class="search-button">
              获取打印机信息
            </button>
          </div>
        </div>
        
        <div class="pdf-file-section">
          <h3>PDF文件打印设置：</h3>
          <div class="search-group">
            <input 
              v-model="pdfFilePath" 
              type="text" 
              placeholder="请输入PDF文件路径" 
              class="printer-input"
            />
            <button @click="handlePrintSpecificPdf" class="pdf-button">
              打印此PDF文件
            </button>
          </div>
          <p class="file-info">当前文件: {{ pdfFilePath }}</p>
        </div>
        
        <div class="response-area">
          <h3>响应日志：</h3>
          <pre>{{ response || '点击按钮测试插件功能...' }}</pre>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

header {
  text-align: center;
  margin-bottom: 3rem;
}

header h1 {
  color: #2c3e50;
  margin-bottom: 0.5rem;
}

header p {
  color: #7f8c8d;
  font-size: 1.1rem;
}

.demo-section {
  background: #f8f9fa;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.button-group {
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
  flex-wrap: wrap;
}

.action-button {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  flex: 1;
  min-width: 150px;
}

.action-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
}

.action-button:nth-child(2) {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.action-button:nth-child(2):hover {
  box-shadow: 0 8px 25px rgba(245, 87, 108, 0.3);
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

.printer-search-section {
  margin: 2rem 0;
  padding: 1.5rem;
  background: white;
  border-radius: 8px;
  border: 1px solid #e1e8ed;
}

.printer-search-section h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  color: #2c3e50;
}

.search-group {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-wrap: wrap;
}

.printer-input {
  flex: 1;
  min-width: 200px;
  padding: 12px 16px;
  border: 2px solid #e1e8ed;
  border-radius: 8px;
  font-size: 1rem;
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
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.search-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(40, 167, 69, 0.3);
}

.pdf-file-section {
  margin: 2rem 0;
  padding: 1.5rem;
  background: white;
  border-radius: 8px;
  border: 1px solid #e1e8ed;
}

.pdf-file-section h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  color: #2c3e50;
}

.pdf-button {
  background: linear-gradient(135deg, #ffa726 0%, #ff7043 100%);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.pdf-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(255, 167, 38, 0.3);
}

.file-info {
  margin-top: 1rem;
  padding: 0.5rem;
  background: #f8f9fa;
  border-radius: 4px;
  color: #6c757d;
  font-size: 0.9rem;
  word-break: break-all;
}

.response-area {
  background: white;
  border-radius: 8px;
  padding: 1.5rem;
  border: 1px solid #e1e8ed;
}

.response-area h3 {
  margin-top: 0;
  color: #2c3e50;
  margin-bottom: 1rem;
}

.response-area pre {
  background: #f1f3f4;
  padding: 1rem;
  border-radius: 6px;
  border-left: 4px solid #667eea;
  font-family: 'Courier New', monospace;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: #2c3e50;
  margin: 0;
  min-height: 100px;
}
</style>
