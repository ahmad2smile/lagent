# lagent

An interactive AI coding agent CLI built with Rust, designed to work with local LLM endpoints.

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
> exit
```

## Tools

| Tool         | Description                                                          |
| ------------ | -------------------------------------------------------------------- |
| `read_file`  | Read the contents of a file at the specified path                    |
| `write_file` | Write content to a file (creates parent dirs, overwrites if exists)  |
| `list_dir`   | List the contents of a directory under the current working directory |

### Tool Permissions

All tools are **sandboxed** to the current working directory:

- Paths are resolved relative to the current directory
- Traversal sequences like `..` are blocked
- Access outside the cwd is denied with a clear error message

## Architecture

```
lagent
├── src/
│   ├── main.rs          # Entry point, agent setup, CLI loop
│   └── utils/
│       ├── mod.rs       # Module exports
│       ├── tools.rs     # Tool definitions (read_file, write_file, list_dir)
│       └── path_utils.rs # Path resolution and sandbox checks
```

## Configuration

The agent uses the following defaults:

- **Model:** `Qwen3.8-9B`
- **Base URL:** `http://localhost:8080/v1/`
- **API Key:** `sk-no-key` (no authentication required)
- **Max Tokens:** `8192`

## Dependencies

- `rig` — Agent and tool orchestration framework
- `tokio` — Async runtime
- `serde` / `serde_json` — Serialization
- `colored` — Colored terminal output
- `anyhow` — Error handling

## License

MIT
