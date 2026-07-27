use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("Windows API 调用失败: {0}")]
    WindowsApi(String),
    #[error("无效输入: {0}")]
    InvalidInput(String),
    #[error("Base64 解码失败: {0}")]
    Base64(String),
    #[error("打印超时: {0}")]
    PrintTimeout(String),
    #[error("WebView2 错误: {0}")]
    WebView2(String),
    #[error("渲染引擎错误: {0}")]
    RenderEngine(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
