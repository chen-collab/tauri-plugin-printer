use std::fs::{File, remove_file as rmf};
use std::io::Write;
use std::path::Path;

use base64::{Engine as _, engine::general_purpose};

use crate::Error;

/// 校验并提取安全的文件名（拒绝路径遍历）
pub fn sanitize_filename(name: &str) -> Result<String, Error> {
    let path = Path::new(name);
    // 拒绝包含路径分隔符、绝对路径、父目录引用
    if path.is_absolute()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
    {
        return Err(Error::InvalidInput(format!("文件名包含非法字符: {}", name)));
    }
    // 提取纯文件名
    match path.file_name().and_then(|n| n.to_str()) {
        Some(fname) if !fname.is_empty() && fname != "." && fname != ".." => Ok(fname.to_string()),
        _ => Err(Error::InvalidInput(format!("无效的文件名: {}", name))),
    }
}

/// 从 base64 字符串创建文件
pub fn create_file_from_base64(base64_string: &str, file_path: &str) -> Result<(), Error> {
    let mut buffer = Vec::<u8>::new();
    general_purpose::STANDARD
        .decode_vec(base64_string, &mut buffer)
        .map_err(|e| Error::Base64(format!("{}", e)))?;

    let path = Path::new(file_path);
    let mut file = File::create(&path)?;
    file.write_all(&buffer)?;
    Ok(())
}

/// 删除文件
pub fn remove_file(file_path: &str) -> Result<(), Error> {
    rmf(file_path)?;
    Ok(())
}