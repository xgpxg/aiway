//! # 安全验证
//! ## 主要功能
//! 对原始请求进行基本的安全验证。
//!
//! ## 基本准则
//! - 由网关系统内置。
//! - 需最先执行，以拦截恶意请求。
//! - 不应提取请求body数据，仅对请求url（含query参数）、header等基础数据进行验证。
//! - 当验证失败时，在Header中设置错误信息，并返回错误信息。
//! - 不应涉及任何网络请求及IO操作，需要在5ms内完成
//!
//! ## 校验规则
//! - IP访问策略：allow:127.0.0.1, deny:1.1.1.1, allow:192.168.0.0/16
//! - Referer策略：allow:https://aaa.com, deny:https://bbb.com
//! - QPS策略：127.0.0.1:8080/1000, */2000
//!
//! ## 获取规则
//! 从控制台定时拉取网Firewall配置
//!
use crate::components::Firewalld;
use crate::report::STATE;
use rocket::fairing::Fairing;
use rocket::{Data, Request};
use context::set_error;

pub struct PreSecurity {}
impl PreSecurity {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for PreSecurity {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "PreSecurity",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let ip = req.client_ip().unwrap();
        let referer = req
            .headers()
            .get_one(context::Headers::REFERER)
            .unwrap_or_default();
        // 调用防火墙校验请求
        if let Err(e) = Firewalld::check(&ip.to_string(), referer).await {
            // 拦截请求后，无效请求数+1
            STATE.inc_request_invalid_count(1);
            // 跳过后续的fairing处理
            set_error!(req, 403, e.to_string());
            return;
        }

        // http连接计数
        // 该计数会在cleaner以及panic hook中-1
        STATE.inc_http_connect_count(1);
    }
}
