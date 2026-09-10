use rig::{client::AgentClientExt, message::Message, providers::openai};
use tokio::io::{self, BufReader};

use crate::{tools::mcp_tools::get_mcp_tools, utils::config::Config};

mod agent_loop;
mod tools;
mod utils;

const MAX_TOKENS: u64 = 8192;
const SYSTEM_PROMPT: &str = r#"You are lagent, an interactive AI coding agent.

You have access to following tools:
- read_file: Read file contents
- write_file: Create or replace file contents
"#;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();

    let client = openai::CompletionsClient::builder()
        .base_url("http://localhost:8080/v1/".to_string())
        .api_key("sk-no-key")
        .build()
        .map_err(|err| anyhow::anyhow!("Failed to connect to the LLM Provider: {err}"))?;

    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut history: Vec<Message> = vec![];

    let mut agent_builder = client
        .agent("Qwen3.8-9B")
        .preamble(SYSTEM_PROMPT)
        .tool(tools::filesystem::ReadFile)
        .tool(tools::filesystem::WriteFile)
        .tool(tools::filesystem::ListDir)
        .max_tokens(MAX_TOKENS);

    for (tools, mcp_client) in get_mcp_tools(config.mcp_servers).await {
        agent_builder = agent_builder.rmcp_tools(tools, mcp_client);
    }

    let agent = agent_builder.build();

    loop {
        if !agent_loop::loop_handler(&agent, &mut stdin, &mut stdout, &mut history).await? {
            break;
        }
    }

    Ok(())
}
