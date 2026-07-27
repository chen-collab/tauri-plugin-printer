<script setup>
import { ref, onMounted, nextTick, computed } from "vue";
import { getPrinters, printTemplate } from "tauri-plugin-printer-v2";
const { hiprint, defaultElementTypeProvider } = window["vue-plugin-hiprint"];

const PAPER_WIDTH = 80;
const PAPER_HEIGHT = 500; // 设计画布高度（足够大），打印时用 printHeight

const isPrinting = ref(false);
const statusMsg = ref("初始化中...");
const printerList = ref([]);
const selectedPrinter = ref("");
const medicineCount = ref(4);
const copies = ref(1);
const grayscale = ref(false);

let hiprintTemplate = null;

// 随机药品名称池
const MEDICINE_NAMES = [
  "阿莫西林胶囊", "头孢克洛胶囊", "布洛芬缓释胶囊", "维生素C片",
  "复方甘草片", "盐酸氨溴索片", "奥美拉唑肠溶胶囊", "二甲双胍片",
  "阿托伐他汀钙片", "氯雷他定片", "蒙脱石散", "对乙酰氨基酚片",
  "左氧氟沙星片", "阿奇霉素片", "硝苯地平控释片", "氨氯地平片",
];

// 随机生成药品列表
const generateItems = (count) => {
  const items = [];
  for (let i = 0; i < count; i++) {
    const name = MEDICINE_NAMES[i % MEDICINE_NAMES.length];
    const price = (Math.random() * 45 + 5).toFixed(2);
    const qty = Math.floor(Math.random() * 4) + 1;
    items.push({ name, price, qty: String(qty), amount: (price * qty).toFixed(2) });
  }
  return items;
};

// 动态计算打印数据
const receiptData = computed(() => {
  const items = generateItems(medicineCount.value);
  const total = items.reduce((sum, item) => sum + parseFloat(item.amount), 0).toFixed(2);
  const insurance = (parseFloat(total) * 0.7).toFixed(2);
  const selfPay = (parseFloat(total) - parseFloat(insurance)).toFixed(2);
  return {
    hospitalName: "XX市第一人民医院",
    date: "2024-07-23",
    serialNo: "SF20240723001",
    name: "张三",
    feeType: "医保",
    items,
    total,
    insurance,
    selfPay,
  };
});

// 根据药品数量自动计算打印高度 (mm)
const printHeight = computed(() => {
  const count = medicineCount.value || 1;
  return Math.max(90 + count * 6, 60); // 基础 90mm, 每行 6mm, 最小 60mm
});

const receiptPanel = {
  panels: [{
    index: 0, height: PAPER_HEIGHT, width: PAPER_WIDTH,
    paperHeader: 10, paperFooter: 1300,
    printElements: [
      { options: { left: 2, top: 8, height: 20, width: 190, title: "XX市第一人民医院", fontSize: 14, fontWeight: "700", textAlign: "center" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 2, top: 26, height: 16, width: 190, title: "收费小票", fontSize: 11, fontWeight: "600", textAlign: "center", color: "#555" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 2, top: 44, height: 6, width: 190 }, printElementType: { type: "hline" } },
      { options: { left: 2, top: 52, height: 12, width: 190, title: "日期：2024-07-23    流水号：SF20240723001", fontSize: 9, textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 2, top: 66, height: 12, width: 190, title: "姓名：张三    费别：医保", fontSize: 9, textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 2, top: 80, height: 6, width: 190 }, printElementType: { type: "hline" } },
      { options: { left: 2, top: 88, height: 900, width: 190, field: "items", fields: [{ text: "项目", field: "name" }, { text: "单价", field: "price" }, { text: "数量", field: "qty" }, { text: "金额", field: "amount" }], columns: [[{ title: "项目", field: "name", width: 70, align: "left" }, { title: "单价", field: "price", width: 40, align: "right" }, { title: "数量", field: "qty", width: 30, align: "center" }, { title: "金额", field: "amount", width: 50, align: "right" }]] }, printElementType: { title: "表格", type: "table", editable: true, columnDisplayEditable: true, columnTitleEditable: true, columnResizable: true } },
      { options: { left: 2, top: 996, height: 6, width: 190 }, printElementType: { type: "hline" } },
      { options: { left: 2, top: 1004, height: 14, width: 190, title: "合计：0 元", fontSize: 11, fontWeight: "700", textAlign: "right" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 2, top: 1020, height: 12, width: 190, title: "医保报销：0 元    自费：0 元", fontSize: 8, textAlign: "center", color: "#666" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 2, top: 1040, height: 12, width: 190, title: "请妥善保管，退费凭此小票办理", fontSize: 8, textAlign: "center", color: "#999" }, printElementType: { title: "自定义文本", type: "text" } },
    ],
    paperNumberLeft: 140, paperNumberTop: 1400
  }]
};

const initDesigner = () => {
  try {
    hiprint.init({ providers: [new defaultElementTypeProvider()] });
    hiprint.PrintElementTypeManager.buildByHtml(window.$(".ep-draggable-item"));
    hiprintTemplate = new hiprint.PrintTemplate({
      template: receiptPanel,
      settingContainer: "#ReceiptElementOptionSetting",
      paginationContainer: "#ReceiptPagination",
      dataMode: 1, history: true,
      onDataChanged: (type) => { statusMsg.value = "模板已修改 (" + type + ")"; },
    });
    hiprintTemplate.design("#ReceiptPrintTemplate");
    statusMsg.value = "结算小票设计器就绪，80mm 宽";
  } catch (e) { statusMsg.value = "初始化失败: " + e.message; console.error(e); }
};

const handlePrint = async () => {
  if (!hiprintTemplate || isPrinting.value) return;
  isPrinting.value = true; statusMsg.value = "正在获取模板...";
  try {
    const data = receiptData.value;
    const height = printHeight.value;
    const templateJson = JSON.stringify(hiprintTemplate.getJson());
    const dataJson = JSON.stringify(data);
    statusMsg.value = "正在发送到打印引擎...";
    const result = await printTemplate({
      template: templateJson,
      data: dataJson,
      paperWidth: PAPER_WIDTH,
      paperHeight: height,
      orientation: "Portrait",
      printerId: selectedPrinter.value || undefined,
      copies: copies.value,
      grayscale: grayscale.value,
    });
    statusMsg.value = "打印成功: " + result;
  } catch (error) { statusMsg.value = "打印失败: " + (error.message || error); }
  finally { isPrinting.value = false; }
};

const handlePreview = () => {
  if (!hiprintTemplate) return;
  statusMsg.value = "正在打开预览...";
  hiprintTemplate.print(receiptData.value, {}, {
    callback: () => { statusMsg.value = "预览窗口已打开"; },
    styleHandler: () => '<link href="/print-lock.css" media="print" rel="stylesheet">',
  });
};

const loadPrinters = async () => {
  try {
    const list = await getPrinters();
    printerList.value = list;
    if (list.length > 0) selectedPrinter.value = list[0].name;
  } catch (error) { console.error("获取打印机列表失败:", error); }
};

onMounted(() => { nextTick(() => { initDesigner(); }); loadPrinters(); });
</script>

<template>
  <div class="receipt-designer">
    <div class="designer-toolbar">
<div class="toolbar-group">
        <span class="toolbar-label">纸张:</span>
        <span class="paper-info">80mm × {{ printHeight }}mm</span>
      </div>
      <div class="toolbar-group">
        <span class="toolbar-label">药品:</span>
        <input v-model.number="medicineCount" type="number" min="1" max="100" class="count-input" @input="medicineCount = Math.max(1, Math.min(100, medicineCount || 1))" />
        <span class="count-label">个</span>
      </div>
      <div class="toolbar-group">
        <span class="toolbar-label">份数:</span>
        <input v-model.number="copies" type="number" min="1" max="99" class="count-input" />
      </div>
      <div class="toolbar-group">
        <label class="grayscale-toggle">
          <input v-model="grayscale" type="checkbox" />
          <span>灰度</span>
        </label>
      </div>
      <div class="toolbar-group">
        <span class="toolbar-label">打印机:</span>
        <select v-model="selectedPrinter" class="printer-select">
          <option value="" disabled>请选择打印机</option>
          <option v-for="p in printerList" :key="p.name" :value="p.name">{{ p.name }}</option>
        </select>
      </div>
      <div class="toolbar-group toolbar-actions">
        <button class="action-btn btn-preview" @click="handlePreview">预览</button>
        <button class="action-btn btn-print" @click="handlePrint" :disabled="isPrinting">{{ isPrinting ? "打印中..." : "打印" }}</button>
      </div>
    </div>
    <div class="designer-main">
      <div class="elements-sidebar">
        <div class="sidebar-title">拖拽组件</div>
        <div class="drag-grid">
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.text"><span class="drag-icon">T</span><span class="drag-label">文本</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.image"><span class="drag-icon">图</span><span class="drag-label">图片</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.longText"><span class="drag-icon">文</span><span class="drag-label">长文</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.tableCustom"><span class="drag-icon">表</span><span class="drag-label">表格</span></a></div>
        </div>
        <div class="sidebar-title">辅助</div>
        <div class="drag-grid">
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.hline"><span class="drag-icon">-</span><span class="drag-label">横线</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.vline"><span class="drag-icon">|</span><span class="drag-label">竖线</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.rect"><span class="drag-icon">口</span><span class="drag-label">矩形</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.oval"><span class="drag-icon">O</span><span class="drag-label">椭圆</span></a></div>
        </div>
        <div class="sidebar-title">条码</div>
        <div class="drag-grid">
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.barcode"><span class="drag-icon">条</span><span class="drag-label">条形码</span></a></div>
          <div class="drag-item"><a class="ep-draggable-item" tid="defaultModule.qrcode"><span class="drag-icon">码</span><span class="drag-label">二维码</span></a></div>
        </div>
      </div>
      <div class="design-canvas">
        <div id="ReceiptPrintTemplate" class="hiprint-template"></div>
      </div>
      <div class="settings-sidebar">
        <div class="sidebar-title">元素属性</div>
        <div id="ReceiptElementOptionSetting" class="settings-content">
          <div class="settings-hint">选中设计面板中的元素，在此编辑属性</div>
        </div>
      </div>
    </div>
    <div id="ReceiptPagination" class="hiprint-pagination"></div>
    <div class="designer-status">{{ statusMsg }}</div>
  </div>
</template>

<style scoped>
.receipt-designer { display: flex; flex-direction: column; height: 100%; background: #f0f2f5; border-radius: 8px; overflow: hidden; }
.designer-toolbar { display: flex; align-items: center; gap: 16px; padding: 8px 16px; background: #fff; border-bottom: 1px solid #e0e0e0; flex-wrap: wrap; }
.toolbar-group { display: flex; align-items: center; gap: 6px; }
.toolbar-label { font-size: 13px; font-weight: 600; color: #555; margin-right: 4px; }
.toolbar-actions { margin-left: auto; }
.paper-info { font-size: 12px; color: #333; font-weight: 600; }
.printer-select { padding: 4px 10px; border: 1px solid #d9d9d9; border-radius: 4px; background: #fff; font-size: 12px; cursor: pointer; max-width: 220px; outline: none; }
.printer-select:hover { border-color: #667eea; }
.count-input { width: 56px; padding: 3px 6px; border: 1px solid #d9d9d9; border-radius: 4px; font-size: 12px; text-align: center; outline: none; -moz-appearance: textfield; transition: border-color 0.15s; }
.count-input::-webkit-inner-spin-button, .count-input::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
.count-input:hover { border-color: #667eea; }
.count-input:focus { border-color: #667eea; box-shadow: 0 0 0 2px rgba(102,126,234,0.15); }
.count-label { font-size: 11px; color: #999; }
.grayscale-toggle { display: flex; align-items: center; gap: 6px; font-size: 12px; color: #555; cursor: pointer; user-select: none; }
.grayscale-toggle input { cursor: pointer; }
.action-btn { padding: 5px 14px; border: 1px solid #d9d9d9; border-radius: 4px; background: #fff; font-size: 13px; cursor: pointer; transition: all 0.15s; }
.action-btn:hover { border-color: #667eea; color: #667eea; }
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-preview { background: #f0f5ff; border-color: #adc6ff; color: #2f54eb; }
.btn-print { background: #f6ffed; border-color: #b7eb8f; color: #52c41a; }
.btn-print:disabled { background: #f5f5f5; }
.designer-main { display: flex; flex: 1; overflow: hidden; }
.elements-sidebar { width: 140px; min-width: 140px; background: #fff; border-right: 1px solid #e0e0e0; overflow-y: auto; padding: 10px; }
.sidebar-title { font-size: 13px; font-weight: 700; color: #333; padding: 8px 0 6px 0; border-bottom: 1px solid #f0f0f0; margin-bottom: 6px; }
.drag-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-bottom: 8px; }
.drag-item > a { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 8px 4px; background: #fafafa; border: 1px solid #e8e8e8; border-radius: 4px; cursor: grab; text-decoration: none; transition: all 0.15s; height: 56px; }
.drag-item > a:hover { background: #e6f7ff; border-color: #91d5ff; }
.drag-icon { font-size: 18px; font-weight: 700; color: #667eea; margin-bottom: 2px; }
.drag-label { font-size: 11px; color: #555; }
.design-canvas { flex: 1; overflow: auto; background: #e8e8e8; padding: 16px; display: flex; justify-content: center; }
.hiprint-template { min-height: 400px; }
.settings-sidebar { width: 260px; min-width: 260px; background: #fff; border-left: 1px solid #e0e0e0; overflow-y: auto; padding: 10px; }
.settings-hint { color: #aaa; font-size: 13px; text-align: center; margin-top: 20px; }
.hiprint-pagination { background: #fafafa; border-top: 1px solid #e0e0e0; padding: 4px 12px; min-height: 26px; }
.designer-status { padding: 5px 16px; background: #001529; color: #a0d8ef; font-size: 12px; }
</style>
