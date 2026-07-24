<template>
  <a-modal :visible="visible" :maskClosable="false"
           @cancel="hideModal" :width="width+'mm'">
    <a-spin :spinning="spinning" style="min-height: 100px">
      <div id="preview_content_design"></div>
    </a-spin>
    <template slot="title">
      <a-space>
        <div style="margin-right: 20px">打印预览</div>
        <a-button :loading="waitShowPrinter" type="primary" icon="printer" @click.stop="print">打印</a-button>
        <a-button type="primary" icon="printer" @click.stop="toPdf">pdf</a-button>
      </a-space>
    </template>
    <template slot="footer">
      <a-button key="close" type="info" @click="hideModal">
        关闭
      </a-button>
    </template>
  </a-modal>
</template>

<script>
export default {
  name: "printPreview",
  props: {},
  data() {
    return {
      visible: false,
      spinning: true,
      waitShowPrinter: false,
      // 纸张宽 mm
      width: 0,
      // 模板
      hiprintTemplate: {},
      // 数据
      printData: {}
    }
  },
  computed: {
    isTauri() {
      return !!(window.__TAURI__ || window.__TAURI_INTERNALS__)
    }
  },
  watch: {},
  created() {
  },
  mounted() {
  },
  methods: {
    hideModal() {
      this.visible = false
    },
    show(hiprintTemplate, printData, width = '210') {
      this.visible = true
      this.spinning = true
      this.width = hiprintTemplate.editingPanel ? hiprintTemplate.editingPanel.width : width;
      this.hiprintTemplate = hiprintTemplate
      this.printData = printData
      setTimeout(() => {
        // eslint-disable-next-line no-undef
        $('#preview_content_design').html(hiprintTemplate.getHtml(printData))
        this.spinning = false
      }, 500)
    },
    async print() {
      this.waitShowPrinter = true
      // Tauri 环境：使用插件 printHtml
      if (this.isTauri) {
        try {
          const htmlResult = this.hiprintTemplate.getHtml(this.printData)
          if (htmlResult && htmlResult.length) {
            const content = htmlResult[0].target.outerHTML
            const w = this.hiprintTemplate.editingPanel ? this.hiprintTemplate.editingPanel.width : 210
            const h = this.hiprintTemplate.editingPanel ? this.hiprintTemplate.editingPanel.height : 297
            const fullHtml = '<!DOCTYPE html>\n<html lang="zh-CN">\n<head>\n<meta charset="UTF-8">\n<style>\n' +
              '  * { box-sizing: border-box; margin: 0; padding: 0; }\n' +
              '  body { font-family: "Microsoft YaHei", "SimHei", sans-serif; }\n' +
              '  @page { size: ' + w + 'mm ' + h + 'mm; margin: 0; }\n' +
              '</style>\n</head>\n<body>' + content + '</body>\n</html>'
            const api = window.__TAURI_INTERNALS__ || window.__TAURI__
            const invoke = typeof api.invoke === 'function' ? api.invoke : api.core?.invoke
            if (invoke) {
              await invoke('plugin:printer|print_html', {
                options: {
                  html: fullHtml,
                  pageWidth: w,
                  pageHeight: h,
                  orientation: w > h ? 'Landscape' : 'Portrait',
                  margin: { top: 0, bottom: 0, left: 0, right: 0, unit: 'mm' },
                  removeAfterPrint: true,
                }
              })
              window.console.log('Tauri 打印成功')
            }
          }
        } catch (e) {
          window.console.error('Tauri 打印失败:', e)
        }
        this.waitShowPrinter = false
        return
      }
      // 浏览器环境：使用 hiprint 客户端
      this.hiprintTemplate.print(this.printData, {}, {
        callback: () => {
          window.console.log('callback')
          this.waitShowPrinter = false
        }
      })
    },
    toPdf() {
      this.hiprintTemplate.toPdf({}, '打印预览');
    },
  }
}
</script>
<style lang="less" scoped>
/deep/ .ant-modal-body {
  padding: 0px;
}

/deep/ .ant-modal-content {
  margin-bottom: 24px;
}
</style>
