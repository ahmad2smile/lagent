use rig::{client::AgentClientExt, message::Message, providers::openai};
use tokio::io::{self, BufReader};

use crate::{tools::mcp_tools::get_mcp_tools, utils::config::Config};

mod agent_loop;
mod tools;
mod utils;

const MAX_TOKENS: u64 = 8192;
const SYSTEM_PROMPT: &str = r#"You are lagent, an interactive AI coding agent."#;

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
    let (tools, _keep_these_services_alive) = get_mcp_tools(config).await;

    let agent = client
        .agent("Qwen3.8-9B")
        .preamble(SYSTEM_PROMPT)
        .tool_server_handle(tools)
        .max_tokens(MAX_TOKENS)
        .build();

    loop {
        if !agent_loop::loop_handler(&agent, &mut stdin, &mut stdout, &mut history).await? {
            break;
        }
    }

    Ok(())
}
