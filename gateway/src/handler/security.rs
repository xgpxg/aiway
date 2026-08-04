use crate::components::Firewalld;
use crate::handler::{HandlerError, HandlerResult, get_real_ip};
use crate::report::STATE;
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;

pub async fn firewall_check(session: &mut Session, _: &mut HttpContext) -> HandlerResult<()> {
    let headers = &session.req_header().headers;

    let fallback = session
        .client_addr()
        .map(|addr| match addr {
            SocketAddr::Inet(addr) => addr.ip().to_string(),
            SocketAddr::Unix(_) => String::new(),
        })
        .unwrap_or_default();
    let ip = get_real_ip(headers, fallback);

    let referer = headers
        .get(Headers::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // 调用防火墙校验请求
    if let Err(e) = Firewalld::check(&ip, referer).await {
        // 拦截请求后，无效请求数+1
        STATE.inc_request_invalid_count(1);

        return Err(HandlerError::new(e.0.as_u16(), &e.1));
    }

    // http连接计数
    // 该计数会在cleaner以及panic hook中-1
    STATE.inc_http_connect_count(1);

    Ok(())
}
