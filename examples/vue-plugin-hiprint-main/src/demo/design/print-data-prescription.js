/** 处方签测试数据 */
export default {
  name: '张三',
  gender: '男',
  age: '45',
  department: '内科',
  patientId: 'MZ2024001234',
  date: '2024-07-23',
  feeType: '医保',
  diagnosis: '临床诊断：上呼吸道感染',
  amount: '金额：86.50',
  medicines: [
    { drugName: '阿莫西林胶囊', spec: '0.5g', dosage: '0.5g', usage: '口服 tid', quantity: '24粒' },
    { drugName: '盐酸氨溴索片', spec: '30mg', dosage: '30mg', usage: '口服 tid', quantity: '20片' },
    { drugName: '布洛芬缓释胶囊', spec: '0.3g', dosage: '0.3g', usage: '口服 bid', quantity: '10粒' },
  ]
}