/**
 * Tauri 打印机插件 - 打印设置示例
 * 使用 SumatraPDF 兼容的参数格式
 */

import { printPdf, printHtml, getPrinters, getPrinterByName } from 'tauri-plugin-printer-v2';

// SumatraPDF 格式打印设置配置示例
const buildPrintSettings = (options = {}) => {
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
};

// ===== 基础打印示例 =====

/**
 * PDF 打印示例
 */
const printPdfWithSettings = async (filePath, printerName = 'default') => {
  try {
    console.log('🖨️ 开始 PDF 打印...');
    
    // 构建打印设置
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color',
      copies: 1
    });
    
    const options = {
      id: `pdf_print_${Date.now()}`,
      path: filePath,
      printer: printerName,
      print_settings: printSettings, // 'portrait,paper=A4,fit,color'
      remove_after_print: false
    };
    
    console.log('📋 打印设置:', printSettings);
    console.log('⚙️ 打印选项:', options);
    
    const result = await printPdf(options);
    console.log('✅ PDF 打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ PDF 打印失败:', error);
    throw error;
  }
};

/**
 * HTML 打印示例
 */
const printHtmlWithSettings = async (htmlContent, printerName = 'default') => {
  try {
    console.log('🖨️ 开始 HTML 打印...');
    
    // 构建打印设置
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color',
      copies: 1
    });
    
    const options = {
      id: `html_print_${Date.now()}`,
      html: htmlContent,
      printer: printerName,
      print_settings: printSettings, // 'portrait,paper=A4,fit,color'
      remove_after_print: true
    };
    
    console.log('📋 打印设置:', printSettings);
    console.log('⚙️ 打印选项:', options);
    
    const result = await printHtml(options);
    console.log('✅ HTML 打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ HTML 打印失败:', error);
    throw error;
  }
};

/**
 * 批量打印示例
 */
const batchPrintWithDifferentSettings = async (files) => {
  console.log('🔄 开始批量打印...');
  
  const results = [];
  
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    
    try {
      // 为每个文件使用不同的设置
      const printSettings = buildPrintSettings({
        orientation: file.orientation || 'portrait',
        paperSize: file.paperSize || 'A4',
        scale: file.scale || 'fit',
        colorMode: file.colorMode || 'color',
        copies: file.copies || 1
      });
      
      const options = {
        id: `batch_print_${Date.now()}_${i}`,
        path: file.path,
        printer: file.printer || 'default',
        print_settings: printSettings,
        remove_after_print: false
      };
      
      console.log(`📄 打印文件 ${i + 1}/${files.length}: ${file.path}`);
      console.log(`📋 设置: ${printSettings}`);
      
      const result = await printPdf(options);
      results.push({ file: file.path, result, success: true });
      
      // 添加延迟避免打印队列拥堵
      await new Promise(resolve => setTimeout(resolve, 1000));
      
    } catch (error) {
      console.error(`❌ 文件 ${file.path} 打印失败:`, error);
      results.push({ file: file.path, error: error.message, success: false });
    }
  }
  
  console.log('📊 批量打印完成，结果汇总:');
  console.table(results);
  
  return results;
};

// ===== 指定打印机示例 =====

/**
 * 按名称指定打印机
 */
const printToSpecificPrinter = async (filePath, printerName) => {
  try {
    console.log(`🎯 指定打印机: ${printerName}`);
    
    // 验证打印机是否存在
    const printer = await getPrinterByName(printerName);
    if (!printer) {
      throw new Error(`打印机 "${printerName}" 未找到`);
    }
    
    console.log('✅ 打印机验证成功:', printer);
    
    // 使用指定打印机打印
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color',
      copies: 1
    });
    
    const options = {
      id: `specific_print_${Date.now()}`,
      path: filePath,
      printer: printerName,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    const result = await printPdf(options);
    console.log('✅ 指定打印机打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ 指定打印机打印失败:', error);
    throw error;
  }
};

/**
 * 按索引指定打印机
 */
const printToIndexedPrinter = async (filePath, printerIndex) => {
  try {
    console.log(`📋 获取打印机列表...`);
    const printers = await getPrinters();
    
    if (printerIndex < 0 || printerIndex >= printers.length) {
      throw new Error(`打印机索引 ${printerIndex} 超出范围 (0-${printers.length - 1})`);
    }
    
    const selectedPrinter = printers[printerIndex];
    console.log(`🎯 选择打印机 [${printerIndex}]: ${selectedPrinter.name}`);
    
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color',
      copies: 1
    });
    
    const options = {
      id: `indexed_print_${Date.now()}`,
      path: filePath,
      printer: selectedPrinter.name,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    const result = await printPdf(options);
    console.log('✅ 按索引打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ 按索引打印失败:', error);
    throw error;
  }
};

/**
 * 智能选择打印机
 */
const smartPrinterSelection = async (filePath) => {
  try {
    console.log('🧠 智能选择打印机...');
    const printers = await getPrinters();
    
    if (printers.length === 0) {
      throw new Error('没有可用的打印机');
    }
    
    // 优先选择默认打印机
    let selectedPrinter = printers.find(p => p.isDefault);
    console.log('🔍 查找默认打印机:', selectedPrinter ? '找到' : '未找到');
    
    // 如果没有默认打印机，选择第一个可用的
    if (!selectedPrinter) {
      selectedPrinter = printers.find(p => p.status === '可用');
      console.log('🔍 查找可用打印机:', selectedPrinter ? '找到' : '未找到');
    }
    
    // 如果还是没有，选择第一个
    if (!selectedPrinter) {
      selectedPrinter = printers[0];
      console.log('🔍 使用第一个打印机:', selectedPrinter.name);
    }
    
    console.log(`🎯 智能选择结果: ${selectedPrinter.name}`);
    
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color',
      copies: 1
    });
    
    const options = {
      id: `smart_print_${Date.now()}`,
      path: filePath,
      printer: selectedPrinter.name,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    const result = await printPdf(options);
    console.log('✅ 智能选择打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ 智能选择打印失败:', error);
    throw error;
  }
};

/**
 * 按特性选择打印机
 */
const printToColorPrinter = async (filePath) => {
  try {
    console.log('🎨 查找彩色打印机...');
    const printers = await getPrinters();
    
    // 查找包含 "color" 关键词的打印机
    const colorPrinter = printers.find(p => 
      p.name.toLowerCase().includes('color') || 
      p.driver.toLowerCase().includes('color')
    );
    
    if (!colorPrinter) {
      console.log('⚠️ 未找到彩色打印机，使用默认打印机');
      return await smartPrinterSelection(filePath);
    }
    
    console.log(`🎯 找到彩色打印机: ${colorPrinter.name}`);
    
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color', // 强制彩色打印
      copies: 1
    });
    
    const options = {
      id: `color_print_${Date.now()}`,
      path: filePath,
      printer: colorPrinter.name,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    const result = await printPdf(options);
    console.log('✅ 彩色打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ 彩色打印失败:', error);
    throw error;
  }
};

/**
 * 按特性选择双面打印机
 */
const printToDuplexPrinter = async (filePath) => {
  try {
    console.log('📄 查找双面打印机...');
    const printers = await getPrinters();
    
    // 查找包含 "duplex" 关键词的打印机
    const duplexPrinter = printers.find(p => 
      p.name.toLowerCase().includes('duplex') || 
      p.driver.toLowerCase().includes('duplex')
    );
    
    if (!duplexPrinter) {
      console.log('⚠️ 未找到双面打印机，使用默认打印机');
      return await smartPrinterSelection(filePath);
    }
    
    console.log(`🎯 找到双面打印机: ${duplexPrinter.name}`);
    
    const printSettings = buildPrintSettings({
      orientation: 'portrait',
      paperSize: 'A4',
      scale: 'fit',
      colorMode: 'color',
      copies: 1,
      duplex: 'duplex' // 启用双面打印
    });
    
    const options = {
      id: `duplex_print_${Date.now()}`,
      path: filePath,
      printer: duplexPrinter.name,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    const result = await printPdf(options);
    console.log('✅ 双面打印成功:', result);
    return result;
    
  } catch (error) {
    console.error('❌ 双面打印失败:', error);
    throw error;
  }
};

// ===== 打印机工具函数 =====

const PrinterUtils = {
  // 构建 SumatraPDF 格式的打印设置
  buildPrintSettings,
  
  // 打印到默认打印机
  printToDefault: async (filePath, settingsOptions = {}) => {
    const printSettings = buildPrintSettings(settingsOptions);
    
    const options = {
      id: `default_print_${Date.now()}`,
      path: filePath,
      printer: 'default',
      print_settings: printSettings,
      remove_after_print: false
    };
    
    console.log('🖨️ 打印到默认打印机:', printSettings);
    return await printPdf(options);
  },
  
  // 打印到第一个可用打印机
  printToFirst: async (filePath, settingsOptions = {}) => {
    const printers = await getPrinters();
    if (printers.length === 0) {
      throw new Error('没有可用的打印机');
    }
    
    const printSettings = buildPrintSettings(settingsOptions);
    
    const options = {
      id: `first_print_${Date.now()}`,
      path: filePath,
      printer: printers[0].name,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    console.log(`🖨️ 打印到第一个打印机 (${printers[0].name}):`, printSettings);
    return await printPdf(options);
  },
  
  // 按名称打印
  printToNamed: async (filePath, printerName, settingsOptions = {}) => {
    const printSettings = buildPrintSettings(settingsOptions);
    
    const options = {
      id: `named_print_${Date.now()}`,
      path: filePath,
      printer: printerName,
      print_settings: printSettings,
      remove_after_print: false
    };
    
    console.log(`🖨️ 打印到指定打印机 (${printerName}):`, printSettings);
    return await printPdf(options);
  },
  
  // 打印到彩色打印机
  printToColor: async (filePath, settingsOptions = {}) => {
    const colorSettings = { ...settingsOptions, colorMode: 'color' };
    return await printToColorPrinter(filePath);
  },
  
  // 打印到双面打印机
  printToDuplex: async (filePath, settingsOptions = {}) => {
    const duplexSettings = { ...settingsOptions, duplex: 'duplex' };
    return await printToDuplexPrinter(filePath);
  },
  
  // 列出所有打印机
  listPrinters: async () => {
    const printers = await getPrinters();
    console.log('📋 可用打印机列表:');
    console.table(printers);
    return printers;
  },
  
  // 获取打印机详细信息
  getPrinterInfo: async (printerName) => {
    const printer = await getPrinterByName(printerName);
    if (printer) {
      console.log(`🖨️ 打印机 "${printerName}" 详细信息:`);
      console.log(printer);
    } else {
      console.log(`❌ 打印机 "${printerName}" 未找到`);
    }
    return printer;
  }
};

// ===== 使用示例 =====

// 导出所有函数供外部使用
export {
  // 基础打印函数
  printPdfWithSettings,
  printHtmlWithSettings,
  batchPrintWithDifferentSettings,
  
  // 指定打印机函数
  printToSpecificPrinter,
  printToIndexedPrinter,
  smartPrinterSelection,
  printToColorPrinter,
  printToDuplexPrinter,
  
  // 工具函数
  PrinterUtils,
  buildPrintSettings
};

// 示例用法（在实际项目中调用）
if (typeof window !== 'undefined') {
  // 浏览器环境示例
  console.log('🚀 Tauri 打印机插件示例已加载');
  console.log('📖 使用方法:');
  console.log('1. printPdfWithSettings("/path/to/file.pdf", "打印机名称")');
  console.log('2. PrinterUtils.listPrinters()');
  console.log('3. smartPrinterSelection("/path/to/file.pdf")');
  
  // 将工具函数添加到全局对象以便调试
  window.PrinterUtils = PrinterUtils;
}

// Node.js 环境示例
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    printPdfWithSettings,
    printHtmlWithSettings,
    batchPrintWithDifferentSettings,
    printToSpecificPrinter,
    printToIndexedPrinter,
    smartPrinterSelection,
    printToColorPrinter,
    printToDuplexPrinter,
    PrinterUtils,
    buildPrintSettings
  };
}

/**
 * 常用配置模板
 */
export const PrintTemplates = {
  // 办公文档
  office: () => buildPrintSettings({
    orientation: 'portrait',
    paperSize: 'A4',
    scale: 'fit',
    colorMode: 'color',
    copies: 1
  }),
  
  // 大幅面图纸
  blueprint: () => buildPrintSettings({
    orientation: 'landscape',
    paperSize: 'A3',
    scale: 'shrink',
    colorMode: 'monochrome',
    copies: 1
  }),
  
  // 批量打印
  batch: (copies = 5) => buildPrintSettings({
    orientation: 'portrait',
    paperSize: 'A4',
    scale: 'fit',
    colorMode: 'monochrome',
    copies: copies
  }),
  
  // 双面打印
  duplex: () => buildPrintSettings({
    orientation: 'portrait',
    paperSize: 'A4',
    scale: 'fit',
    colorMode: 'color',
    copies: 1,
    duplex: 'duplex'
  }),
  
  // 高质量彩色打印
  highQuality: () => buildPrintSettings({
    orientation: 'portrait',
    paperSize: 'A4',
    scale: 'noscale',
    colorMode: 'color',
    copies: 1
  })
};

console.log('📋 可用的打印模板:', Object.keys(PrintTemplates));