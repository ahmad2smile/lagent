# la' Agent

Minimalistic and focused on Security by tool use restrictions but still in auto mode. No permissions model.

NOTE: Code is handcrafted but docs (+anything below) is AI written using this lagent but manually reviewed and edited.

## Features

- 🤖 **Interactive CLI** — Chat with an AI agent directly from your terminal
- 🔧 **File System Tools** — Read, write, and list directories with sandboxed permissions
- 📦 **Tool Calling** — The agent can discover and use tools to perform actions
- 🔄 **Streaming Responses** — Real-time output as the model generates responses
- 💾 **Conversation History** — Maintains context across multiple turns
- 🛡️ **Sandboxed Access** — Tools are restricted to the current working directory

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- A local LLM server (e.g., [Ollama](https://ollama.ai)) running on `http://localhost:8080`
- [MCP servers](https://modelcontextprotocol.io/introduction) (optional, for remote tool integration)

## Installation

```bash
cargo install --path .
```

## Usage

```bash
lagent
```

Then in your terminal, you can interact with the agent:

```
> Read the file ./README.md
> List the contents of ./src
> Write a new file with the greeting "Hello from lagent!"
> /help
```

## Commands

| Command                | Description                                                                |
| ---------------------- | -------------------------------------------------------------------------- |
| `/exit`, `/quit`, `/q` | Exit the agent and return to the terminal                                  |
| `/help`, `/?`          | Show available commands                                                    |
| `/clear`, `/new`       | Clear the conversation / start a new session                               |
| `!<any>`               | Use this to provide output of some command to agent only, ex: `cargo test` |

## Environment Variables

The following environment variables can be used to configure the agent:

| Variable         | Description                      | Default                     |
| ---------------- | -------------------------------- | --------------------------- |
| `LLM_BASE_URL`   | The base URL of the LLM endpoint | `http://localhost:8080/v1/` |
| `LLM_API_KEY`    | The API key for authentication   | `sk-no-key`                 |
| `LLM_MODEL`      | The model name to use            | `Qwen3.8-9B`                |
| `LLM_MAX_TOKENS` | Maximum tokens per response      | `8192`                      |
| `MCP_SERVERS`    | Comma-separated MCP server URLs  | `http://127.0.0.1:8765/mcp` |

### Example

```bash
export LLM_BASE_URL="http://localhost:8080/v1/"
export LLM_API_KEY="sk-no-key"
export LLM_MODEL="Qwen3.8-9B"
export LLM_MAX_TOKENS="4096"
export MCP_SERVERS="http://127.0.0.1:8765/mcp"

lagent
```

## Tools

| Tool         | Description                                                          |
| ------------ | -------------------------------------------------------------------- |
| `read_file`  | Read the contents of a file at the specified path                    |
| `write_file` | Write content to a file (creates parent dirs, overwrites if exists)  |
| `list_dir`   | List the contents of a directory under the current working directory |

### Tool Permissions

All tools are **sandboxed** to the current working directory. Paths are resolved relative to the cwd, traversal sequences like `..` are blocked, and access outside the cwd is denied.

## Architecture

```
lagent
├── src/
│   ├── main.rs              # Entry point, config, CLI setup
│   ├── agent_loop.rs        # Interactive CLI handler with streaming responses
│   ├── tools/               # Tool definitions and execution
│   │   ├── mod.rs           # Tool module exports
│   │   ├── commands.rs      # CLI command parsing
│   │   ├── filesystem.rs    # File operations (read, write, list)
│   │   └── mcp_tools.rs     # MCP service registration
│   └── utils/               # Utilities
│       ├── mod.rs           # Utility module exports
│       ├── config.rs        # Environment variable parsing
│       ├── executor_utils.rs # Shell command execution
│       └── path_utils.rs    # Path resolution and sandbox checks
```

## License

MIT
