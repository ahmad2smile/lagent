use std::env;

pub(crate) struct Config {
    pub name: String,
    pub version: String,
    pub mcp_servers: Vec<String>,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        Self {
            name: "lagent".into(),
            version: "0.0.1".into(),
            mcp_servers: env::var("MCP_SERVERS").map_or(vec![], |val| {
                val.split(',')
                    .map(|str| str.to_string())
                    .collect::<Vec<String>>()
            }),
        }
    }
}
