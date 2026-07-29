use crate::Args;
use crate::gateway::Gateway;
use crate::service::LocalService;
use aiway_protocol::common::constants::GATEWAY_LOCAL_SOCK_PATH;
use pingora::apps::HttpServerOptions;
use pingora::prelude::*;
use pingora::proxy::ProxyServiceBuilder;

/// 启动 HTTP 服务器
pub fn start_http_server(args: &Args) -> anyhow::Result<()> {
    let mut server = Server::new(None)?;
    server.bootstrap();
    {
        let service = Gateway::new(args);
        let mut server_options = HttpServerOptions::default();
        server_options.h2c = true;

        let mut proxy = ProxyServiceBuilder::new(&server.configuration, service)
            .server_options(server_options)
            .build();

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
            cpu_cores
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
