use protocol::gateway;
use rocket::form::FromForm;
use rocket::fs::TempFile;

#[derive(Debug, FromForm)]
pub struct PluginAddOrUpdateReq<'a> {
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
