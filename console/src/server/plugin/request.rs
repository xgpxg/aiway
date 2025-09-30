use protocol::common::req::PageReq;
use protocol::{gateway, impl_pagination};
use rocket::form::FromForm;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, FromForm)]
pub struct PluginAddReq<'a> {
    /// 插件名称，全局唯一
    pub name: String,
    /// 插件描述
    pub description: String,
    /// 插件版本
    pub version: String,
    /// 插件文件，目前仅支持`.so`
    pub file: TempFile<'a>,
    /// 插件的默认配置,YAML格式。
    /// - 该配置在全局插件配置及路由插件配置时展示，修改后的配置关联到[`gateway::ConfiguredPlugin`]
    /// - 该配置仅可在插件管理处修改
    pub default_config: Option<String>,
}
#[derive(Debug, FromForm)]
pub struct PluginUpdateReq<'a> {
    pub id: i64,
    /// 插件描述
    pub description: String,
    /// 插件版本
    pub version: String,
    /// 插件文件，目前仅支持`.so`
    pub file: TempFile<'a>,
    /// 插件的默认配置,YAML格式。
    /// - 该配置在全局插件配置及路由插件配置时展示，修改后的配置关联到[`gateway::ConfiguredPlugin`]
    /// - 该配置仅可在插件管理处修改
    pub default_config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginListReq {
    /// 模糊搜索：插件名称、描述
    pub filter_text: String,
    page: PageReq,
}
impl_pagination!(PluginListReq);
