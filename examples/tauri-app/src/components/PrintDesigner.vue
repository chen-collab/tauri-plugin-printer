<script setup>
import { ref, onMounted, nextTick, computed } from "vue";
import { printHtml } from "tauri-plugin-printer-v2";
// vue-plugin-hiprint 已在 index.html 中以全局脚本加载，直接使用 window.hiprint
const { hiprint, defaultElementTypeProvider } = window["vue-plugin-hiprint"];
import panel from "./panel.js";
import printData from "./print-data.js";

// ========== 纸张类型配置 ==========
const PAPER_TYPES = {
  "A4 纵向": { width: 210, height: 297 },
  "A4 横向": { width: 297, height: 210 },
  "A5": { width: 148, height: 210 },
  "B5": { width: 176, height: 250 },
  "58x40mm": { width: 58, height: 40 },
  "80x80mm": { width: 80, height: 80 },
  "80x297mm": { width: 80, height: 297 },
};

// ========== 状态 ==========
const curPaper = ref({ type: "A4 纵向", width: 210, height: 297 });
const scaleValue = ref(1);
const scaleMax = 5;
const scaleMin = 0.5;
const isPrinting = ref(false);
const statusMsg = ref("初始化中...");

let hiprintTemplate = null;

// 当前纸张类型名称
const curPaperType = computed(() => {
  let type = "自定义";
  const types = PAPER_TYPES;
  for (const key in types) {
    const item = types[key];
    if (item.width === curPaper.value.width && item.height === curPaper.value.height) {
      type = key;
      break;
    }
  }
  return type;
});

// ========== 初始化 ==========
const initDesigner = () => {
  try {
    hiprint.init({ providers: [new defaultElementTypeProvider()] });
    // 自定义配置
    hiprint.setConfig({
      movingDistance: 2.5,
      text: {
        supportOptions: [
          { name: "styler", hidden: true },
          { name: "formatter", hidden: true },
        ],
      },
    });
// 构建可拖拽元素（必须使用 jQuery 选择器，hiprint 内部依赖 jQuery 方法）
    hiprint.PrintElementTypeManager.buildByHtml(window.$(".ep-draggable-item"));
    // 创建模板
    hiprintTemplate = new hiprint.PrintTemplate({
      template: panel,
      settingContainer: "#PrintElementOptionSetting",
      paginationContainer: "#hiprintPagination",
      dataMode: 1,
      history: true,
      onDataChanged: (type) => {
        statusMsg.value = "模板已修改 (" + type + ")";
      },
    });
    hiprintTemplate.design("#hiprint-printTemplate");
    scaleValue.value = hiprintTemplate.editingPanel ? hiprintTemplate.editingPanel.scale || 1 : 1;
    statusMsg.value = "设计器就绪，拖拽元素到面板中开始设计";
  } catch (e) {
    statusMsg.value = "初始化失败: " + e.message;
    console.error(e);
  }
};

// ========== 纸张切换 ==========
const setPaper = (type, value) => {
  try {
    curPaper.value = { type, width: value.width, height: value.height };
    hiprintTemplate.setPaper(value.width, value.height);
    statusMsg.value = "已切换到 " + type + " (" + value.width + "x" + value.height + "mm)";
  } catch (error) {
    statusMsg.value = "切换纸张失败: " + error;
  }
};

// ========== 缩放 ==========
const changeScale = (big) => {
  let sv = scaleValue.value;
  if (big) {
    sv += 0.1;
    if (sv > scaleMax) sv = 5;
  } else {
    sv -= 0.1;
    if (sv < scaleMin) sv = 0.5;
  }
  if (hiprintTemplate) {
    hiprintTemplate.zoom(sv);
    scaleValue.value = sv;
  }
};

// ========== 打印（接入 Tauri 插件） ==========
const handlePrint = async () => {
  if (!hiprintTemplate || isPrinting.value) return;
  isPrinting.value = true;
  statusMsg.value = "正在生成打印内容...";
  try {
    const paper = curPaper.value;
// 从 hiprint 获取渲染后的 HTML（返回 jQuery 对象，包裹 <div class="hiprint-printTemplate">）
    const htmlResult = hiprintTemplate.getHtml(printData);
    if (!htmlResult || !htmlResult.length) throw new Error("请先添加打印元素");
    // 获取面板渲染的 HTML 内容（jQuery 对象的 .html() 返回内部 HTML 字符串）
    const htmlContent = htmlResult.html();
    // 构建完整 HTML 文档
    const fullHtml = "<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n<meta charset=\"UTF-8\">\n<style>\n  * { box-sizing: border-box; margin: 0; padding: 0; }\n  body { font-family: \"Microsoft YaHei\", \"SimHei\", sans-serif; }\n  @page { size: " + paper.width + "mm " + paper.height + "mm; margin: 0; }\n</style>\n</head>\n<body>" + htmlContent + "</body>\n</html>";
    statusMsg.value = "正在发送到打印机...";
    // 调用 Tauri 原生打印
    const result = await printHtml({
      html: fullHtml,
      pageWidth: paper.width,
      pageHeight: paper.height,
      orientation: paper.width > paper.height ? "Landscape" : "Portrait",
      margin: { top: 0, bottom: 0, left: 0, right: 0, unit: "mm" },
      removeAfterPrint: true,
    });
    statusMsg.value = "打印成功: " + result;
  } catch (error) {
    statusMsg.value = "打印失败: " + (error.message || error);
  } finally {
    isPrinting.value = false;
  }
};

// ========== 预览 ==========
const handlePreview = () => {
  if (!hiprintTemplate) return;
  statusMsg.value = "正在打开预览...";
  hiprintTemplate.print(printData, {}, {
    callback: () => { statusMsg.value = "预览窗口已打开"; },
    styleHandler: () => {
      return '<link href="/print-lock.css" media="print" rel="stylesheet">';
    },
  });
};

// ========== 清空 ==========
const handleClear = () => {
  try {
    hiprintTemplate.clear();
    statusMsg.value = "模板已清空";
  } catch (error) {
    statusMsg.value = "清空失败: " + error;
  }
};

// ========== 模板保存/加载 ==========
const handleSave = () => {
  if (!hiprintTemplate) return;
  const json = hiprintTemplate.getJson();
  const saveData = { paper: curPaper.value, template: json };
  localStorage.setItem("hiprint_medical_template", JSON.stringify(saveData));
  statusMsg.value = "模板已保存到本地";
};

const handleLoad = () => {
  const saved = localStorage.getItem("hiprint_medical_template");
  if (!saved) { statusMsg.value = "没有已保存的模板"; return; }
  try {
    const { paper: savedPaper, template } = JSON.parse(saved);
    if (savedPaper) {
      curPaper.value = savedPaper;
      hiprintTemplate.setPaper(savedPaper.width, savedPaper.height);
    }
    hiprintTemplate.update(template);
    statusMsg.value = "模板已加载: " + (savedPaper ? savedPaper.type : "未知");
  } catch (e) {
    statusMsg.value = "加载模板失败: " + e.message;
  }
};

onMounted(() => {
  nextTick(() => { initDesigner(); });
});
</script>
<template>
  <div class="print-designer">
    <!-- 顶部工具栏 -->
    <div class="designer-toolbar">
      <!-- 纸张选择 -->
      <div class="toolbar-group">
        <span class="toolbar-label">纸张:</span>
        <button
          v-for="(value, type) in PAPER_TYPES"
          :key="type"
          :class="['paper-btn', { active: curPaperType === type }]"
          @click="setPaper(type, value)"
        >{{ type }}</button>
      </div>

      <!-- 缩放 -->
      <div class="toolbar-group">
        <button class="action-btn" @click="changeScale(false)" title="缩小">-</button>
        <span class="scale-display">{{ (scaleValue * 100).toFixed(0) }}%</span>
        <button class="action-btn" @click="changeScale(true)" title="放大">+</button>
      </div>

      <!-- 操作按钮 -->
      <div class="toolbar-group toolbar-actions">
        <button class="action-btn btn-preview" @click="handlePreview">预览</button>
        <button class="action-btn btn-print" @click="handlePrint" :disabled="isPrinting">
          {{ isPrinting ? "打印中..." : "打印" }}
        </button>
        <button class="action-btn btn-save" @click="handleSave">保存</button>
        <button class="action-btn btn-load" @click="handleLoad">加载</button>
        <button class="action-btn btn-clear" @click="handleClear">清空</button>
      </div>
    </div>

    <!-- 主区域 -->
    <div class="designer-main">
      <!-- 左侧：可拖拽元素面板 -->
      <div class="elements-sidebar">
        <div class="sidebar-title">拖拽组件</div>
        <div class="drag-grid">
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.text">
              <span class="drag-icon">T</span>
              <span class="drag-label">文本</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.image">
              <span class="drag-icon">图</span>
              <span class="drag-label">图片</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.longText">
              <span class="drag-icon">文</span>
              <span class="drag-label">长文</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.tableCustom">
              <span class="drag-icon">表</span>
              <span class="drag-label">表格</span>
            </a>
          </div>
        </div>
        <div class="sidebar-title">辅助</div>
        <div class="drag-grid">
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.hline">
              <span class="drag-icon">-</span>
              <span class="drag-label">横线</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.vline">
              <span class="drag-icon">|</span>
              <span class="drag-label">竖线</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.rect">
              <span class="drag-icon">口</span>
              <span class="drag-label">矩形</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.oval">
              <span class="drag-icon">O</span>
              <span class="drag-label">椭圆</span>
            </a>
          </div>
        </div>
        <div class="sidebar-title">条码</div>
        <div class="drag-grid">
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.barcode">
              <span class="drag-icon">条</span>
              <span class="drag-label">条形码</span>
            </a>
          </div>
          <div class="drag-item">
            <a class="ep-draggable-item" tid="defaultModule.qrcode">
              <span class="drag-icon">码</span>
              <span class="drag-label">二维码</span>
            </a>
          </div>
        </div>
      </div>

      <!-- 中间：设计画布 -->
      <div class="design-canvas">
        <div id="hiprint-printTemplate" class="hiprint-template"></div>
      </div>

      <!-- 右侧：元素属性面板 -->
      <div class="settings-sidebar">
        <div class="sidebar-title">元素属性</div>
        <div id="PrintElementOptionSetting" class="settings-content">
          <div class="settings-hint">选中设计面板中的元素，在此编辑属性</div>
        </div>
      </div>
    </div>

    <!-- 分页容器 -->
    <div id="hiprintPagination" class="hiprint-pagination"></div>

    <!-- 状态栏 -->
    <div class="designer-status">{{ statusMsg }}</div>
  </div>
</template>
<style scoped>
.print-designer {
  display: flex; flex-direction: column; height: 100%;
  background: #f0f2f5; border-radius: 8px; overflow: hidden;
}

/* 工具栏 */
.designer-toolbar {
  display: flex; align-items: center; gap: 16px;
  padding: 8px 16px; background: #fff; border-bottom: 1px solid #e0e0e0;
  flex-wrap: wrap;
}
.toolbar-group { display: flex; align-items: center; gap: 6px; }
.toolbar-label { font-size: 13px; font-weight: 600; color: #555; margin-right: 4px; }
.toolbar-actions { margin-left: auto; }

.paper-btn {
  padding: 4px 10px; border: 1px solid #d9d9d9; border-radius: 4px;
  background: #fff; font-size: 12px; cursor: pointer; transition: all 0.15s;
  white-space: nowrap;
}
.paper-btn:hover { border-color: #667eea; color: #667eea; }
.paper-btn.active { background: #667eea; color: #fff; border-color: #667eea; }

.action-btn {
  padding: 5px 14px; border: 1px solid #d9d9d9; border-radius: 4px;
  background: #fff; font-size: 13px; cursor: pointer; transition: all 0.15s;
}
.action-btn:hover { border-color: #667eea; color: #667eea; }
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-preview { background: #f0f5ff; border-color: #adc6ff; color: #2f54eb; }
.btn-print { background: #f6ffed; border-color: #b7eb8f; color: #52c41a; }
.btn-print:disabled { background: #f5f5f5; }
.btn-save { background: #fff7e6; border-color: #ffd591; color: #fa8c16; }
.btn-load { background: #e6fffb; border-color: #87e8de; color: #13c2c2; }
.btn-clear { background: #fff1f0; border-color: #ffa39e; color: #f5222d; }

.scale-display {
  display: inline-block; width: 50px; text-align: center;
  font-size: 13px; font-weight: 600; color: #333;
}

/* 主区域 */
.designer-main {
  display: flex; flex: 1; overflow: hidden;
}

/* 左侧元素面板 */
.elements-sidebar {
  width: 140px; min-width: 140px; background: #fff;
  border-right: 1px solid #e0e0e0; overflow-y: auto; padding: 10px;
}
.sidebar-title {
  font-size: 13px; font-weight: 700; color: #333;
  padding: 8px 0 6px 0; border-bottom: 1px solid #f0f0f0; margin-bottom: 6px;
}
.drag-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-bottom: 8px; }
.drag-item { }
.drag-item > a {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 8px 4px; background: #fafafa; border: 1px solid #e8e8e8;
  border-radius: 4px; cursor: grab; text-decoration: none; transition: all 0.15s;
  height: 56px;
}
.drag-item > a:hover { background: #e6f7ff; border-color: #91d5ff; }
.drag-item > a:active { cursor: grabbing; }
.drag-icon { font-size: 18px; font-weight: 700; color: #667eea; margin-bottom: 2px; }
.drag-label { font-size: 11px; color: #555; }

/* 设计画布 */
.design-canvas {
  flex: 1; overflow: auto; background: #e8e8e8;
  padding: 16px; display: flex; justify-content: center;
}
.hiprint-template { min-height: 400px; }

/* 右侧属性面板 */
.settings-sidebar {
  width: 260px; min-width: 260px; background: #fff;
  border-left: 1px solid #e0e0e0; overflow-y: auto; padding: 10px;
}
.settings-content { }
.settings-hint {
  color: #aaa; font-size: 13px; text-align: center; margin-top: 20px;
}

/* 分页容器 */
.hiprint-pagination {
  background: #fafafa; border-top: 1px solid #e0e0e0;
  padding: 4px 12px; min-height: 26px;
}

/* 状态栏 */
.designer-status {
  padding: 5px 16px; background: #001529; color: #a0d8ef;
  font-size: 12px;
}
</style>
