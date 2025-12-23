use crate::server::db::Pool;
use crate::server::db::models::service::{Service, ServiceStatus};
use rbs::value;

pub(crate) async fn models() -> anyhow::Result<Vec<protocol::model::Model>> {
    Ok(vec![protocol::model::Model {
        name: "qwen-plus".to_string(),
        providers: vec![protocol::model::Provider {
            name: "qwen".to_string(),
            api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key: "".to_string().into(),
            weight: 0,
        }],
        lb: protocol::model::LbStrategy::Random,
    }])
}
