<script setup>
import { ref, onMounted, nextTick, computed } from "vue";
import { getPrinters, printTemplate } from "tauri-plugin-printer-v2";
const { hiprint, defaultElementTypeProvider } = window["vue-plugin-hiprint"];

const PAPER_WIDTH = 48;    // 48mm 窄幅热敏纸（取药/发药小票）
const DESIGN_HEIGHT = 100; // 设计画布默认高度 100mm（仅用于可视化编辑；打印时高度随内容自动撑开）

// 表格与页脚布局参数（mm，与用户 panel 一致）
const TABLE_TOP = 30;          // 药品列表顶部（紧跟第二条分隔线 line_2）
const HEADER_TR_H = 4.5;       // 表头行高
const ROW_H = 4.2;             // 每行药品行高
const DEFAULT_DRUG_COUNT = 5;  // 设计画布默认展示的药品行数
const DESIGN_TABLE_H = HEADER_TR_H + DEFAULT_DRUG_COUNT * ROW_H; // 设计态表格高度 ≈ 25.5
const DESIGN_FOOTER_BASE = TABLE_TOP + DESIGN_TABLE_H;           // 页脚区顶部（紧跟药品列表）

// 页脚 4 个元素相对“表格底部”的偏移（与用户 panel 的 +N 一致）
// line_3(+2) / total_fee(+4) / pay_fee(+9) / remark(+14)
const FOOTER_OFFSETS = [2, 4, 9, 14];

const isPrinting = ref(false);
const statusMsg = ref("初始化中...");
const printerList = ref([]);
const selectedPrinter = ref("");
const drugCount = ref(5);
const copies = ref(1);
const grayscale = ref(false);

let hiprintTemplate = null;

// 中医门诊示例药品池
const SAMPLE_DRUGS = [
  { drugName: "黄芪颗粒", spec: "10g*10袋", num: "2盒", price: "45.00" },
  { drugName: "当归补血口服液", spec: "10ml*10支", num: "3盒", price: "68.00" },
  { drugName: "六味地黄丸", spec: "200丸", num: "1瓶", price: "32.00" },
  { drugName: "三七粉", spec: "3g*20袋", num: "1盒", price: "88.00" },
  { drugName: "金银花露", spec: "250ml", num: "2瓶", price: "26.00" },
  { drugName: "补中益气丸", spec: "200丸", num: "1瓶", price: "29.00" },
  { drugName: "板蓝根颗粒", spec: "10g*20袋", num: "1盒", price: "18.50" },
];

// 动态计算打印数据（字段与用户 panel 的占位符一一对应）
const dispenseData = computed(() => {
  const drugs = [];
  for (let i = 0; i < drugCount.value; i++) {
    const d = SAMPLE_DRUGS[i % SAMPLE_DRUGS.length];
    drugs.push({ drugName: d.drugName, spec: d.spec, num: d.num, price: d.price });
  }
  const total = drugs.reduce((sum, d) => sum + parseFloat(d.price), 0).toFixed(2);
  return {
    patientName: "张伟",
    sex: "男",
    age: "35",
    dept: "中医内科",
    doctorName: "李时珍",
    visitId: "MZ20260727001",
    visitDate: "2026-07-27",
    drugs,
    totalAmount: total,
    payAmount: total,
    remark: "请遵医嘱按时服用，忌生冷辛辣。",
  };
});

// 取药/发药小票模板（由用户提供的 panel 转换：type/options/id → hiprint 标准 printElementType）
const dispensePanel = {
  panels: [{
    index: 0, height: DESIGN_HEIGHT, width: PAPER_WIDTH,
    paperHeader: 0, paperFooter: 0,
    panelPageRule: "none", // 不分页模式：热敏连续纸由内容撑开高度
    printElements: [
      // 眉栏
      { options: { id: "title_hospital", left: 4, top: 3, height: 6, width: 40, title: "伊森中医门诊", fontSize: 14, fontWeight: "bold", textAlign: "center" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "line_1", left: 2, top: 11, height: 1, width: 44 }, printElementType: { type: "hline" } },
      { options: { id: "patient_name", left: 3, top: 13, height: 4.5, width: 20, title: "姓名：{{patientName}}", fontSize: 9 }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "patient_sex_age", left: 24, top: 13, height: 4.5, width: 21, title: "性别：{{sex}} 年龄：{{age}}", fontSize: 9 }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "patient_dept_doctor", left: 3, top: 18, height: 4.5, width: 42, title: "科室：{{dept}} 接诊医生：{{doctorName}}", fontSize: 9 }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "visit_no", left: 3, top: 23, height: 4.5, width: 42, title: "就诊单号：{{visitId}} 日期：{{visitDate}}", fontSize: 9 }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "line_2", left: 2, top: 28.5, height: 1, width: 44 }, printElementType: { type: "hline" } },
      // 药品列表（高度随药品数量自适应，打印时由 measureReflow 重写）
      {
        options: {
          id: "drug_table",
          left: 2, top: TABLE_TOP, height: DESIGN_TABLE_H, width: 44,
          field: "drugs",
          autoExpand: true,
          fontSize: 8.5,
          trHeight: 4.2,
          headerTrHeight: 4.5,
          borderWidth: 0.5,
          fields: [
            { text: "药品名称", field: "drugName" },
            { text: "规格", field: "spec" },
            { text: "数量", field: "num" },
            { text: "单价", field: "price" },
          ],
          columns: [[
            { title: "药品名称", field: "drugName", width: 22, align: "left" },
            { title: "规格", field: "spec", width: 8, align: "center" },
            { title: "数量", field: "num", width: 6, align: "center" },
            { title: "单价", field: "price", width: 8, align: "right" },
          ]],
        },
        printElementType: { title: "表格", type: "table", editable: true, columnDisplayEditable: true, columnTitleEditable: true, columnResizable: true },
      },
      // 页脚区（默认紧跟药品列表底部；打印时按实际行数由 measureReflow 重排）
      { options: { id: "line_3", left: 2, top: DESIGN_FOOTER_BASE + 2, height: 1, width: 44 }, printElementType: { type: "hline" } },
      { options: { id: "total_fee", left: 3, top: DESIGN_FOOTER_BASE + 4, height: 5, width: 42, title: "应收总金额：￥{{totalAmount}}", fontSize: 10, fontWeight: "bold", textAlign: "right" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "pay_fee", left: 3, top: DESIGN_FOOTER_BASE + 9, height: 4.5, width: 42, title: "实付金额：￥{{payAmount}}", fontSize: 9, textAlign: "right" }, printElementType: { title: "自定义文本", type: "text" } },
      { options: { id: "remark", left: 3, top: DESIGN_FOOTER_BASE + 14, height: 4, width: 42, title: "备注：{{remark}}", fontSize: 8 }, printElementType: { title: "自定义文本", type: "text" } },
    ],
  }],
};

// 构建“打印用”基础模板：开启不分页模式，表格 box 高度先置 0（真实高度由 measureReflow 量算回填）
const buildPrintTemplate = () => {
  const tpl = JSON.parse(JSON.stringify(dispensePanel));
  const panel = tpl.panels[0];
  panel.panelPageRule = "none";
  panel.paperFooter = 0;
  const table = panel.printElements.find((e) => e.printElementType && e.printElementType.type === "table");
  if (table) table.options.height = 0;
  return tpl;
};

// 离屏渲染模板、量取表格真实渲染高度，把页脚区（模板末尾 4 个元素）重排到表格正下方，
// 回填真实表格高度与页脚 top。返回修正后的模板交给引擎渲染。
const measureReflow = (tpl, data) => {
  try {
    const widthPx = (PAPER_WIDTH * 96) / 25.4; // 与引擎渲染宽度保持一致（96DPI）
    const holder = document.createElement("div");
    holder.style.cssText = "position:absolute;left:-99999px;top:0;visibility:hidden;width:" + widthPx + "px;";
    document.body.appendChild(holder);

    const t = new hiprint.PrintTemplate({ template: tpl });
    const $html = t.getHtml(data);
    const paperDom = $html && $html[0] ? $html[0] : $html;
    holder.appendChild(paperDom);

    const tableTarget = paperDom.querySelector(".hiprint-printElement-tableTarget");
    if (tableTarget) {
      const wrapper = tableTarget.closest(".hiprint-printElement") || tableTarget.parentElement;
      wrapper.style.height = "auto";
      wrapper.style.overflow = "visible";
      const tableTop = parseFloat(wrapper.style.top) || wrapper.offsetTop || TABLE_TOP;
      const realTableH = tableTarget.getBoundingClientRect().height;
      const tableBottom = tableTop + realTableH;

      // 页脚区 = 模板末尾 4 个元素，紧跟表格底部重排（与设计态相对间距一致）
      const footerEls = tpl.panels[0].printElements.slice(-4);
      const tops = FOOTER_OFFSETS.map((o) => Math.round(tableBottom + o));
      footerEls.forEach((el, i) => { if (el && el.options) el.options.top = tops[i]; });

      // 回填真实表格高度（+2 安全余量，避免 wrapper overflow:hidden 裁掉末行）
      const tableEl = tpl.panels[0].printElements.find((e) => e.printElementType && e.printElementType.type === "table");
      if (tableEl) tableEl.options.height = Math.round(realTableH) + 2;
    }
    document.body.removeChild(holder);
  } catch (e) {
    console.warn("measureReflow 失败，回退为估算布局:", e);
  }
  return tpl;
};

const initDesigner = () => {
  try {
    hiprint.init({ providers: [new defaultElementTypeProvider()] });
    hiprint.PrintElementTypeManager.buildByHtml(window.$(".ep-draggable-item"));
    hiprintTemplate = new hiprint.PrintTemplate({
      template: dispensePanel,
      settingContainer: "#DispenseElementOptionSetting",
      paginationContainer: "#DispensePagination",
      dataMode: 1, history: true,
      onDataChanged: (type) => { statusMsg.value = "模板已修改 (" + type + ")"; },
    });
    hiprintTemplate.design("#DispensePrintTemplate");
    statusMsg.value = "取药小票设计器就绪，48mm 宽";
  } catch (e) { statusMsg.value = "初始化失败: " + e.message; console.error(e); }
};

const handlePrint = async () => {
  if (!hiprintTemplate || isPrinting.value) return;
  isPrinting.value = true; statusMsg.value = "正在获取模板...";
  try {
    const data = dispenseData.value;
    // 离屏量取真实表格高度并重排页脚，再交给引擎按内容自动计算高度
    const printTpl = measureReflow(buildPrintTemplate(), data);
    const templateJson = JSON.stringify(printTpl);
    const dataJson = JSON.stringify(data);
    statusMsg.value = "正在发送到打印引擎...";
    const result = await printTemplate({
      template: templateJson,
      data: dataJson,
      paperWidth: PAPER_WIDTH,
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
  // 预览与打印一致：按当前药品数量离屏量取真实高度并重排页脚，页脚紧接列表、高度自适应
  const previewTpl = measureReflow(buildPrintTemplate(), dispenseData.value);
  const previewTemplate = new hiprint.PrintTemplate({ template: previewTpl, dataMode: 1, history: false });
  statusMsg.value = "正在打开预览...";
  previewTemplate.print(dispenseData.value, {}, {
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
  <div class="dispense-designer">
    <div class="designer-toolbar">
      <div class="toolbar-group">
        <span class="toolbar-label">纸张:</span>
        <span class="paper-info">48mm · 高度自适应</span>
      </div>
      <div class="toolbar-group">
        <span class="toolbar-label">药品:</span>
        <input v-model.number="drugCount" type="number" min="1" max="100" class="count-input" @input="drugCount = Math.max(1, Math.min(100, drugCount || 1))" />
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
        <div id="DispensePrintTemplate" class="hiprint-template"></div>
      </div>
      <div class="settings-sidebar">
        <div class="sidebar-title">元素属性</div>
        <div id="DispenseElementOptionSetting" class="settings-content">
          <div class="settings-hint">选中设计面板中的元素，在此编辑属性</div>
        </div>
      </div>
    </div>
    <div id="DispensePagination" class="hiprint-pagination"></div>
    <div class="designer-status">{{ statusMsg }}</div>
  </div>
</template>

<style scoped>
.dispense-designer { display: flex; flex-direction: column; height: 100%; background: #f0f2f5; border-radius: 8px; overflow: hidden; }
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
