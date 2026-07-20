use crate::Args;
use crate::gateway::Gateway;
use crate::service::LocalService;
use aiway_protocol::common::constants::GATEWAY_LOCAL_SOCK_PATH;
use pingora::prelude::*;

/// 启动 HTTP 服务器
pub fn start_http_server(args: &Args) -> anyhow::Result<()> {
    let mut server = Server::new(None)?;
    server.bootstrap();
    {
        let service = Gateway::new(args);
        let mut proxy = http_proxy_service(&server.configuration, service);

        let addr = format!("{}:{}", args.address, args.port);
        proxy.add_tcp(addr.as_str());

        let cpu_cores = num_cpus::get();
        proxy.threads = Some(cpu_cores);

        server.add_service(proxy);

        log::info!(
            "gateway started success, current version: {}, listening on: {}:{}, workers: {}",
            crate::VERSION,
            args.address,
            args.port,
            cpu_cores.max(4)
        );
    }

    {
        let service = LocalService::new(args);
        let mut proxy = http_proxy_service(&server.configuration, service);

        proxy.add_uds(GATEWAY_LOCAL_SOCK_PATH, None);

        let cpu_cores = num_cpus::get();
        proxy.threads = Some(cpu_cores);

        server.add_service(proxy);
    }

    std::thread::spawn(move || {
        server.run_forever();
    });

    Ok(())
}
