import './assets/main.css'

import { createApp } from 'vue'
import App from './App.vue'

// vue-plugin-hiprint 已在 index.html 中以全局脚本方式加载
// 全局可用: window.hiprint, window["vue-plugin-hiprint"]
const { hiPrintPlugin } = window['vue-plugin-hiprint']

const app = createApp(App)

// 全局注册 hiprint 插件
app.use(hiPrintPlugin, '$hiprint')
// 禁用 hiprint 自动 socket 连接
hiPrintPlugin.disAutoConnect()

app.mount('#app')