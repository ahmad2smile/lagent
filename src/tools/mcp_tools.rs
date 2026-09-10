use rig::tool::{
    rmcp::McpClientHandler,
    server::{ToolServer, ToolServerHandle},
};
use rmcp::{
    RoleClient,
    model::{ClientCapabilities, ClientInfo, Implementation},
    service::RunningService,
    transport::StreamableHttpClientTransport,
};

use crate::{tools, utils::config::Config};

pub(crate) async fn get_mcp_tools(
    config: Config,
) -> (
    ToolServerHandle,
    Vec<RunningService<RoleClient, McpClientHandler>>,
) {
    let tool_server_handle = ToolServer::new()
        .tool(tools::filesystem::ReadFile)
        .tool(tools::filesystem::WriteFile)
        .tool(tools::filesystem::ListDir)
        .run();

    let client_info = ClientInfo::new(
        ClientCapabilities::default(),
        Implementation::new(config.name, config.version),
    );
    let mut running_services = vec![];

    for server in config.mcp_servers {
        let transport = StreamableHttpClientTransport::from_uri(server.as_str());
        let handler = McpClientHandler::new(client_info.clone(), tool_server_handle.clone());
        match handler.connect(transport).await {
            Ok(service) => {
                running_services.push(service);
                println!("Connected to MCP at: {server}")
            }
            Err(err) => println!("Error connecting to MCP at: {server}, {err}"),
        }
    }

    (tool_server_handle, running_services)
}
