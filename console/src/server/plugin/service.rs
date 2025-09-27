use crate::server::auth::UserPrincipal;
use crate::server::db::models::plugin::{Plugin, PluginBuilder};
use crate::server::db::{Pool, tools};
use crate::server::file::file_util::{make_download_file, make_save_file};
use crate::server::plugin::request::PluginAddOrUpdateReq;
use anyhow::bail;
use common::{data_dir, id};
use protocol::common::req::IdsReq;
use rbs::value;
use rocket::tokio::fs;

pub async fn add(req: PluginAddOrUpdateReq<'_>, user: UserPrincipal) -> anyhow::Result<()> {
    let mut plugin = PluginBuilder::default()
        .id(Some(id::next()))
        .name(Some(req.name))
        .description(Some(req.description))
        .version(Some(req.version))
        .phase(Some(req.phase))
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;

    // 名称唯一
    let name = plugin.name.as_ref().unwrap();
    if check_exists(&plugin, None).await? {
        bail!("Plugin with name {} already exists", name);
    }

    let mut file = req.file;
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
    plugin.url = Some(url);

    Plugin::insert(Pool::get()?, &plugin).await?;
    Ok(())
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
    Plugin::delete_by_map(Pool::get()?, value! { "id": req.ids}).await?;
    Ok(())
}
