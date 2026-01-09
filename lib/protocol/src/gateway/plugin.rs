use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// 插件配置
///
/// 注意：插件应无状态化
#[derive(Debug, Serialize, Eq, PartialEq, Deserialize)]
pub struct Plugin {
    /// 插件名称，全局唯一
    pub name: String,
    /// 下载地址
    /// - 相对地址：从控制台下载，如`/file/download/xxx.so`
    /// - 绝对地址：从给定的地址下载，如`https://xxx.com/xxx.so`
    pub url: String,
    /// 插件版本，只增不减的语义化版本号。
    pub version: String,
}

/// 已配置的插件
///
/// 该类型在具体的插件配置和运行时使用。
///
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfiguredPlugin {
    /// 插件名称
    pub name: String,
    /// 插件配置
    pub config: serde_json::Value,
}

impl Plugin {
    /// 通过控制台地址，构建下载地址。
    ///
    /// 插件地址可以是相对地址和绝对地址，如果是相对地址，则从控制台下载，否则直接下载。
    #[inline]
    pub fn build_url_with_console(&self, console_addr: &str) -> String {
        format!("http://{}{}", console_addr, self.url)
    }

    /// 判断插件url是否是相对地址。
    #[inline]
    pub fn is_relative_download_url(&self) -> bool {
        !self.url.starts_with("http://") && !self.url.starts_with("https://")
    }
}
