use crate::server::auth::UserPrincipal;
use crate::server::db;
use crate::server::db::models::model_provider::ModelProvider;
use crate::server::db::models::plugin::{Plugin, PluginBuilder};
use crate::server::db::models::route::Route;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::db::{Pool, tools};
use crate::server::file::file_util::{
    delete_download_file, make_download_file, make_save_file, sha256_hex,
};
use crate::server::plugin::request::{PluginAddReq, PluginInfoReq, PluginListReq, PluginUpdateReq};
use crate::server::plugin::response::{PluginInfoRes, PluginListRes};
use aiway_protocol::gateway::GlobalPlugin;
use anyhow::bail;
use busi::req::{IdsReq, Pagination};
use busi::res::{IntoPageRes, PageRes};
use common::id;
use logging::log;
use rbs::value;
use rocket::fs::TempFile;
use rocket::tokio::io;
use std::collections::HashSet;
use std::path::Path;

pub async fn info(req: PluginInfoReq<'_>, _user: UserPrincipal) -> anyhow::Result<PluginInfoRes> {
    let mut stream = req.file.open().await?;
    let mut buffer = Vec::new();
    io::copy(&mut stream, &mut buffer).await?;
    let plugin = plugin_manager::plugin_from_bytes(&buffer)
        .map_err(|e| anyhow::anyhow!("Invalid plugin: {e}"))?;
    let info = plugin.info().clone();
    let res = PluginInfoRes {
        name: plugin.name().to_string(),
        version: info.version,
        default_config: info.default_config,
        description: info.description,
        readme: info.readme,
    };
    drop(plugin);
    Ok(res)
}

pub async fn add(mut req: PluginAddReq<'_>, user: UserPrincipal) -> anyhow::Result<()> {
    let mut plugin = PluginBuilder::default()
        .id(Some(id::next()))
        .name(Some(req.name))
        .description(Some(req.description))
        .version(Some(req.version))
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;

    let default_config = match req.default_config {
        Some(config) => serde_json::Value::from(config),
        None => serde_json::Value::default(),
    };

    plugin.default_config = Some(default_config);

    // 名称唯一
    let name = plugin.name.as_ref().unwrap();
    if check_exists(&plugin, None).await? {
        bail!("Plugin with name {} already exists", name);
    }

    let (url, checksum) = save_file_and_gen_plugin_url(&mut req.file).await?;
    plugin.url = Some(url);
    plugin.checksum = Some(checksum);
    // readme 取自插件包，与 checksum 同步刷新
    plugin.readme = parse_plugin_readme(&req.file).await?;

    Plugin::insert(Pool::get()?, &plugin).await?;
    Ok(())
}

/// 从插件文件解析 readme（Markdown）
async fn parse_plugin_readme(file: &TempFile<'_>) -> anyhow::Result<Option<String>> {
    let mut stream = file.open().await?;
    let mut buffer = Vec::new();
    io::copy(&mut stream, &mut buffer).await?;
    let plugin = plugin_manager::plugin_from_bytes(&buffer)
        .map_err(|e| anyhow::anyhow!("Invalid plugin: {e}"))?;
    Ok(plugin.info().readme.clone())
}

async fn save_file_and_gen_plugin_url(file: &mut TempFile<'_>) -> anyhow::Result<(String, String)> {
    // 原始文件名
    let file_name = file
        .raw_name()
        .unwrap()
        .dangerous_unsafe_unsanitized_raw()
        .as_str();
    // 保存的文件名和路径
    let (save_file_name, save_file_path) = make_save_file(file_name)?;

    // 注意：不要使用 persist_to，避免跨文件系统错误
    // 在openEuler上如果用persist_to，会报错：Invalid cross-device link (os error 18)
    let mut dest = Path::new(&save_file_path);
    std::fs::create_dir_all(dest.parent().unwrap())?;

    file.copy_to(&mut dest).await?;

    // 计算文件SHA256，供网关下载后校验
    let checksum = sha256_hex(&std::fs::read(&save_file_path)?);

    let url = make_download_file(&save_file_name);

    Ok((url, checksum))
}

async fn check_exists(plugin: &Plugin, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = Plugin::select_by_map(
        Pool::get()?,
        value! {
            "name": &plugin.name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub async fn delete(req: IdsReq) -> anyhow::Result<()> {
    let plugins = Plugin::select_by_map(Pool::get()?, value! { "id": &req.ids }).await?;
    if plugins.is_empty() {
        return Ok(());
    }

    // 引用检查：被路由、全局配置或模型提供商引用的插件禁止删除
    let names: HashSet<String> = plugins.iter().filter_map(|p| p.name.clone()).collect();
    let references = find_plugin_references(&names).await?;
    if !references.is_empty() {
        let names = names.into_iter().collect::<Vec<_>>().join(", ");
        bail!(
            "插件 「{}」 在以下位置被引用：{}，请先解除引用后再删除。",
            names,
            references.join("、")
        );
    }

    let urls: Vec<String> = plugins.iter().filter_map(|p| p.url.clone()).collect();
    let tx = Pool::get()?.acquire_begin().await?;
    if let Err(e) = Plugin::delete_by_map(&tx, value! { "id": &req.ids }).await {
        tx.rollback().await?;
        return Err(e.into());
    }
    tx.commit().await?;

    for url in urls {
        if let Err(e) = delete_download_file(&url) {
            log::error!("delete plugin file failed, url: {}, error: {}", url, e);
        }
    }
    Ok(())
}

/// 查找插件被引用的位置，返回引用描述列表（空列表表示无引用）
///
/// 引用来源：路由插件配置、全局插件配置、模型提供商插件。
async fn find_plugin_references(names: &HashSet<String>) -> anyhow::Result<Vec<String>> {
    let mut refs = Vec::new();

    // 路由插件引用
    for route in Route::select_all(Pool::get()?).await? {
        let Some(plugins) = &route.plugins else {
            continue;
        };
        if plugins.iter().any(|p| names.contains(&p.name)) {
            refs.push(format!("路由「{}」", route.name.unwrap_or_default()));
        }
    }

    // 全局插件配置引用
    let configs = SystemConfig::select_by_map(
        Pool::get()?,
        value! { "config_key": ConfigKey::GlobalPlugin },
    )
    .await?;

    if let Some(config) = configs.first()
        && let Some(value) = &config.config_value
        && let Ok(global) = serde_json::from_str::<GlobalPlugin>(value)
        && global.plugins.iter().any(|p| names.contains(&p.name))
    {
        refs.push("全局插件".to_string());
    }

    // 模型提供商插件引用
    for provider in ModelProvider::select_all(Pool::get()?).await? {
        if let Some(p) = &provider.plugins
            && names.contains(&p.name)
        {
            refs.push(format!(
                "模型提供商「{}」",
                provider.name.unwrap_or_default()
            ));
        }
    }

    Ok(refs)
}

pub async fn list(req: PluginListReq) -> anyhow::Result<PageRes<PluginListRes>> {
    let page = db::models::plugin::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| PluginListRes { inner: item })
            .collect::<Vec<_>>()
    });
    Ok(list)
}

pub async fn update(req: PluginUpdateReq<'_>, user: UserPrincipal) -> anyhow::Result<()> {
    let tx = Pool::get()?;
    let old = Plugin::select_by_map(tx, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("Plugin not found")
    }
    //let old = old.first().unwrap();

    // if semver::Version::parse(&req.version)?
    //     < semver::Version::parse(&old.version.clone().unwrap())?
    // {
    //     bail!("Plugin version must be greater or equal than the current version")
    // }

    let mut update = PluginBuilder::default()
        .description(req.description)
        .version(Some(req.version))
        .update_user_id(Some(user.id))
        .update_time(Some(tools::now()))
        .build()?;

    let default_config = match req.default_config {
        Some(config) => serde_json::Value::from(config),
        None => serde_json::Value::default(),
    };
    update.default_config = Some(default_config);

    if let Some(mut file) = req.file {
        let (url, checksum) = save_file_and_gen_plugin_url(&mut file).await?;
        update.url = Some(url);
        update.checksum = Some(checksum);
        // readme 随插件包刷新
        update.readme = parse_plugin_readme(&file).await?;
    }

    Plugin::update_by_map(tx, &update, value! { "id": req.id}).await?;

    Ok(())
}
