<script setup>
import { ref } from 'vue'
import {
  ping,
  getPrinters,
  getPrinterByName,
  printPdf,
  printHtml,
  getJobs,
  resumeJob,
  restartJob,
  pauseJob,
  removeJob,
  createTempFile,
  removeTempFile,
} from 'tauri-plugin-printer-v2'
import { open } from '@tauri-apps/plugin-dialog'
import PrintDesigner from './components/PrintDesigner.vue'

const currentTab = ref('printer') // 'printer' | 'designer'

const response = ref('')
const printerName = ref('')
const pdfFilePath = ref('')
const selectedFileName = ref('')
const printersList = ref([])
const selectedPrinter = ref('')

const jobPrinterName = ref('')
const jobsList = ref([])
const jobId = ref('')

const tempFileBase64 = ref('')
const tempFileName = ref('')
const tempFilePath = ref('')
const isPrinting = ref(false)

const MEDICAL_TEMPLATES = [
  { id: 1, name: 'A4 处方笺', icon: '📋', paperSize: 'A4 纵向', options: { pageSize: 'A4', orientation: 'Portrait', margin: { top: 10, bottom: 10, left: 10, right: 10, unit: 'mm' } } },
  { id: 2, name: 'A5 检验报告', icon: '🔬', paperSize: 'A5', options: { pageSize: 'A5', margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' } } },
  { id: 3, name: 'A4 知情同意书', icon: '📝', paperSize: 'A4 横向', options: { pageSize: 'A4', orientation: 'Landscape', margin: { top: 15, bottom: 15, left: 15, right: 15, unit: 'mm' } } },
  { id: 4, name: '58x40mm 腕带标签', icon: '🏷️', paperSize: '58x40mm', options: { pageWidth: 58, pageHeight: 40, margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' } } },
  { id: 5, name: '80x80mm 输液标签', icon: '💉', paperSize: '80x80mm', options: { pageWidth: 80, pageHeight: 80, margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' } } },
  { id: 6, name: '80x297mm 小票', icon: '🧾', paperSize: '80x297mm', options: { pageWidth: 80, pageHeight: 297, margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' } } },
]

const updateResponse = (returnValue) => {
  const timestamp = new Date().toLocaleTimeString()
  const value = typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)
  response.value += `[${timestamp}] ${value}\n`
}

const handlePing = async () => {
  try { const result = await ping('Pong!'); updateResponse(result) } catch (error) { updateResponse(error) }
}

const handleGetPrinters = async () => {
  try {
    const result = await getPrinters()
    updateResponse(`打印机列表: ${JSON.stringify(result)}`)
    try {
      const parsedResult = typeof result === 'string' ? JSON.parse(result) : result
      if (Array.isArray(parsedResult)) {
        printersList.value = parsedResult.map((printer, index) => ({
          id: index + 1,
          name: printer.Name || printer.name || printer,
          status: printer.PrinterStatus || '未知',
          isDefault: printer.isDefault || false,
          driver: printer.DriverName || '未知',
          port: printer.PortName || '未知',
        }))
        updateResponse(`成功解析 ${printersList.value.length} 台打印机`)
      }
    } catch (parseError) {
      const printerNames = (typeof result === 'string' ? result : JSON.stringify(result)).split(',').filter((name) => name.trim())
      printersList.value = printerNames.map((name, index) => ({
        id: index + 1, name: name.trim(), status: '可用', isDefault: index === 0, driver: '未知', port: '未知',
      }))
    }
  } catch (error) {
    updateResponse(`获取打印机列表失败: ${error}`)
    printersList.value = []
  }
}

const handleGetPrinterByName = async () => {
  if (!printerName.value.trim()) { updateResponse('请输入打印机名称'); return }
  try {
    const result = await getPrinterByName(printerName.value.trim())
    updateResponse(`打印机信息 [${printerName.value}]: ${JSON.stringify(result)}`)
  } catch (error) { updateResponse(`获取打印机信息失败 [${printerName.value}]: ${error}`) }
}

const handleSelectPrinter = (printer) => {
  selectedPrinter.value = printer.name
  printerName.value = printer.name
  jobPrinterName.value = printer.name
  updateResponse(`已选择打印机: ${printer.name}`)
}

const handleSelectPdfFile = async () => {
  try {
    const selected = await open({ multiple: false, filters: [{ name: 'PDF文件', extensions: ['pdf'] }] })
    if (selected && typeof selected === 'string') {
      pdfFilePath.value = selected
      const fileName = selected.split('\\').pop() || selected.split('/').pop() || selected
      selectedFileName.value = fileName
      updateResponse(`已选择PDF文件: ${fileName}`)
    } else { updateResponse('未选择文件') }
  } catch (error) { updateResponse(`选择文件失败: ${error}`) }
}

const handlePrintSpecificPdf = async () => {
  if (!pdfFilePath.value.trim()) { updateResponse('请先选择要打印的PDF文件'); return }
  updateResponse(`开始打印PDF文件: ${selectedFileName.value || pdfFilePath.value}`)
  const currentPrinter = selectedPrinter.value || printerName.value.trim()
  if (!currentPrinter) { updateResponse('警告: 未指定打印机，将使用默认打印机') }
  try {
    const printId = `pdf_print_${Date.now()}`
    const result = await printPdf({ id: printId, path: pdfFilePath.value, printerSetting: currentPrinter || 'default', removeAfterPrint: false })
    updateResponse(`PDF打印任务已成功提交: ${result}`)
  } catch (error) { updateResponse(`打印PDF失败: ${error.message || error}`) }
}

const handleGetJobs = async () => {
  const pn = jobPrinterName.value.trim() || selectedPrinter.value || printerName.value.trim()
  if (!pn) { updateResponse('请先选择或输入打印机名称'); return }
  try {
    const result = await getJobs(pn)
    jobsList.value = Array.isArray(result) ? result : []
    updateResponse(`获取到 ${jobsList.value.length} 个打印任务`)
  } catch (error) { updateResponse(`获取打印任务失败: ${error}`); jobsList.value = [] }
}

const handleJobAction = async (action, job) => {
  const pn = jobPrinterName.value.trim() || selectedPrinter.value || printerName.value.trim()
  if (!pn) { updateResponse('请先选择打印机'); return }
  const jid = String(job.id || jobId.value)
  if (!jid) { updateResponse('请指定任务ID'); return }
  try {
    const actions = { resume: resumeJob, restart: restartJob, pause: pauseJob, remove: removeJob }
    await actions[action](pn, jid)
    updateResponse(`任务 ${jid}: ${action} 成功`)
    await handleGetJobs()
  } catch (error) { updateResponse(`任务操作失败: ${error}`) }
}

const handleCreateTempFile = async () => {
  if (!tempFileBase64.value.trim() || !tempFileName.value.trim()) { updateResponse('请输入Base64数据和文件名'); return }
  try {
    const result = await createTempFile(tempFileBase64.value.trim(), tempFileName.value.trim())
    tempFilePath.value = result
    updateResponse(`临时文件已创建: ${result}`)
  } catch (error) { updateResponse(`创建临时文件失败: ${error}`) }
}

const handleRemoveTempFile = async () => {
  const fn = tempFileName.value.trim() || tempFilePath.value
  if (!fn) { updateResponse('请输入文件名'); return }
  try {
    const result = await removeTempFile(fn)
    updateResponse(`删除临时文件: ${result}`)
    tempFilePath.value = ''
  } catch (error) { updateResponse(`删除临时文件失败: ${error}`) }
}

const buildHtml = (title, bodyContent) =>
  '<!DOCTYPE html>\n<html lang="zh-CN">\n<head>\n<meta charset="UTF-8">\n<style>\n' +
  '  * { box-sizing: border-box; margin: 0; padding: 0; }\n' +
  '  body { font-family: "Microsoft YaHei", "SimHei", sans-serif; font-size: 12px; color: #000; }\n' +
  '  .title { text-align: center; font-size: 16px; font-weight: bold; margin-bottom: 8px; }\n' +
  '  .subtitle { text-align: center; font-size: 11px; color: #555; margin-bottom: 10px; }\n' +
  '  table { width: 100%; border-collapse: collapse; margin: 6px 0; }\n' +
  '  th, td { border: 1px solid #000; padding: 4px 6px; text-align: left; font-size: 11px; }\n' +
  '  th { background: #f0f0f0; font-weight: bold; }\n' +
  '  .info-row { display: flex; justify-content: space-between; font-size: 11px; margin: 4px 0; }\n' +
  '  .info-row span { display: inline-block; }\n' +
  '  .signature { margin-top: 24px; display: flex; justify-content: space-between; font-size: 12px; }\n' +
  '  .footer { text-align: center; font-size: 10px; color: #888; margin-top: 16px; }\n' +
  '</style>\n</head>\n<body>\n<h1 class="title">' + title + '</h1>\n' + bodyContent + '\n</body>\n</html>'

const getPrintOptions = (overrides = {}) => {
  const printer = selectedPrinter.value || printerName.value.trim() || undefined
  return { printerId: printer, removeAfterPrint: true, ...overrides }
}

const handlePrintTemplate = async (template) => {
  if (isPrinting.value) return
  isPrinting.value = true
  updateResponse(`开始打印 ${template.name}...`)
  try {
    const html = getTemplateHtml(template.id)
    const result = await printHtml({ html, ...getPrintOptions(template.options) })
    updateResponse(`${template.name} 打印成功: ${result}`)
  } catch (error) { updateResponse(`${template.name} 打印失败: ${error}`) }
  finally { isPrinting.value = false }
}

const T1 = () =>
  '<div class="info-row"><span>姓名：张三</span><span>性别：男</span><span>年龄：45</span><span>科别：内科</span></div>\n' +
  '<div class="info-row"><span>门诊号：MZ2024001234</span><span>日期：2024-07-23</span><span>费别：医保</span></div>\n' +
  '<div class="subtitle">临床诊断：上呼吸道感染</div>\n' +
  '<table>\n  <tr><th>药品名称</th><th>规格</th><th>用量</th><th>用法</th><th>数量</th></tr>\n' +
  '  <tr><td>阿莫西林胶囊</td><td>0.5g</td><td>0.5g</td><td>口服 tid</td><td>24粒</td></tr>\n' +
  '  <tr><td>盐酸氨溴索片</td><td>30mg</td><td>30mg</td><td>口服 tid</td><td>20片</td></tr>\n' +
  '  <tr><td>布洛芬缓释胶囊</td><td>0.3g</td><td>0.3g</td><td>口服 bid</td><td>10粒</td></tr>\n' +
  '</table>\n<div class="signature"><span>医师签名：___________</span><span>药师签名：___________</span><span>金额：86.50</span></div>\n' +
  '<div class="footer">本处方当日有效 | 请遵医嘱用药</div>\n'

const T2 = () =>
  '<div class="info-row"><span>姓名：李四</span><span>性别：女</span><span>年龄：32</span><span>样本编号：S20240723001</span></div>\n' +
  '<div class="info-row"><span>科室：检验科</span><span>送检医生：王医生</span><span>采样时间：2024-07-23 08:30</span></div>\n' +
  '<table>\n  <tr><th>项目名称</th><th>结果</th><th>参考范围</th><th>单位</th><th>标志</th></tr>\n' +
  '  <tr><td>白细胞计数 (WBC)</td><td>6.8</td><td>3.5-9.5</td><td>10^9/L</td><td>正常</td></tr>\n' +
  '  <tr><td>红细胞计数 (RBC)</td><td>4.5</td><td>3.8-5.1</td><td>10^12/L</td><td>正常</td></tr>\n' +
  '  <tr><td>血红蛋白 (HGB)</td><td>135</td><td>115-150</td><td>g/L</td><td>正常</td></tr>\n' +
  '  <tr><td>血小板计数 (PLT)</td><td>220</td><td>125-350</td><td>10^9/L</td><td>正常</td></tr>\n' +
  '  <tr><td>血糖 (GLU)</td><td>5.6</td><td>3.9-6.1</td><td>mmol/L</td><td>正常</td></tr>\n' +
  '</table>\n<div class="signature"><span>检验者：___________</span><span>审核者：___________</span><span>报告时间：2024-07-23 10:15</span></div>\n' +
  '<div class="footer">本报告仅对本次样本负责</div>\n'

const T3 = () =>
  '<div class="info-row"><span>患者姓名：王五</span><span>性别：男</span><span>年龄：58</span><span>住院号：ZY2024005678</span></div>\n' +
  '<div class="info-row"><span>科室：外科</span><span>病区：三病区</span><span>床号：12床</span></div>\n' +
  '<div class="subtitle">手术知情同意书</div>\n' +
  '<div style="margin:12px 0; line-height:1.8; font-size:11px;">\n' +
  '  <p><strong>术前诊断：</strong>急性阑尾炎</p>\n' +
  '  <p><strong>拟行手术：</strong>腹腔镜下阑尾切除术</p>\n' +
  '  <p><strong>麻醉方式：</strong>全身麻醉</p>\n' +
  '  <p style="margin-top:8px;"><strong>手术风险告知：</strong></p>\n' +
  '  <p>1. 麻醉意外，可能出现心脑血管意外等严重并发症。</p>\n' +
  '  <p>2. 术中可能出现大出血、周围脏器损伤等风险。</p>\n' +
  '  <p>3. 术后可能出现切口感染、腹腔感染、肠粘连等并发症。</p>\n' +
  '  <p>4. 术中如发现其他病变，可能需要扩大手术范围。</p>\n' +
  '  <p>5. 其他不可预见的意外情况。</p>\n' +
  '  <p style="margin-top:8px;"><strong>医生已向本人详细解释上述手术相关情况，本人已充分了解手术风险，同意接受手术治疗。</strong></p>\n' +
  '</div>\n' +
  '<div class="signature"><span>患者签名：___________</span><span>家属签名：___________</span><span>与患者关系：___________</span></div>\n' +
  '<div class="signature" style="margin-top:12px;"><span>主治医师签名：___________</span><span>日期：2024年07月23日</span></div>\n'

const T4 = () =>
  '<div style="text-align:center; font-size:10px; margin:2px 0;">\n' +
  '  <div style="font-weight:bold; font-size:13px; margin-bottom:3px;">XX市第一人民医院</div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:3px 0;"><span>姓名：<strong>赵六</strong></span><span>性别：男</span><span>年龄：65</span></div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:3px 0;"><span>住院号：ZY2024009</span><span>床号：05</span></div>\n' +
  '  <div style="margin:3px 0;">科室：心内科</div>\n' +
  '  <div style="border:1px dashed #000; padding:4px; margin:4px 0; font-family:monospace; font-size:9px; letter-spacing:2px;">||| | |||| || | ||||| || ||| ||||</div>\n' +
  '</div>\n'

const T5 = () =>
  '<div style="text-align:center; font-size:10px;">\n' +
  '  <div style="font-weight:bold; font-size:14px; margin-bottom:4px; border-bottom:1px solid #000; padding-bottom:3px;">输液标签</div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:3px 0;"><span>姓名：孙七</span><span>床号：08</span></div>\n' +
  '  <div style="margin:3px 0;">药品：0.9%氯化钠注射液 250ml</div>\n' +
  '  <div style="margin:3px 0;">加药：注射用头孢呋辛钠 1.5g</div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:3px 0;"><span>用法：静脉滴注 bid</span><span>滴速：40滴/分</span></div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:3px 0;"><span>配药时间：07-23 09:00</span><span>护士：___________</span></div>\n' +
  '</div>\n'

const T6 = () =>
  '<div style="font-size:10px;">\n' +
  '  <div style="text-align:center; font-weight:bold; font-size:13px; margin-bottom:4px; border-bottom:1px dashed #000; padding-bottom:3px;">XX市第一人民医院 收费小票</div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:2px 0;"><span>日期：2024-07-23 10:30</span><span>流水号：SF20240723001</span></div>\n' +
  '  <div style="display:flex; justify-content:space-between; margin:2px 0;"><span>姓名：张三</span><span>费别：医保</span></div>\n' +
  '  <div style="border-bottom:1px dashed #000; margin:4px 0;"></div>\n' +
  '  <table style="font-size:9px;">\n' +
  '    <tr><th>项目</th><th>单价</th><th>数量</th><th>金额</th></tr>\n' +
  '    <tr><td>挂号费</td><td>15.00</td><td>1</td><td>15.00</td></tr>\n' +
  '    <tr><td>血常规</td><td>25.00</td><td>1</td><td>25.00</td></tr>\n' +
  '    <tr><td>阿莫西林胶囊</td><td>18.50</td><td>2</td><td>37.00</td></tr>\n' +
  '    <tr><td>输液费</td><td>9.50</td><td>1</td><td>9.50</td></tr>\n' +
  '  </table>\n' +
  '  <div style="border-bottom:1px dashed #000; margin:4px 0;"></div>\n' +
  '  <div style="display:flex; justify-content:space-between; font-weight:bold; font-size:11px;"><span>合计</span><span>86.50</span></div>\n' +
  '  <div style="text-align:center; margin-top:6px; font-size:9px;">医保报销：60.55 | 自费：25.95</div>\n' +
  '  <div style="text-align:center; margin-top:4px; font-size:9px;">请妥善保管，退费凭此小票办理</div>\n' +
  '</div>\n'

const TEMPLATE_HTMLS = { 1: T1, 2: T2, 3: T3, 4: T4, 5: T5, 6: T6 }

const getTemplateHtml = (id) => {
  const fn = TEMPLATE_HTMLS[id]
  return fn ? buildHtml(MEDICAL_TEMPLATES[id - 1].name, fn()) : ''
}
</script>

<template>
  <div class="container">
    <header>
      <h1>Tauri Plugin Printer Demo</h1>
      <p>基于 Vue 3 + Vite 的 Tauri 插件演示应用</p>
    </header>

    <!-- 标签页导航 -->
    <div class="tab-nav">
      <button
        class="tab-btn"
        :class="{ active: currentTab === 'printer' }"
        @click="currentTab = 'printer'"
      >打印机管理</button>
      <button
        class="tab-btn"
        :class="{ active: currentTab === 'designer' }"
        @click="currentTab = 'designer'"
      >排版设计</button>
    </div>

    <!-- 打印机管理 -->
    <main v-if="currentTab === 'printer'" class="desktop-layout">
      <div class="control-panel">
        <div class="section-card">
          <h3>基础功能</h3>
          <div class="button-group vertical">
            <button @click="handlePing" class="action-button">测试 Ping 功能</button>
            <button @click="handleGetPrinters" class="action-button">获取打印机列表</button>
          </div>
        </div>

        <div class="section-card">
          <h3>PDF 文件打印</h3>
          <div class="pdf-section">
            <button @click="handleSelectPdfFile" class="action-button file-select-button">选择 PDF 文件</button>
            <div v-if="selectedFileName" class="selected-file-info">
              <div class="file-icon">📄</div>
              <div class="file-details">
                <div class="file-name">{{ selectedFileName }}</div>
                <div class="file-path">{{ pdfFilePath }}</div>
              </div>
            </div>
            <button @click="handlePrintSpecificPdf" class="action-button pdf-print-button" :disabled="!pdfFilePath">打印选中的PDF</button>
          </div>
        </div>

        <div class="section-card">
          <h3>打印机查询</h3>
          <div class="search-group">
            <input v-model="printerName" type="text" placeholder="请输入打印机名称" class="printer-input" @keyup.enter="handleGetPrinterByName" />
            <button @click="handleGetPrinterByName" class="search-button">查询</button>
          </div>
        </div>

        <div class="section-card">
          <h3>打印任务管理</h3>
          <div class="search-group">
            <input v-model="jobPrinterName" type="text" placeholder="打印机名称" class="printer-input" />
            <button @click="handleGetJobs" class="search-button">获取任务</button>
          </div>
          <div style="margin-top: 8px; display: flex; gap: 6px; flex-wrap: wrap;">
            <input v-model="jobId" type="text" placeholder="任务ID" class="printer-input" style="flex: 1; min-width: 80px;" />
            <button @click="handleJobAction('resume', {})" class="mini-btn" title="恢复">▶</button>
            <button @click="handleJobAction('restart', {})" class="mini-btn" title="重启">↻</button>
            <button @click="handleJobAction('pause', {})" class="mini-btn" title="暂停">⏸</button>
            <button @click="handleJobAction('remove', {})" class="mini-btn mini-btn-danger" title="删除">✕</button>
          </div>
          <div v-if="jobsList.length > 0" style="margin-top: 8px; max-height: 200px; overflow-y: auto;">
            <div v-for="j in jobsList" :key="j.id" class="job-item">
              <span>{{ j.documentName || '未知' }}</span>
              <span style="font-size: 11px; color: #888;">ID: {{ j.id }}</span>
              <div class="job-actions">
                <button @click="handleJobAction('resume', j)" class="mini-btn">▶</button>
                <button @click="handleJobAction('pause', j)" class="mini-btn">⏸</button>
                <button @click="handleJobAction('remove', j)" class="mini-btn mini-btn-danger">✕</button>
              </div>
            </div>
          </div>
        </div>

        <div class="section-card">
          <h3>临时文件管理</h3>
          <div class="search-group">
            <input v-model="tempFileName" type="text" placeholder="文件名" class="printer-input" style="flex: 2;" />
            <input v-model="tempFileBase64" type="text" placeholder="Base64数据" class="printer-input" style="flex: 1;" />
          </div>
          <div style="margin-top: 8px; display: flex; gap: 6px;">
            <button @click="handleCreateTempFile" class="search-button" style="flex: 1;">创建临时文件</button>
            <button @click="handleRemoveTempFile" class="action-button" style="flex: 1; background: #dc3545;">删除临时文件</button>
          </div>
          <div v-if="tempFilePath" style="margin-top: 8px; font-size: 12px; color: #28a745; word-break: break-all;">{{ tempFilePath }}</div>
        </div>
      </div>

      <div class="info-panel">
        <div class="printer-list-section" v-if="printersList.length > 0">
          <h3>可用打印机列表：</h3>
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
                <tr v-for="printer in printersList" :key="printer.id" :class="{ 'selected-row': selectedPrinter === printer.name }">
                  <td>{{ printer.id }}</td>
                  <td class="printer-name">{{ printer.name }}</td>
                  <td>
                    <span class="status-badge" :class="printer.status === '可用' ? 'status-available' : 'status-unknown'">{{ printer.status }}</span>
                  </td>
                  <td><span v-if="printer.isDefault" class="default-badge">✓</span><span v-else>-</span></td>
                  <td class="driver-info">{{ printer.driver }}</td>
                  <td class="port-info">{{ printer.port }}</td>
                  <td>
                    <button @click="handleSelectPrinter(printer)" class="select-button" :class="{ selected: selectedPrinter === printer.name }">
                      {{ selectedPrinter === printer.name ? '已选择' : '选择' }}
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="section-card medical-section">
          <h3>医疗打印测试</h3>
          <div class="medical-grid">
            <div v-for="tmpl in MEDICAL_TEMPLATES" :key="tmpl.id" class="medical-card">
              <div class="medical-icon">{{ tmpl.icon }}</div>
              <div class="medical-name">{{ tmpl.name }}</div>
              <span class="paper-badge">{{ tmpl.paperSize }}</span>
               <button @click="handlePrintTemplate(tmpl)" :disabled="isPrinting" class="medical-print-btn">打印</button>
            </div>
          </div>
        </div>

        <div class="response-area">
          <h3>响应日志</h3>
          <pre>{{ response || '点击按钮测试插件功能...' }}</pre>
        </div>
      </div>
    </main>

    <!-- 排版设计 -->
    <main v-if="currentTab === 'designer'" class="designer-layout">
      <PrintDesigner />
    </main>
  </div>
</template>

<style scoped>
.container { max-width: 1400px; margin: 0 auto; padding: 2rem; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; }
header { text-align: center; margin-bottom: 1.5rem; }
header h1 { color: #2c3e50; margin-bottom: 0.5rem; font-size: 2.5rem; }
header p { color: #7f8c8d; font-size: 1.1rem; }

/* 标签页导航 */
.tab-nav {
  display: flex; gap: 0; margin-bottom: 1.5rem;
  border-bottom: 2px solid #e1e8ed;
}
.tab-btn {
  padding: 12px 28px; border: none; background: transparent;
  font-size: 1rem; font-weight: 600; color: #7f8c8d;
  cursor: pointer; transition: all 0.2s; border-bottom: 3px solid transparent;
  margin-bottom: -2px;
}
.tab-btn:hover { color: #2c3e50; }
.tab-btn.active { color: #667eea; border-bottom-color: #667eea; }

.designer-layout { height: calc(100vh - 220px); min-height: 600px; }

.desktop-layout { display: grid; grid-template-columns: 400px 1fr; gap: 2rem; min-height: 80vh; }
.control-panel { display: flex; flex-direction: column; gap: 1.5rem; }

.section-card { background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08); border: 1px solid #e1e8ed; }
.section-card h3 { margin: 0 0 1rem 0; color: #2c3e50; font-size: 1.1rem; font-weight: 600; }

.info-panel { display: flex; flex-direction: column; gap: 1.5rem; }
.button-group { display: flex; gap: 1rem; flex-wrap: wrap; }
.button-group.vertical { flex-direction: column; gap: 0.75rem; }

.pdf-section { display: flex; flex-direction: column; gap: 1rem; }
.selected-file-info { display: flex; align-items: center; gap: 1rem; padding: 1rem; background: #f8f9fa; border-radius: 8px; border: 2px dashed #dee2e6; }
.file-icon { font-size: 2rem; }
.file-details { flex: 1; min-width: 0; }
.file-name { font-weight: 600; color: #2c3e50; margin-bottom: 0.25rem; word-break: break-word; }
.file-path { font-size: 0.85rem; color: #6c757d; word-break: break-all; font-family: 'Courier New', monospace; }

@media (max-width: 1024px) {
  .desktop-layout { grid-template-columns: 1fr; gap: 1.5rem; }
  .container { padding: 1rem; }
  header h1 { font-size: 2rem; }
}

.action-button {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white; border: none; padding: 12px 20px; border-radius: 8px; font-size: 0.95rem;
  font-weight: 600; cursor: pointer; transition: all 0.3s ease; width: 100%;
  text-align: center; display: flex; align-items: center; justify-content: center; gap: 0.5rem;
}
.action-button:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3); }
.action-button:disabled { background: #6c757d; cursor: not-allowed; opacity: 0.6; }
.action-button:disabled:hover { transform: none; box-shadow: none; }

.file-select-button { background: linear-gradient(135deg, #17a2b8 0%, #138496 100%) !important; }
.file-select-button:hover:not(:disabled) { box-shadow: 0 8px 25px rgba(23, 162, 184, 0.3) !important; }
.pdf-print-button { background: linear-gradient(135deg, #ffa726 0%, #ff7043 100%) !important; }
.pdf-print-button:hover { box-shadow: 0 8px 25px rgba(255, 167, 38, 0.3) !important; }

.printer-list-section { background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08); border: 1px solid #e1e8ed; }
.printer-list-section h3 { margin: 0 0 1rem 0; color: #2c3e50; font-size: 1.1rem; font-weight: 600; }

.current-printer { margin-bottom: 1rem; padding: 0.75rem 1rem; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border-radius: 6px; display: flex; align-items: center; gap: 0.5rem; }
.current-label { font-weight: 600; }
.current-name { font-weight: 700; background: rgba(255, 255, 255, 0.2); padding: 0.25rem 0.5rem; border-radius: 4px; }

.table-container { overflow-x: auto; border-radius: 8px; border: 1px solid #e1e8ed; }
.printers-table { width: 100%; border-collapse: collapse; background: white; }
.printers-table th { background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%); color: #495057; font-weight: 600; padding: 1rem 0.75rem; text-align: left; border-bottom: 2px solid #dee2e6; font-size: 0.9rem; }
.printers-table td { padding: 0.75rem; border-bottom: 1px solid #f1f3f4; vertical-align: middle; }
.printers-table tbody tr { transition: all 0.2s ease; }
.printers-table tbody tr:hover { background: #f8f9fa; }

.selected-row { background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%) !important; border-left: 4px solid #667eea; }
.printer-name { font-weight: 600; color: #2c3e50; max-width: 200px; word-break: break-word; }
.status-badge { padding: 0.25rem 0.5rem; border-radius: 12px; font-size: 0.8rem; font-weight: 600; text-transform: uppercase; }
.status-available { background: #d4edda; color: #155724; }
.status-unknown { background: #f8d7da; color: #721c24; }
.default-badge { color: #28a745; font-weight: bold; font-size: 1.2rem; }
.driver-info, .port-info { color: #6c757d; font-size: 0.9rem; max-width: 120px; word-break: break-word; }

.select-button { background: linear-gradient(135deg, #28a745 0%, #20c997 100%); color: white; border: none; padding: 0.5rem 1rem; border-radius: 6px; font-size: 0.9rem; font-weight: 600; cursor: pointer; transition: all 0.3s ease; min-width: 70px; }
.select-button:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3); }
.select-button.selected { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); cursor: default; }
.select-button.selected:hover { transform: none; box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3); }

.search-group { display: flex; gap: 0.75rem; align-items: center; flex-wrap: wrap; }
.printer-input { flex: 1; min-width: 200px; padding: 10px 14px; border: 2px solid #e1e8ed; border-radius: 6px; font-size: 0.95rem; transition: border-color 0.3s ease; }
.printer-input:focus { outline: none; border-color: #667eea; box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1); }
.search-button { background: linear-gradient(135deg, #28a745 0%, #20c997 100%); color: white; border: none; padding: 10px 16px; border-radius: 6px; font-size: 0.95rem; font-weight: 600; cursor: pointer; transition: all 0.3s ease; white-space: nowrap; }
.search-button:hover { transform: translateY(-1px); box-shadow: 0 6px 20px rgba(40, 167, 69, 0.3); }

.mini-btn { background: #667eea; color: white; border: none; padding: 6px 10px; border-radius: 4px; font-size: 0.8rem; cursor: pointer; transition: all 0.2s; }
.mini-btn:hover { background: #5a6fd6; }
.mini-btn-danger { background: #dc3545; }
.mini-btn-danger:hover { background: #c82333; }

.job-item { display: flex; align-items: center; justify-content: space-between; padding: 8px; border-bottom: 1px solid #eee; font-size: 13px; gap: 8px; }
.job-actions { display: flex; gap: 4px; }

.response-area { background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08); border: 1px solid #e1e8ed; flex: 1; }
.response-area h3 { margin: 0 0 1rem 0; color: #2c3e50; font-size: 1.1rem; font-weight: 600; }
.response-area pre { background: #f8f9fa; padding: 1.5rem; border-radius: 8px; border-left: 4px solid #667eea; font-family: 'Consolas', 'Monaco', 'Courier New', monospace; white-space: pre-wrap; word-wrap: break-word; color: #2c3e50; margin: 0; min-height: 300px; max-height: 500px; overflow-y: auto; font-size: 0.9rem; line-height: 1.5; }

.medical-section { border: 2px solid #e8f5e9; }
.medical-section h3 { color: #2e7d32; }
.medical-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.medical-card { background: #f5fdf6; border: 1px solid #c8e6c9; border-radius: 10px; padding: 16px 12px; display: flex; flex-direction: column; align-items: center; gap: 8px; transition: all 0.2s; }
.medical-card:hover { transform: translateY(-2px); box-shadow: 0 6px 16px rgba(46, 125, 50, 0.15); border-color: #66bb6a; }
.medical-icon { font-size: 2rem; }
.medical-name { font-size: 0.9rem; font-weight: 600; color: #2c3e50; text-align: center; }
.paper-badge { background: #e8f5e9; color: #2e7d32; font-size: 0.7rem; padding: 2px 8px; border-radius: 10px; font-weight: 500; }
.medical-print-btn { background: linear-gradient(135deg, #43a047 0%, #2e7d32 100%); color: white; border: none; padding: 8px 24px; border-radius: 6px; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s; width: 100%; }
.medical-print-btn:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(46, 125, 50, 0.3); }
.medical-print-btn:disabled { background: #6c757d; cursor: not-allowed; opacity: 0.6; transform: none; box-shadow: none; }

@media (max-width: 1024px) {
  .medical-grid { grid-template-columns: repeat(2, 1fr); }
}
</style>