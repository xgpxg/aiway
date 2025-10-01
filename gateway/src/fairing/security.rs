//! # 安全验证
//! ## 主要功能
//! 对原始请求进行基本的安全验证。
//!
//! ## 基本准则
//! - 由网关系统内置，可通过系统级别的配置开启或关闭。
//! - 需最先执行，以拦截恶意请求。
//! - 不应提取请求body数据，仅对请求url（含query参数）、header等基础数据进行验证。
//! - 当验证失败时，更改uri到指定端点，返回错误信息。
//! - 不应涉及任何网络请求及IO操作，需要在5ms内完成
//!
//! ## 校验规则
//! - IP访问策略：allow:127.0.0.1, deny:1.1.1.1, allow:192.168.0.0/16
//! - Referer策略：allow:https://aaa.com, deny:https://bbb.com
//! - QPS策略：127.0.0.1:8080/1000, */2000
//!
//! ## 获取规则
//! 从控制台定时拉取网关配置，取其中的firewall配置
use crate::report::STATE;
use crate::router::Firewalld;
use crate::set_error;
use rocket::fairing::Fairing;
use rocket::{Data, Request};

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
        // TODO 校验请求
        let ip = req.client_ip().unwrap();
        let referer = req.headers().get_one("Referer").unwrap_or_default();
        if let Err(e) = Firewalld::check(&ip.to_string(), referer).await {
            // 拦截请求后，无效请求数+1
            STATE.inc_request_invalid_count(1);
            // 跳过后续的fairing处理
            set_error!(req, 403, e.to_string());
            return;
        }

        // 请求计数（不含无效请求）
        STATE.inc_request_count(1);
        // http连接计数
        // 该计数会在cleaner以及panic hook中-1
        STATE.inc_http_connect_count(1);

        //println!("Run PreSecurity on request");
    }
}
