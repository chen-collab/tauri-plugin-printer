# Tauri 打印机插件 - 打印设置指南

本指南详细说明如何在 Tauri 打印机插件中使用打印设置参数。

## 打印设置参数说明

### `print_settings` 参数格式

打印设置采用 SumatraPDF 兼容的逗号分隔格式：

```javascript
// SumatraPDF 格式示例
const printSettings = 'portrait,paper=A4,fit,color,2x';

// 参数说明：
// - portrait/landscape: 打印方向
// - paper=A4: 纸张大小
// - fit/shrink/noscale: 缩放方式
// - color/monochrome: 颜色模式
// - 2x: 打印份数（大于1时）
```

### 支持的参数选项

#### 打印方向
- `portrait` - 纵向打印
- `landscape` - 横向打印

#### 纸张大小
- `paper=A4` - A4 纸张 (210×297mm)
- `paper=A3` - A3 纸张 (297×420mm)
- `paper=Letter` - Letter 纸张 (216×279mm)
- `paper=Legal` - Legal 纸张 (216×356mm)
- `paper=A5` - A5 纸张 (148×210mm)

#### 缩放方式
- `fit` - 适应页面大小
- `shrink` - 仅缩小（不放大）
- `noscale` - 不缩放

#### 颜色模式
- `color` - 彩色打印
- `monochrome` - 黑白打印

#### 打印份数
- `2x` - 打印2份
- `3x` - 打印3份
- 等等...（省略1x，默认为1份）

#### 双面打印
- `duplex` - 双面打印（长边翻转）
- `duplexshort` - 双面打印（短边翻转）
- `simplex` - 单面打印

## 1. PDF 打印示例

### 基础 PDF 打印

```javascript
import { printPdf } from 'tauri-plugin-printer-v2';

const printPdfWithSettings = async () => {
  const options = {
    id: `pdf_print_${Date.now()}`,
    path: '/path/to/document.pdf',
    printer: 'HP LaserJet Pro',
    print_settings: 'portrait,paper=A4,fit,color',
    remove_after_print: false
  };
  
  try {
    const result = await printPdf(options);
    console.log('PDF 打印成功:', result);
  } catch (error) {
    console.error('PDF 打印失败:', error);
  }
};
```

### 高级 PDF 打印设置

```javascript
// 横向打印，A3纸张，黑白，2份
const advancedPdfPrint = async () => {
  const options = {
    id: `pdf_advanced_${Date.now()}`,
    path: '/path/to/large-document.pdf',
    printer: 'Canon PIXMA',
    print_settings: 'landscape,paper=A3,shrink,monochrome,2x',
    remove_after_print: false
  };
  
  const result = await printPdf(options);
  return result;
};
```

## 2. HTML 打印示例

### 基础 HTML 打印

```javascript
import { printHtml } from 'tauri-plugin-printer-v2';

const printHtmlWithSettings = async () => {
  const htmlContent = `
    <!DOCTYPE html>
    <html>
    <head>
        <title>测试页面</title>
        <style>
            body { font-family: Arial, sans-serif; }
            .header { text-align: center; color: #333; }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>HTML 打印测试</h1>
            <p>这是一个测试页面</p>
        </div>
    </body>
    </html>
  `;
  
  const options = {
    id: `html_print_${Date.now()}`,
    html: htmlContent,
    printer: 'default',
    print_settings: 'portrait,paper=A4,fit,color',
    remove_after_print: true
  };
  
  try {
    const result = await printHtml(options);
    console.log('HTML 打印成功:', result);
  } catch (error) {
    console.error('HTML 打印失败:', error);
  }
};
```

## 3. 指定打印机的方法

### 按名称指定打印机

```javascript
import { getPrinters, printPdf } from 'tauri-plugin-printer-v2';

const printToNamedPrinter = async (printerName, filePath) => {
  const options = {
    id: `named_print_${Date.now()}`,
    path: filePath,
    printer: printerName,
    print_settings: 'portrait,paper=A4,fit,color',
    remove_after_print: false
  };
  
  return await printPdf(options);
};

// 使用示例
await printToNamedPrinter('HP LaserJet Pro', '/path/to/document.pdf');
```

### 按索引指定打印机

```javascript
const printToIndexedPrinter = async (printerIndex, filePath) => {
  const printers = await getPrinters();
  
  if (printerIndex >= 0 && printerIndex < printers.length) {
    const selectedPrinter = printers[printerIndex];
    
    const options = {
      id: `indexed_print_${Date.now()}`,
      path: filePath,
      printer: selectedPrinter.name,
      print_settings: 'portrait,paper=A4,fit,color',
      remove_after_print: false
    };
    
    return await printPdf(options);
  } else {
    throw new Error('打印机索引超出范围');
  }
};
```

### 智能选择打印机

```javascript
const smartPrinterSelection = async (filePath) => {
  const printers = await getPrinters();
  
  // 优先选择默认打印机
  let selectedPrinter = printers.find(p => p.isDefault);
  
  // 如果没有默认打印机，选择第一个可用的
  if (!selectedPrinter) {
    selectedPrinter = printers.find(p => p.status === '可用');
  }
  
  // 如果还是没有，选择第一个
  if (!selectedPrinter && printers.length > 0) {
    selectedPrinter = printers[0];
  }
  
  if (!selectedPrinter) {
    throw new Error('没有可用的打印机');
  }
  
  const options = {
    id: `smart_print_${Date.now()}`,
    path: filePath,
    printer: selectedPrinter.name,
    print_settings: 'portrait,paper=A4,fit,color',
    remove_after_print: false
  };
  
  return await printPdf(options);
};
```

## 4. 打印设置工具函数

```javascript
const PrinterUtils = {
  // 生成 SumatraPDF 格式的打印设置字符串
  buildPrintSettings: (options = {}) => {
    const {
      orientation = 'portrait',
      paperSize = 'A4',
      scale = 'fit',
      colorMode = 'color',
      copies = 1,
      duplex = null
    } = options;
    
    const settings = [];
    
    // 添加打印方向
    settings.push(orientation.toLowerCase());
    
    // 添加纸张大小
    settings.push(`paper=${paperSize}`);
    
    // 添加缩放设置
    settings.push(scale);
    
    // 添加颜色设置
    settings.push(colorMode);
    
    // 添加打印份数（如果大于1）
    if (copies > 1) {
      settings.push(`${copies}x`);
    }
    
    // 添加双面打印设置
    if (duplex) {
      settings.push(duplex);
    }
    
    return settings.join(',');
  },
  
  // 打印到默认打印机
  printToDefault: async (filePath, settingsOptions = {}) => {
    const printSettings = PrinterUtils.buildPrintSettings(settingsOptions);
    
    const options = {
      id: `default_print_${Date.now()}`,
      path: filePath,
      printer: 'default',
      print_settings: printSettings,
      remove_after_print: false
    };
    
    return await printPdf(options);
  },
  
  // 打印到第一个可用打印机
  printToFirst: async (filePath, settingsOptions = {}) => {
    const printers = await getPrinters();
    if (printers.length === 0) {
      throw new Error('没有可用的打印机');
    }
    
    const printSettings = PrinterUtils.buildPrintSettings(settingsOptions);
    
    const options = {
      id: `first_print_${Date.now()}`,
      path: filePath,
      printer: printers[0].name,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    return await printPdf(options);
  },
  
  // 列出所有打印机
  listPrinters: async () => {
    const printers = await getPrinters();
    console.table(printers);
    return printers;
  }
};
```

## 5. 使用示例

### 在 Vue 应用中使用

```javascript
// 在 Vue 组件中
export default {
  data() {
    return {
      printSettings: {
        orientation: 'Portrait',
        paperSize: 'A4',
        copies: 1,
        quality: 300,
        grayscale: false
      }
    };
  },
  
  methods: {
    async handlePrint() {
      // 构建 SumatraPDF 格式的设置字符串
      const settings = [];
      
      // 添加打印方向
      if (this.printSettings.orientation === 'Landscape') {
        settings.push('landscape');
      } else {
        settings.push('portrait');
      }
      
      // 添加纸张大小
      settings.push(`paper=${this.printSettings.paperSize}`);
      
      // 添加缩放设置
      settings.push('fit');
      
      // 添加颜色设置
      if (this.printSettings.grayscale) {
        settings.push('monochrome');
      } else {
        settings.push('color');
      }
      
      // 添加打印份数（如果大于1）
      if (this.printSettings.copies > 1) {
        settings.push(`${this.printSettings.copies}x`);
      }
      
      const printSettingsString = settings.join(',');
      
      // 执行打印
      const options = {
        id: `vue_print_${Date.now()}`,
        path: this.selectedFilePath,
        printer: this.selectedPrinter || 'default',
        print_settings: printSettingsString,
        remove_after_print: false
      };
      
      try {
        const result = await printPdf(options);
        console.log('打印成功:', result);
      } catch (error) {
        console.error('打印失败:', error);
      }
    }
  }
};
```

## 常用配置模板

### 办公文档打印
```javascript
const officeSettings = 'portrait,paper=A4,fit,color';
```

### 大幅面图纸打印
```javascript
const blueprintSettings = 'landscape,paper=A3,shrink,monochrome';
```

### 批量打印
```javascript
const batchSettings = 'portrait,paper=A4,fit,monochrome,5x';
```

### 双面打印
```javascript
const duplexSettings = 'portrait,paper=A4,fit,color,duplex';
```

## 重要注意事项

1. **参数顺序**: SumatraPDF 格式的参数顺序不重要，但建议保持一致的顺序以便维护

2. **纸张大小**: 确保指定的纸张大小在目标打印机上可用

3. **打印机名称**: 打印机名称必须与系统中的实际名称完全匹配（区分大小写）

4. **错误处理**: 始终使用 try-catch 块来处理打印错误

5. **资源清理**: 对于临时文件，建议设置 `remove_after_print: true`

## 故障排除

### 常见问题

1. **打印机未找到**
   - 检查打印机名称是否正确
   - 确认打印机已安装并可用
   - 使用 `getPrinters()` 获取可用打印机列表

2. **纸张大小不支持**
   - 检查打印机是否支持指定的纸张大小
   - 尝试使用 `fit` 或 `shrink` 缩放选项

3. **打印质量问题**
   - 调整缩放设置（fit/shrink/noscale）
   - 检查颜色模式设置
   - 确认打印机驱动程序是否最新

### 调试技巧

1. **启用详细日志**
   ```javascript
   console.log('打印设置:', printSettingsString);
   console.log('打印选项:', options);
   ```

2. **测试打印机连接**
   ```javascript
   const printers = await getPrinters();
   console.log('可用打印机:', printers);
   ```

3. **分步测试**
   - 先测试简单的打印设置
   - 逐步添加复杂的参数
   - 验证每个参数的效果

## 完整示例

查看 `examples/printer-settings-example.js` 文件获取更多完整的使用示例，包括：

- 基础打印示例（PDF、HTML、批量打印）
- 指定打印机示例（按名称、按索引、智能选择、按特性选择）
- `PrinterUtils` 工具函数的完整实现
- 错误处理和调试技巧
- 实际项目中的最佳实践

通过这些示例和指南，您可以充分利用 Tauri 打印机插件的所有功能，实现灵活、可靠的打印解决方案。