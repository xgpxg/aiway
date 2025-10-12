use crate::server::auth::UserPrincipal;
use crate::server::db;
use crate::server::db::models::plugin::{Plugin, PluginBuilder};
use crate::server::db::{Pool, tools};
use crate::server::file::file_util::{make_download_file, make_save_file};
use crate::server::plugin::request::{PluginAddReq, PluginListReq, PluginUpdateReq};
use crate::server::plugin::response::PluginListRes;
use anyhow::bail;
use common::id;
use protocol::common::req::{IdsReq, Pagination};
use protocol::common::res::{IntoPageRes, PageRes};
use rbs::value;
use rocket::fs::TempFile;

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

    plugin.url = Some(save_file_and_gen_plugin_url(&mut req.file).await?);

    Plugin::insert(Pool::get()?, &plugin).await?;
    Ok(())
}

async fn save_file_and_gen_plugin_url(file: &mut TempFile<'_>) -> anyhow::Result<String> {
    // 原始文件名
    let file_name = file
        .raw_name()
        .unwrap()
        .dangerous_unsafe_unsanitized_raw()
        .as_str();
    // 保存的文件名和路径
    let (save_file_name, save_file_path) = make_save_file(file_name)?;
    file.persist_to(&save_file_path).await?;

    let url = make_download_file(&save_file_name);

    Ok(url)
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
    //TODO 删除文件
    Plugin::delete_by_map(Pool::get()?, value! { "id": req.ids}).await?;
    Ok(())
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

pub async fn update(mut req: PluginUpdateReq<'_>, user: UserPrincipal) -> anyhow::Result<()> {
    let tx = Pool::get()?;
    let old = Plugin::select_by_map(tx, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("Plugin not found")
    }
    let old = old.first().unwrap();

    if semver::Version::parse(&req.version)?
        <= semver::Version::parse(&old.version.clone().unwrap())?
    {
        bail!("Plugin version must be greater than the current version")
    }

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
        update.url = Some(save_file_and_gen_plugin_url(&mut file).await?);
    }

    Plugin::update_by_map(tx, &update, value! { "id": req.id}).await?;

    Ok(())
}
