use rmcp::{
    ServiceExt,
    model::{ClientInfo, Tool},
    service::ServerSink,
    transport::StreamableHttpClientTransport,
};

pub(crate) async fn get_mcp_tools(mcp_servers: Vec<String>) -> Vec<(Vec<Tool>, ServerSink)> {
    let mut tools: Vec<(Vec<Tool>, ServerSink)> = vec![];

    for server in mcp_servers {
        let transport = StreamableHttpClientTransport::from_uri(server.as_str());

        match ClientInfo::default().serve(transport).await {
            Ok(client) => {
                match client.list_tools(Default::default()).await {
                    Ok(server_tools) => {
                        tools.push((server_tools.tools, client.peer().to_owned()));
                    }
                    Err(err) => {
                        eprintln!("Failed to get tools for MCP at {server} error: {err}");
                    }
                };
            }
            Err(err) => {
                eprintln!("Failed to connect to MCP at {server} error: {err}");
            }
        }
    }

    tools
}
