use crate::server::db::Pool;
use crate::server::db::models::mcp_server::{McpServer, McpServerStatus};
use crate::server::db::models::mcp_tool::{McpTool, McpToolStatus};
use aiway_protocol::mcp::mcp::McpTool as Tool;
use aiway_protocol::mcp::mcp::{McpServer as Server, Route};
use itertools::Itertools;
use rbs::value;
pub(crate) async fn all_mcp_servers() -> anyhow::Result<Vec<Server>> {
    let tx = Pool::get()?;
    let servers = McpServer::select_by_map(
        tx,
        value! {
            "status": McpServerStatus::Ok
        },
    )
    .await?;
    if servers.is_empty() {
        return Ok(vec![]);
    }

    let tools = McpTool::select_by_map(
        tx,
        value! {
            "status": McpToolStatus::Ok
        },
    )
    .await?;

    let tools_group = tools
        .into_iter()
        .into_group_map_by(|tool| tool.mcp_server_id);

    let mut list = vec![];

    for server in servers {
        let server_tools = tools_group
            .get(&server.id)
            .cloned()
            .map(|tools| tools.into_iter().collect::<Vec<McpTool>>())
            .unwrap_or_default();
        let server_tools_map = server_tools
            .into_iter()
            .map(|tool| {
                (
                    tool.name.clone().unwrap(),
                    Tool {
                        name: tool.name.unwrap(),
                        description: tool.description.unwrap(),
                        input_schema: tool.input_schema,
                        route: Route {
                            route_type: tool.route_type.unwrap(),
                            service_name: tool.service_name,
                            service_path: tool.service_path,
                            url: tool.url,
                            method: tool.method,
                            request_param: tool.request_param,
                        },
                    },
                )
            })
            .collect();
        let mcp_server = Server {
            name: server.name.unwrap(),
            description: server.description,
            server_type: server.server_type.unwrap(),
            tools: server_tools_map,
            proxy_config: server.proxy_config,
        };
        list.push(mcp_server);
    }

    Ok(list)
}
