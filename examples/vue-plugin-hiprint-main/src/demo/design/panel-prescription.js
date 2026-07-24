/** 处方签模板 — A4 竖版 (210×297mm) */
export default {
  "panels": [
    {
      "index": 0,
      "name": "处方签",
      "height": 297,
      "width": 210,
      "paperHeader": 48,
      "paperFooter": 780,
      "printElements": [
        {
          "options": {
            "left": 60,
            "top": 12,
            "height": 24,
            "width": 480,
            "title": "XX市第一人民医院",
            "fontSize": 18,
            "fontWeight": "700",
            "textAlign": "center",
            "fontFamily": "SimHei"
          },
          "printElementType": { "title": "医院名称", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 38,
            "height": 16,
            "width": 480,
            "title": "处  方  笺",
            "fontSize": 15,
            "fontWeight": "700",
            "textAlign": "center",
            "fontFamily": "SimHei"
          },
          "printElementType": { "title": "处方笺标题", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 60,
            "height": 10,
            "width": 480,
            "borderWidth": 0.5
          },
          "printElementType": { "title": "分割线", "type": "hline" }
        },
        {
          "options": {
            "left": 60,
            "top": 75,
            "height": 13,
            "width": 120,
            "title": "姓名",
            "field": "name",
            "testData": "张三",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "姓名", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 75,
            "height": 13,
            "width": 120,
            "title": "姓名",
            "field": "name",
            "testData": "张三",
            "textAlign": "center",
            "fontSize": 11,
            "fields": [
              { "text": "姓名", "field": "name" },
              { "text": "性别", "field": "gender" },
              { "text": "年龄", "field": "age" },
              { "text": "科别", "field": "department" },
              { "text": "门诊号", "field": "patientId" },
              { "text": "日期", "field": "date" },
              { "text": "费别", "field": "feeType" }
            ]
          },
          "printElementType": { "title": "姓名", "type": "text" }
        },
        {
          "options": {
            "left": 184,
            "top": 75,
            "height": 13,
            "width": 80,
            "title": "性别",
            "field": "gender",
            "testData": "男",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "性别", "type": "text" }
        },
        {
          "options": {
            "left": 268,
            "top": 75,
            "height": 13,
            "width": 80,
            "title": "年龄",
            "field": "age",
            "testData": "45",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "年龄", "type": "text" }
        },
        {
          "options": {
            "left": 352,
            "top": 75,
            "height": 13,
            "width": 80,
            "title": "科别",
            "field": "department",
            "testData": "内科",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "科别", "type": "text" }
        },
        {
          "options": {
            "left": 436,
            "top": 75,
            "height": 13,
            "width": 104,
            "title": "门诊号",
            "field": "patientId",
            "testData": "MZ2024001234",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "门诊号", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 92,
            "height": 13,
            "width": 120,
            "title": "日期",
            "field": "date",
            "testData": "2024-07-23",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "日期", "type": "text" }
        },
        {
          "options": {
            "left": 184,
            "top": 92,
            "height": 13,
            "width": 160,
            "title": "费别",
            "field": "feeType",
            "testData": "医保",
            "textAlign": "center",
            "fontSize": 11
          },
          "printElementType": { "title": "费别", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 110,
            "height": 13,
            "width": 480,
            "title": "临床诊断",
            "field": "diagnosis",
            "testData": "临床诊断：上呼吸道感染",
            "fontSize": 11,
            "fontWeight": "600",
            "textAlign": "left",
            "fontFamily": "SimHei"
          },
          "printElementType": { "title": "临床诊断", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 128,
            "height": 10,
            "width": 480,
            "borderWidth": 0.5
          },
          "printElementType": { "title": "分割线", "type": "hline" }
        },
        {
          "options": {
            "left": 60,
            "top": 142,
            "height": 13,
            "width": 480,
            "title": "Rp",
            "fontSize": 12,
            "fontWeight": "700",
            "textAlign": "left"
          },
          "printElementType": { "title": "Rp", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 160,
            "height": 120,
            "width": 480,
            "field": "medicines",
            "tableHeaderRepeat": "first",
            "tableFooterRepeat": "last",
            "tableHeaderBorder": "bottomBorder",
            "tableHeaderCellBorder": "border",
            "tableBodyRowBorder": "bottomBorder",
            "fields": [
              { "text": "药品名称", "field": "drugName" },
              { "text": "规格", "field": "spec" },
              { "text": "用量", "field": "dosage" },
              { "text": "用法", "field": "usage" },
              { "text": "数量", "field": "quantity" }
            ],
            "columns": [
              [
                { "width": 175, "title": "药品名称", "field": "drugName", "checked": true, "align": "left" },
                { "width": 80, "title": "规格", "field": "spec", "checked": true, "align": "center" },
                { "width": 70, "title": "用量", "field": "dosage", "checked": true, "align": "center" },
                { "width": 90, "title": "用法", "field": "usage", "checked": true, "align": "center" },
                { "width": 65, "title": "数量", "field": "quantity", "checked": true, "align": "center" }
              ]
            ]
          },
          "printElementType": {
            "title": "药品表格",
            "type": "table",
            "editable": true,
            "columnDisplayEditable": true,
            "columnDisplayIndexEditable": true,
            "columnTitleEditable": true,
            "columnResizable": true,
            "columnAlignEditable": true,
            "isEnableEditField": true,
            "isEnableContextMenu": true,
            "isEnableInsertRow": true,
            "isEnableDeleteRow": true,
            "isEnableInsertColumn": true,
            "isEnableDeleteColumn": true,
            "isEnableMergeCell": true
          }
        },
        {
          "options": {
            "left": 60,
            "top": 290,
            "height": 10,
            "width": 480,
            "borderWidth": 0.5
          },
          "printElementType": { "title": "分割线", "type": "hline" }
        },
        {
          "options": {
            "left": 60,
            "top": 305,
            "height": 13,
            "width": 150,
            "title": "医师签名：___________",
            "fontSize": 11,
            "textAlign": "left"
          },
          "printElementType": { "title": "医师签名", "type": "text" }
        },
        {
          "options": {
            "left": 230,
            "top": 305,
            "height": 13,
            "width": 150,
            "title": "药师签名：___________",
            "fontSize": 11,
            "textAlign": "left"
          },
          "printElementType": { "title": "药师签名", "type": "text" }
        },
        {
          "options": {
            "left": 400,
            "top": 305,
            "height": 13,
            "width": 140,
            "title": "金额",
            "field": "amount",
            "testData": "金额：86.50",
            "fontSize": 11,
            "fontWeight": "600",
            "textAlign": "right"
          },
          "printElementType": { "title": "金额", "type": "text" }
        },
        {
          "options": {
            "left": 60,
            "top": 325,
            "height": 13,
            "width": 480,
            "title": "本处方当日有效  |  请遵医嘱用药",
            "fontSize": 10,
            "textAlign": "center",
            "color": "#888888"
          },
          "printElementType": { "title": "页脚", "type": "text" }
        }
      ],
      "paperNumberLeft": 565.5,
      "paperNumberTop": 773,
      "paperNumberContinue": true,
      "watermarkOptions": {
        "content": "处方签",
        "rotate": 30,
        "timestamp": true,
        "format": "YYYY-MM-DD HH:mm"
      }
    }
  ]
}