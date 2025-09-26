use crate::server::auth::UserPrincipal;
use crate::server::key::request::ApiKeyAddOrUpdateReq;
use protocol::gateway::ApiKey;
use serde_json::Value;
use cache::caches::CacheKey;
use common::constants::ENCRYPT_KEY;

pub async fn add(req: ApiKeyAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let api_key = ApiKey::new().encrypt(ENCRYPT_KEY);
    println!("{}", api_key);
    cache::set(CacheKey::ApiKey(api_key).to_string(), &Value::Null, None).await?;
    Ok(())
}
