use crate::components::Firewalld;
use crate::handler::{HttpError, HttpResult};
use crate::report::STATE;
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;

pub async fn firewall_check(session: &mut Session, _: &mut HttpContext) -> HttpResult<()> {
    let addr = session.client_addr();
    let ip = addr
        .map(|addr| match addr {
            SocketAddr::Inet(addr) => addr.ip().to_string(),
            SocketAddr::Unix(_) => {
                unimplemented!()
            }
        })
        .unwrap_or_default();
    let referer = session
        .req_header()
        .headers
        .get(Headers::REFERER)
        .map(|v| v.to_str().unwrap())
        .unwrap_or_default();

    // 调用防火墙校验请求
    if let Err(e) = Firewalld::check(&ip, referer).await {
        // 拦截请求后，无效请求数+1
        STATE.inc_request_invalid_count(1);

        return Err(HttpError::new(403, &e.to_string()));
    }

    // http连接计数
    // 该计数会在cleaner以及panic hook中-1
    STATE.inc_http_connect_count(1);

    Ok(())
}
