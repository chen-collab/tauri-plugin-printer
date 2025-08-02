<script setup>
import { ref } from 'vue'
import { ping, getPrinters, getPrinterByName } from 'tauri-plugin-printer-api'

const response = ref('')
const printerName = ref('')

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
