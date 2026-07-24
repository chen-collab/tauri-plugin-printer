// 医疗打印示例数据
export default {
  name: "张三",
  gender: "男",
  age: "45",
  department: "内科",
  diagnosis: "上呼吸道感染",
  date: "2024-07-23",
  visitNo: "MZ2024001234",
  hospitalName: "XX市第一人民医院",
  feeType: "医保",
  doctor: "李医生",
  pharmacist: "王药师",
  total: "86.50",
  sampleNo: "S20240723001",
  sampleTime: "2024-07-23 08:30",
  reportTime: "2024-07-23 10:15",
  inNo: "ZY2024005678",
  bedNo: "12床",
  preDiagnosis: "急性阑尾炎",
  surgery: "腹腔镜下阑尾切除术",
  anesthesia: "全身麻醉",
  serialNo: "SF20240723001",
  insurance: "60.55",
  selfPay: "25.95",
  // 处方药品列表
  medicines: [
    { name: "阿莫西林胶囊", spec: "0.5g", dosage: "0.5g", usage: "口服 tid", qty: "24粒", price: "18.50" },
    { name: "盐酸氨溴索片", spec: "30mg", dosage: "30mg", usage: "口服 tid", qty: "20片", price: "12.00" },
    { name: "布洛芬缓释胶囊", spec: "0.3g", dosage: "0.3g", usage: "口服 bid", qty: "10粒", price: "15.00" },
  ],
  // 检验项目
  labItems: [
    { name: "白细胞计数(WBC)", result: "6.8", range: "3.5-9.5", unit: "10^9/L", flag: "正常" },
    { name: "红细胞计数(RBC)", result: "4.5", range: "3.8-5.1", unit: "10^12/L", flag: "正常" },
    { name: "血红蛋白(HGB)", result: "135", range: "115-150", unit: "g/L", flag: "正常" },
    { name: "血小板计数(PLT)", result: "220", range: "125-350", unit: "10^9/L", flag: "正常" },
    { name: "血糖(GLU)", result: "5.6", range: "3.9-6.1", unit: "mmol/L", flag: "正常" },
  ],
  // 收费明细
  feeItems: [
    { name: "挂号费", price: "15.00", qty: "1", amount: "15.00" },
    { name: "血常规", price: "25.00", qty: "1", amount: "25.00" },
    { name: "阿莫西林胶囊", price: "18.50", qty: "2", amount: "37.00" },
    { name: "输液费", price: "9.50", qty: "1", amount: "9.50" },
  ],
};
