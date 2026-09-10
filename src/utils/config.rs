use std::env;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub name: String,
    pub version: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub system_prompt: String,
    pub max_token: u64,
    pub mcp_servers: Vec<String>,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        Self {
            name: "lagent".into(),
            version: "0.0.1".into(),
            system_prompt: env::var("LLM_SYSTEM_PROMPT")
                .unwrap_or(r#"You are lagent, an interactive AI coding agent."#.into()),
            base_url: env::var("LLM_BASE_URL").unwrap_or("http://localhost:8080/v1/".into()),
            api_key: env::var("LLM_API_KEY").unwrap_or("sk-no-key".into()),
            model: env::var("LLM_MODEL").unwrap_or("Qwen3.8-9B".into()),
            max_token: env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(u64::MAX),
            mcp_servers: env::var("MCP_SERVERS").map_or(vec![], |val| {
                val.split(',')
                    .map(|str| str.into())
                    .collect::<Vec<String>>()
            }),
        }
    }
}
