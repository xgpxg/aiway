use crate::server::db::tools;
use anyhow::bail;
use common::data_dir;

/// 生成保存文件名
///
/// 文件名保存格式：YYYYMMDD-10位随机字符串_原始文件名。
pub fn make_save_file(file_name: &str) -> anyhow::Result<(String, String)> {
    let today = tools::now().format("YYYYMMDD");
    let dir = data_dir!("file", &today);
    match std::fs::exists(&dir) {
        Ok(exists) => {
            if !exists {
                std::fs::create_dir_all(&dir)?;
            }
        }
        Err(e) => {
            bail!("make save dir failed: {}", e);
        }
    }

    // 保存的文件名：随机字符串-日期_原始文件名
    let save_file_name = format!("{}-{}_{}", today, nanoid::nanoid!(10), file_name);
    let save_file = format!("{}/{}", dir.display(), save_file_name);

    Ok((save_file_name, save_file))
}

pub fn make_download_file(file_name: &str) -> String {
    format!("/file/download/{}", file_name)
}

/// 计算文件内容 SHA256，返回十六进制字符串
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}
