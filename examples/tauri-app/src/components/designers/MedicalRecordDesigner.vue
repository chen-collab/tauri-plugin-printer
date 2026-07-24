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

let hiprintTemplate = null;

const recordData = {
  name: "张三", gender: "男", age: "45", department: "内科",
  inNo: "ZY2024005678", bedNo: "12床", admitDate: "2024-07-20",
  chiefComplaint: "发热、咳嗽、咽痛3天",
  presentIllness: "患者3天前受凉后出现发热，体温最高38.5C，伴咳嗽、咽痛、鼻塞。自行服用退热药后体温可暂时下降，但反复发热。今来我院就诊。",
  pastHistory: "既往体健，否认高血压、糖尿病等慢性病史，否认药物过敏史。",
  physicalExam: "T: 38.2C, P: 88次/分, R: 20次/分, BP: 125/80mmHg。咽部充血，扁桃体II度肿大。",
  labResults: [
    { name: "白细胞计数(WBC)", result: "12.5", range: "3.5-9.5", unit: "10^9/L", flag: "↑" },
    { name: "中性粒细胞比例", result: "78.5", range: "40-75", unit: "%", flag: "↑" },
    { name: "C反应蛋白(CRP)", result: "35", range: "0-10", unit: "mg/L", flag: "↑" },
    { name: "降钙素原(PCT)", result: "0.25", range: "<0.5", unit: "ng/mL", flag: "正常" },
  ],
  diagnosis: "急性上呼吸道感染",
  treatment: "1. 头孢呋辛酯片 0.25g 口服 bid\n2. 复方氨酚烷胺胶囊 1粒 口服 tid\n3. 布洛芬缓释胶囊 0.3g 口服 必要时\n4. 建议多饮水、休息，如症状加重及时复诊",
  doctor: "李医生",
  date: "2024-07-23",
  hospitalName: "XX市第一人民医院",
};

const recordPanel = {
  panels: [{
    index: 0, height: PAPER_HEIGHT, width: PAPER_WIDTH,
    paperHeader: 15, paperFooter: 550,
    printElements: [
      { options: { left: 5, top: 5, height: 18, width: 350, title: "XX市第一人民医院 门(急)诊病历", fontSize: 13, fontWeight: "700", textAlign: "center" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 28, height: 6, width: 350 }, printElementType: { type: "hline" } },
      { options: { left: 5, top: 38, height: 14, width: 350, title: "姓名: 张三    性别: 男    年龄: 45", fontSize: 9, textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 54, height: 14, width: 350, title: "科别: 内科    住院号: ZY2024005678    床号: 12床", fontSize: 9, textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 70, height: 14, width: 350, title: "入院日期: 2024-07-20    记录日期: 2024-07-23", fontSize: 9, textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 88, height: 6, width: 350 }, printElementType: { type: "hline" } },
      { options: { left: 5, top: 98, height: 14, width: 350, title: "主诉: 发热、咳嗽、咽痛3天", fontSize: 10, fontWeight: "600", textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 115, height: 40, width: 350, title: "现病史: 患者3天前受凉后出现发热，体温最高38.5C，伴咳嗽、咽痛、鼻塞。", fontSize: 9, textAlign: "left", lineHeight: 1.6 }, printElementType: { title: "自定义文本", type: "longText" } },
      { options: { left: 5, top: 160, height: 30, width: 350, title: "既往史: 既往体健，否认高血压、糖尿病等慢性病史。", fontSize: 9, textAlign: "left", lineHeight: 1.6 }, printElementType: { title: "自定义文本", type: "longText" } },
      { options: { left: 5, top: 195, height: 6, width: 350 }, printElementType: { type: "hline" } },
      { options: { left: 5, top: 205, height: 14, width: 350, title: "体格检查", fontSize: 10, fontWeight: "600", textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 222, height: 40, width: 350, title: "T: 38.2C, P: 88次/分, R: 20次/分, BP: 125/80mmHg。咽部充血，扁桃体II度肿大。", fontSize: 9, textAlign: "left", lineHeight: 1.6 }, printElementType: { title: "自定义文本", type: "longText" } },
      { options: { left: 5, top: 268, height: 6, width: 350 }, printElementType: { type: "hline" } },
      { options: { left: 5, top: 278, height: 14, width: 350, title: "辅助检查", fontSize: 10, fontWeight: "600", textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 296, height: 80, width: 350, field: "labResults", fields: [{ text: "项目", field: "name" }, { text: "结果", field: "result" }, { text: "参考", field: "range" }, { text: "单位", field: "unit" }, { text: "标志", field: "flag" }], columns: [[{ title: "项目", field: "name", width: 100, align: "left" }, { title: "结果", field: "result", width: 50, align: "center" }, { title: "参考范围", field: "range", width: 70, align: "left" }, { title: "单位", field: "unit", width: 70, align: "left" }, { title: "标志", field: "flag", width: 30, align: "center" }]] }, printElementType: { title: "表格", type: "table", editable: true, columnDisplayEditable: true, columnTitleEditable: true, columnResizable: true } },
      { options: { left: 5, top: 384, height: 6, width: 350 }, printElementType: { type: "hline" } },
      { options: { left: 5, top: 394, height: 14, width: 350, title: "诊断: 急性上呼吸道感染", fontSize: 10, fontWeight: "600", textAlign: "left" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { left: 5, top: 412, height: 50, width: 350, title: "治疗意见: 1.头孢呋辛酯片 0.25g 口服 bid 2.布洛芬缓释胶囊 0.3g 口服 必要时", fontSize: 9, textAlign: "left", lineHeight: 1.6 }, printElementType: { title: "自定义文本", type: "longText" } },
      { options: { left: 5, top: 468, height: 14, width: 350, title: "医师签名: 李医生", fontSize: 9, textAlign: "right" }, printElementType: { title: "自定义文本", type: "text" } },
    ],
    paperNumberLeft: 280, paperNumberTop: 540
  }]
};

const initDesigner = () => {
  try {
    hiprint.init({ providers: [new defaultElementTypeProvider()] });
    hiprint.PrintElementTypeManager.buildByHtml(window.$(".ep-draggable-item"));
    hiprintTemplate = new hiprint.PrintTemplate({
      template: recordPanel,
      settingContainer: "#RecordElementOptionSetting",
      paginationContainer: "#RecordPagination",
      dataMode: 1, history: true,
      onDataChanged: (type) => { statusMsg.value = "模板已修改 (" + type + ")"; },
    });
    hiprintTemplate.design("#RecordPrintTemplate");
    statusMsg.value = "病历设计器就绪，A5 多页模拟";
  } catch (e) { statusMsg.value = "初始化失败: " + e.message; console.error(e); }
};

const handlePrint = async () => {
  if (!hiprintTemplate || isPrinting.value) return;
  isPrinting.value = true; statusMsg.value = "正在生成打印内容...";
  try {
    const htmlResult = hiprintTemplate.getHtml(recordData);
    if (!htmlResult || !htmlResult.length) throw new Error("请先添加打印元素");
    const htmlContent = htmlResult.html();
    const fullHtml = '<!DOCTYPE html>\n<html lang="zh-CN">\n<head>\n<meta charset="UTF-8">\n<style>\n  * { box-sizing: border-box; margin: 0; padding: 0; }\n  body { font-family: "Microsoft YaHei", "SimHei", sans-serif; }\n  @page { size: ' + PAPER_WIDTH + 'mm ' + PAPER_HEIGHT + 'mm; margin: 0; }\n</style>\n</head>\n<body>' + htmlContent + '</body>\n</html>';
    statusMsg.value = "正在发送到打印机...";
    const result = await printHtml({
      html: fullHtml, pageWidth: PAPER_WIDTH, pageHeight: PAPER_HEIGHT,
      orientation: "Portrait",
      margin: { top: 0, bottom: 0, left: 0, right: 0, unit: "mm" },
      printerId: selectedPrinter.value || undefined,
      removeAfterPrint: true,
    });
    statusMsg.value = "打印成功: " + result;
  } catch (error) { statusMsg.value = "打印失败: " + (error.message || error); }
  finally { isPrinting.value = false; }
};

const handlePreview = () => {
  if (!hiprintTemplate) return;
  statusMsg.value = "正在打开预览...";
  hiprintTemplate.print(recordData, {}, {
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
  <div class="record-designer">
    <div class="designer-toolbar">
      <div class="toolbar-group">
        <span class="toolbar-label">纸张:</span>
        <span class="paper-info">A5 (148x210mm) 多页病历</span>
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
        <div id="RecordPrintTemplate" class="hiprint-template"></div>
      </div>
      <div class="settings-sidebar">
        <div class="sidebar-title">元素属性</div>
        <div id="RecordElementOptionSetting" class="settings-content">
          <div class="settings-hint">选中设计面板中的元素，在此编辑属性</div>
        </div>
      </div>
    </div>
    <div id="RecordPagination" class="hiprint-pagination"></div>
    <div class="designer-status">{{ statusMsg }}</div>
  </div>
</template>

<style scoped>
.record-designer { display: flex; flex-direction: column; height: 100%; background: #f0f2f5; border-radius: 8px; overflow: hidden; }
.designer-toolbar { display: flex; align-items: center; gap: 16px; padding: 8px 16px; background: #fff; border-bottom: 1px solid #e0e0e0; flex-wrap: wrap; }
.toolbar-group { display: flex; align-items: center; gap: 6px; }
.toolbar-label { font-size: 13px; font-weight: 600; color: #555; margin-right: 4px; }
.toolbar-actions { margin-left: auto; }
.paper-info { font-size: 12px; color: #333; font-weight: 600; }
.printer-select { padding: 4px 10px; border: 1px solid #d9d9d9; border-radius: 4px; background: #fff; font-size: 12px; cursor: pointer; max-width: 220px; outline: none; }
.printer-select:hover { border-color: #667eea; }
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
