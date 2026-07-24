// 医疗处方笺默认模板
export default {
  "panels": [{
    "index": 0,
    "height": 297,
    "width": 210,
    "paperHeader": 49.5,
    "paperFooter": 780,
    "printElements": [
      // 标题
      {
        "options": {
          "left": 60, "top": 10.5, "height": 27, "width": 400,
          "title": "XX市第一人民医院 处方笺",
          "fontSize": 19, "fontWeight": "700", "textAlign": "center"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      },
      // 页眉横线
      {
        "options": { "left": 30, "top": 42, "height": 10, "width": 460 },
        "printElementType": { "type": "hline" }
      },
      // 患者信息行
      {
        "options": {
          "left": 30, "top": 55, "height": 16, "width": 460,
          "title": "姓名：${name}    性别：${gender}    年龄：${age}    科别：${department}",
          "fontSize": 12, "textAlign": "left", "fontFamily": "Microsoft YaHei"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      },
      {
        "options": {
          "left": 30, "top": 75, "height": 16, "width": 460,
          "title": "门诊号：${visitNo}    日期：${date}    费别：${feeType}",
          "fontSize": 12, "textAlign": "left", "fontFamily": "Microsoft YaHei"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      },
      // 诊断
      {
        "options": {
          "left": 30, "top": 95, "height": 16, "width": 460,
          "title": "临床诊断：${diagnosis}",
          "fontSize": 12, "textAlign": "left", "fontFamily": "Microsoft YaHei"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      },
      // 分割线
      {
        "options": { "left": 30, "top": 116, "height": 10, "width": 460 },
        "printElementType": { "type": "hline" }
      },
      // 药品表格
      {
        "options": {
          "left": 30, "top": 130, "height": 120, "width": 460,
          "field": "medicines",
          "fields": [
            { "text": "药品名称", "field": "name" },
            { "text": "规格", "field": "spec" },
            { "text": "用量", "field": "dosage" },
            { "text": "用法", "field": "usage" },
            { "text": "数量", "field": "qty" },
            { "text": "单价", "field": "price" }
          ],
          "columns": [[
            { "title": "药品名称", "field": "name", "width": 100, "align": "left" },
            { "title": "规格", "field": "spec", "width": 60, "align": "center" },
            { "title": "用量", "field": "dosage", "width": 60, "align": "center" },
            { "title": "用法", "field": "usage", "width": 80, "align": "center" },
            { "title": "数量", "field": "qty", "width": 60, "align": "center" },
            { "title": "单价", "field": "price", "width": 60, "align": "right" }
          ]]
        },
        "printElementType": {
          "title": "表格", "type": "table",
          "editable": true, "columnDisplayEditable": true,
          "columnTitleEditable": true, "columnResizable": true
        }
      },
      // 分割线
      {
        "options": { "left": 30, "top": 260, "height": 10, "width": 460 },
        "printElementType": { "type": "hline" }
      },
      // 金额
      {
        "options": {
          "left": 350, "top": 275, "height": 16, "width": 140,
          "title": "金额合计：${total} 元",
          "fontSize": 13, "fontWeight": "700", "textAlign": "right", "fontFamily": "Microsoft YaHei"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      },
      // 签名区
      {
        "options": {
          "left": 30, "top": 300, "height": 16, "width": 460,
          "title": "医师签名：${doctor}        药师签名：${pharmacist}",
          "fontSize": 12, "textAlign": "left", "fontFamily": "Microsoft YaHei"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      },
      // 页脚
      {
        "options": {
          "left": 30, "top": 330, "height": 14, "width": 460,
          "title": "本处方当日有效  |  请遵医嘱用药  |  ${hospitalName}",
          "fontSize": 10, "textAlign": "center", "color": "#888", "fontFamily": "Microsoft YaHei"
        },
        "printElementType": { "title": "自定义文本", "type": "text" }
      }
    ],
    "paperNumberLeft": 430,
    "paperNumberTop": 819
  }]
};
