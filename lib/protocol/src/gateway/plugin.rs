use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// 插件配置
///
/// 注意：插件应无状态化
#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Plugin {
    /// 插件名称，全局唯一
    pub name: String,
    /// 插件执行阶段
    //pub phase: PluginPhase,
    /// 下载地址
    /// - 相对地址：从网关下载，如`/file/download/xxx.so`
    /// - 绝对地址：从给定的地址下载，如`https://xxx.com/xxx.so`
    pub url: String,
    /// 插件版本，只增不减的语义化版本号。
    pub version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
pub enum PluginPhase {
    /// 全局有效，请求阶段执行
    GlobalPre,
    /// 全局有效，响应阶段执行
    GlobalPost,
    /// 路由有效，请求阶段执行
    Pre,
    /// 路由有效，响应阶段执行
    Post,
}
impl Display for PluginPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginPhase::GlobalPre => write!(f, "GlobalPre"),
            PluginPhase::GlobalPost => write!(f, "GlobalPost"),
            PluginPhase::Pre => write!(f, "Pre"),
            PluginPhase::Post => write!(f, "Post"),
        }
    }
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
