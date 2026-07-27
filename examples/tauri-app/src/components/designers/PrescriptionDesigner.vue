<script setup>
import { ref, onMounted, nextTick } from "vue";
import { getPrinters, printHtml } from "tauri-plugin-printer-v2";
const { hiprint, defaultElementTypeProvider } = window["vue-plugin-hiprint"];

const PAPER_WIDTH = 148;
const PAPER_HEIGHT = 210;

const isPrinting = ref(false);
const statusMsg = ref("初始化中...");
const printerList = ref([]);
const selectedPrinter = ref("");
const copies = ref(1);
const grayscale = ref(false);

let hiprintTemplate = null;

const rxData = {
  name: "张三", gender: "男", age: "45", department: "内科",
  visitNo: "MZ2024001234", date: "2024-07-23", feeType: "医保",
  diagnosis: "上呼吸道感染合并细菌感染",
  medicines: [
    { name: "阿莫西林胶囊", spec: "0.5g", dosage: "0.5g", usage: "口服 tid", qty: "24粒", price: "18.50" },
    { name: "盐酸氨溴索片", spec: "30mg", dosage: "30mg", usage: "口服 tid", qty: "20片", price: "12.00" },
    { name: "布洛芬缓释胶囊", spec: "0.3g", dosage: "0.3g", usage: "口服 bid", qty: "10粒", price: "15.00" },
    { name: "复方甘草片", spec: "100mg", dosage: "100mg", usage: "口服 tid", qty: "30片", price: "8.50" },
    { name: "维生素C片", spec: "100mg", dosage: "100mg", usage: "口服 tid", qty: "30片", price: "3.00" },
    { name: "头孢克洛胶囊", spec: "0.25g", dosage: "0.25g", usage: "口服 tid", qty: "18粒", price: "22.00" },
    { name: "奥美拉唑肠溶胶囊", spec: "20mg", dosage: "20mg", usage: "口服 qd", qty: "14粒", price: "25.00" },
    { name: "二甲双胍片", spec: "0.5g", dosage: "0.5g", usage: "口服 tid", qty: "42片", price: "9.50" },
    { name: "阿托伐他汀钙片", spec: "10mg", dosage: "10mg", usage: "口服 qd", qty: "28片", price: "35.00" },
    { name: "氯雷他定片", spec: "10mg", dosage: "10mg", usage: "口服 qd", qty: "10片", price: "16.00" },
    { name: "蒙脱石散", spec: "3g", dosage: "3g", usage: "口服 tid", qty: "18袋", price: "14.50" },
    { name: "对乙酰氨基酚片", spec: "500mg", dosage: "500mg", usage: "口服 prn", qty: "12片", price: "6.50" },
    { name: "左氧氟沙星片", spec: "0.5g", dosage: "0.5g", usage: "口服 qd", qty: "14片", price: "28.00" },
    { name: "阿奇霉素片", spec: "0.25g", dosage: "0.5g", usage: "口服 qd", qty: "6片", price: "32.00" },
    { name: "硝苯地平控释片", spec: "30mg", dosage: "30mg", usage: "口服 qd", qty: "28片", price: "42.00" },
    { name: "氨氯地平片", spec: "5mg", dosage: "5mg", usage: "口服 qd", qty: "28片", price: "19.00" },
    { name: "美托洛尔缓释片", spec: "47.5mg", dosage: "47.5mg", usage: "口服 qd", qty: "28片", price: "22.50" },
    { name: "福辛普利钠片", spec: "10mg", dosage: "10mg", usage: "口服 qd", qty: "28片", price: "31.00" },
    { name: "氢氯噻嗪片", spec: "25mg", dosage: "25mg", usage: "口服 qd", qty: "28片", price: "5.00" },
    { name: "螺内酯片", spec: "20mg", dosage: "20mg", usage: "口服 qd", qty: "28片", price: "12.00" },
    { name: "瑞格列奈片", spec: "0.5mg", dosage: "0.5mg", usage: "口服 ac", qty: "60片", price: "38.00" },
    { name: "阿卡波糖片", spec: "50mg", dosage: "50mg", usage: "口服 ac", qty: "42片", price: "45.00" },
    { name: "格列美脲片", spec: "2mg", dosage: "2mg", usage: "口服 qd", qty: "28片", price: "16.50" },
    { name: "吡格列酮片", spec: "30mg", dosage: "30mg", usage: "口服 qd", qty: "28片", price: "38.00" },
  ],
  total: "501.50",
  doctor: "李医生", pharmacist: "王药师",
  hospitalName: "XX市第一人民医院",
};

const rxPanel = {
  panels: [{
    index: 0, height: PAPER_HEIGHT, width: PAPER_WIDTH,
    paperHeader: 35, paperFooter: 550,
    printElements: [
      { options: { left: 40, top: 8, height: 22, width: 330, title: "XX市第一人民医院 处方笺", fontSize: 16, fontWeight: "700", textAlign: "center" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 20, top: 34, height: 8, width: 370 }, printElementType: { type: "hline" } },
      { options: { left: 20, top: 44, height: 14, width: 370, title: "姓名: 张三    性别: 男    年龄: 45    科别: 内科", fontSize: 10, textAlign: "left", fontFamily: "Microsoft YaHei" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 20, top: 60, height: 14, width: 370, title: "门诊号: MZ2024001234    日期: 2024-07-23    费别: 医保", fontSize: 10, textAlign: "left", fontFamily: "Microsoft YaHei" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 20, top: 76, height: 14, width: 370, title: "临床诊断: 上呼吸道感染合并细菌感染", fontSize: 10, textAlign: "left", fontFamily: "Microsoft YaHei" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 20, top: 94, height: 8, width: 370 }, printElementType: { type: "hline" } },
      { options: { left: 20, top: 106, height: 380, width: 370, field: "medicines", fields: [{ text: "药品名称", field: "name" }, { text: "规格", field: "spec" }, { text: "用量", field: "dosage" }, { text: "用法", field: "usage" }, { text: "数量", field: "qty" }, { text: "单价", field: "price" }], columns: [[{ title: "药品名称", field: "name", width: 80, align: "left" }, { title: "规格", field: "spec", width: 50, align: "center" }, { title: "用量", field: "dosage", width: 50, align: "center" }, { title: "用法", field: "usage", width: 65, align: "center" }, { title: "数量", field: "qty", width: 45, align: "center" }, { title: "单价", field: "price", width: 50, align: "right" }]] }, printElementType: { title: "表格", type: "table", editable: true, columnDisplayEditable: true, columnTitleEditable: true, columnResizable: true } },
      { options: { left: 20, top: 494, height: 8, width: 370 }, printElementType: { type: "hline" } },
      { options: { left: 250, top: 506, height: 14, width: 140, title: "金额合计: 501.50 元", fontSize: 11, fontWeight: "700", textAlign: "right", fontFamily: "Microsoft YaHei" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 20, top: 524, height: 14, width: 370, title: "医师签名: 李医生        药师签名: 王药师", fontSize: 10, textAlign: "left", fontFamily: "Microsoft YaHei" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 20, top: 558, height: 12, width: 370, title: "本处方当日有效  |  请遵医嘱用药  |  XX市第一人民医院", fontSize: 9, textAlign: "center", color: "#888", fontFamily: "Microsoft YaHei" }, printElementType: { title: "自定义文本", type: "text" } },
    ],
    paperNumberLeft: 300, paperNumberTop: 580
  }]
};

const initDesigner = () => {
  try {
    hiprint.init({ providers: [new defaultElementTypeProvider()] });
    hiprint.PrintElementTypeManager.buildByHtml(window.$(".ep-draggable-item"));
    hiprintTemplate = new hiprint.PrintTemplate({
      template: rxPanel,
      settingContainer: "#RxElementOptionSetting",
      paginationContainer: "#RxPagination",
      dataMode: 1, history: true,
      onDataChanged: (type) => { statusMsg.value = "模板已修改 (" + type + ")"; },
    });
    hiprintTemplate.design("#RxPrintTemplate");
    statusMsg.value = "处方设计器就绪，A4 两页打印，含页眉页脚及页码";
  } catch (e) { statusMsg.value = "初始化失败: " + e.message; console.error(e); }
};

// 图片转 Base64（隐藏 WebView 窗口无法加载网络/鉴权图片）
const ensureImagesBase64 = async (html) => {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, "text/html");
  const imgs = doc.querySelectorAll("img");
  if (imgs.length === 0) return html;
  const toBase64 = (src) => new Promise((resolve) => {
    if (src.startsWith("data:") || src.startsWith("blob:")) { resolve(src); return; }
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      try {
        const canvas = document.createElement("canvas");
        canvas.width = img.naturalWidth;
        canvas.height = img.naturalHeight;
        canvas.getContext("2d").drawImage(img, 0, 0);
        resolve(canvas.toDataURL("image/png"));
      } catch { resolve(src); }
    };
    img.onerror = () => resolve(src);
    img.src = src;
  });
  const promises = Array.from(imgs).map(async (img) => {
    const src = img.getAttribute("src");
    if (!src) return;
    img.setAttribute("src", await toBase64(src));
  });
  await Promise.all(promises);
  return doc.documentElement.outerHTML;
};

const handlePrint = async () => {
  if (!hiprintTemplate || isPrinting.value) return;
  isPrinting.value = true; statusMsg.value = "正在生成打印内容...";
  try {
    const htmlResult = hiprintTemplate.getHtml(rxData);
    if (!htmlResult || !htmlResult.length) throw new Error("请先添加打印元素");
    const htmlContent = htmlResult.html();
    let fullHtml = '<!DOCTYPE html>\n<html lang="zh-CN">\n<head>\n<meta charset="UTF-8">\n<style>\n  * { box-sizing: border-box; margin: 0; padding: 0; }\n  body { font-family: "Microsoft YaHei", "SimHei", sans-serif; }\n  @page { size: ' + PAPER_WIDTH + 'mm ' + PAPER_HEIGHT + 'mm; margin: 0; }\n</style>\n</head>\n<body>' + htmlContent + '</body>\n</html>';
    statusMsg.value = "正在处理图片资源...";
    fullHtml = await ensureImagesBase64(fullHtml);
    statusMsg.value = "正在发送到打印机...";
    const result = await printHtml({
      html: fullHtml, pageWidth: PAPER_WIDTH, pageHeight: PAPER_HEIGHT,
      orientation: "Portrait",
      margin: { top: 0, bottom: 0, left: 0, right: 0, unit: "mm" },
      printerId: selectedPrinter.value || undefined,
      removeAfterPrint: true,
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
  hiprintTemplate.print(rxData, {}, {
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
  <div class="rx-designer">
    <div class="designer-toolbar">
      <div class="toolbar-group">
        <span class="toolbar-label">纸张:</span>
        <span class="paper-info">A5 (148x210mm) 处方笺 - 24个药品自动分页</span>
      </div>
      <div class="toolbar-group">
        <span class="toolbar-label">打印机:</span>
        <select v-model="selectedPrinter" class="printer-select">
          <option value="" disabled>请选择打印机</option>
          <option v-for="p in printerList" :key="p.name" :value="p.name">{{ p.name }}</option>
        </select>
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
        <div id="RxPrintTemplate" class="hiprint-template"></div>
      </div>
      <div class="settings-sidebar">
        <div class="sidebar-title">元素属性</div>
        <div id="RxElementOptionSetting" class="settings-content">
          <div class="settings-hint">选中设计面板中的元素，在此编辑属性</div>
        </div>
      </div>
    </div>
    <div id="RxPagination" class="hiprint-pagination"></div>
    <div class="designer-status">{{ statusMsg }}</div>
  </div>
</template>

<style scoped>
.rx-designer { display: flex; flex-direction: column; height: 100%; background: #f0f2f5; border-radius: 8px; overflow: hidden; }
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
