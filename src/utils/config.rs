use std::env;

pub(crate) struct Config {
    pub(crate) mcp_servers: Vec<String>,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        Self {
            mcp_servers: env::var("MCP_SERVERS").map_or(vec![], |val| {
                val.split(',')
                    .map(|str| str.to_string())
                    .collect::<Vec<String>>()
            }),
        }
    }
}
