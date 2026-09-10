use rig::{client::AgentClientExt, message::Message, providers::openai};
use tokio::io::{self, BufReader};

use crate::{tools::mcp_tools::get_mcp_tools, utils::config::Config};

mod agent_loop;
mod tools;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();

    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut history: Vec<Message> = vec![];
    // NOTE: Hack to keep ref to MCP services, so connection stays alive
    let (tools, _keep_these_services_alive) = get_mcp_tools(config.clone()).await;

    let agent = openai::CompletionsClient::builder()
        .base_url(config.base_url)
        .api_key(config.api_key)
        .build()
        .map_err(|err| anyhow::anyhow!("Failed to connect to the LLM Provider: {err}"))?
        .agent(config.model)
        .preamble(&config.system_prompt)
        .tool_server_handle(tools)
        .max_tokens(config.max_token)
        .build();

    loop {
        if !agent_loop::loop_handler(&agent, &mut stdin, &mut stdout, &mut history).await? {
            break;
        }
    }

    Ok(())
}
