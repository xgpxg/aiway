use crate::server::auth::UserPrincipal;
use crate::server::db::models::api_key::ApiKeyStatus;
use crate::server::db::{Pool, models, tools};
use crate::server::key::ApiKeyListReq;
use crate::server::key::request::ApiKeyAddOrUpdateReq;
use crate::server::key::response::ApiKeyListRes;
use cache::caches::CacheKey;
use common::constants::ENCRYPT_KEY;
use common::id;
use protocol::common::req::{IdsReq, Pagination};
use protocol::common::res::{IntoPageRes, PageRes};
use protocol::gateway::ApiKey;
use rbs::value;
use serde_json::Value;

pub async fn add(req: ApiKeyAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let ak = match &req.principal {
        None => ApiKey::new().encrypt(ENCRYPT_KEY),
        Some(principal) => ApiKey::new_with_principal(principal).encrypt(ENCRYPT_KEY),
    };
    println!("{}", ak);
    cache::set(CacheKey::ApiKey(ak.clone()).to_string(), &Value::Null, None).await?;

    let api_key = models::api_key::ApiKeyBuilder::default()
        .id(Some(id::next()))
        .name(Some(req.name))
        .principal(req.principal)
        .secret(Some(ak))
        .status(Some(ApiKeyStatus::Ok))
        .eff_time(Some(tools::now()))
        .exp_time(req.exp_time)
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;

    models::api_key::ApiKey::insert(Pool::get()?, &api_key).await?;

    Ok(())
}

pub async fn delete(req: IdsReq) -> anyhow::Result<()> {
    for id in req.ids.iter() {
        let api_key =
            models::api_key::ApiKey::select_by_map(Pool::get()?, value! {"id": id}).await?;
        if api_key.is_empty() {
            continue;
        }
        let api_key = api_key.first().unwrap();
        let secret = api_key.secret.clone().unwrap();
        cache::remove(&CacheKey::ApiKey(secret).to_string()).await?;
        models::api_key::ApiKey::delete_by_map(Pool::get()?, value! {"id": id}).await?;
    }
    Ok(())
}

pub async fn list(req: ApiKeyListReq) -> anyhow::Result<PageRes<ApiKeyListRes>> {
    let page = models::api_key::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| ApiKeyListRes { inner: item })
            .collect::<Vec<_>>()
    });
    Ok(list)
}
