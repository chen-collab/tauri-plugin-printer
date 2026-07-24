/** 输液标签模板 — 100×80mm 自定义标签 */
export default {
  "panels": [
    {
      "index": 0,
      "name": "输液标签",
      "height": 80,
      "width": 100,
      "paperHeader": 5,
      "paperFooter": 75,
      "printElements": [
        {
          "options": {
            "left": 5,
            "top": 3,
            "height": 14,
            "width": 180,
            "title": "XX市第一人民医院",
            "fontSize": 13,
            "fontWeight": "700",
            "textAlign": "center",
            "fontFamily": "SimHei",
            "borderWidth": 0,
            "borderStyle": "solid"
          },
          "printElementType": { "title": "医院名称", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 18,
            "height": 10,
            "width": 180,
            "borderWidth": 0.5
          },
          "printElementType": { "title": "分割线", "type": "hline" }
        },
        {
          "options": {
            "left": 5,
            "top": 30,
            "height": 11,
            "width": 130,
            "title": "姓名",
            "field": "name",
            "testData": "姓名：孙七",
            "fontSize": 10,
            "fontWeight": "600",
            "textAlign": "left",
            "fontFamily": "SimHei"
          },
          "printElementType": { "title": "姓名", "type": "text" }
        },
        {
          "options": {
            "left": 140,
            "top": 30,
            "height": 11,
            "width": 45,
            "title": "床号",
            "field": "bedNo",
            "testData": "床号：08",
            "fontSize": 10,
            "fontWeight": "600",
            "textAlign": "left",
            "fontFamily": "SimHei"
          },
          "printElementType": { "title": "床号", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 44,
            "height": 11,
            "width": 105,
            "title": "科室",
            "field": "department",
            "testData": "科室：心内科",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "科室", "type": "text" }
        },
        {
          "options": {
            "left": 115,
            "top": 44,
            "height": 11,
            "width": 70,
            "title": "住院号",
            "field": "patientId",
            "testData": "住院号：ZY2024009",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "住院号", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 58,
            "height": 11,
            "width": 180,
            "title": "药品",
            "field": "drugName",
            "testData": "药品：0.9%氯化钠注射液 250ml",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "药品", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 72,
            "height": 11,
            "width": 180,
            "title": "加药",
            "field": "addDrug",
            "testData": "加药：注射用头孢呋辛钠 1.5g",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "加药", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 86,
            "height": 10,
            "width": 180,
            "borderWidth": 0.5,
            "borderStyle": "dashed"
          },
          "printElementType": { "title": "虚线", "type": "hline" }
        },
        {
          "options": {
            "left": 5,
            "top": 99,
            "height": 11,
            "width": 90,
            "title": "用法",
            "field": "usage",
            "testData": "用法：静脉滴注 bid",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "用法", "type": "text" }
        },
        {
          "options": {
            "left": 100,
            "top": 99,
            "height": 11,
            "width": 85,
            "title": "滴速",
            "field": "dripRate",
            "testData": "滴速：40滴/分",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "滴速", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 113,
            "height": 11,
            "width": 105,
            "title": "配药时间",
            "field": "prepTime",
            "testData": "配药：07-23 09:00",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "配药时间", "type": "text" }
        },
        {
          "options": {
            "left": 115,
            "top": 113,
            "height": 11,
            "width": 70,
            "title": "护士",
            "field": "nurse",
            "testData": "护士：___________",
            "fontSize": 10,
            "textAlign": "left"
          },
          "printElementType": { "title": "护士", "type": "text" }
        },
        {
          "options": {
            "left": 5,
            "top": 128,
            "height": 10,
            "width": 180,
            "borderWidth": 0.5,
            "borderStyle": "dashed"
          },
          "printElementType": { "title": "虚线", "type": "hline" }
        },
        {
          "options": {
            "left": 5,
            "top": 141,
            "height": 30,
            "width": 80,
            "title": "barcode",
            "field": "barcode",
            "testData": "1234567890"
          },
          "printElementType": { "title": "条形码", "type": "barcode" }
        },
        {
          "options": {
            "left": 95,
            "top": 141,
            "height": 30,
            "width": 30,
            "title": "qrcode",
            "field": "qrcode",
            "testData": "QRC"
          },
          "printElementType": { "title": "二维码", "type": "qrcode" }
        },
        {
          "options": {
            "left": 135,
            "top": 141,
            "height": 11,
            "width": 50,
            "title": "核对",
            "fontSize": 10,
            "textAlign": "center",
            "fontWeight": "600"
          },
          "printElementType": { "title": "核对", "type": "text" }
        }
      ],
      "paperNumberLeft": 150,
      "paperNumberTop": 170,
      "paperNumberContinue": false,
      "watermarkOptions": {
        "content": "输液标签",
        "rotate": 30,
        "timestamp": true,
        "format": "YYYY-MM-DD HH:mm"
      }
    }
  ]
}